//! `rustdocumenter check` — verify all pub items are documented.

use std::path::Path;

/// Check that all public items have `///` doc comments.
/// Returns (exit_code, output_lines).
pub fn run(root: &Path) -> (i32, Vec<String>) {
    let docs = rustdocumenter::collect_docs(root);
    let mut lines = Vec::new();
    let mut missing = 0;

    for doc in &docs {
        for item in &doc.items {
            if item.doc.is_empty() {
                lines.push(format!(
                    "{}:{}: error rust/docs/doc-required: pub item `{}` has empty doc comment",
                    doc.source, item.line, item.name
                ));
                missing += 1;
            }
        }
    }

    if missing > 0 {
        lines.push(format!("rustdocumenter: {} items with missing docs", missing));
        (1, lines)
    } else {
        lines.push("rustdocumenter: all pub items documented".to_string());
        (0, lines)
    }
}
