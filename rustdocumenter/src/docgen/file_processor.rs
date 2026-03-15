//! Per-file doc insertion: generates and writes `///` comments for one file.

use std::path::Path;

use crate::gateway;
use crate::manifest::SourceDoc;

use crate::docgen::DocGenResult;
use crate::docgen::item_processor;

/// Generate and insert `///` docs for all undocumented items in one source file.
pub fn process_file(root: &Path, source_doc: &SourceDoc, docgen_state: &mut DocGenResult) {
    let file_path = root.join(&source_doc.source);
    let content = match gateway::read_file(&file_path) {
        Ok(c) => c,
        Err(e) => {
            docgen_state.log.push(format!("  skip {}: {e}", source_doc.source));
            return;
        }
    };

    let (rewritten, count) = item_processor::rewrite(&content, source_doc, docgen_state);

    if count > 0 {
        match gateway::write_file(&file_path, &rewritten) {
            Ok(()) => docgen_state.generated += count,
            Err(e) => docgen_state.log.push(format!("  error writing {}: {e}", source_doc.source)),
        }
    }
}
