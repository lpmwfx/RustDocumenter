//! Iterates source docs and dispatches per-file processing.

use std::path::Path;

use crate::manifest::SourceDoc;

use crate::docgen::DocGenResult;
use crate::docgen::file_processor;

/// Process all source files and insert AI-generated docs.
pub fn process_all(root: &Path, docs: &[SourceDoc]) -> DocGenResult {
    let mut docgen_state = DocGenResult { generated: 0, skipped: 0, log: Vec::new() };

    for source_doc in docs {
        if source_doc.items.iter().any(|item| item.doc.is_empty()) {
            file_processor::process_file(root, source_doc, &mut docgen_state);
        }
    }

    docgen_state
}
