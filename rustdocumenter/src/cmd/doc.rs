//! `rustdocumenter doc` — auto-generate /// doc comments via AI.

use std::path::Path;

/// Auto-generate `///` doc comments for undocumented items via AI.
/// Returns output lines for the caller to print.
pub fn run(root: &Path) -> Vec<String> {
    let docgen_output = rustdocumenter::docgen::document_project(
        root,
        &rustdocumenter::RS_PATTERNS,
        &rustdocumenter::SLINT_PATTERNS,
    );
    let mut lines = docgen_output.log;
    lines.push(format!(
        "\nrustdocumenter doc: {} items documented, {} skipped",
        docgen_output.generated, docgen_output.skipped
    ));
    lines
}
