use super::formula::Formula;
use super::geometry::Geometry;
use super::inline::Inline;
use super::prov::Prov;
use super::style::LostProperties;
use super::table::Table;

/// Which pages a header or footer applies to.
#[derive(Debug, Clone, PartialEq)]
pub enum HeaderFooterApply {
    /// All pages.
    All,
    /// Even-numbered pages only.
    Even,
    /// Odd-numbered pages only.
    Odd,
    /// First page only.
    First,
}

/// A list item, which may itself contain nested blocks (including sub-lists).
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    /// Block-level content of this item.
    pub content: Vec<Block>,
}

/// A block-level content node.
///
/// The `lost` field on several variants carries HWP-specific properties that
/// have no DocLang v0.6 equivalent.  In `Lean` mode these are forwarded to the
/// `LossReport`; in `Preserve` mode they are emitted as `<custom>` elements.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// A body paragraph.
    Paragraph {
        content: Vec<Inline>,
        /// HWP properties with no DocLang representation.
        lost: Option<LostProperties>,
        /// Model provenance for v2 `<location>` join; `None` when unknown or
        /// when location is disabled.
        prov: Option<Prov>,
    },

    /// A section heading (`<heading level=N>`), level 1–6.
    Heading {
        /// 1-based heading level (1 = top, 6 = deepest).
        level: u8,
        content: Vec<Inline>,
        lost: Option<LostProperties>,
        /// Model provenance for v2 `<location>` join.
        prov: Option<Prov>,
    },

    /// An ordered or unordered list.
    List {
        /// `true` for numbered lists, `false` for bulleted.
        ordered: bool,
        items: Vec<ListItem>,
        lost: Option<LostProperties>,
        /// Model provenance for v2 `<location>` join. A list coalesces several
        /// list-item paragraphs; this is the provenance of the *first* item so
        /// the resolved box anchors the list at its leading line.
        prov: Option<Prov>,
    },

    /// A table (maps to DocLang OTSL).
    Table(Table),

    /// An embedded picture (image).
    Picture {
        /// Raw image bytes (already decompressed by rhwp).
        data: Vec<u8>,
        /// File extension in lower-case ASCII (e.g. `"png"`, `"jpg"`, `"emf"`).
        extension: String,
        /// Geometry captured for v2 `<location>` support; not emitted in v1.
        geometry: Option<Geometry>,
        lost: Option<LostProperties>,
        /// Model provenance for v2 `<location>` join.
        prov: Option<Prov>,
    },

    /// A mathematical formula.
    Formula(Formula),

    /// An explicit page break (`<page_break/>`).
    PageBreak,

    /// A footnote definition.  The `number` is 1-based and sequential.
    Footnote {
        number: usize,
        content: Vec<Block>,
        /// Model provenance for v2 `<location>` join.
        prov: Option<Prov>,
    },

    /// A page header block.
    PageHeader {
        content: Vec<Block>,
        apply: HeaderFooterApply,
        /// Model provenance for v2 `<location>` join.
        prov: Option<Prov>,
    },

    /// A page footer block.
    PageFooter {
        content: Vec<Block>,
        apply: HeaderFooterApply,
        /// Model provenance for v2 `<location>` join.
        prov: Option<Prov>,
    },

    /// Marks the start of a text thread (column or text-box flow).
    ///
    /// DocLang `<thread id="…">`.  The `thread_id` is a stable string key
    /// assigned by the adapter (e.g. `"col-0"`, `"textbox-3"`).
    ThreadStart { thread_id: String },

    /// Marks the continuation (subsequent column / text-box segment) of a
    /// previously opened thread.
    ThreadContinuation { thread_id: String },

    /// A custom/unknown HWP element preserved as an opaque payload.
    ///
    /// Used in `Preserve` mode for shapes, text-boxes, and any `Control`
    /// variant the adapter does not have a typed mapping for.
    Custom {
        /// XML namespace suffix, e.g. `"hwp:floating"`, `"hwp:ruby"`.
        namespace: String,
        /// Serialised payload (JSON or XML fragment).
        payload: String,
    },
}
