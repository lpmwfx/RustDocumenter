# `rustdocumenter/src/manifest.rs`

## `pub enum ItemKind`
*Line 8 · enum*

Kind of documented item.

---

## `pub fn label(&self) -> &'static str`
*Line 22 · fn*

> ⚠ **undocumented** — add a `///` doc comment

---

## `pub struct DocItem`
*Line 41 · struct*

A single public item extracted from source.
`doc` is empty if no `///` comment was found above the item.

---

## `pub struct SourceDoc`
*Line 53 · struct*

All items from a single source file (documented + undocumented).

---

## `pub struct ManifestEntry`
*Line 61 · struct*

Summary entry in MANIFEST.json — one row per source file.

---

## `pub struct ManifestItem`
*Line 71 · struct*

Summary item entry for MANIFEST all_items table.

---

## `pub struct Manifest`
*Line 81 · struct*

Top-level MANIFEST.json.

---

