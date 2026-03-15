//! `rustdocumenter gen` — generate man/ documentation.

use std::path::Path;

/// Generate man/ documentation and return warning lines for missing docs.
pub fn run(root: &Path) -> Vec<String> {
    let docs = rustdocumenter::collect_docs(root);
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    rustdocumenter::generator::generate(root, project_name, &docs);

    let mut lines = Vec::new();
    let mut missing_count = 0;
    for doc in &docs {
        for item in &doc.items {
            if item.doc.is_empty() {
                lines.push(format!(
                    "{}:{}:1: warning rust/docs/doc-required: missing /// doc comment on {} `{}`",
                    doc.source, item.line, item.kind.label(), item.name
                ));
                missing_count += 1;
            }
        }
    }
    if missing_count > 0 {
        lines.push(format!("rustdocumenter: {} items missing /// doc comments", missing_count));
    }
    lines
}
