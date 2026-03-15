//! IO gateway — all filesystem and process operations go through here.
//!
//! Library code must not perform IO directly. This module is the single
//! point of contact for `std::fs`, `std::process`, and `walkdir`.

mod ai_runners;
mod fs;

pub use ai_runners::{run_claude, run_codex};
pub use fs::{collect_docs, read_file, write_file};
