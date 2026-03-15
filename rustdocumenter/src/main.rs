//! rustdocumenter CLI — extract /// doc comments and generate man/ documentation.
//!
//! Usage:
//!   rustdocumenter gen [PATH]    # parse .rs + .slint → write man/ JSON + MD + warnings
//!   rustdocumenter check [PATH]  # verify all pub items are documented, exit 1 if not
//!   rustdocumenter doc [PATH]    # auto-generate /// for undocumented items via AI

mod cmd;

use std::env;
use std::path::PathBuf;

/// CLI subcommand.
enum Cmd {
    Gen,
    Check,
    Doc,
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
    }
}

fn parse_args(args: &[String]) -> (Cmd, PathBuf) {
    let cmd = match args.get(1).map(|s| s.as_str()) {
        Some("gen") | None => Cmd::Gen,
        Some("check")      => Cmd::Check,
        Some("doc")        => Cmd::Doc,
        Some(_) => {
            eprintln!("Usage: rustdocumenter <gen|check|doc> [PATH]");
            std::process::exit(1);
        }
    };
    let path = args
        .get(PATH_ARG_INDEX)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    (cmd, path)
}
