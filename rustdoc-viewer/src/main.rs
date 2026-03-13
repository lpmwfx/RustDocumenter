//! rustdoc-viewer — Slint GUI for browsing man/ documentation.
//!
//! Usage:
//!   rustdoc-viewer [PATH]
//!
//! PATH can be:
//!   - a project root (looks for man/MANIFEST.json)
//!   - a man/ directory directly
//!
//! Navigation sidebar mirrors the source code folder hierarchy.
//! Click a file to load its man/*.md into the markdown viewer.

use std::path::{Path, PathBuf};
use std::{env, fs};

use slint::ComponentHandle;
use slint_ui_templates::{docs, dsl::BgStyle, platform, DocBlock, DocsApp, NavItem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = env::args().nth(1).map(PathBuf::from);
    let man_dir = resolve_man_dir(arg.as_deref())?;

    let pages = collect_pages(&man_dir);

    if pages.is_empty() {
        eprintln!(
            "rustdoc-viewer: no man/*.md files found in {}",
            man_dir.display()
        );
        eprintln!("  Run `rustdocumenter gen <project-root>` first.");
        std::process::exit(1);
    }

    let project_name = resolve_project_name(&man_dir);

    // ── Build UI ──────────────────────────────────────────────────────────────
    let ui = DocsApp::new()?;
    ui.set_doc_title(project_name.clone().into());

    // Build nav items sorted by folder hierarchy (path order)
    let nav: Vec<NavItem> = pages
        .iter()
        .map(|(id, label, _path)| NavItem {
            id:    id.clone().into(),
            label: label.clone().into(),
            icon:  "".into(),
        })
        .collect();
    ui.set_nav_items(slint::ModelRc::new(slint::VecModel::from(nav)));

    // Load first page
    if let Some((first_id, _, _)) = pages.first() {
        push_page(&ui, first_id, &pages);
        ui.set_active_view(first_id.clone().into());
    }

    // Navigate callback
    let pages_clone = pages.clone();
    let weak = ui.as_weak();
    ui.on_navigate(move |id| {
        if let Some(h) = weak.upgrade() {
            push_page(&h, id.as_str(), &pages_clone);
        }
    });

    // Theme toggle
    let weak = ui.as_weak();
    ui.on_request_bg_style(move |style| {
        if let Some(h) = weak.upgrade() {
            let bg = match style.as_str() {
                "mica"    => BgStyle::Mica,
                "acrylic" => BgStyle::Acrylic,
                _         => BgStyle::Solid,
            };
            platform::apply_backdrop(h.window(), bg);
        }
    });

    ui.show()?;
    platform::apply_backdrop(ui.window(), BgStyle::Solid);
    ui.run()?;
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// (id, sidebar-label, md-path) sorted by path so folder hierarchy is visible.
type PageList = Vec<(String, String, PathBuf)>;

fn collect_pages(man_dir: &Path) -> PageList {
    let mut pages: PageList = Vec::new();

    // Walk man/ recursively for .md files, skip MANIFEST.md
    collect_md(man_dir, man_dir, &mut pages);
    pages.sort_by(|a, b| a.0.cmp(&b.0));
    pages
}

fn collect_md(base: &Path, dir: &Path, out: &mut PageList) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_md(base, &path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "MANIFEST.md" {
                continue;
            }
            // id = relative path from man/, e.g. "src/lib"
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .with_extension("");
            let id = rel.to_string_lossy().replace('\\', "/");

            // label = same but with folder indent markers for visual hierarchy
            let label = make_label(&id);

            out.push((id, label, path));
        }
    }
}

/// Add indent based on folder depth.
/// "src/checks/magic_numbers" → "  checks/magic_numbers"
fn make_label(id: &str) -> String {
    let parts: Vec<&str> = id.split('/').collect();
    if parts.len() <= 1 {
        return id.to_string();
    }
    // Show last two segments, indent by depth
    let depth = parts.len().saturating_sub(2);
    let indent = "  ".repeat(depth);
    let display = parts[parts.len().saturating_sub(2)..].join("/");
    format!("{}{}", indent, display)
}

fn push_page(ui: &DocsApp, id: &str, pages: &PageList) {
    let blocks: Vec<DocBlock> = if let Some((_, _, md_path)) = pages.iter().find(|(i, ..)| i == id)
    {
        match fs::read_to_string(md_path) {
            Ok(md) => docs::parse(&md),
            Err(e) => docs::parse(&format!(
                "# Error\n\nCould not read `{}`:\n\n```\n{e}\n```\n",
                md_path.display()
            )),
        }
    } else {
        docs::parse(&format!("# {id}\n\nNo documentation found.\n"))
    };

    ui.set_doc_blocks(slint::ModelRc::new(slint::VecModel::from(blocks)));
    ui.set_doc_title(id.into());
    ui.set_status_text(format!("man/{id}.md").into());
}

fn resolve_man_dir(arg: Option<&Path>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base = arg.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        env::current_dir().expect("no cwd")
    });

    // If arg is already a man/ dir
    if base.join("MANIFEST.md").exists() || base.file_name().map_or(false, |n| n == "man") {
        return Ok(base);
    }

    // Try base/man/
    let man = base.join("man");
    if man.is_dir() {
        return Ok(man);
    }

    Err(format!(
        "Cannot find man/ directory in {}\nRun `rustdocumenter gen .` first.",
        base.display()
    )
    .into())
}

fn resolve_project_name(man_dir: &Path) -> String {
    // man_dir is <project>/man — go up one level
    man_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}
