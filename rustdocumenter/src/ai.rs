//! AI-powered doc comment generation.
//!
//! Uses Claude Code CLI in print mode (Claude MAX license) as primary,
//! falls back to Codex CLI (ChatGPT license).
//! No API keys needed — both use existing subscriptions.
//!
//! This module is pure logic — IO is delegated to `gateway`.

use crate::gateway;

const SMART_LEFT_SINGLE: char = '\u{2018}';
const SMART_RIGHT_SINGLE: char = '\u{2019}';
const SMART_LEFT_DOUBLE: char = '\u{201C}';
const SMART_RIGHT_DOUBLE: char = '\u{201D}';

/// Generate a doc comment for a public item using AI.
///
/// Returns the doc text (without `///` prefix) or an error message.
/// Tries Claude Code CLI first, then Codex CLI as fallback.
pub fn generate_doc(signature: &str, body: &str, kind: &str, name: &str) -> Result<String, String> {
    let prompt = build_prompt(signature, body, kind, name);

    if let Ok(text) = gateway::run_claude(&prompt) {
        return Ok(clean_response(&text));
    }

    match gateway::run_codex(&prompt) {
        Ok(text) => Ok(clean_response(&text)),
        Err(e) => Err(e),
    }
}

const SLINT_KINDS: &[&str] = &["property", "callback", "component"];

/// Build the prompt sent to the AI model.
fn build_prompt(signature: &str, body: &str, kind: &str, _name: &str) -> String {
    let lang = if SLINT_KINDS.contains(&kind) { "Slint" } else { "Rust" };
    // For single-line items body == signature; for multi-line body is the full block.
    let code = if body.is_empty() || body == signature { signature.to_string() } else { body.to_string() };

    format!(
        "```\n{code}\n```\n\n\
         Write a 1-3 line /// doc comment for this {lang} {kind}. \
         Output ONLY the plain text of the comment. \
         Do not include /// prefix. Do not include code fences. \
         Do not ask questions. Do not include any preamble."
    )
}

/// Strip code fences, `///` prefixes, and unicode smart quotes from the AI response.
fn clean_response(text: &str) -> String {
    text.lines()
        .filter(|l| !l.trim().starts_with("```"))
        .map(|l| {
            let trimmed = l.trim_start();
            let line = if trimmed.starts_with("/// ") {
                trimmed[4..].to_string()
            } else if trimmed == "///" {
                String::new()
            } else {
                l.to_string()
            };
            line.replace(SMART_LEFT_SINGLE, "'")
                .replace(SMART_RIGHT_SINGLE, "'")
                .replace(SMART_LEFT_DOUBLE, "\"")
                .replace(SMART_RIGHT_DOUBLE, "\"")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
