//! Extracts all public items from .rs and .slint source files.
//!
//! Items with `///` doc comments above them get a populated `doc` field.
//! Items without doc comments are still emitted with `doc = ""` so that
//! the viewer shows the full public surface and `check` can flag gaps.

use std::path::Path;
use regex::Regex;
use std::sync::LazyLock;

use crate::manifest::{DocItem, ItemKind};

// ─── Rust patterns ────────────────────────────────────────────────────────────

static PUB_FN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+(?:async\s+)?fn\s+(\w+)").unwrap()
});
static PUB_STRUCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+struct\s+(\w+)").unwrap()
});
static PUB_ENUM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+enum\s+(\w+)").unwrap()
});
static PUB_TRAIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+trait\s+(\w+)").unwrap()
});
static PUB_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+type\s+(\w+)").unwrap()
});
static PUB_MOD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+mod\s+(\w+)").unwrap()
});
static PUB_CONST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+(?:const|static)\s+(\w+)").unwrap()
});
static PUB_USE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+use\s+").unwrap()
});
static ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*#\[").unwrap()
});

// ─── Slint patterns ───────────────────────────────────────────────────────────

static SLINT_COMPONENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*export\s+component\s+(\w+)").unwrap()
});
static SLINT_STRUCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*export\s+struct\s+(\w+)").unwrap()
});
static SLINT_ENUM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*export\s+enum\s+(\w+)").unwrap()
});
static SLINT_CALLBACK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:pure\s+)?callback\s+(\w[\w-]*)").unwrap()
});
static SLINT_PROPERTY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:in|out|in-out|private)\s+property\s+<[^>]+>\s+(\w[\w-]*)").unwrap()
});

// ─── Public API ───────────────────────────────────────────────────────────────

/// Extract all public items from a `.rs` file.
/// Items without `///` above them are included with `doc = ""`.
pub fn parse_rs(path: &Path, content: &str) -> Vec<DocItem> {
    let lines: Vec<&str> = content.lines().collect();
    let mut items = Vec::new();
    let mut doc_buf: Vec<String> = Vec::new();

    for (idx, &line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("/// ") || trimmed == "///" {
            let text = if trimmed.len() > 4 { &trimmed[4..] } else { "" };
            doc_buf.push(text.to_string());
            continue;
        }

        // Attribute lines — keep doc buffer alive
        if ATTRIBUTE.is_match(line) {
            continue;
        }

        // Blank line — reset doc buffer
        if trimmed.is_empty() {
            doc_buf.clear();
            continue;
        }

        // Skip pub use re-exports
        if PUB_USE.is_match(line) {
            doc_buf.clear();
            continue;
        }

        // Emit ALL matched public items — doc="" if no /// above
        if let Some((name, kind)) = match_rs_item(line) {
            let sig = extract_rs_signature(&lines, idx);
            items.push(DocItem {
                name,
                kind,
                signature: sig,
                line: idx + 1,
                doc: doc_buf.join("\n"),
            });
        }

        // Non-comment, non-blank, non-attribute resets the doc buffer
        if !trimmed.starts_with("//") {
            doc_buf.clear();
        }
    }

    let _ = path;
    items
}

/// Extract all exported items from a `.slint` file.
/// Items without `///` above them are included with `doc = ""`.
pub fn parse_slint(path: &Path, content: &str) -> Vec<DocItem> {
    let lines: Vec<&str> = content.lines().collect();
    let mut items = Vec::new();
    let mut doc_buf: Vec<String> = Vec::new();

    for (idx, &line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("/// ") || trimmed == "///" {
            let text = if trimmed.len() > 4 { &trimmed[4..] } else { "" };
            doc_buf.push(text.to_string());
            continue;
        }

        if trimmed.is_empty() {
            doc_buf.clear();
            continue;
        }

        // Emit ALL matched exported items — doc="" if no /// above
        if let Some((name, kind)) = match_slint_item(line) {
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

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn match_rs_item(line: &str) -> Option<(String, ItemKind)> {
    if let Some(c) = PUB_FN.captures(line)     { return Some((c[1].to_string(), ItemKind::Fn)); }
    if let Some(c) = PUB_STRUCT.captures(line) { return Some((c[1].to_string(), ItemKind::Struct)); }
    if let Some(c) = PUB_ENUM.captures(line)   { return Some((c[1].to_string(), ItemKind::Enum)); }
    if let Some(c) = PUB_TRAIT.captures(line)  { return Some((c[1].to_string(), ItemKind::Trait)); }
    if let Some(c) = PUB_TYPE.captures(line)   { return Some((c[1].to_string(), ItemKind::Type)); }
    if let Some(c) = PUB_MOD.captures(line)    { return Some((c[1].to_string(), ItemKind::Mod)); }
    if let Some(c) = PUB_CONST.captures(line)  { return Some((c[1].to_string(), ItemKind::Const)); }
    None
}

fn match_slint_item(line: &str) -> Option<(String, ItemKind)> {
    if let Some(c) = SLINT_COMPONENT.captures(line) { return Some((c[1].to_string(), ItemKind::Component)); }
    if let Some(c) = SLINT_STRUCT.captures(line)    { return Some((c[1].to_string(), ItemKind::Struct)); }
    if let Some(c) = SLINT_ENUM.captures(line)      { return Some((c[1].to_string(), ItemKind::Enum)); }
    if let Some(c) = SLINT_CALLBACK.captures(line)  { return Some((c[1].to_string(), ItemKind::Callback)); }
    if let Some(c) = SLINT_PROPERTY.captures(line)  { return Some((c[1].to_string(), ItemKind::Property)); }
    None
}

/// Extract the signature of a Rust item starting at `start_idx`.
fn extract_rs_signature(lines: &[&str], start_idx: usize) -> String {
    let mut parts = Vec::new();
    for i in start_idx..lines.len().min(start_idx + 8) {
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
