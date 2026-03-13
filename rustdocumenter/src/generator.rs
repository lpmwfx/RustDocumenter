//! Writes man/ directory: JSON files (viewer) and MD files (AI/human).

use std::fs;
use std::path::Path;

use crate::manifest::{Manifest, ManifestEntry, ManifestItem, SourceDoc};

/// Generate man/ directory under `root` from collected `docs`.
pub fn generate(root: &Path, project_name: &str, docs: &[SourceDoc]) {
    let man_dir = root.join("man");
    fs::create_dir_all(&man_dir).expect("cannot create man/");

    let mut entries: Vec<ManifestEntry> = Vec::new();
    let mut all_items: Vec<ManifestItem> = Vec::new();
    let mut total_items: usize = 0;
    let mut undoc_items: usize = 0;

    for doc in docs {
        // ALL source files — even those with no pub items — get a man/ page
        let rel = Path::new(&doc.source);
        let stem = rel.with_extension("");
        let json_rel = stem.with_extension("json");
        let md_rel   = stem.with_extension("md");

        let json_path = man_dir.join(&json_rel);
        let md_path   = man_dir.join(&md_rel);

        if let Some(parent) = json_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        // Write JSON
        let json = serde_json::to_string_pretty(doc).expect("json serialization");
        fs::write(&json_path, json).expect("write json");

        // Write Markdown
        let md = render_md(doc);
        fs::write(&md_path, md).expect("write md");

        let man_path = format!("man/{}", json_rel.to_string_lossy().replace(r"\\", "/"));

        for item in &doc.items {
            total_items += 1;
            if item.doc.is_empty() {
                undoc_items += 1;
            }
            all_items.push(ManifestItem {
                name:   item.name.clone(),
                kind:   item.kind.label().to_string(),
                source: doc.source.clone(),
                line:   item.line,
                documented: !item.doc.is_empty(),
            });
        }

        entries.push(ManifestEntry {
            source:        doc.source.clone(),
            man_path,
            item_count:    doc.items.len(),
            undoc_count:   doc.items.iter().filter(|i| i.doc.is_empty()).count(),
        });
    }

    // Write MANIFEST.json
    let manifest = Manifest {
        generated:   chrono::Local::now().to_rfc3339(),
        project:     project_name.to_string(),
        files:       entries,
        all_items:   all_items.clone(),
    };
    let manifest_json = serde_json::to_string_pretty(&manifest).expect("manifest json");
    fs::write(man_dir.join("MANIFEST.json"), manifest_json).expect("write MANIFEST.json");

    // Write MANIFEST.md (AI-readable index)
    let manifest_md = render_manifest_md(&manifest, total_items, undoc_items);
    fs::write(man_dir.join("MANIFEST.md"), manifest_md).expect("write MANIFEST.md");

    let doc_count = total_items - undoc_items;
    let pct = if total_items > 0 { doc_count * 100 / total_items } else { 100 };
    if undoc_items > 0 {
        eprintln!(
            "rustdocumenter: {} source files, {}/{} items documented ({}%) — {} undocumented → man/",
            manifest.files.len(), doc_count, total_items, pct, undoc_items
        );
    } else {
        println!(
            "rustdocumenter: {} source files, {} items, 100% documented → man/",
            manifest.files.len(), total_items
        );
    }
}

// ─── Markdown renderers ───────────────────────────────────────────────────────

fn render_md(doc: &SourceDoc) -> String {
    let mut out = String::new();
    out.push_str(&format!("# `{}`\n\n", doc.source));

    if doc.items.is_empty() {
        out.push_str("*No public items found in this file.*\n");
        return out;
    }

    for item in &doc.items {
        out.push_str(&format!("## `{}`\n", item.signature));
        out.push_str(&format!("*Line {} · {}*\n\n", item.line, item.kind.label()));
        if item.doc.is_empty() {
            out.push_str("> ⚠ **undocumented** — add a `///` doc comment\n");
        } else {
            out.push_str(&item.doc);
            out.push('\n');
        }
        out.push_str("\n---\n\n");
    }

    out
}

fn render_manifest_md(manifest: &Manifest, total: usize, undoc: usize) -> String {
    let mut out = String::new();
    let doc = total.saturating_sub(undoc);
    let pct = if total > 0 { doc * 100 / total } else { 100 };

    out.push_str("# Documentation Index\n\n");
    out.push_str(&format!(
        "Generated: {}  \nProject: `{}`  \nCoverage: **{}/{}** items documented (**{}%**)",
        manifest.generated, manifest.project, doc, total, pct
    ));
    if undoc > 0 {
        out.push_str(&format!(" — ⚠ **{} undocumented**", undoc));
    }
    out.push_str("\n\n");

    // File table with coverage column
    out.push_str("## Files\n\n");
    out.push_str("| Source File | Items | Undocumented |\n|---|---|---|\n");
    for entry in &manifest.files {
        let md_path = entry.man_path.replace(".json", ".md");
        let undoc_cell = if entry.undoc_count > 0 {
            format!("⚠ {}", entry.undoc_count)
        } else if entry.item_count > 0 {
            "✓".to_string()
        } else {
            "—".to_string()
        };
        out.push_str(&format!(
            "| [{}]({}) | {} | {} |\n",
            entry.source, md_path, entry.item_count, undoc_cell
        ));
    }

    // All items table with documented column
    out.push_str("\n## All Items\n\n");
    out.push_str("| Item | Kind | Source | Line | Documented |\n|---|---|---|---|---|\n");
    for item in &manifest.all_items {
        let doc_cell = if item.documented { "✓" } else { "⚠" };
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            item.name, item.kind, item.source, item.line, doc_cell
        ));
    }

    out
}
