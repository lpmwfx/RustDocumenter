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
const CARGO_TOML: &str = "Cargo.toml";

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

/// Auto-generate `///` doc comments for all undocumented public items.
///
/// Designed to be called from `build.rs` as a `[build-dependencies]` entry.
/// Traverses `.rs` and `.slint` source files in the project, generates doc
/// comments via AI (Claude Code CLI, fallback Codex CLI), and writes them
/// directly into the source files.
///
/// On the first build, missing docs are filled in and Cargo detects the
/// changed files — the project recompiles with the new comments in place.
/// On subsequent builds, all items are already documented and nothing is
/// written, so the build is fast.
///
/// Emits `cargo:warning=` lines for each item documented (visible in
/// `cargo build` output). Silent when all items are already documented.
pub fn document_project() {
    use std::io::Write;

    let root = resolve_build_root();
    let doc_result = docgen::document_project(&root, &RS_PATTERNS, &SLINT_PATTERNS);

    for line in &doc_result.log {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let _ = writeln!(std::io::stdout(), "cargo:warning=rustdocumenter: {trimmed}");
        }
    }

    if doc_result.generated > 0 {
        let _ = writeln!(
            std::io::stdout(),
            "cargo:warning=rustdocumenter: {} item(s) documented",
            doc_result.generated
        );
    }

    if doc_result.skipped > 0 {
        let _ = writeln!(
            std::io::stdout(),
            "cargo:warning=rustdocumenter: {} item(s) skipped (AI unavailable)",
            doc_result.skipped
        );
    }
}

/// Resolve the project root for build-script context.
///
/// Walks up from `CARGO_MANIFEST_DIR` to find the workspace root
/// (a `Cargo.toml` containing `[workspace]`), falling back to the
/// manifest directory itself for single-crate projects.
fn resolve_build_root() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

    let mut dir = manifest_dir.clone();
    loop {
        let cargo_toml = dir.join(CARGO_TOML);
        if cargo_toml.exists() {
            if let Ok(content) = gateway::read_file(&cargo_toml) {
                if content.contains("[workspace]") {
                    return dir;
                }
            }
        }
        match dir.parent().map(|p| p.to_path_buf()) {
            Some(parent) if parent != dir => dir = parent,
            _ => return manifest_dir,
        }
    }
}
