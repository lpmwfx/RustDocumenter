//! rustdocumenter — build-time documentation scanner
//!
//! Scans .rs and .slint files for /// doc comments.
//! Generates man/ documentation and warns if comments are missing.

pub mod manifest;
pub mod parser;
pub mod generator;

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use manifest::SourceDoc;

const SKIP_DIRS: &[&str] = &["target", ".git", ".cargo", "man"];

/// Scan the project for /// doc comments.
/// Generates man/ documentation.
/// Emits warnings for missing doc comments.
pub fn scan_project() {
    let root = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    scan_project_at(&root);
}

/// Scan project at a specific path.
pub fn scan_project_at(root: &Path) {
    let docs = collect_docs(root);
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    // Generate man/ documentation
    generator::generate(root, project_name, &docs);

    // Emit warnings for missing doc comments
    let mut missing_count = 0;
    for doc in &docs {
        for item in &doc.items {
            if item.doc.is_empty() {
                eprintln!(
                    "{}:{}:1: warning rust/docs/doc-required: missing /// doc comment on {} `{}`",
                    doc.source, item.line, item.kind.label(), item.name
                );
                missing_count += 1;
            }
        }
    }

    if missing_count > 0 {
        eprintln!(
            "rustdocumenter: {} items missing /// doc comments",
            missing_count
        );
    }
}

// ─── Collection ───────────────────────────────────────────────────────────────

pub fn collect_docs(root: &Path) -> Vec<SourceDoc> {
    let mut docs = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip ignored directories
        if path.components().any(|c| {
            SKIP_DIRS.contains(&c.as_os_str().to_str().unwrap_or(""))
        }) {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let items = match ext {
            "rs" => {
                let content = std::fs::read_to_string(path).unwrap_or_default();
                parser::parse_rs(path, &content)
            }
            "slint" => {
                let content = std::fs::read_to_string(path).unwrap_or_default();
                parser::parse_slint(path, &content)
            }
            _ => continue,
        };

        let source = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        docs.push(SourceDoc { source, items });
    }

    docs
}
