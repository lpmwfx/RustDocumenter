# RustDocumenter

Documentation system for Rust + Slint projects — like DocC for Swift, but for Rust.

Enforced by `rust/docs/doc-required` in [RustScanners](https://github.com/lpmwfx/RustScanners) and [RulesTools](https://github.com/lpmwfx/RulesTools).

## Quick start

```bash
# Install (both binaries)
cargo install --git https://github.com/lpmwfx/RustDocumenter --bins

# Generate man/ in your project
rustdocumenter gen /path/to/project

# Browse in the viewer
rustdoc-viewer /path/to/project
```

## Binaries

| Binary | Purpose |
|---|---|
| `rustdocumenter gen [PATH]` | Parse `///` comments → `man/` JSON + MD |
| `rustdocumenter check [PATH]` | Verify all pub items have `///`, exit 1 if not |
| `rustdoc-viewer [PATH]` | Slint GUI docs browser |

## Source convention

Add `///` doc comments above every public item:

```rust
/// Parse a CommonMark string into a flat list of DocBlock items.
pub fn parse(input: &str) -> Vec<DocBlock> { ... }

/// Scanner configuration loaded from proj/rulestools.toml.
pub struct Config { ... }
```

For Slint:

```slint
/// Main application window with sidebar navigation and markdown viewer.
export component DocsApp inherits Window { ... }
```

## Output

```
man/
  MANIFEST.md      ← AI-readable index (all items, all files)
  MANIFEST.json    ← viewer index
  src/
    lib.md         ← one MD per source file
    lib.json
    checks/
      mod.md
      magic_numbers.md
```

## Viewer

The `rustdoc-viewer` uses [SlintUITemplates](https://github.com/lpmwfx/SlintUITemplates)
`DocsApp` component — sidebar navigation mirrors the source folder hierarchy,
right pane renders the man/*.md markdown.

## Integration with install.sh

Add to `RulesTools/install.sh` update section:

```bash
cargo install --quiet --git https://github.com/lpmwfx/RustDocumenter --bins
```
