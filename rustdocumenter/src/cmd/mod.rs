//! CLI subcommand implementations.

/// Verify all pub items are documented.
pub mod check;
/// Verify AI backend availability.
pub mod diag;
/// Auto-generate `///` doc comments via AI.
pub mod doc;
/// Generate `man/` documentation output.
pub mod gen;
