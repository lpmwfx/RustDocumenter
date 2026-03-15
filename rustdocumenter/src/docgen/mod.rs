//! Auto-generate `///` doc comments for undocumented public items.
//!
//! Reads each source file, finds items with empty docs, calls the AI
//! to generate doc text, and inserts `///` lines above the item.
//!
//! This module is pure orchestration — IO is delegated to `gateway`.

use std::path::Path;

use crate::gateway;
use crate::parser;

mod file_processor;
mod inserter;
mod item_processor;

/// Result of auto-documenting a project.
pub struct DocGenResult {
    /// Number of items that got AI-generated docs.
    pub generated: usize,
    /// Number of items skipped due to errors.
    pub skipped: usize,
    /// Log lines for each processed item.
    pub log: Vec<String>,
}

/// Auto-document all undocumented public items in the project.
///
/// Returns a result with counts and log lines. Caller is responsible for printing.
pub fn document_project(
    root: &Path,
    rs_patterns: &parser::RsPatterns,
    slint_patterns: &parser::SlintPatterns,
) -> DocGenResult {
    let skip_dirs = &["target", ".git", ".cargo", "man"];
    let docs = gateway::collect_docs(root, skip_dirs, rs_patterns, slint_patterns);
    inserter::process_all(root, &docs)
}
