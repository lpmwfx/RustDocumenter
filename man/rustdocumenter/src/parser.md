# `rustdocumenter/src/parser.rs`

## `pub fn parse_rs(path: &Path, content: &str) -> Vec<DocItem>`
*Line 65 · fn*

Extract all public items from a `.rs` file.
Items without `///` above them are included with `doc = ""`.

---

## `pub fn parse_slint(path: &Path, content: &str) -> Vec<DocItem>`
*Line 120 · fn*

Extract all exported items from a `.slint` file.
Items without `///` above them are included with `doc = ""`.

---

