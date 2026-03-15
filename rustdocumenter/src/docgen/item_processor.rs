//! Generates AI doc comments and rewrites file content for one source file.

use crate::ai;
use crate::manifest::SourceDoc;
use crate::parser;

use crate::docgen::DocGenResult;

/// Generate docs for all undocumented items, log results, return count generated.
pub fn generate_all(content: &str, source_doc: &SourceDoc, docgen_state: &mut DocGenResult) -> usize {
    let mut count = 0;
    let mut items_sorted: Vec<_> = source_doc.items.iter().filter(|i| i.doc.is_empty()).collect();
    items_sorted.sort_by(|a, b| b.line.cmp(&a.line));

    for item in &items_sorted {
        let body = parser::extract_body(content, item.line);
        match ai::generate_doc(&item.signature, &body, item.kind.label(), &item.name) {
            Ok(_) => {
                count += 1;
                docgen_state.log.push(format!("  {} {}::{}", item.kind.label(), source_doc.source, item.name));
            }
            Err(e) => {
                docgen_state.log.push(format!("  skip {}: {e}", item.name));
                docgen_state.skipped += 1;
            }
        }
    }
    count
}

/// Rewrite file content by inserting AI-generated `///` lines above each undocumented item.
pub fn rewrite(content: &str, source_doc: &SourceDoc, _docgen_state: &mut DocGenResult) -> String {
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let mut items_sorted: Vec<_> = source_doc.items.iter().filter(|i| i.doc.is_empty()).collect();
    items_sorted.sort_by(|a, b| b.line.cmp(&a.line));

    for item in &items_sorted {
        let body = parser::extract_body(content, item.line);
        if let Ok(doc_text) = ai::generate_doc(&item.signature, &body, item.kind.label(), &item.name) {
            insert_doc(&mut lines, item.line, &doc_text);
        }
    }

    let joined = lines.join("\n");
    if content.ends_with('\n') && !joined.ends_with('\n') {
        format!("{joined}\n")
    } else {
        joined
    }
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
