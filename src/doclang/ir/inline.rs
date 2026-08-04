use super::style::StyleFlags;

/// An inline content node inside a paragraph or heading.
///
/// The tree is intentionally shallow: `Styled` nesting mirrors DocLang's
/// element nesting model (`<bold><italic>…</italic></bold>`) rather than a
/// flat run sequence, making the writer straightforward.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain UTF-8 text with no additional formatting.
    Text(String),
    /// A formatted span: the flags describe which DocLang inline elements wrap
    /// the inner content.  Multiple flags set simultaneously produce nested
    /// elements.
    Styled(StyleFlags, Vec<Inline>),
    /// A reference to a footnote by its 1-based sequential number.
    FootnoteRef(usize),
    /// Hard line break (`\n` equivalent within a paragraph).
    LineBreak,
    /// Tab character.
    Tab,
    /// A hyperlink span: `content` is the anchor inline content and `uri` the
    /// link target. Maps to DocLang `<href uri="…">…</href>`. Produced from a
    /// HWP hyperlink field whose `field_ranges` cover the anchor text.
    Href { uri: String, content: Vec<Inline> },
}
