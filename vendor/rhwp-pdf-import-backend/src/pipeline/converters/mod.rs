//! Output converters for the text extraction pipeline.
//!
//! This module provides the OutputConverter trait and implementations for
//! converting ordered text spans to various output formats.
//!
//! # Available Converters
//!
//! - [`MarkdownOutputConverter`]: Convert to Markdown format
//! - [`HtmlOutputConverter`]: Convert to HTML format
//! - [`PlainTextConverter`]: Convert to plain text
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::pipeline::converters::{OutputConverter, MarkdownOutputConverter};
//! use pdf_oxide::pipeline::TextPipelineConfig;
//!
//! let converter = MarkdownOutputConverter::new();
//! let config = TextPipelineConfig::default();
//! let output = converter.convert(&ordered_spans, &config)?;
//! ```

mod html;
mod markdown;
mod plain_text;
pub mod toc_detector;

pub use html::HtmlOutputConverter;
pub use markdown::MarkdownOutputConverter;
pub use plain_text::PlainTextConverter;
pub use toc_detector::{TocDetector, TocEntry};

use crate::error::Result;
use crate::layout::TextSpan;
use crate::pipeline::{OrderedTextSpan, StructRole, TextPipelineConfig};
use crate::structure::table_extractor::Table;

/// Bullet glyphs commonly used as list markers in PDFs.
const BULLET_CHARS: &[char] = &[
    '►', '•', '▪', '▸', '‣', '◦', '●', '■', '◆', '○', '□', '❍', '❖', '✓', '✔', '➢', '➤', '\x7f',
];

/// Whether `text` is a lone bullet glyph (the whole span is the marker).
pub(crate) fn is_bullet_span(text: &str) -> bool {
    let t = text.trim();
    let mut chars = t.chars();
    matches!((chars.next(), chars.next()), (Some(c), None) if BULLET_CHARS.contains(&c))
}

/// Whether `text` begins with a bullet glyph (inline bullet + body).
pub(crate) fn starts_with_bullet(text: &str) -> bool {
    text.trim_start()
        .chars()
        .next()
        .is_some_and(|c| BULLET_CHARS.contains(&c))
}

/// Parse an ordered-list marker (`1.`, `a)`, `iv.`) at the start of `text`,
/// returning the numeric position when it is numeric. `Some(_)` means "this
/// text starts a numbered/lettered list item".
pub(crate) fn is_ordered_list_marker(text: &str) -> Option<u32> {
    let t = text.trim_start();
    let bytes = t.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut idx = 0;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() && idx < 3 {
        idx += 1;
    }
    let numeric_n = if idx > 0 {
        std::str::from_utf8(&bytes[..idx])
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
    } else {
        None
    };
    // Single ASCII letter / roman-numeral form (a) / b. / iv.).
    if idx == 0 && bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() {
        let mut roman_end = 0;
        while roman_end < bytes.len().min(4)
            && matches!(bytes[roman_end], b'i' | b'v' | b'x' | b'I' | b'V' | b'X')
        {
            roman_end += 1;
        }
        if roman_end >= 1 && bytes.len() > roman_end {
            let punct = bytes[roman_end];
            if matches!(punct, b'.' | b')') && bytes.get(roman_end + 1).copied() == Some(b' ') {
                return Some(1);
            }
        }
        if bytes.len() >= 3 && matches!(bytes[1], b'.' | b')') && bytes[2] == b' ' {
            return Some(1);
        }
        return None;
    }
    if idx > 0 && bytes.len() > idx {
        let punct = bytes[idx];
        if matches!(punct, b'.' | b')') && bytes.get(idx + 1).copied() == Some(b' ') {
            return numeric_n;
        }
    }
    None
}

/// The base (body) font size used to derive heading levels from font ratios.
///
/// Uses the mode of spans ≥9pt (excluding bullet glyphs / subscripts / footnote
/// markers that would skew it down), capped at 12pt. Shared by the markdown and
/// HTML converters so both agree on heading levels for untagged documents.
pub(crate) fn base_heading_font_size(spans: &[&OrderedTextSpan], detect_headings: bool) -> f32 {
    if !detect_headings {
        return 12.0;
    }
    let mut size_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for s in spans {
        let sz = s.span.font_size;
        if sz < 9.0 {
            continue;
        }
        *size_counts.entry((sz * 2.0).round() as u32).or_insert(0) += 1;
    }
    size_counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
        .map(|(bucket, _)| bucket as f32 / 2.0)
        .unwrap_or(12.0)
        .min(12.0)
}

/// Whether a `/Link` annotation URI is safe to emit as a hyperlink.
///
/// Only well-known navigable schemes are allowed; `javascript:`, `data:`,
/// `vbscript:`, `file:` and any other/unknown scheme are rejected so a
/// malicious PDF cannot inject an active-content link into the HTML/markdown
/// output (XSS). Anchor text is still emitted for rejected URIs — only the
/// link target is dropped.
pub(crate) fn is_safe_link_uri(uri: &str) -> bool {
    let lower = uri.trim_start().to_ascii_lowercase();
    [
        "http://", "https://", "mailto:", "tel:", "ftp://", "ftps://",
    ]
    .iter()
    .any(|scheme| lower.starts_with(scheme))
}

/// Whether a structure-tree role marks the span as part of a list item.
pub(crate) fn is_list_item_role(role: Option<StructRole>) -> bool {
    matches!(
        role,
        Some(StructRole::ListItem | StructRole::ListItemLabel | StructRole::ListItemBody)
    )
}

/// Trait for converting ordered text spans to output formats.
///
/// Implementations transform a sequence of ordered text spans into a specific
/// output format (Markdown, HTML, plain text, etc.).
///
/// This trait provides a clean abstraction layer between the PDF extraction
/// pipeline and the output generation, following the PDF spec compliance goal
/// of separating PDF representation from output formatting.
pub trait OutputConverter: Send + Sync {
    /// Convert ordered spans to the target format.
    ///
    /// # Arguments
    ///
    /// * `spans` - Ordered text spans from the reading order strategy
    /// * `config` - Pipeline configuration affecting output formatting
    ///
    /// # Returns
    ///
    /// The formatted output string.
    fn convert(&self, spans: &[OrderedTextSpan], config: &TextPipelineConfig) -> Result<String>;

    /// Convert ordered spans to the target format, with pre-detected tables.
    ///
    /// Table regions are rendered using the converter's table formatting
    /// (markdown tables, HTML tables, or tab-delimited text). Spans that
    /// fall within table bounding boxes are excluded from normal rendering.
    ///
    /// Default implementation ignores tables and falls back to `convert()`.
    fn convert_with_tables(
        &self,
        spans: &[OrderedTextSpan],
        tables: &[Table],
        config: &TextPipelineConfig,
    ) -> Result<String> {
        let _ = tables;
        self.convert(spans, config)
    }

    /// Return the name of this converter for debugging.
    fn name(&self) -> &'static str;

    /// Return the MIME type for the output format.
    fn mime_type(&self) -> &'static str;
}

/// Returns `true` if `c` is a CJK character (Chinese, Japanese, or Korean).
fn is_cjk_char(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{309F}' |   // Hiragana
        '\u{30A0}'..='\u{30FF}' |   // Katakana
        '\u{4E00}'..='\u{9FFF}' |   // CJK Unified Ideographs
        '\u{AC00}'..='\u{D7AF}' |   // Hangul
        '\u{3400}'..='\u{4DBF}' |   // CJK Extension A
        '\u{20000}'..='\u{2A6DF}'   // CJK Extension B
    )
}

/// Returns `true` if `c` is a fullwidth or mathematical operator that is
/// commonly embedded inside CJK text without surrounding spaces.
///
/// These characters have slightly wider advances than typical ASCII characters,
/// which can trigger the gap heuristic and insert a spurious space when they
/// appear between CJK glyphs (e.g. `25000≤Q＜40000`).
fn is_fullwidth_or_math_op(c: char) -> bool {
    matches!(c,
        '\u{FF0B}' |                // ＋
        '\u{FF0D}' |                // －
        '\u{FF1A}' |                // ：
        '\u{FF1B}' |                // ；
        '\u{FF1C}'..='\u{FF1E}' |  // ＜ ＝ ＞
        '\u{2260}' |               // ≠
        '\u{2248}' |               // ≈
        '\u{2264}'..='\u{2265}' |  // ≤ ≥
        '\u{00B5}' |               // µ
        '\u{03BC}' |               // μ
        '\u{00B1}' |               // ±
        '\u{00D7}' |               // ×
        '\u{00F7}'                 // ÷
    )
}

/// Check whether two horizontally adjacent spans have a visible gap between them.
///
/// Returns `true` when the horizontal distance between the end of `prev` and
/// the start of `current` exceeds a small fraction of the font size but is not
/// unreasonably large (which would indicate a column break rather than a word
/// gap).
///
/// CJK scripts do not use spaces between words.  When one side of the boundary
/// is a CJK character and the other side is CJK or a fullwidth/math operator
/// (e.g. `≤`, `＜`, `μ`), no space is inserted even if the geometric gap
/// exceeds the threshold.  This mirrors the CJK-pair suppression in the text
/// extraction path (`document.rs`).
/// True for a character belonging to a right-to-left script.
///
/// Covers Hebrew, Arabic and its supplements and presentation forms, Syriac,
/// Thaana and NKo — the ranges ISO 32000-1:2008 14.8.2.3.3 has in mind when it
/// describes right-to-left show-text runs. Used to keep left-to-right
/// positional reasoning from being applied to a script that advances the other
/// way.
fn is_rtl_char(c: char) -> bool {
    matches!(c as u32,
        0x0590..=0x05FF   // Hebrew
        | 0x0600..=0x06FF // Arabic
        | 0x0700..=0x074F // Syriac
        | 0x0750..=0x077F // Arabic Supplement
        | 0x0780..=0x07BF // Thaana
        | 0x07C0..=0x07FF // NKo
        | 0x08A0..=0x08FF // Arabic Extended-A
        | 0xFB1D..=0xFDFF // Hebrew/Arabic Presentation Forms-A
        | 0xFE70..=0xFEFF // Arabic Presentation Forms-B
    )
}

/// A footnote or citation marker set immediately after a prose word.
///
/// ISO 32000-1:2008 §9.4.4 makes the glyph advance the only thing that moves
/// the text position, so a marker typeset at the base word's advance edge is
/// not separated by geometry at all — `has_horizontal_gap` correctly reports no
/// gap, and the two run together as `phosphorylation55`.
///
/// Gap size cannot settle it, and the measurements are inverted from the
/// intuition: the footnote markers sit at 0.10 em from their word while genuine
/// maths subscripts (`W2`, `CP3`, `H1`) sit at 0.14 em and larger. Any
/// threshold that splits the first fuses on the second.
///
/// What does separate them is the same distinction `merge_sub_superscript_spans`
/// already draws for the text path: a sub/superscript *host* is a symbol, not a
/// prose word. `H`, `x`, `ADP` and `SO` are hosts; `phosphorylation` is not. So
/// this fires only where the base is a word, the marker is a bare numeral, and
/// the run is set smaller in a different font — the shape a reference callout
/// has and a subscript does not.
///
/// Deliberately kept out of `has_horizontal_gap`: that function also serves
/// `render_cell_html` and `cell_plain_text`, and table rendering must not move.
pub(crate) fn is_reference_marker_boundary(prev: &TextSpan, current: &TextSpan) -> bool {
    // Only in the band the gap rule already declines; never override a real gap.
    if (prev.rotation_degrees - current.rotation_degrees).abs() > 0.5
        || has_horizontal_gap(prev, current)
    {
        return false;
    }
    // An italic base is a mathematical expression, not prose.
    if prev.is_italic || current.is_italic {
        return false;
    }
    // The marker is a distinctly smaller run in a different font resource.
    if prev.font_name == current.font_name
        || current.font_size >= prev.font_size * 0.85
        || current.font_size <= 0.0
    {
        return false;
    }
    let em = prev.font_size.max(current.font_size).max(1.0);
    let gap = current.bbox.x - (prev.bbox.x + prev.bbox.width);
    if gap <= 0.5 || gap >= em * 3.0 {
        return false;
    }
    // The base ends in a prose word: three or more letters, ending lowercase.
    // That excludes every sub/superscript host — a lone symbol, an element pair
    // or a trailing acronym.
    let base = prev.text.trim_end();
    let tail: String = base
        .chars()
        .rev()
        .take_while(|c| c.is_alphabetic())
        .collect();
    if tail.chars().count() < 3 || !base.ends_with(|c: char| c.is_ascii_lowercase()) {
        return false;
    }
    // The marker is a bare numeral, optionally a list or range of them.
    let head: String = current
        .text
        .trim_start()
        .chars()
        .take_while(|c| {
            c.is_ascii_digit() || matches!(c, ',' | '-' | '\u{2013}' | '\u{2014}' | '\u{2212}')
        })
        .collect();
    !head.is_empty()
        && head.chars().next().is_some_and(|c| c.is_ascii_digit())
        && head.chars().any(|c| c.is_ascii_digit())
}

/// True when a run reads as a piece of a sentence rather than a title.
///
/// The heading predicates are built from typography — font size, weight, word
/// count, capitalisation. On a page whose text layer is garbled those signals
/// survive intact while the words themselves stop forming titles, and a body
/// fragment carrying a large font gets promoted. A 1919 broadsheet produced
/// `## Furthermore, one reads in the` and `### palaces league.` that way: both
/// clear every existing test, because the first leads with a capital and is
/// only five words, and the second is two words, below the five-word floor the
/// lowercase-initial rule uses.
///
/// Two shapes settle it without appealing to layout:
///
/// A title does not end on a function word. `in`, `the`, `of` and their kin
/// exist to attach what follows them, so a run ending in one has had its
/// continuation cut away. The check needs three words before it applies, which
/// keeps `About`, `Contact Us` and titles that genuinely end in such a word
/// out of its reach.
///
/// A run that opens lowercase and closes on a full stop is a sentence with its
/// head removed. Headings that legitimately begin lowercase — a product name,
/// a stylised mark — do not also terminate in a period.
///
/// Both tests are English-shaped and both only ever *reject*, so a heading in
/// another language is untouched: its words match no entry in the list, and
/// scripts without case report `is_lowercase() == false`.
pub(crate) fn reads_as_a_sentence_fragment(text: &str) -> bool {
    let trimmed = text.trim();

    // Opens lowercase, closes on a full stop.
    if trimmed.ends_with('.') {
        if let Some(first) = trimmed.chars().find(|c| c.is_alphabetic()) {
            if first.is_lowercase() {
                return true;
            }
        }
    }

    // Ends on a function word, with enough words for that to mean anything.
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() < 3 {
        return false;
    }
    let last: String = words[words.len() - 1]
        .chars()
        .filter(|c| c.is_alphabetic())
        .flat_map(|c| c.to_lowercase())
        .collect();
    // Articles, prepositions, the coordinating conjunctions and the two
    // complementisers. Deliberately no auxiliaries or pronouns: `Let It Be`,
    // `Yes We Can` and `Doctor Who` are titles that end exactly that way.
    matches!(
        last.as_str(),
        "a" | "an"
            | "the"
            | "of"
            | "in"
            | "on"
            | "at"
            | "to"
            | "for"
            | "with"
            | "from"
            | "by"
            | "into"
            | "onto"
            | "upon"
            | "over"
            | "under"
            | "between"
            | "among"
            | "through"
            | "during"
            | "against"
            | "about"
            | "than"
            | "without"
            | "within"
            | "and"
            | "or"
            | "but"
            | "nor"
            | "that"
            | "which"
    )
}

/// True for a character that establishes left-to-right reading on its own.
///
/// Unicode Standard Annex #9 sorts characters into strong, weak and neutral
/// types, and only the strong ones carry a direction. A Latin, Greek, Cyrillic
/// or Han letter is strong; digits and `/`, `%`, `-`, `#` are weak or neutral
/// and take their direction from whatever surrounds them. ISO 32000-1:2008
/// Table 344 defers to that annex by name — a writing mode's
/// inline-progression direction "is subject to local override within the text
/// being laid out, as described in Unicode Standard Annex #9, The
/// Bidirectional Algorithm".
///
/// So a run of digits is evidence of nothing. It reads left-to-right in a Latin
/// paragraph and right-to-left in an Arabic one, and its geometry alone cannot
/// say which.
fn is_strong_ltr_char(c: char) -> bool {
    c.is_alphabetic() && !is_rtl_char(c)
}

pub(crate) fn has_horizontal_gap(prev: &TextSpan, current: &TextSpan) -> bool {
    // Runs on different writing axes are not comparable along page-x at all.
    // ISO 32000-1:2008 9.4.4: a glyph's displacement is interpreted in text
    // space, so a 90-degree run's `bbox.width` is its advance along a
    // physically vertical axis. Subtracting it from a horizontal run's x gave
    // `72 - (32 + 343.30) = -303.30` for a rotated marginal stamp beside a body
    // line, which read as "no gap" and glued the two together.
    if (prev.rotation_degrees - current.rotation_degrees).abs() > 0.5 {
        return true;
    }
    let font_size = prev.font_size.max(current.font_size).max(1.0);
    let prev_end_x = prev.bbox.x + prev.bbox.width;
    let gap = current.bbox.x - prev_end_x;
    let threshold = font_size * 0.15;
    // Sub-em gaps are inter-glyph kerning — no space needed.  ANY gap
    // larger than that, including gaps >5 em (column boundaries on
    // wide tables — issue 487 pr-138-example.pdf), must result in a
    // space.  The previous `gap < 5 em` upper bound made the caller
    // concatenate without separator for huge gaps, gluing tokens like
    // `3.80%` + `4.41%` into `3.80%4.41%` when the rate-table cells
    // sit ~265 pt apart and the table detector wasn't able to capture
    // them as a real grid.
    // A span that ends before the previous one begins cannot be a
    // continuation of it: the two are separated by a reading discontinuity —
    // a new line, a new column, or a re-ordered run on an OCR text layer
    // whose baselines jitter enough to scramble the row grouping. Treating
    // that as "no gap" concatenated tokens that were never adjacent, so
    // `It is the` came out as `theisIt`. Only a *complete* backward step
    // counts: a small negative gap is glyph overlap (accent composition, an
    // over-wide advance estimate) and must stay unseparated.
    //
    // The premise holds only for a left-to-right run. In Arabic, Hebrew and
    // the other right-to-left scripts a continuation steps *leftward* by
    // definition, so every glyph pair in a right-to-left word satisfies the
    // test and the word is split into single letters. Word-final forms often
    // carry a zero advance here as well, which makes the apparent step even
    // larger. So the rule is scoped to runs with no right-to-left character on
    // either side; a right-to-left run falls back to the plain gap test, which
    // is what separated its words correctly before.
    // A *large* backward jump is a discontinuity even when the two boxes still
    // overlap. Requiring a complete step missed the case where a second
    // overlaid layer, or a run re-ordered on an OCR text layer, starts well to
    // the left of where the previous run ended but stretches past its start:
    // "…Labeling Technologies" ending at 396.07 followed by a run beginning at
    // 205.31 is a gap of −190.76 pt, and the two were concatenated as
    // "TechnologiesA Guide". The bound mirrors the one `extract_text` uses for
    // the same judgement (twenty ems), which is far outside the range of the
    // glyph overlap this rule exists to tolerate.
    let backward_em = prev.font_size.max(current.font_size).max(6.0) * 20.0;
    //
    // Carrying no right-to-left character does not establish that a run is
    // left-to-right, because digits and their separators establish no direction
    // at all. A Persian form's `1403/09/19` is drawn right-to-left like the
    // words around it, but `19`, `/` and `09` hold no strong character of
    // either script, so a guard keyed on right-to-left characters never sees
    // them and the rule split every date, percentage and section number it met:
    // `19 / 09 /1403`, `50 %`, `5 2`.
    //
    // The test needs positive evidence rather than the absence of contrary
    // evidence, so it applies only where some strong left-to-right character is
    // present and no right-to-left one is.
    let steps_backward = (current.bbox.x + current.bbox.width <= prev.bbox.x || gap < -backward_em)
        && !prev.text.chars().any(is_rtl_char)
        && !current.text.chars().any(is_rtl_char)
        && (prev.text.chars().any(is_strong_ltr_char)
            || current.text.chars().any(is_strong_ltr_char));
    if gap <= threshold && !steps_backward {
        return false;
    }

    // Suppress space insertion when one side is CJK and the other is CJK or a
    // fullwidth/math operator.  This mirrors the CJK-pair suppression in the
    // text extraction path (document.rs:5587-5605).
    let prev_last = prev.text.chars().next_back();
    let curr_first = current.text.chars().next();
    if let (Some(p), Some(c)) = (prev_last, curr_first) {
        let p_cjk = is_cjk_char(p);
        let c_cjk = is_cjk_char(c);
        if (p_cjk || is_fullwidth_or_math_op(p)) && (c_cjk || is_fullwidth_or_math_op(c)) {
            // At least one side must actually be CJK (not two pure math ops).
            if p_cjk || c_cjk {
                return false;
            }
        }
    }

    true
}

/// Two spans a table cell renders one after the other need a separator when
/// they are not on the same line, whatever their horizontal relationship.
///
/// The cell renderers asked only `has_horizontal_gap`, which compares x. A
/// cell that stacks its members vertically — a CAD sheet's contour labels, a
/// wrapped sentence — has consecutive spans at nearly the same x and different
/// y, so that test found no gap and ran them together, inventing words the
/// page never draws (`128` above `126` became `128126`). The paragraph path
/// has always inserted a separator between lines; the cell path had no
/// equivalent.
///
/// The 0.5 × font-size threshold is the same line test used throughout.
pub(crate) fn spans_are_stacked(prev: &TextSpan, current: &TextSpan) -> bool {
    let font_size = prev.font_size.max(current.font_size).max(1.0);
    (current.bbox.y - prev.bbox.y).abs() >= font_size * 0.5
}

/// Return the index of the table whose bounding box contains the span's
/// origin AND that has a cell whose bbox also contains the span — i.e.
/// the table is actually going to render this span as part of a cell.
///
/// Returning `Some(idx)` causes `convert_semantic_mode` (md/html) to skip
/// the span from paragraph flow on the assumption that the table render
/// will emit it.  If the span sits inside the table's *outer* bbox but
/// the spatial column-clustering missed the column it belongs to (a
/// sparse / variable-width score column on wide sailing-results grids
/// — issue 486 / 487), no cell will contain it and the table render
/// drops the content.  Treating that span as "outside the table" lets
/// the paragraph flow pick it up so the text is not lost.
pub(crate) fn span_in_table(span: &OrderedTextSpan, tables: &[Table]) -> Option<usize> {
    let sx = span.span.bbox.x;
    let sy = span.span.bbox.y;

    // Two tiers ahead of the geometric fallback, so this predicate answers from
    // what the table ACTUALLY renders rather than re-deriving ownership from a
    // different test over a different span population.
    //
    // The detector claims a span by its **centre**, with a 3 pt snap, over
    // *populated* cells. This function historically claimed by the span's
    // **origin**, with a 2 pt slack, over the *full lattice* — placeholder cells
    // included. Those disagree at the edges: a span whose centre snaps in from
    // outside is rendered by the cell AND emitted in prose (duplication, at the
    // left and bottom edges), while one whose origin lands in an empty lattice
    // square is suppressed from prose and rendered by nobody (loss, at the top
    // and right).
    //
    // `TableCell::spans` is the bridge: it holds the word spans the cell really
    // renders, and a placeholder cell has none. That distinction already exists,
    // so no field and no struct change is needed. Identity matching was tried
    // before and reverted because the detector consumes
    // `extract_table_word_spans` while the converters walk flow spans — two
    // populations whose `sequence` values do not correspond. Geometry against
    // the cell's own spans works where identity cannot.
    //
    // A tier that applies is **decisive** for its table: it answers both ways,
    // so the span it declines is left to prose rather than falling through to a
    // rule that would suppress it and leave it rendered by nobody.
    let mut undecided: Vec<usize> = Vec::with_capacity(tables.len());
    for (i, table) in tables.iter().enumerate() {
        // Tier 1 — marked content. Exact for tagged PDFs, and the same test
        // `extract_text` applies.
        let table_has_mcids = table
            .rows
            .iter()
            .any(|r| r.cells.iter().any(|c| !c.mcids.is_empty()));
        if table_has_mcids {
            if let Some(mcid) = span.span.mcid {
                if table
                    .rows
                    .iter()
                    .any(|r| r.cells.iter().any(|c| c.mcids.contains(&mcid)))
                {
                    return Some(i);
                }
                continue;
            }
        }

        // Tier 2 — the cell's own ink. Claim the span only where the runs the
        // cell renders actually cover it, on the same line.
        let table_has_spans = table
            .rows
            .iter()
            .any(|r| r.cells.iter().any(|c| !c.spans.is_empty()));
        if table_has_spans {
            let sw = span.span.bbox.width;
            let s_end = sx + sw;
            let band = span.span.font_size.max(1.0) * 0.5;
            let mut covered = 0.0f32;
            for row in &table.rows {
                for cell in &row.cells {
                    for member in &cell.spans {
                        if (member.bbox.y - sy).abs() > band {
                            continue;
                        }
                        let m_end = member.bbox.x + member.bbox.width;
                        let lo = sx.max(member.bbox.x);
                        let hi = s_end.min(m_end);
                        if hi > lo {
                            covered += hi - lo;
                        }
                    }
                }
            }
            if sw > 0.0 && covered >= sw * 0.5 {
                return Some(i);
            }
            continue;
        }

        // Tier 3 — a table whose cells carry neither MCIDs nor spans
        // (MCID-built tables and unit-test fixtures). The legacy geometric rule
        // is all there is.
        undecided.push(i);
    }

    for i in undecided {
        let table = &tables[i];
        let Some(ref bbox) = table.bbox else { continue };
        let tolerance = 2.0;
        let in_outer_bbox = sx >= bbox.x - tolerance
            && sx <= bbox.x + bbox.width + tolerance
            && sy >= bbox.y - tolerance
            && sy <= bbox.y + bbox.height + tolerance;
        if !in_outer_bbox {
            continue;
        }
        // Span is geometrically inside the table — verify a cell will
        // own it.  Walks all rows / cells once; tables that get through
        // is_real_grid are typically small enough (≤30 rows × ≤25 cols)
        // that this is negligible vs. the cost of running the conversion.
        //
        // Special case: a Table with no cell bboxes at all (e.g. when
        // built from MCID-based tagged-PDF extraction, or in unit-test
        // fixtures) carries the rendering responsibility wholesale —
        // there is no per-cell layout to consult.  Fall back to the
        // outer-bbox containment for that case so we don't silently
        // skip the table rendering.
        let has_any_cell_bbox = table
            .rows
            .iter()
            .any(|row| row.cells.iter().any(|c| c.bbox.is_some()));
        if !has_any_cell_bbox {
            return Some(i);
        }
        let span_owned = table.rows.iter().any(|row| {
            row.cells.iter().any(|cell| {
                let Some(cb) = cell.bbox else { return false };
                sx >= cb.x - tolerance
                    && sx <= cb.x + cb.width + tolerance
                    && sy >= cb.y - tolerance
                    && sy <= cb.y + cb.height + tolerance
            })
        });
        if span_owned {
            return Some(i);
        }
        // Span sits in the outer bbox but no cell claims it; fall through
        // to paragraph flow so the content is not silently dropped.
    }
    None
}

/// Post-process rendered text to merge key-value pairs that were split across
/// lines due to column-based reading order.
///
/// Detects the pattern where a text label (e.g. "Grand Total") appears on one
/// line and its corresponding value (e.g. "$750.00") appears alone on the next
/// line.  When detected, the two lines are merged into one with a separating
/// space (e.g. "Grand Total $750.00").
///
/// A line is considered a "value" if it is short (< 30 chars), starts with a
/// digit, currency symbol, or parenthesized number, and does not look like a
/// sentence continuation.  A line is considered a "label" if it ends with
/// alphabetic text (no trailing punctuation that would indicate a complete
/// sentence).
pub(crate) fn merge_key_value_pairs(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 2 {
        return text.to_string();
    }

    // Determine which lines are "value-only" lines that should merge upward.
    // A value line is short and starts with a digit, $, (, -, or similar
    // numeric indicator.
    fn is_value_line(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.len() > 30 {
            return false;
        }
        // A markdown list item (bullet or ordered marker like "1. Finalize")
        // is never a value — merging it into the line above would glue a list
        // item onto a heading or label.
        if is_ordered_list_marker(trimmed).is_some() || starts_with_bullet(trimmed) {
            return false;
        }
        let mut chars = trimmed.chars();
        let first = chars.next().unwrap();
        match first {
            '0'..='9' | '$' | '€' | '£' | '¥' | '(' => true,
            // '-' / '.' indicate a value ("-$42.50", "-50", ".50") unless
            // followed by a space — i.e. a markdown list bullet "- ", which must
            // NOT merge a list item into the line above it (e.g. a heading).
            '-' | '.' => !matches!(chars.next(), Some(' ') | None),
            _ => false,
        }
    }

    // A label line: non-empty, ends with a word character (letter or digit),
    // does not end with sentence-terminal punctuation.  We also reject lines
    // that are themselves value-only (to avoid merging two values).
    fn is_label_line(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }
        // Must not itself be a value-only line
        if is_value_line(line) {
            return false;
        }
        // Last non-whitespace character should be alphanumeric or ')' or ':'
        // (not sentence-ending like '.', '!', '?')
        let last = trimmed.chars().next_back().unwrap();
        last.is_alphanumeric() || last == ')' || last == ':'
    }

    let mut result = String::with_capacity(text.len());
    let mut i = 0;
    while i < lines.len() {
        // Pattern 1: label immediately followed by value (no blank line)
        if i + 1 < lines.len() && is_label_line(lines[i]) && is_value_line(lines[i + 1]) {
            result.push_str(lines[i].trim_end());
            result.push(' ');
            result.push_str(lines[i + 1].trim_start());
            result.push('\n');
            i += 2;
        }
        // Pattern 2: label, blank line, value (paragraph break between them)
        else if i + 2 < lines.len()
            && is_label_line(lines[i])
            && lines[i + 1].trim().is_empty()
            && is_value_line(lines[i + 2])
        {
            result.push_str(lines[i].trim_end());
            result.push(' ');
            result.push_str(lines[i + 2].trim_start());
            result.push('\n');
            i += 3;
        } else {
            result.push_str(lines[i]);
            result.push('\n');
            i += 1;
        }
    }

    // Restore the exact trailing-newline count of the original input.
    // `text.lines()` strips all trailing empty lines, so we count them here
    // and re-append them after processing.
    let orig_trailing_newlines = text.chars().rev().take_while(|&c| c == '\n').count();
    // Strip any trailing newlines we added, then re-append the original count.
    while result.ends_with('\n') {
        result.pop();
    }
    for _ in 0..orig_trailing_newlines {
        result.push('\n');
    }

    result
}

/// Create a converter based on the output format name.
pub fn create_converter(format: &str) -> Option<Box<dyn OutputConverter>> {
    match format.to_lowercase().as_str() {
        "markdown" | "md" => Some(Box::new(MarkdownOutputConverter::new())),
        "html" => Some(Box::new(HtmlOutputConverter::new())),
        "text" | "plain" | "txt" => Some(Box::new(PlainTextConverter::new())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_converter_markdown() {
        let converter = create_converter("markdown").unwrap();
        assert_eq!(converter.name(), "MarkdownOutputConverter");
        assert_eq!(converter.mime_type(), "text/markdown");
    }

    #[test]
    fn test_create_converter_html() {
        let converter = create_converter("html").unwrap();
        assert_eq!(converter.name(), "HtmlOutputConverter");
        assert_eq!(converter.mime_type(), "text/html");
    }

    #[test]
    fn test_create_converter_text() {
        let converter = create_converter("text").unwrap();
        assert_eq!(converter.name(), "PlainTextConverter");
        assert_eq!(converter.mime_type(), "text/plain");
    }

    #[test]
    fn test_create_converter_unknown() {
        assert!(create_converter("unknown").is_none());
    }

    // ========================================================================
    // Key-value pair merging tests
    // ========================================================================

    #[test]
    fn test_key_value_pair_merging_basic() {
        let input = "Grand Total\n$750.00\nNet Amount\n$250.00\n";
        let expected = "Grand Total $750.00\nNet Amount $250.00\n";
        assert_eq!(merge_key_value_pairs(input), expected);
    }

    #[test]
    fn test_key_value_pair_merging_no_false_positive_on_sentences() {
        // Lines ending with period should not be treated as labels.
        let input = "This is a sentence.\n$100.00\n";
        assert_eq!(merge_key_value_pairs(input), input);
    }

    #[test]
    fn test_key_value_pair_merging_negative_numbers() {
        let input = "Balance Due\n-$42.50\n";
        let expected = "Balance Due -$42.50\n";
        assert_eq!(merge_key_value_pairs(input), expected);
    }

    #[test]
    fn test_key_value_pair_merging_plain_numbers() {
        let input = "Account Number\n434508032\n";
        let expected = "Account Number 434508032\n";
        assert_eq!(merge_key_value_pairs(input), expected);
    }

    #[test]
    fn test_is_safe_link_uri_allows_navigable_and_rejects_active_content() {
        for ok in [
            "https://example.com",
            "http://x",
            "mailto:a@b.com",
            "tel:+15551234",
            "FTP://h",
        ] {
            assert!(is_safe_link_uri(ok), "{ok} should be allowed");
        }
        for bad in [
            "javascript:alert(1)",
            "  javascript:alert(1)",
            "JavaScript:alert(1)",
            "data:text/html,<script>",
            "vbscript:msgbox",
            "file:///etc/passwd",
            "relative/path",
            "",
        ] {
            assert!(!is_safe_link_uri(bad), "{bad} should be rejected");
        }
    }

    #[test]
    fn test_key_value_merge_does_not_glue_list_items_to_headings() {
        // A heading/label must not absorb the following list item, whether
        // the item is a bullet ("- ...") or an ordered marker ("1. ...").
        let bullet = "## Highlights\n- Revenue grew steadily.\n";
        assert_eq!(merge_key_value_pairs(bullet), bullet, "bullet item glued");
        let ordered = "## Next Steps\n1. Finalize the budget.\n";
        assert_eq!(merge_key_value_pairs(ordered), ordered, "ordered item glued");
    }

    #[test]
    fn test_key_value_pair_merging_skips_long_values() {
        // A long "value" line should not be merged (it is probably a paragraph).
        let input = "Introduction\nThis is a full paragraph of text that continues.\n";
        assert_eq!(merge_key_value_pairs(input), input);
    }

    #[test]
    fn test_key_value_pair_merging_preserves_blank_lines() {
        let input = "Section A\n\nTotal\n$100\n";
        let expected = "Section A\n\nTotal $100\n";
        assert_eq!(merge_key_value_pairs(input), expected);
    }

    #[test]
    fn test_key_value_pair_merging_consecutive_pairs() {
        let input = "Subtotal\n$200.00\nTax\n$18.00\nTotal\n$218.00\n";
        let expected = "Subtotal $200.00\nTax $18.00\nTotal $218.00\n";
        assert_eq!(merge_key_value_pairs(input), expected);
    }

    #[test]
    fn test_key_value_pair_merging_euro_and_pound() {
        let input = "Price\n€49.99\nShipping\n£5.00\n";
        let expected = "Price €49.99\nShipping £5.00\n";
        assert_eq!(merge_key_value_pairs(input), expected);
    }

    #[test]
    fn test_key_value_pair_merging_parenthesized_negative() {
        let input = "Net Loss\n(1,234.56)\n";
        let expected = "Net Loss (1,234.56)\n";
        assert_eq!(merge_key_value_pairs(input), expected);
    }

    #[test]
    fn test_key_value_pair_merging_no_merge_value_value() {
        // Two consecutive value-only lines should not merge.
        let input = "$100\n$200\n";
        assert_eq!(merge_key_value_pairs(input), input);
    }

    #[test]
    fn test_key_value_pair_merging_empty_input() {
        assert_eq!(merge_key_value_pairs(""), "");
        assert_eq!(merge_key_value_pairs("single line\n"), "single line\n");
    }

    // ========================================================================
    // has_horizontal_gap CJK suppression tests (#485)
    // ========================================================================

    /// Build a minimal TextSpan for gap tests.
    ///
    /// `x` is the left edge of the span, `w` is its width, `text` is the
    /// content.  Font size is set to 10 so that the 0.15em threshold = 1.5.
    fn make_span(x: f32, w: f32, text: &str) -> crate::layout::TextSpan {
        crate::layout::TextSpan {
            text: text.to_string(),
            bbox: crate::geometry::Rect::new(x, 0.0, w, 10.0),
            font_size: 10.0,
            ..Default::default()
        }
    }

    // ========================================================================
    // sentence-fragment rejection for heading promotion
    // ========================================================================

    /// The guard is English-shaped and must only ever reject, so a heading in a
    /// language it knows nothing about has to pass untouched — including
    /// scripts with no case, where `is_lowercase()` is false for every
    /// character.
    #[test]
    fn test_heading_in_another_script_is_untouched() {
        for heading in [
            "\u{7b2c}\u{4e00}\u{7ae0}",
            "\u{627}\u{644}\u{645}\u{642}\u{62f}\u{645}\u{629}",
            "\u{41f}\u{440}\u{435}\u{434}\u{438}\u{441}\u{43b}\u{43e}\u{432}\u{438}\u{435}",
            "\u{7d50}\u{8ad6}\u{3068}\u{8003}\u{5bdf}",
        ] {
            assert!(
                !reads_as_a_sentence_fragment(heading),
                "{heading:?} must not be rejected by an English-shaped rule"
            );
        }
    }

    // ========================================================================
    // has_horizontal_gap right-to-left scoping
    // ========================================================================

    /// Build a span at an explicit x/width/size, for the RTL geometry below.
    fn rtl_span(x: f32, w: f32, fs: f32, text: &str) -> crate::layout::TextSpan {
        crate::layout::TextSpan {
            text: text.to_string(),
            bbox: crate::geometry::Rect::new(x, 0.0, w, fs),
            font_size: fs,
            ..Default::default()
        }
    }

    #[test]
    fn arabic_glyphs_stepping_leftward_are_not_separated() {
        // Measured from the page that exposed this: two glyphs of one Arabic
        // word, the second starting well left of the first and carrying a zero
        // advance. A right-to-left continuation steps leftward by definition,
        // so the backward-step discontinuity rule must not claim it.
        let prev = rtl_span(497.37, 0.0, 22.0, "\u{629}");
        let curr = rtl_span(386.46, 0.0, 22.0, "\u{632}");
        assert!(
            !has_horizontal_gap(&prev, &curr),
            "an RTL continuation is not a reading discontinuity"
        );
    }

    #[test]
    fn hebrew_glyphs_stepping_leftward_are_not_separated() {
        let prev = rtl_span(300.0, 0.0, 12.0, "\u{5e9}");
        let curr = rtl_span(288.0, 0.0, 12.0, "\u{5dc}");
        assert!(!has_horizontal_gap(&prev, &curr), "Hebrew advances right-to-left too");
    }

    #[test]
    fn test_latin_backward_step_is_still_a_discontinuity() {
        // The rule the RTL guard narrows must still fire for left-to-right
        // text, where a run beginning left of the previous run's start is a
        // new line or column — this is what stops `It is the` becoming
        // `theisIt`.
        let prev = rtl_span(300.0, 20.0, 12.0, "the");
        let curr = rtl_span(100.0, 12.0, 12.0, "It");
        assert!(
            has_horizontal_gap(&prev, &curr),
            "a Latin run starting 200pt back is a discontinuity"
        );
    }

    #[test]
    fn adjacent_latin_kerning_is_still_not_a_gap() {
        let prev = rtl_span(100.0, 18.0, 12.0, "Effi");
        let curr = rtl_span(118.1, 26.0, 12.0, "ciency");
        assert!(!has_horizontal_gap(&prev, &curr), "a sub-em gap is inter-glyph kerning");
    }

    /// The four span pairs of a Persian form's issue date, at the geometry the
    /// file actually draws. The digits are laid down left-to-right —
    /// `1403` `/` `09` `/` `19` at ascending x — and the bidi pass reverses
    /// them into reading order, so each consecutive pair in the flow steps
    /// leftward. Nothing here carries an Arabic character, so a guard keyed on
    /// right-to-left characters cannot see that this is a right-to-left run.
    ///
    /// The last pair is the tell: `/`→`1403` ends at 160.32 against a previous
    /// start of 160.22, missing the backward-step test by a tenth of a point
    /// where the other three met it. That is why the damage was asymmetric —
    /// `19 / 09 /1403`, a space before every separator but not after the last.
    #[test]
    fn test_date_in_a_right_to_left_run_is_not_split_at_its_separators() {
        let date = [
            (176.54, 11.28, "19"),
            (174.02, 2.48, "/"),
            (162.74, 11.28, "09"),
            (160.22, 2.48, "/"),
            (137.90, 22.42, "1403"),
        ];
        for pair in date.windows(2) {
            let (px, pw, pt) = pair[0];
            let (cx, cw, ct) = pair[1];
            let prev = rtl_span(px, pw, 12.0, pt);
            let curr = rtl_span(cx, cw, 12.0, ct);
            assert!(
                !has_horizontal_gap(&prev, &curr),
                "a date must not gain a break between {pt:?} and {ct:?}"
            );
        }
    }

    /// A percentage from the same page. `%` is a bidi terminator and the digits
    /// are European numbers; neither establishes a direction, so neither can
    /// justify reading a leftward step as a discontinuity.
    #[test]
    fn test_percentage_in_a_right_to_left_run_keeps_its_sign() {
        let prev = rtl_span(202.49, 5.63, 12.0, "5");
        let curr = rtl_span(197.21, 5.24, 12.0, "%");
        assert!(
            !has_horizontal_gap(&prev, &curr),
            "a percent sign must not be separated from its number"
        );
    }

    /// The counter-case that keeps the narrowing honest. Identical leftward
    /// geometry, but with strong left-to-right letters on both sides this is a
    /// genuine reading discontinuity and must still separate — it is what stops
    /// a re-ordered OCR layer emitting `It is the` as `theisIt`.
    #[test]
    fn test_latin_backward_step_with_letters_still_separates() {
        let prev = rtl_span(176.54, 11.28, 12.0, "is");
        let curr = rtl_span(160.22, 11.28, 12.0, "the");
        assert!(
            has_horizontal_gap(&prev, &curr),
            "a backward step between Latin words is still a discontinuity"
        );
    }

    #[test]
    fn test_has_horizontal_gap_cjk_cjk_suppressed() {
        // CJK char followed by CJK char with a gap > 0.15em → no space.
        let prev = make_span(0.0, 10.0, "数"); // ends with CJK
        let curr = make_span(12.0, 10.0, "学"); // starts with CJK; gap = 2.0 > 1.5
        assert!(!has_horizontal_gap(&prev, &curr), "CJK→CJK should suppress space insertion");
    }

    #[test]
    fn test_has_horizontal_gap_cjk_fullwidth_suppressed() {
        // CJK char followed by fullwidth operator → no space.
        let prev = make_span(0.0, 10.0, "Q"); // ends with ASCII (not CJK alone)
                                              // override: use a CJK ending character
        let prev_cjk = make_span(0.0, 10.0, "量");
        let curr = make_span(12.0, 10.0, "＜"); // starts with fullwidth '<'; gap = 2.0
        assert!(
            !has_horizontal_gap(&prev_cjk, &curr),
            "CJK→fullwidth-op should suppress space insertion"
        );
        let _ = prev; // silence unused warning
    }

    #[test]
    fn test_has_horizontal_gap_fullwidth_cjk_suppressed() {
        // Fullwidth operator followed by CJK char → no space.
        let prev = make_span(0.0, 10.0, "≤"); // ends with math op
        let curr = make_span(12.0, 10.0, "Q"); // pure ASCII start — not suppressed
                                               // For suppression we need curr to start with CJK
        let curr_cjk = make_span(12.0, 10.0, "量");
        assert!(
            !has_horizontal_gap(&prev, &curr_cjk),
            "fullwidth-op→CJK should suppress space insertion"
        );
        let _ = curr; // silence unused warning
    }

    #[test]
    fn test_has_horizontal_gap_latin_latin_unchanged() {
        // Latin→Latin: gap-based logic unchanged — gap > threshold → true.
        let prev = make_span(0.0, 10.0, "hello");
        let curr = make_span(12.0, 10.0, "world"); // gap = 2.0 > 1.5
        assert!(
            has_horizontal_gap(&prev, &curr),
            "Latin→Latin with gap > threshold should still insert space"
        );
    }

    #[test]
    fn test_has_horizontal_gap_latin_latin_no_gap() {
        // Latin→Latin: gap ≤ threshold → false (no change from CJK fix).
        let prev = make_span(0.0, 10.0, "hello");
        let curr = make_span(11.0, 10.0, "world"); // gap = 1.0 < 1.5
        assert!(
            !has_horizontal_gap(&prev, &curr),
            "Latin→Latin below threshold should not insert space"
        );
    }

    #[test]
    fn test_has_horizontal_gap_two_pure_math_ops_unchanged() {
        // Two pure math operators (neither is CJK): gap-based logic unchanged.
        let prev = make_span(0.0, 10.0, "≤");
        let curr = make_span(12.0, 10.0, "≥"); // gap = 2.0 > 1.5; neither is CJK
        assert!(
            has_horizontal_gap(&prev, &curr),
            "math-op→math-op (no CJK) should still apply gap-based logic"
        );
    }

    // ========================================================================
    // span_in_table cell-aware regression tests (#486 / #487)
    //
    // These guarantee that:
    //   * a span inside the outer table bbox is still "in table" when no cell
    //     bbox exists (e.g. MCID-based tagged-PDF tables, or unit-test
    //     fixtures) — preserves the legacy contract
    //   * a span inside the outer bbox but outside every cell bbox is NOT
    //     "in table" — sparse score columns whose cells never got detected
    //     fall through to paragraph flow instead of being silently dropped
    //     (issue 486 / 487)
    // ========================================================================

    fn make_table_no_cells(x: f32, y: f32, width: f32, height: f32) -> Table {
        let mut t = Table::new();
        t.bbox = Some(crate::geometry::Rect::new(x, y, width, height));
        t
    }

    fn make_table_with_cell(
        table_bbox: (f32, f32, f32, f32),
        cell_bbox: (f32, f32, f32, f32),
    ) -> Table {
        use crate::structure::table_extractor::{TableCell, TableRow};
        let mut t = Table::new();
        t.bbox = Some(crate::geometry::Rect::new(
            table_bbox.0,
            table_bbox.1,
            table_bbox.2,
            table_bbox.3,
        ));
        let mut row = TableRow::new(false);
        let mut cell = TableCell::new(String::new(), false);
        cell.bbox =
            Some(crate::geometry::Rect::new(cell_bbox.0, cell_bbox.1, cell_bbox.2, cell_bbox.3));
        row.cells.push(cell);
        t.rows.push(row);
        t.col_count = 1;
        t
    }

    fn make_ordered_span(x: f32, y: f32) -> crate::pipeline::OrderedTextSpan {
        let span = crate::layout::TextSpan {
            text: "test".to_string(),
            bbox: crate::geometry::Rect::new(x, y, 5.0, 10.0),
            font_size: 10.0,
            ..Default::default()
        };
        crate::pipeline::OrderedTextSpan::new(span, 0)
    }

    /// Span inside outer bbox of a Table that has no cells at all — legacy
    /// passthrough must still return Some.  Covers unit-test fixtures and
    /// MCID-based tagged-PDF Tables built without per-cell layout.
    #[test]
    fn span_in_table_no_cells_legacy_passthrough() {
        let table = make_table_no_cells(10.0, 50.0, 200.0, 100.0);
        let span = make_ordered_span(50.0, 70.0); // inside outer bbox
        assert_eq!(
            span_in_table(&span, &[table]),
            Some(0),
            "no-cell Table preserves legacy outer-bbox contract"
        );
    }

    /// Span inside the outer bbox AND owned by a cell → Some.
    #[test]
    fn span_in_table_owned_by_cell() {
        let table = make_table_with_cell(
            (10.0, 50.0, 200.0, 100.0), // outer
            (40.0, 60.0, 100.0, 20.0),  // cell at (40..140, 60..80)
        );
        let span = make_ordered_span(50.0, 70.0); // inside cell
        assert_eq!(span_in_table(&span, &[table]), Some(0));
    }

    /// Span inside outer bbox but outside every cell — sparse score column
    /// case from issue 486.  Must return None so paragraph flow picks it up.
    #[test]
    fn span_in_table_outer_bbox_only_returns_none() {
        let table = make_table_with_cell(
            (10.0, 50.0, 200.0, 100.0), // outer: x=10..210, y=50..150
            (10.0, 50.0, 50.0, 100.0),  // cell only covers x=10..60
        );
        // Span at x=150 sits inside outer bbox (10..210) but outside cell
        // (10..60) — represents a column the detector missed.
        let span = make_ordered_span(150.0, 70.0);
        assert_eq!(
            span_in_table(&span, &[table]),
            None,
            "span outside every cell must NOT be marked in_table — \
             paragraph flow needs to pick it up instead of dropping"
        );
    }

    /// Span outside every table's outer bbox → None.
    #[test]
    fn span_in_table_outside_all_tables() {
        let table = make_table_with_cell((10.0, 50.0, 200.0, 100.0), (40.0, 60.0, 100.0, 20.0));
        let span = make_ordered_span(500.0, 500.0);
        assert_eq!(span_in_table(&span, &[table]), None);
    }

    // ========================================================================
    // reference-marker boundary
    // ========================================================================

    /// A span at an explicit x/width/size/font, for the marker geometry below.
    fn marker_span(x: f32, w: f32, fs: f32, font: &str, text: &str) -> crate::layout::TextSpan {
        crate::layout::TextSpan {
            text: text.to_string(),
            bbox: crate::geometry::Rect::new(x, 0.0, w, fs),
            font_size: fs,
            font_name: font.to_string(),
            ..Default::default()
        }
    }

    /// The measured geometry from a real paper: the marker sits 0.99 pt after
    /// the word at 9.96 pt body size, i.e. 0.10 em — well inside the 0.15 em
    /// bar, so `has_horizontal_gap` declines and the two glue as
    /// `phosphorylation55`.
    #[test]
    fn test_footnote_marker_after_a_word_is_a_boundary() {
        let word = marker_span(124.60, 189.36, 9.96, "TWJDIY+SFRM1000", "phosphorylation");
        let marker = marker_span(314.95, 6.0, 6.97, "MIOKXQ+SFRM0700", "55.");
        assert!(
            !has_horizontal_gap(&word, &marker),
            "precondition: the gap rule must decline this, or the test proves nothing"
        );
        assert!(
            is_reference_marker_boundary(&word, &marker),
            "a numeral set smaller after a prose word is a reference callout"
        );
    }

    /// The counter-case, and the reason gap size cannot be the discriminator:
    /// a maths subscript sits *further* from its base (0.14 em) than the
    /// footnote marker does (0.10 em), so any threshold that splits the marker
    /// fuses this. The base being a symbol rather than a prose word is what
    /// separates them — the same rule `merge_sub_superscript_spans` uses.
    #[test]
    fn test_maths_subscript_is_not_a_boundary() {
        for (base, sub) in [
            ("W", "2"),
            ("H", "2"),
            ("ADP", "3"),
            ("SO", "4"),
            ("x", "2"),
        ] {
            let b = marker_span(100.0, 8.0, 11.96, "BODY+Font", base);
            let s = marker_span(109.63, 4.0, 6.97, "SUB+Font", sub);
            assert!(
                !is_reference_marker_boundary(&b, &s),
                "{base}+{sub} is a subscript on a symbol host and must stay joined"
            );
        }
    }

    /// An italic base is a mathematical expression, not prose.
    #[test]
    fn test_italic_base_is_not_a_prose_word() {
        let mut base = marker_span(100.0, 30.0, 10.0, "BODY+Font", "alpha");
        base.is_italic = true;
        let marker = marker_span(131.0, 5.0, 6.5, "SUB+Font", "2");
        assert!(!is_reference_marker_boundary(&base, &marker));
    }

    /// The marker must be a numeral. A letter following a word at the advance
    /// edge is a glyph-split word, not a callout, and must stay joined.
    #[test]
    fn test_letter_continuation_is_not_a_marker() {
        let base = marker_span(100.0, 30.0, 10.0, "BODY+Font", "ultrasonographi");
        let cont = marker_span(131.0, 20.0, 10.0, "BODY+Font", "cally");
        assert!(!is_reference_marker_boundary(&base, &cont));
    }

    /// Same font means one run, whatever the size — no callout.
    #[test]
    fn test_same_font_resource_is_not_a_boundary() {
        let base = marker_span(100.0, 30.0, 10.0, "SAME+Font", "phosphorylation");
        let marker = marker_span(131.0, 5.0, 6.5, "SAME+Font", "55");
        assert!(!is_reference_marker_boundary(&base, &marker));
    }
}

#[cfg(test)]
mod span_ownership_tests {
    use super::*;
    use crate::layout::TextSpan;
    use crate::pipeline::ordered_span::OrderedTextSpan;
    use crate::structure::table_extractor::{Table, TableCell, TableRow};

    fn span_at(text: &str, x: f32, y: f32, width: f32) -> TextSpan {
        TextSpan {
            text: text.to_string(),
            bbox: crate::geometry::Rect {
                x,
                y,
                width,
                height: 10.0,
            },
            font_size: 10.0,
            ..Default::default()
        }
    }

    /// A table whose single populated cell renders one run, plus an empty
    /// placeholder cell in the lattice beside it. `bbox` covers both.
    fn table_with_a_placeholder_cell() -> Table {
        let mut populated = TableCell::new("Region".to_string(), false);
        populated.bbox = Some(crate::geometry::Rect {
            x: 100.0,
            y: 700.0,
            width: 60.0,
            height: 12.0,
        });
        populated.spans = vec![span_at("Region", 102.0, 701.0, 34.0)];

        // The lattice square the detector left empty — no text was placed in it.
        let mut placeholder = TableCell::new(String::new(), false);
        placeholder.bbox = Some(crate::geometry::Rect {
            x: 160.0,
            y: 700.0,
            width: 60.0,
            height: 12.0,
        });

        let mut row = TableRow::new(false);
        row.add_cell(populated);
        row.add_cell(placeholder);

        let mut table = Table::new();
        table.add_row(row);
        table.bbox = Some(crate::geometry::Rect {
            x: 100.0,
            y: 700.0,
            width: 120.0,
            height: 12.0,
        });
        table
    }

    /// The loss half. A span whose origin lands in an empty lattice square is
    /// rendered by no cell, so suppressing it from prose deletes it from the
    /// document. The origin test said it belonged to the table; the cell's own
    /// ink says otherwise, and the ink is what gets rendered.
    #[test]
    fn test_span_no_cell_renders_is_left_to_prose() {
        let table = table_with_a_placeholder_cell();
        let orphan = OrderedTextSpan::new(span_at("footnote", 165.0, 703.0, 40.0), 0);

        assert_eq!(
            span_in_table(&orphan, std::slice::from_ref(&table)),
            None,
            "a span sitting in an empty lattice square was suppressed from prose \
             and is rendered by nobody, so it is lost from every surface"
        );
    }

    /// The counter-case, and the reason the predicate cannot simply answer
    /// `None`: a span the cell really does render must stay out of prose, or it
    /// appears twice.
    #[test]
    fn test_span_a_cell_renders_is_claimed_by_it() {
        let table = table_with_a_placeholder_cell();
        let owned = OrderedTextSpan::new(span_at("Region", 102.0, 701.0, 34.0), 0);

        assert_eq!(
            span_in_table(&owned, std::slice::from_ref(&table)),
            Some(0),
            "a span the cell renders was also emitted into prose, so it appears twice"
        );
    }

    /// A tagged table answers from marked content, which is exact — and it
    /// answers both ways, so an unclaimed span is not swept up by geometry.
    #[test]
    fn marked_content_decides_a_tagged_table_both_ways() {
        let mut cell = TableCell::new("Region".to_string(), false);
        cell.mcids = vec![7];
        cell.bbox = Some(crate::geometry::Rect {
            x: 100.0,
            y: 700.0,
            width: 60.0,
            height: 12.0,
        });
        let mut row = TableRow::new(false);
        row.add_cell(cell);
        let mut table = Table::new();
        table.add_row(row);
        table.bbox = Some(crate::geometry::Rect {
            x: 100.0,
            y: 700.0,
            width: 120.0,
            height: 12.0,
        });

        let mut inside = span_at("Region", 102.0, 701.0, 34.0);
        inside.mcid = Some(7);
        assert_eq!(
            span_in_table(&OrderedTextSpan::new(inside, 0), std::slice::from_ref(&table)),
            Some(0)
        );

        let mut other = span_at("caption", 102.0, 701.0, 34.0);
        other.mcid = Some(9);
        assert_eq!(
            span_in_table(&OrderedTextSpan::new(other, 0), std::slice::from_ref(&table)),
            None,
            "a span the table's marked content does not claim must reach prose, \
             even though it sits inside the table's bbox"
        );
    }

    /// A table carrying neither marked content nor cell spans still falls back
    /// to the geometric rule, which is all there is for it.
    #[test]
    fn test_bare_table_still_uses_the_geometric_rule() {
        let mut table = Table::new();
        table.add_row(TableRow::new(false));
        table.bbox = Some(crate::geometry::Rect {
            x: 100.0,
            y: 700.0,
            width: 120.0,
            height: 12.0,
        });
        let inside = OrderedTextSpan::new(span_at("Region", 102.0, 701.0, 34.0), 0);
        assert_eq!(span_in_table(&inside, std::slice::from_ref(&table)), Some(0));
    }
}
