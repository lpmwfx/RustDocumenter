//! rustdocumenter — extract /// doc comments and generate man/ documentation.
//!
//! Usage:
//!   rustdocumenter gen [PATH]    # parse .rs + .slint → write man/ JSON + MD
//!   rustdocumenter check [PATH]  # verify all pub items are documented, exit 1 if not

mod manifest;
mod parser;
mod generator;

use std::env;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use manifest::SourceDoc;

const SKIP_DIRS: &[&str] = &["target", ".git", ".cargo", "man"];

fn main() {
    let args: Vec<String> = env::args().collect();
    let (cmd, root) = parse_args(&args);

    match cmd.as_str() {
        "gen" => cmd_gen(&root),
        "check" => cmd_check(&root),
        _ => {
            eprintln!("Usage: rustdocumenter <gen|check> [PATH]");
            std::process::exit(1);
        }
    }
}

fn parse_args(args: &[String]) -> (String, PathBuf) {
    let cmd = args.get(1).cloned().unwrap_or_else(|| "gen".to_string());
    let path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
        env::current_dir().expect("no cwd")
    });
    (cmd, path)
}

// ─── Commands ─────────────────────────────────────────────────────────────────

fn cmd_gen(root: &Path) {
    let docs = collect_docs(root);
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    generator::generate(root, project_name, &docs);
}

fn cmd_check(root: &Path) -> ! {
    let docs = collect_docs(root);
    let mut missing = 0;

    for doc in &docs {
        for item in &doc.items {
            if item.doc.is_empty() {
                eprintln!(
                    "{}:{}: error rust/docs/doc-required: pub item `{}` has empty doc comment",
                    doc.source, item.line, item.name
                );
                missing += 1;
            }
        }
    }

    if missing > 0 {
        eprintln!("rustdocumenter: {} items with missing docs", missing);
        std::process::exit(1);
    } else {
        println!("rustdocumenter: all pub items documented");
        std::process::exit(0);
    }
}

// ─── Collection ───────────────────────────────────────────────────────────────

fn collect_docs(root: &Path) -> Vec<SourceDoc> {
    let mut docs = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip ignored directories
        if path.components().any(|c| {
            SKIP_DIRS.contains(&c.as_os_str().to_str().unwrap_or(""))
        }) {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let items = match ext {
            "rs" => {
                let content = std::fs::read_to_string(path).unwrap_or_default();
                parser::parse_rs(path, &content)
            }
            "slint" => {
                let content = std::fs::read_to_string(path).unwrap_or_default();
                parser::parse_slint(path, &content)
            }
            _ => continue,
        };

        let source = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        docs.push(SourceDoc { source, items });
    }

    docs
}
