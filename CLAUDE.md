# RustDocumenter — CLAUDE.md

Documentation system for Rust + Slint projects.

## What this repo contains

| Binary | Purpose |
|---|---|
| `rustdocumenter` | Parse `///` comments → write `man/` (JSON + MD) |
| `rustdoc-viewer` | Slint GUI browser for `man/` documentation |

## Workflow

```bash
rustdocumenter gen <project-root>   # generate man/ in the project
rustdoc-viewer <project-root>       # browse the documentation
```

## man/ file format

One `.md` and one `.json` per source file, plus `MANIFEST.md` and `MANIFEST.json`.

```
man/
  MANIFEST.json    ← machine-readable index (for viewer)
  MANIFEST.md      ← AI-readable master index table
  src/
    lib.md         ← one MD per source file
    lib.json       ← same as JSON
    checks/
      mod.md
```

## Source convention

Use `///` doc comments above all public items in `.rs` and `.slint` files:

```rust
/// Scan the project and return the error count.
pub fn scan_project() -> usize { ... }
```

```slint
/// Main application window with sidebar navigation.
export component DocsApp inherits Window { ... }
```

## Architecture

- `rustdocumenter/src/parser.rs` — extract `///` blocks + pub item signatures
- `rustdocumenter/src/generator.rs` — write man/ JSON + MD
- `rustdocumenter/src/manifest.rs` — serde types
- `rustdoc-viewer/src/main.rs` — load MANIFEST.json, use `slint-ui-templates::DocsApp`

## Scanner enforcement

The `rust/docs/doc-required` rule is enforced by:
- **RustScanners** — `src/checks/doc_comments.rs` — build-time ERROR
- **RulesTools** — `rust/checks/doc_required.py` — `rulestools scan` ERROR

Both reference rule: `rust/docs/doc-required`


---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=20" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
