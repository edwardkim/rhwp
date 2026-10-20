//! Types for PDF logical structure trees.
//!
//! Implements structure element types according to ISO 32000-1:2008 Section 14.7.2.

use crate::object::{Object, ObjectRef};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Source of a Marked Content Identifier (MCID).
///
/// Per ISO 32000-1:2008 §14.7.4.3, MCIDs are NOT page-global. Each
/// content stream that can carry `/StructParents` defines its own MCID
/// namespace:
///
/// - A **page** content stream (`Page::StructParents`).
/// - A **Form XObject** content stream (`/Subtype /Form`,
///   `Form::StructParents`).
/// - A **Tiling Pattern** content stream (`/PatternType 1`,
///   `Pattern::StructParents`).
///
/// Two Form XObjects rendered on the same page may both emit MCID `0`
/// inside their respective streams; they refer to different structure
/// elements and must NOT collapse onto a single key. The same applies
/// to two Tiling Patterns. This enum is the namespace-discriminator that
/// keeps them apart.
///
/// `ActualTextIndex` and `TextSpan::mcid_scope` use this enum as the
/// scope half of their `(scope, mcid)` lookup key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub enum McidScope {
    /// MCID drawn directly by the page content stream. The `u32` is the
    /// 0-based page index.
    Page(u32),
    /// MCID drawn by a Form XObject content stream. The `ObjectRef`
    /// identifies the form (so two distinct forms on one page stay
    /// distinct).
    Form(ObjectRef),
    /// MCID drawn by a Tiling Pattern content stream. The `ObjectRef`
    /// identifies the pattern (so two distinct patterns on one page
    /// stay distinct).
    Pattern(ObjectRef),
}

impl McidScope {
    /// Returns the page index when the scope is `Page(p)`, otherwise
    /// `None`. Useful for assemblers that want a fast-path on the
    /// common case.
    pub fn page(&self) -> Option<u32> {
        match self {
            McidScope::Page(p) => Some(*p),
            _ => None,
        }
    }
}

/// Bare marked-content ids that exactly one content stream numbers.
///
/// ISO 32000-1:2008 §14.7.4.2 scopes an `/MCID` to its content stream, and
/// §14.7.4.3 gives a marked-content reference an `/Stm` entry naming that
/// stream. A structure element whose kid is a bare integer carries no `/Stm`,
/// so it resolves against the page's own stream — yet a producer that draws
/// part of a page through a Form XObject routinely numbers one continuous id
/// space across the page stream and the form and references all of it that
/// way. The reference's scope and the glyphs' scope then disagree although
/// nothing collides.
///
/// An id that only one stream numbers has exactly one possible referent, so
/// matching it across scopes cannot pick the wrong marked content. An id two
/// streams both number is a real collision: there the scope is the only thing
/// that tells them apart and it must be honoured.
pub(crate) fn unambiguous_mcid_scopes<'k>(
    keys: impl Iterator<Item = &'k (McidScope, u32)>,
) -> HashMap<u32, McidScope> {
    let mut sole: HashMap<u32, Option<McidScope>> = HashMap::new();
    for (scope, mcid) in keys {
        match sole.get(mcid) {
            None => {
                sole.insert(*mcid, Some(scope.clone()));
            },
            Some(Some(seen)) if seen != scope => {
                sole.insert(*mcid, None);
            },
            _ => {},
        }
    }
    sole.into_iter()
        .filter_map(|(mcid, scope)| scope.map(|s| (mcid, s)))
        .collect()
}

/// The key under which `map` holds what `key` names: `key` itself when the map
/// has it, otherwise the sole holder of the same bare id.
///
/// The fallback fires only when the id is unambiguous on **both** sides of the
/// match — `unambiguous` describes the map, and the caller must have
/// established that its own side numbers the id once too. See
/// [`unambiguous_mcid_scopes`].
pub(crate) fn resolve_mcid_key<V>(
    map: &HashMap<(McidScope, u32), V>,
    unambiguous: &HashMap<u32, McidScope>,
    key: &(McidScope, u32),
) -> Option<(McidScope, u32)> {
    if map.contains_key(key) {
        return Some(key.clone());
    }
    let scope = unambiguous.get(&key.1)?;
    let alt = (scope.clone(), key.1);
    map.contains_key(&alt).then_some(alt)
}

/// The root of a PDF structure tree (StructTreeRoot dictionary).
///
/// This is the entry point for accessing a document's logical structure.
/// According to PDF spec Section 14.7.2, the StructTreeRoot contains:
/// - `/Type` - Must be `/StructTreeRoot`
/// - `/K` - The immediate child or children of the structure tree root
/// - `/ParentTree` - Maps marked content to structure elements
/// - `/RoleMap` - Maps non-standard structure types to standard ones
#[derive(Debug, Clone)]
pub struct StructTreeRoot {
    /// Root structure element(s)
    pub root_elements: Vec<StructElem>,

    /// Parent tree mapping MCIDs to structure elements (optional)
    pub parent_tree: Option<ParentTree>,

    /// Role map for custom structure types (optional)
    pub role_map: HashMap<String, String>,
}

impl StructTreeRoot {
    /// Create a new structure tree root
    pub fn new() -> Self {
        Self {
            root_elements: Vec::new(),
            parent_tree: None,
            role_map: HashMap::new(),
        }
    }

    /// Add a root element to the structure tree
    pub fn add_root_element(&mut self, elem: StructElem) {
        self.root_elements.push(elem);
    }
}

impl Default for StructTreeRoot {
    fn default() -> Self {
        Self::new()
    }
}

/// A structure element (StructElem) in the structure tree.
///
/// According to PDF spec Section 14.7.2, each StructElem has:
/// - `/S` - Structure type (e.g., /Document, /P, /H1, /Sect)
/// - `/K` - Children (structure elements or marked content references)
/// - `/P` - Parent structure element
/// - `/Pg` - Page containing this element (optional)
/// - `/A` - Attributes (optional)
/// - `/Alt` - Alternate description (optional, per Section 14.9.3)
/// - `/E` - Expansion for abbreviations (optional, per Section 14.9.5)
#[derive(Debug, Clone)]
pub struct StructElem {
    /// Structure type (e.g., "Document", "P", "H1", "Sect")
    pub struct_type: StructType,

    /// Child elements (structure elements or content references)
    pub children: Vec<StructChild>,

    /// Page number this element appears on (if known)
    pub page: Option<u32>,

    /// Attributes (optional)
    pub attributes: HashMap<String, Object>,

    /// Alternate description for accessibility (optional)
    /// Per PDF spec Section 14.9.3, this provides a human-readable
    /// description of the element's content (e.g., formula LaTeX or description)
    pub alt_text: Option<String>,

    /// Expansion for abbreviations (optional)
    /// Per PDF spec Section 14.9.5, the /E entry provides the expanded form
    /// of an abbreviation or acronym (e.g., "PDF" -> "Portable Document Format")
    pub expansion: Option<String>,

    /// Actual text replacement (optional)
    /// Per PDF spec Section 14.9.4, /ActualText provides exact replacement text
    /// for structure elements, overriding all descendant content.
    pub actual_text: Option<String>,

    /// Original role name before RoleMap resolution (optional).
    /// When a custom structure type (e.g., "Heading1") is mapped to a standard
    /// type (e.g., "H1") via the RoleMap, this field preserves the original name.
    /// None when the element's /S value is already a standard type.
    pub source_role: Option<String>,
}

impl StructElem {
    /// Create a new structure element
    pub fn new(struct_type: StructType) -> Self {
        Self {
            struct_type,
            children: Vec::new(),
            page: None,
            attributes: HashMap::new(),
            alt_text: None,
            expansion: None,
            actual_text: None,
            source_role: None,
        }
    }

    /// Add a child to this structure element
    pub fn add_child(&mut self, child: StructChild) {
        self.children.push(child);
    }
}

/// Child of a structure element (either another struct elem or marked content reference)
#[derive(Debug, Clone)]
pub enum StructChild {
    /// Another structure element (recursive hierarchy)
    StructElem(Box<StructElem>),

    /// Reference to marked content by MCID (Marked Content ID)
    MarkedContentRef {
        /// Marked Content ID
        mcid: u32,
        /// Page number containing this marked content
        page: u32,
        /// Content-stream scope this MCID belongs to (ISO 32000-1:2008
        /// §14.7.4.3). `Page(_)` for MCIDs drawn directly by the page
        /// content stream (the default when an MCR omits `/Stm`).
        /// `Form(_)` / `Pattern(_)` when the MCR's `/Stm` resolves to
        /// a Form XObject or Tiling Pattern stream.
        scope: McidScope,
    },

    /// Object reference (indirect reference to another StructElem)
    ObjectRef(u32, u16), // (object_num, generation)
}

/// Standard structure types from PDF spec Section 14.8.4.
///
/// These are the standard structure types defined by the PDF specification.
/// Custom types can be mapped to standard types via the RoleMap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructType {
    // Document-level structure types
    /// Document root
    Document,
    /// Part (major division)
    Part,
    /// Article
    Art,
    /// Section
    Sect,
    /// Division
    Div,

    // Paragraph-level structure types
    /// Paragraph
    P,
    /// Heading level 1-6
    H,
    /// Heading level 1
    H1,
    /// Heading level 2
    H2,
    /// Heading level 3
    H3,
    /// Heading level 4
    H4,
    /// Heading level 5
    H5,
    /// Heading level 6
    H6,

    // List structure types
    /// List
    L,
    /// List item
    LI,
    /// Label (list item marker)
    Lbl,
    /// List body (list item content)
    LBody,

    // Table structure types
    /// Table
    Table,
    /// Table row
    TR,
    /// Table header cell
    TH,
    /// Table data cell
    TD,
    /// Table header group
    THead,
    /// Table body group
    TBody,
    /// Table footer group
    TFoot,

    // Inline structure types
    /// Span (inline generic)
    Span,
    /// Quote
    Quote,
    /// Note
    Note,
    /// Reference
    Reference,
    /// Bibliographic entry
    BibEntry,
    /// Code
    Code,
    /// Link
    Link,
    /// Annotation
    Annot,
    /// Word break - explicit word boundary in structure tree (PDF 1.5+)
    ///
    /// Per PDF spec Section 14.8.4.4, WB elements mark word boundaries
    /// in languages that do not use spaces between words (e.g., CJK).
    WB,

    // Illustration structure types
    /// Figure
    Figure,
    /// Formula
    Formula,
    /// Form (input field)
    Form,

    // Non-standard or custom type
    /// Custom structure type not defined in the PDF specification
    Custom(String),
}

impl StructType {
    /// Parse structure type from string (e.g., "/P" -> StructType::P)
    pub fn from_str(s: &str) -> Self {
        match s {
            "Document" => Self::Document,
            "Part" => Self::Part,
            "Art" => Self::Art,
            "Sect" => Self::Sect,
            "Div" => Self::Div,
            "P" => Self::P,
            "H" => Self::H,
            "H1" => Self::H1,
            "H2" => Self::H2,
            "H3" => Self::H3,
            "H4" => Self::H4,
            "H5" => Self::H5,
            "H6" => Self::H6,
            "L" => Self::L,
            "LI" => Self::LI,
            "Lbl" => Self::Lbl,
            "LBody" => Self::LBody,
            "Table" => Self::Table,
            "TR" => Self::TR,
            "TH" => Self::TH,
            "TD" => Self::TD,
            "THead" => Self::THead,
            "TBody" => Self::TBody,
            "TFoot" => Self::TFoot,
            "Span" => Self::Span,
            "Quote" => Self::Quote,
            "Note" => Self::Note,
            "Reference" => Self::Reference,
            "BibEntry" => Self::BibEntry,
            "Code" => Self::Code,
            "Link" => Self::Link,
            "Annot" => Self::Annot,
            "WB" => Self::WB,
            "Figure" => Self::Figure,
            "Formula" => Self::Formula,
            "Form" => Self::Form,
            _ => Self::Custom(s.to_string()),
        }
    }

    /// Check if this is a heading type (H, H1-H6)
    pub fn is_heading(&self) -> bool {
        matches!(self, Self::H | Self::H1 | Self::H2 | Self::H3 | Self::H4 | Self::H5 | Self::H6)
    }

    /// Check if this is a block-level element
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            Self::Document
                | Self::Part
                | Self::Art
                | Self::Sect
                | Self::Div
                | Self::P
                | Self::H
                | Self::H1
                | Self::H2
                | Self::H3
                | Self::H4
                | Self::H5
                | Self::H6
                | Self::Table
                | Self::Figure
                | Self::Formula
        )
    }

    /// Get heading level (1-6) if this is a heading type
    pub fn heading_level(&self) -> Option<u8> {
        match self {
            Self::H | Self::H1 => Some(1),
            Self::H2 => Some(2),
            Self::H3 => Some(3),
            Self::H4 => Some(4),
            Self::H5 => Some(5),
            Self::H6 => Some(6),
            _ => None,
        }
    }

    /// Check if this is a list type (L, LI, Lbl, LBody)
    pub fn is_list(&self) -> bool {
        matches!(self, Self::L | Self::LI | Self::Lbl | Self::LBody)
    }

    /// Check if this is a word break element (WB)
    ///
    /// Word break elements mark explicit word boundaries in languages
    /// that don't use spaces (e.g., CJK). When encountered during text
    /// extraction, a space should be inserted.
    pub fn is_word_break(&self) -> bool {
        matches!(self, Self::WB)
    }

    /// Get markdown prefix for this structure type
    pub fn markdown_prefix(&self) -> Option<&'static str> {
        match self {
            Self::H1 => Some("# "),
            Self::H2 => Some("## "),
            Self::H3 => Some("### "),
            Self::H4 => Some("#### "),
            Self::H5 => Some("##### "),
            Self::H6 => Some("###### "),
            Self::Lbl => Some("- "),
            _ => None,
        }
    }
}

/// Parent tree that maps marked content IDs to structure elements.
///
/// According to PDF spec Section 14.7.4.4, the parent tree is a number tree
/// that maps MCID values to the structure elements that own them.
#[derive(Debug, Clone)]
pub struct ParentTree {
    /// Mapping from page number to MCID mappings for that page
    pub page_mappings: HashMap<u32, HashMap<u32, ParentTreeEntry>>,
}

impl ParentTree {
    /// Create a new parent tree
    pub fn new() -> Self {
        Self {
            page_mappings: HashMap::new(),
        }
    }

    /// Get the structure element that owns the given MCID on the given page
    pub fn get_parent(&self, page: u32, mcid: u32) -> Option<&ParentTreeEntry> {
        self.page_mappings
            .get(&page)
            .and_then(|page_map| page_map.get(&mcid))
    }
}

impl Default for ParentTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Entry in the parent tree
#[derive(Debug, Clone)]
pub enum ParentTreeEntry {
    /// Direct reference to a structure element
    StructElem(Box<StructElem>),
    /// Object reference (indirect)
    ObjectRef(u32, u16), // (object_num, generation)
}

/// MarkInfo dictionary from the document catalog.
///
/// Per ISO 32000-1:2008 Section 14.7.1, the MarkInfo dictionary contains:
/// - `/Marked` - Whether the document conforms to Tagged PDF conventions
/// - `/Suspects` - Whether the document contains suspect content that
///   may not render properly or has questionable structure
/// - `/UserProperties` - Whether the document contains user properties
///
/// When `suspects` is true, reading order strategies should consider
/// falling back to geometric ordering instead of relying on the
/// potentially unreliable structure tree.
#[derive(Debug, Clone, Default)]
pub struct MarkInfo {
    /// Whether the document is marked (conforms to Tagged PDF conventions)
    pub marked: bool,

    /// Whether the document contains suspect content
    ///
    /// Per PDF spec Section 14.9.2, when this is true, the structure tree
    /// may contain errors or unreliable content. Reading order strategies
    /// should consider falling back to geometric ordering.
    pub suspects: bool,

    /// Whether the document uses user-defined properties
    pub user_properties: bool,
}

impl MarkInfo {
    /// Create a new MarkInfo with default values (all false)
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the structure tree should be considered reliable
    ///
    /// Returns true if the document is marked as Tagged PDF AND
    /// does not have suspected unreliable content.
    pub fn is_structure_reliable(&self) -> bool {
        self.marked && !self.suspects
    }
}

/// Pre-computed index of structure-element `/ActualText` replacements
/// resolved over the entire structure tree.
///
/// Per ISO 32000-1:2008 §14.9.4, a structure element may carry an
/// `/ActualText` entry that replaces the entire content of its
/// descendants for text-extraction purposes. The replacement scope is
/// the subtree rooted at the bearing element. Nested ActualText
/// declarations override their ancestors for the spans they cover.
///
/// ## Data model: keyed by `(McidScope, mcid)`
///
/// Per ISO 32000-1:2008 §14.7.4.3, MCIDs are NOT globally unique — they
/// are unique only within a single content stream (page content stream,
/// Form XObject content stream, or Tiling Pattern content stream). Two
/// Form XObjects on the same page can both emit MCID `0` referring to
/// distinct structure elements. Keying by `(McidScope, mcid)` keeps
/// every namespace separate.
///
/// For page-scoped MCIDs, `McidScope::Page(page_index)` plays exactly
/// the role the previous `(page, mcid)` keying did; the cross-page
/// emit-once rule still applies via [`Self::suppress_only`]. Form- and
/// Pattern-scoped MCIDs are intrinsically per-stream — every covered
/// MCID emits at its anchor (the form/pattern's content stream covers
/// a single namespace).
///
/// ## Emission model: mcid-driven, consecutive-run dedup
///
/// Consumers iterate per-page MCIDs in structure-tree order and emit
/// `mcid_to_actual_text[(page, mcid)]` whenever the replacement
/// changes between consecutive covered MCIDs. A subtree covering
/// `[5, 6, 7]` with one replacement string therefore emits ONE
/// replacement; a subtree where a nested ActualText overrides MCID 6
/// gives THREE emissions (outer at 5, inner at 6, outer at 7), which
/// is the correct inner-wins shape. Run-end is detected at the next
/// non-covered MCID or at a covered MCID with a different replacement.
///
/// ## Visibility (OCG filtering)
///
/// When a covered MCID has been filtered out by an excluded OCG layer,
/// the consumer skips emission for that MCID but does NOT break the
/// consecutive-run dedup — so partial OCG visibility still emits the
/// replacement at the first visible covered MCID in the run.
#[derive(Debug, Clone, Default)]
pub struct ActualTextIndex {
    /// Map from `(McidScope, mcid)` → its innermost replacement text.
    ///
    /// The "innermost" resolution honours nesting: when a descendant
    /// element redeclares `/ActualText`, the descendant's text replaces
    /// the ancestor's for `(scope, mcid)` keys in the inner subtree.
    pub mcid_to_actual_text: HashMap<(McidScope, u32), Arc<str>>,

    /// `(McidScope, mcid)` pairs whose raw glyph spans must be
    /// suppressed during assembly.
    ///
    /// Every key in [`Self::mcid_to_actual_text`] is present here, and
    /// keys in [`Self::suppress_only`] are present here too. Suppression
    /// prevents duplicate output: the replacement is emitted via the
    /// mcid-driven walk, and raw glyphs for the same `(scope, mcid)`
    /// are dropped.
    pub covered_mcids: HashSet<(McidScope, u32)>,

    /// `(McidScope, mcid)` pairs that are covered (raw glyphs
    /// suppressed) but must NOT emit a replacement.
    ///
    /// For page-scoped subtrees, this encodes "emit-once on the first
    /// page" semantics: the bearing element's first-page `(Page(p), mcid)`
    /// entries land in [`Self::mcid_to_actual_text`]; every other page's
    /// `(Page(p), mcid)` entries land here so the raw glyphs are still
    /// suppressed but no second emission fires. Form- and Pattern-
    /// scoped subtrees do not populate this set: each
    /// `Form` / `Pattern` covers a single namespace.
    pub suppress_only: HashSet<(McidScope, u32)>,
}

impl ActualTextIndex {
    /// Construct an empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true when no ActualText scopes were discovered.
    pub fn is_empty(&self) -> bool {
        self.covered_mcids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_type_parsing() {
        assert_eq!(StructType::from_str("P"), StructType::P);
        assert_eq!(StructType::from_str("H1"), StructType::H1);
        assert_eq!(StructType::from_str("Document"), StructType::Document);

        // Custom types
        match StructType::from_str("CustomType") {
            StructType::Custom(s) => assert_eq!(s, "CustomType"),
            _ => panic!("Expected Custom type"),
        }
    }

    #[test]
    fn test_is_heading() {
        assert!(StructType::H1.is_heading());
        assert!(StructType::H2.is_heading());
        assert!(StructType::H.is_heading());
        assert!(!StructType::P.is_heading());
        assert!(!StructType::Document.is_heading());
    }

    #[test]
    fn test_is_block() {
        assert!(StructType::P.is_block());
        assert!(StructType::H1.is_block());
        assert!(StructType::Document.is_block());
        assert!(!StructType::Span.is_block());
        assert!(!StructType::Link.is_block());
    }

    #[test]
    fn test_heading_level() {
        assert_eq!(StructType::H.heading_level(), Some(1));
        assert_eq!(StructType::H1.heading_level(), Some(1));
        assert_eq!(StructType::H2.heading_level(), Some(2));
        assert_eq!(StructType::H3.heading_level(), Some(3));
        assert_eq!(StructType::H4.heading_level(), Some(4));
        assert_eq!(StructType::H5.heading_level(), Some(5));
        assert_eq!(StructType::H6.heading_level(), Some(6));
        assert_eq!(StructType::P.heading_level(), None);
        assert_eq!(StructType::Document.heading_level(), None);
    }

    #[test]
    fn test_is_list() {
        assert!(StructType::L.is_list());
        assert!(StructType::LI.is_list());
        assert!(StructType::Lbl.is_list());
        assert!(StructType::LBody.is_list());
        assert!(!StructType::P.is_list());
        assert!(!StructType::H1.is_list());
        assert!(!StructType::Table.is_list());
    }

    #[test]
    fn test_markdown_prefix() {
        assert_eq!(StructType::H1.markdown_prefix(), Some("# "));
        assert_eq!(StructType::H2.markdown_prefix(), Some("## "));
        assert_eq!(StructType::H3.markdown_prefix(), Some("### "));
        assert_eq!(StructType::H4.markdown_prefix(), Some("#### "));
        assert_eq!(StructType::H5.markdown_prefix(), Some("##### "));
        assert_eq!(StructType::H6.markdown_prefix(), Some("###### "));
        assert_eq!(StructType::Lbl.markdown_prefix(), Some("- "));
        assert_eq!(StructType::P.markdown_prefix(), None);
        assert_eq!(StructType::Table.markdown_prefix(), None);
    }
}
