//! Generates AI doc comments and rewrites file content for one source file.

use crate::ai;
use crate::manifest::SourceDoc;
use crate::parser;

use crate::docgen::DocGenResult;

/// Generate AI docs and insert `///` lines for all undocumented items in one pass.
///
/// Returns the rewritten file content and the number of items documented.
/// Each item triggers a single AI call — result is logged and inserted immediately.
pub fn rewrite(content: &str, source_doc: &SourceDoc, docgen_state: &mut DocGenResult) -> (String, usize) {
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut count = 0;

    let mut items_sorted: Vec<_> = source_doc.items.iter().filter(|i| i.doc.is_empty()).collect();
    items_sorted.sort_by(|a, b| b.line.cmp(&a.line));

    for item in &items_sorted {
        let body = parser::extract_body(content, item.line);
        match ai::generate_doc(&item.signature, &body, item.kind.label(), &item.name) {
            Ok(doc_text) => {
                insert_doc(&mut lines, item.line, &doc_text);
                docgen_state.log.push(format!("  {} {}::{}", item.kind.label(), source_doc.source, item.name));
                count += 1;
            }
            Err(e) => {
                docgen_state.log.push(format!("  skip {}: {e}", item.name));
                docgen_state.skipped += 1;
            }
        }
    }

    let joined = lines.join("\n");
    let rewritten = if content.ends_with('\n') && !joined.ends_with('\n') {
        format!("{joined}\n")
    } else {
        joined
    };
    (rewritten, count)
}

/// Insert `///` comment lines above `item_line` (1-based) in `lines`.
fn insert_doc(lines: &mut Vec<String>, item_line: usize, doc_text: &str) {
    let item_idx = item_line - 1;
    let indent = if item_idx < lines.len() {
        let ln = &lines[item_idx];
        &ln[..ln.len() - ln.trim_start().len()]
    } else {
        ""
    };
    let doc_lines: Vec<String> = doc_text
        .lines()
        .map(|l| if l.is_empty() { format!("{indent}///") } else { format!("{indent}/// {l}") })
        .collect();
    for (i, doc_line) in doc_lines.iter().enumerate() {
        lines.insert(item_idx + i, doc_line.clone());
    }
}
