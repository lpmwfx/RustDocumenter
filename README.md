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

## Usage workflow

### 1. Install

```bash
# Install both binaries (to ~/.cargo/bin/)
cargo install --git https://github.com/lpmwfx/RustDocumenter --bins

# Verify installation
rustdocumenter --help
rustdoc-viewer --help
```

### 2. Generate documentation

In your project root:

```bash
# Generate man/ directory from all /// comments in src/
rustdocumenter gen

# Or specify a project path
rustdocumenter gen /path/to/project
```

This creates:
- `man/MANIFEST.json` — machine-readable index (for the viewer)
- `man/MANIFEST.md` — AI-readable overview of all documented items
- `man/src/lib.json` + `man/src/lib.md` — one pair per source file
- `proj/ISSUES` — list of all undocumented public items

Example output:
```
Parsing 12 source files...
Parsed 156 public items
Generated man/MANIFEST.{json,md}
Coverage: 148/156 (94.9%)
```

### 3. Browse documentation

```bash
# Open the Slint viewer
rustdoc-viewer /path/to/project

# Or use the rustman wrapper to auto-discover
cd /path/to/project
rustman

# Or generate and immediately browse
rustman gen
```

The viewer shows:
- Left sidebar: project folder structure
- Right panel: item details with markdown-rendered docs
- Search: find items by name or description

### 4. Enforce with scanners

Build-time check via RustScanners (Rust projects):
```rust
// build.rs
fn main() {
    if rustscanners::scan_project() > 0 {
        std::process::exit(1);
    }
}
```

Or at CI time via RulesTools:
```bash
rulestools scan /path/to/project
# Exits 1 if any pub items lack /// docs
```

## Wrapper scripts (rustman / slintman)

Both binaries are installed to `~/.cargo/bin/` and can be invoked directly from your shell.
Additionally, `rustman.bat` and `slintman.bat` are included in the workspace `/scripts/` folder for Windows users.

```bash
# Auto-discover man/ by walking up directory tree
rustman              # find nearest man/MANIFEST.json and open viewer

# Explicit path
rustman /path/to/project

# Generate and browse
rustman gen          # generate man/ in CWD, then open viewer

# Check coverage
rustman check        # run coverage scan, print results

# Alias for Slint projects
slintman             # delegates to rustman
```

## Practical example

```bash
# In a Rust project with pub items to document
$ cd my-rust-project
$ rustman check
Error: 17 items lack /// doc comments (11%)
  src/lib.rs:42  pub fn parse()
  src/config.rs:18  pub struct Config
  ...

$ # Edit files to add /// comments above each pub item

$ rustman gen
Generated 156 items (100%)

$ rustman
# Opens viewer in browser
```

## Integration with install.sh

Update `RulesTools/install.sh` to install both binaries:

```bash
# rustdocumenter and rustdoc-viewer must use separate cargo install calls
# because they are distinct binary crates in the same repo
cargo install --quiet --force --git https://github.com/lpmwfx/RustDocumenter rustdocumenter
cargo install --quiet --force --git https://github.com/lpmwfx/RustDocumenter rustdoc-viewer
```

Note: Use `--force` flag to ensure clean reinstalls, since both crates build separate binaries.


---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=20" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
