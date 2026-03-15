//! rustdocumenter — build-time documentation scanner
//!
//! Scans .rs and .slint files for /// doc comments.
//! Generates man/ documentation and warns if comments are missing.

/// AI-powered doc comment generation via Claude Code CLI and Codex CLI fallback.
pub mod ai;
/// Orchestrates auto-documentation: finds undocumented items and inserts AI-generated `///` comments.
pub mod docgen;
/// IO gateway — filesystem and process operations.
pub mod gateway;
/// Shared serde types for the `man/` file format.
pub mod manifest;
/// Extracts public items and `///` doc comments from `.rs` and `.slint` files.
pub mod parser;
/// Writes `man/` output: per-file JSON + Markdown pages and MANIFEST indexes.
pub mod generator;

use std::sync::{LazyLock, OnceLock};

use manifest::SourceDoc;

const SKIP_DIRS: &[&str] = &["target", ".git", ".cargo", "man"];

/// Compiled Rust regex patterns — initialised once, owned by the mother.
pub static RS_PATTERNS: LazyLock<parser::RsPatterns> = LazyLock::new(|| {
    parser::RsPatterns::new().unwrap_or_else(|e| {
        INIT_ERROR.set(format!("Rust regex: {e}")).ok();
        // Safety: LazyLock must return a value; if init fails the error is
        // stored and collect_docs will return empty, main will exit cleanly.
        parser::RsPatterns::empty()
    })
});

/// Compiled Slint regex patterns — initialised once, owned by the mother.
pub static SLINT_PATTERNS: LazyLock<parser::SlintPatterns> = LazyLock::new(|| {
    parser::SlintPatterns::new().unwrap_or_else(|e| {
        INIT_ERROR.set(format!("Slint regex: {e}")).ok();
        parser::SlintPatterns::empty()
    })
});

/// Stores a regex compilation error if one occurred during initialisation.
pub static INIT_ERROR: OnceLock<String> = OnceLock::new();

/// Recursively collects and parses source documents from Rust and Slint files.
pub fn collect_docs(root: &std::path::Path) -> Vec<SourceDoc> {
    gateway::collect_docs(root, SKIP_DIRS, &RS_PATTERNS, &SLINT_PATTERNS)
}
