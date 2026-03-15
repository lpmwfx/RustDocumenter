//! Filesystem IO operations.

use std::path::Path;
use walkdir::WalkDir;

use crate::manifest::{SourceDoc, SourceLang};
use crate::parser;

/// Walk the project tree and collect all documented and undocumented public items.
pub fn collect_docs(
    root: &Path,
    skip_dirs: &[&str],
    rs_patterns: &parser::RsPatterns,
    slint_patterns: &parser::SlintPatterns,
) -> Vec<SourceDoc> {
    let mut docs = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        if path.components().any(|c| {
            skip_dirs.contains(&c.as_os_str().to_str().unwrap_or(""))
        }) {
            continue;
        }

        let lang = match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => SourceLang::Rust,
            Some("slint") => SourceLang::Slint,
            _ => continue,
        };

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let items = match lang {
            SourceLang::Rust => parser::parse_rs(path, &content, rs_patterns),
            SourceLang::Slint => parser::parse_slint(path, &content, slint_patterns),
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

/// Read a file to string. Returns `Err` with a message on failure.
pub fn read_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Write content to a file. Returns `Err` with a message on failure.
pub fn write_file(path: &Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| format!("{}: {e}", path.display()))
}
