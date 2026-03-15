//! Extracts all public items from .rs and .slint source files.
//!
//! Items with `///` doc comments above them get a populated `doc` field.
//! Items without doc comments are still emitted with `doc = ""` so that
//! the viewer shows the full public surface and `check` can flag gaps.
//!
//! This module is stateless — all compiled regex patterns live in the
//! mother module (`lib.rs`) and are passed via `RsPatterns`/`SlintPatterns`.

use std::path::Path;
use regex::Regex;

use crate::manifest::{DocItem, ItemKind};

const DOC_PREFIX_LEN: usize = 4;
const MAX_SIG_LINES: usize = 8;
const MAX_BODY_LINES: usize = 50;

// ─── Pattern types ───────────────────────────────────────────────────────────

/// A compiled regex paired with the item kind it matches.
pub struct ItemPattern {
    pub regex: Regex,
    pub kind: ItemKind,
}

/// Compiled regex patterns for parsing `.rs` files.
pub struct RsPatterns {
    pub items: Vec<ItemPattern>,
    pub attribute: Regex,
    pub pub_use: Regex,
}

/// Compiled regex patterns for parsing `.slint` files.
pub struct SlintPatterns {
    pub items: Vec<ItemPattern>,
}

impl RsPatterns {
    /// Return an empty pattern set (used as fallback when compilation fails).
    pub fn empty() -> Self {
        // "$^" never matches any input — guaranteed valid, safe to unwrap_or
        let never = Regex::new("$^").unwrap_or_else(|_| Regex::new("a^").unwrap_or_else(|_| unreachable!()));
        Self { items: Vec::new(), attribute: never.clone(), pub_use: never }
    }

    /// Compile all Rust item patterns. Returns `Err` if a regex is invalid.
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            items: vec![
                ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+(?:async\s+)?fn\s+(\w+)")?, kind: ItemKind::Fn },
                ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+struct\s+(\w+)")?, kind: ItemKind::Struct },
                ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+enum\s+(\w+)")?, kind: ItemKind::Enum },
                ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+trait\s+(\w+)")?, kind: ItemKind::Trait },
                ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+type\s+(\w+)")?, kind: ItemKind::Type },
                ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+mod\s+(\w+)")?, kind: ItemKind::Mod },
                ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+(?:const|static)\s+(\w+)")?, kind: ItemKind::Const },
            ],
            attribute: Regex::new(r"^\s*#\[")?,
            pub_use: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+use\s+")?,
        })
    }
}

impl SlintPatterns {
    /// Return an empty pattern set (used as fallback when compilation fails).
    pub fn empty() -> Self {
        Self { items: Vec::new() }
    }

    /// Compile all Slint item patterns. Returns `Err` if a regex is invalid.
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            items: vec![
                ItemPattern { regex: Regex::new(r"^\s*export\s+component\s+(\w+)")?, kind: ItemKind::Component },
                ItemPattern { regex: Regex::new(r"^\s*export\s+struct\s+(\w+)")?, kind: ItemKind::Struct },
                ItemPattern { regex: Regex::new(r"^\s*export\s+enum\s+(\w+)")?, kind: ItemKind::Enum },
                ItemPattern { regex: Regex::new(r"^\s*(?:pure\s+)?callback\s+(\w[\w-]*)")?, kind: ItemKind::Callback },
                ItemPattern { regex: Regex::new(r"^\s*(?:in|out|in-out|private)\s+property\s+<[^>]+>\s+(\w[\w-]*)")?, kind: ItemKind::Property },
            ],
        })
    }
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Extract all public items from a `.rs` file.
/// Items without `///` above them are included with `doc = ""`.
pub fn parse_rs(path: &Path, content: &str, patterns: &RsPatterns) -> Vec<DocItem> {
    let lines: Vec<&str> = content.lines().collect();
    let mut items = Vec::new();
    let mut doc_buf: Vec<String> = Vec::new();

    for (idx, &line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("/// ") || trimmed == "///" {
            let text = if trimmed.len() > DOC_PREFIX_LEN { &trimmed[DOC_PREFIX_LEN..] } else { "" };
            doc_buf.push(text.to_string());
            continue;
        }

        if patterns.attribute.is_match(line) {
            continue;
        }

        if trimmed.is_empty() {
            doc_buf.clear();
            continue;
        }

        if patterns.pub_use.is_match(line) {
            doc_buf.clear();
            continue;
        }

        if let Some((name, kind)) = match_item(line, &patterns.items) {
            let sig = extract_rs_signature(&lines, idx);
            items.push(DocItem {
                name,
                kind,
                signature: sig,
                line: idx + 1,
                doc: doc_buf.join("\n"),
            });
        }

        if !trimmed.starts_with("//") {
            doc_buf.clear();
        }
    }

    let _ = path;
    items
}

/// Extract all exported items from a `.slint` file.
/// Items without `///` above them are included with `doc = ""`.
pub fn parse_slint(path: &Path, content: &str, patterns: &SlintPatterns) -> Vec<DocItem> {
    let lines: Vec<&str> = content.lines().collect();
    let mut items = Vec::new();
    let mut doc_buf: Vec<String> = Vec::new();

    for (idx, &line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("/// ") || trimmed == "///" {
            let text = if trimmed.len() > DOC_PREFIX_LEN { &trimmed[DOC_PREFIX_LEN..] } else { "" };
            doc_buf.push(text.to_string());
            continue;
        }

        if trimmed.is_empty() {
            doc_buf.clear();
            continue;
        }

        if let Some((name, kind)) = match_item(line, &patterns.items) {
            let sig = extract_slint_signature(&lines, idx);
            items.push(DocItem {
                name,
                kind,
                signature: sig,
                line: idx + 1,
                doc: doc_buf.join("\n"),
            });
        }

        if !trimmed.starts_with("//") {
            doc_buf.clear();
        }
    }

    let _ = path;
    items
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Match a line against a list of item patterns.
fn match_item(line: &str, patterns: &[ItemPattern]) -> Option<(String, ItemKind)> {
    for pat in patterns {
        if let Some(c) = pat.regex.captures(line) {
            return Some((c[1].to_string(), pat.kind.clone()));
        }
    }
    None
}

/// Extract the signature of a Rust item starting at `start_idx`.
fn extract_rs_signature(lines: &[&str], start_idx: usize) -> String {
    let mut parts = Vec::new();
    for i in start_idx..lines.len().min(start_idx + MAX_SIG_LINES) {
        let trimmed = lines[i].trim();
        parts.push(trimmed);
        if trimmed.ends_with('{') || trimmed.ends_with(';') || trimmed.ends_with("{}") {
            break;
        }
    }
    let joined = parts.join(" ");
    joined
        .trim_end_matches(|c| c == '{' || c == ' ')
        .to_string()
}

/// Extract the signature of a Slint item starting at `start_idx`.
fn extract_slint_signature(lines: &[&str], start_idx: usize) -> String {
    let line = lines[start_idx].trim();
    if let Some(brace) = line.find(" {") {
        return line[..brace].trim().to_string();
    }
    line.trim_end_matches(|c: char| c == '{' || c == ' ').to_string()
}

/// Extract the body of an item starting at a 1-based line number.
///
/// Counts braces from the item line to the matching `}`.
/// Returns the body content as a string (max 50 lines for AI context).
pub fn extract_body(content: &str, start_line: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start_idx = start_line.saturating_sub(1);
    if start_idx >= lines.len() {
        return String::new();
    }

    let mut depth: i32 = 0;
    let mut body_lines = Vec::new();
    let mut has_open_brace = false;

    for line in &lines[start_idx..] {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                has_open_brace = true;
            } else if ch == '}' {
                depth -= 1;
            }
        }
        body_lines.push(*line);

        if has_open_brace && depth <= 0 {
            break;
        }

        if body_lines.len() >= MAX_BODY_LINES {
            break;
        }
    }

    body_lines.join("\n")
}
