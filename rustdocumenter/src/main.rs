//! rustdocumenter CLI — extract /// doc comments and generate man/ documentation.
//!
//! Usage:
//!   rustdocumenter [PATH]        # (default) auto-generate /// for undocumented items via AI
//!   rustdocumenter doc [PATH]    # same as default
//!   rustdocumenter gen [PATH]    # parse .rs + .slint → write man/ JSON + MD + warnings
//!   rustdocumenter check [PATH]  # verify all pub items are documented, exit 1 if not
//!   rustdocumenter diag          # verify AI backend (Claude/Codex) is available

mod cmd;

use std::env;
use std::path::PathBuf;

/// CLI subcommand.
enum Cmd {
    Gen,
    Check,
    Doc,
    Diag,
}

const PATH_ARG_INDEX: usize = 2;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (cmd, root) = parse_args(&args);

    match cmd {
        Cmd::Gen => {
            for line in cmd::gen::run(&root) { eprintln!("{line}"); }
        }
        Cmd::Check => {
            let (code, lines) = cmd::check::run(&root);
            for line in &lines {
                if code == 0 { println!("{line}"); } else { eprintln!("{line}"); }
            }
            std::process::exit(code);
        }
        Cmd::Doc => {
            for line in cmd::doc::run(&root) { println!("{line}"); }
        }
        Cmd::Diag => {
            let (code, lines) = cmd::diag::run();
            for line in &lines { println!("{line}"); }
            std::process::exit(code);
        }
    }
}

fn parse_args(args: &[String]) -> (Cmd, PathBuf) {
    let cmd = match args.get(1).map(|s| s.as_str()) {
        Some("gen")        => Cmd::Gen,
        None               => Cmd::Doc,
        Some("check")      => Cmd::Check,
        Some("doc")        => Cmd::Doc,
        Some("diag")       => Cmd::Diag,
        Some(_) => {
            eprintln!("Usage: rustdocumenter <gen|check|doc|diag> [PATH]");
            std::process::exit(1);
        }
    };
    let path = args
        .get(PATH_ARG_INDEX)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    (cmd, path)
}
