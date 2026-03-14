//! rustdocumenter CLI — extract /// doc comments and generate man/ documentation.
//!
//! Usage:
//!   rustdocumenter gen [PATH]    # parse .rs + .slint → write man/ JSON + MD + warnings
//!   rustdocumenter check [PATH]  # verify all pub items are documented, exit 1 if not

use rustdocumenter::*;
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (cmd, root) = parse_args(&args);

    match cmd.as_str() {
        "gen" => scan_project_at(&root),
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

fn cmd_check(root: &std::path::Path) -> ! {
    let docs = crate::collect_docs(root);
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
