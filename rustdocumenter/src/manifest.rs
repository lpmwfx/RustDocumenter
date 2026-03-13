//! Shared types for man/ file format.

use serde::{Deserialize, Serialize};

/// Kind of documented item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Fn,
    Struct,
    Enum,
    Trait,
    Type,
    Mod,
    Const,
    Component, // Slint export component
    Property,  // Slint in/out property
    Callback,  // Slint callback
}

impl ItemKind {
    pub fn label(&self) -> &'static str {
        match self {
            ItemKind::Fn        => "fn",
            ItemKind::Struct    => "struct",
            ItemKind::Enum      => "enum",
            ItemKind::Trait     => "trait",
            ItemKind::Type      => "type",
            ItemKind::Mod       => "mod",
            ItemKind::Const     => "const",
            ItemKind::Component => "component",
            ItemKind::Property  => "property",
            ItemKind::Callback  => "callback",
        }
    }
}

/// A single public item extracted from source.
/// `doc` is empty if no `///` comment was found above the item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    pub name: String,
    pub kind: ItemKind,
    /// Full public signature, possibly multi-line joined with space.
    pub signature: String,
    pub line: usize,
    /// Joined `///` comment lines. Empty string = undocumented.
    pub doc: String,
}

/// All items from a single source file (documented + undocumented).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDoc {
    /// Relative path from project root, e.g. "src/lib.rs"
    pub source: String,
    pub items: Vec<DocItem>,
}

/// Summary entry in MANIFEST.json — one row per source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub source: String,
    pub man_path: String,
    pub item_count: usize,
    /// Number of items without a `///` doc comment.
    pub undoc_count: usize,
}

/// Summary item entry for MANIFEST all_items table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestItem {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub line: usize,
    pub documented: bool,
}

/// Top-level MANIFEST.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub generated: String,
    pub project: String,
    pub files: Vec<ManifestEntry>,
    pub all_items: Vec<ManifestItem>,
}
