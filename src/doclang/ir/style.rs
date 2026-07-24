/// Six character-level formatting flags that DocLang expresses directly.
///
/// All six correspond 1-to-1 with rhwp `CharShape` booleans and with DocLang
/// inline elements `<bold>`, `<italic>`, `<underline>`, `<strikethrough>`,
/// `<superscript>`, and `<subscript>`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StyleFlags {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub superscript: bool,
    pub subscript: bool,
}

impl StyleFlags {
    /// Returns `true` when no flag is set (plain text run).
    pub fn is_plain(&self) -> bool {
        !self.bold
            && !self.italic
            && !self.underline
            && !self.strike
            && !self.superscript
            && !self.subscript
    }
}

/// HWP-specific properties that have no direct DocLang v0.6 representation.
///
/// Populated during adapter lowering.  In `Lean` mode the fields are included
/// in the `LossReport`; in `Preserve` mode they are serialised as
/// `<custom ns="hwp:style">` payload.
///
/// All fields are optional — only the subset actually present in the source
/// paragraph or character shape is filled in.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LostProperties {
    /// Named paragraph style (e.g. "본문", "제목1").
    pub named_style: Option<String>,
    /// Font name (may vary by script; this stores the Latin/default face).
    pub font_name: Option<String>,
    /// Font size in tenth-points (HWP internal unit), e.g. 100 = 10 pt.
    pub font_size: Option<i32>,
    /// Text foreground color as 0xRRGGBB.
    pub text_color: Option<u32>,
    /// Section-level information (column layout, page margins, etc.) serialised
    /// as an opaque JSON-like string for preserve mode.
    pub section_info: Option<String>,
    /// Catch-all for any other HWP properties not covered by the typed fields
    /// above.  Stored as `(key, value)` pairs where both strings are
    /// human-readable / round-trip safe.
    pub extras: Vec<(String, String)>,
}
