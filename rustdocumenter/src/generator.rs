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

    for doc in docs {
        if doc.items.is_empty() {
            continue;
        }

        // Derive man/ path: "src/lib.rs" → "man/src/lib.json"
        let rel = Path::new(&doc.source);
        let stem = rel.with_extension("");
        let json_rel = stem.with_extension("json");
        let md_rel = stem.with_extension("md");

        let json_path = man_dir.join(&json_rel);
        let md_path = man_dir.join(&md_rel);

        if let Some(parent) = json_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        // Write JSON
        let json = serde_json::to_string_pretty(doc).expect("json serialization");
        fs::write(&json_path, json).expect("write json");

        // Write Markdown
        let md = render_md(doc);
        fs::write(&md_path, md).expect("write md");

        let man_path = format!("man/{}", json_rel.display());
        for item in &doc.items {
            all_items.push(ManifestItem {
                name: item.name.clone(),
                kind: item.kind.label().to_string(),
                source: doc.source.clone(),
                line: item.line,
            });
        }

        entries.push(ManifestEntry {
            source: doc.source.clone(),
            man_path,
            item_count: doc.items.len(),
        });
    }

    // Write MANIFEST.json
    let manifest = Manifest {
        generated: chrono::Local::now().to_rfc3339(),
        project: project_name.to_string(),
        files: entries,
        all_items: all_items.clone(),
    };
    let manifest_json = serde_json::to_string_pretty(&manifest).expect("manifest json");
    fs::write(man_dir.join("MANIFEST.json"), manifest_json).expect("write MANIFEST.json");

    // Write MANIFEST.md (AI-readable index)
    let manifest_md = render_manifest_md(&manifest);
    fs::write(man_dir.join("MANIFEST.md"), manifest_md).expect("write MANIFEST.md");

    let total_items: usize = manifest.files.iter().map(|e| e.item_count).sum();
    println!(
        "rustdocumenter: {} source files, {} documented items → man/",
        manifest.files.len(),
        total_items
    );
}

// ─── Markdown renderers ───────────────────────────────────────────────────────

fn render_md(doc: &SourceDoc) -> String {
    let mut out = String::new();
    out.push_str(&format!("# `{}`\n\n", doc.source));

    for item in &doc.items {
        out.push_str(&format!("## `{}`\n", item.signature));
        out.push_str(&format!("*Line {} · {}*\n\n", item.line, item.kind.label()));
        if !item.doc.is_empty() {
            out.push_str(&item.doc);
            out.push('\n');
        }
        out.push_str("\n---\n\n");
    }

    out
}

fn render_manifest_md(manifest: &Manifest) -> String {
    let mut out = String::new();
    out.push_str("# Documentation Index\n\n");
    out.push_str(&format!(
        "Generated: {}  \nProject: `{}`\n\n",
        manifest.generated, manifest.project
    ));

    // File table
    out.push_str("## Files\n\n");
    out.push_str("| Source File | Items |\n|---|---|\n");
    for entry in &manifest.files {
        let md_path = entry.man_path.replace(".json", ".md");
        out.push_str(&format!(
            "| [{}]({}) | {} |\n",
            entry.source, md_path, entry.item_count
        ));
    }

    // All items table
    out.push_str("\n## All Items\n\n");
    out.push_str("| Item | Kind | Source | Line |\n|---|---|---|---|\n");
    for item in &manifest.all_items {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            item.name, item.kind, item.source, item.line
        ));
    }

    out
}
