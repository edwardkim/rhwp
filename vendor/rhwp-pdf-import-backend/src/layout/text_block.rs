//! Text block representation for layout analysis.
//!
//! This module defines structures for representing text elements in a PDF document
//! with their geometric and styling information.

use crate::extractors::text::ArtifactType;
use crate::geometry::{Point, Rect};
use crate::structure::McidScope;
use std::collections::HashMap;

/// A text span (complete string from a Tj/TJ operator).
///
/// This represents text as the PDF specification provides it - complete strings
/// from text showing operators, not individual characters. This is the correct
/// approach per PDF spec ISO 32000-1:2008.
///
/// Extracting complete strings instead of individual characters:
/// - Avoids overlapping character issues
/// - Preserves PDF's text positioning intent
/// - Matches industry best practices
/// - More robust for complex layouts
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct TextSpan {
    /// The complete text string
    pub text: String,
    /// Bounding box of the entire span in PDF coordinates (points)
    pub bbox: Rect,
    /// Font name/family
    pub font_name: String,
    /// Font size in points
    pub font_size: f32,
    /// Font weight (normal or bold)
    pub font_weight: FontWeight,
    /// Font style: italic or normal
    pub is_italic: bool,
    /// Whether the font is monospaced (from PDF font descriptor FixedPitch flag)
    pub is_monospace: bool,
    /// Text color
    pub color: Color,
    /// Marked Content ID (for Tagged PDFs)
    pub mcid: Option<u32>,
    /// Content-stream scope of [`Self::mcid`] (ISO 32000-1:2008 §14.7.4.3).
    ///
    /// MCIDs are scoped to a single content stream — page, Form
    /// XObject, or Tiling Pattern — not to a page globally. When this
    /// span's `mcid` was emitted inside a Form XObject's content
    /// stream, `mcid_scope` is `Form(<form_ref>)`; inside a Tiling
    /// Pattern, `Pattern(<pattern_ref>)`; otherwise `Page(page_index)`
    /// for the page that owns the top-level content stream the span
    /// came from.
    ///
    /// The struct-tree `/ActualText` applier keys lookups by
    /// `(mcid_scope, mcid)` so two Form XObjects on the same page that
    /// each carry MCID 0 do not collide and overwrite each other's
    /// replacements.
    ///
    /// `None` for spans extracted before page-index attribution
    /// completes (e.g. mid-extraction internal spans) or for synthetic
    /// test fixtures.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub mcid_scope: Option<McidScope>,
    /// Extraction sequence number
    pub sequence: usize,
    /// If true, this span was created by splitting fused words
    pub split_boundary_before: bool,
    /// If true, this span was created by the TJ processor as a space
    pub offset_semantic: bool,
    /// Character spacing (Tc parameter)
    pub char_spacing: f32,
    /// Word spacing (Tw parameter)
    pub word_spacing: f32,
    /// Horizontal scaling (Tz parameter)
    pub horizontal_scaling: f32,
    /// If true, was created by WordBoundaryDetector primary detection.
    pub primary_detected: bool,
    /// Artifact type classification for filtered content (PDF Spec Section 14.8.2.2)
    pub artifact_type: Option<ArtifactType>,
    /// Per-character advance widths in user-space points.
    /// When non-empty and matching text length, to_chars() uses these
    /// for accurate per-glyph bounding boxes instead of uniform division.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub char_widths: Vec<f32>,
    /// Accurate per-glyph baseline x-origins (user-space points), sourced from
    /// the spec-aligned char-level extractor that `extract_chars` uses.
    ///
    /// When non-empty and matching `text.chars().count()`, [`Self::to_chars`]
    /// uses these directly for each glyph's `origin_x` / `bbox.x` instead of
    /// prefix-summing the nominal `char_widths` from `bbox.x`. The nominal
    /// widths omit ISO 32000-1:2008 §9.4.3 TJ-array kerning adjustments and the
    /// full §9.4.4 text-space displacement, so prefix-summing them drifts
    /// cumulatively along a line; these offsets carry the real positions and so
    /// do not drift. Empty (the default) preserves the legacy prefix-sum path
    /// byte-for-byte.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub char_x_offsets: Vec<f32>,
    /// Heading level (1-6) when this span belongs to a document heading.
    /// Populated either from the source PDF's structure tree
    /// (`StructRole::Heading(n)`) or from a font-size-ratio heuristic when
    /// the PDF is untagged. Layout-preserving DOCX export uses this to
    /// emit `<w:pStyle w:val="HeadingN"/>` so the output document
    /// preserves heading semantics for accessibility, navigation, and
    /// outline panes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_level: Option<u8>,
    /// Display rotation of the run in degrees, from `atan2(b, a)` of the composed
    /// text rendering matrix (`T_m × CTM`, ISO 32000-1 §9.4.4), normalised so a
    /// near-quadrant angle snaps to `0` / `90` / `180` / `-90`. `0.0` for ordinary
    /// horizontal text. Reading order segregates non-zero-rotation runs out of the
    /// horizontal flow so they are ordered as their own blocks rather than
    /// interleaved (the axis-aligned assumptions in the row-band / XY-cut sort do
    /// not hold for rotated text).
    #[serde(skip_serializing_if = "is_zero_f32", default)]
    pub rotation_degrees: f32,
    /// Writing mode under which the glyphs in this span were emitted.
    ///
    /// `0` = horizontal (the overwhelming default), `1` = vertical (tategaki
    /// / lateral CJK). Set from `GraphicsState::text_wmode` when the span
    /// is constructed, so each span carries its own writing-mode metadata
    /// even on mixed-mode pages (e.g. horizontal headings above vertical
    /// body copy). The reading-order sort consults this to advance
    /// downward-then-right-to-left within blocks of vertical spans while
    /// leaving horizontal spans on their existing top-to-bottom,
    /// left-to-right path.
    #[serde(skip_serializing_if = "is_zero_u8", default)]
    pub wmode: u8,
    /// Baseline shift of this run as a ratio of font size (`Ts ÷ font_size`),
    /// from the text-rise text-state parameter (ISO 32000-1 §9.3.7). `Ts > 0`
    /// raises the baseline (superscript), `Ts < 0` lowers it (subscript); `0.0`
    /// for ordinary on-baseline text. Stored as a ratio (rather than raw points)
    /// so it is independent of the text/CTM scale and directly comparable to the
    /// font-size-ratio used by the sub/superscript rejoin. Reading order uses a
    /// non-zero value as the authoritative, untagged-available signal that the
    /// run is an off-baseline super/subscript to be rejoined inline rather than
    /// split onto its own line by the row-band sort.
    #[serde(skip_serializing_if = "is_zero_f32", default)]
    pub text_rise: f32,
    /// True when this span's glyphs were drawn **right-to-left** — successive
    /// glyphs placed at decreasing x (the producer stored RTL text in LOGICAL
    /// order and positioned each glyph individually, ISO 32000-1 §14.8.2.3.3
    /// method 1). Such a span's characters are already in logical order, so the
    /// structure-path `push_span_text_bidi` must NOT apply its visual→logical
    /// character reversal to it. Detected from raw draw geometry before any
    /// reading-order sort (`detect_rtl_draw_direction`) and OR-ed through
    /// `merge_adjacent_spans`. Default `false` = VISUAL storage (glyphs drawn
    /// left-to-right, the common case), kept byte-identical. The draw direction
    /// is the only signal that separates logical- from visual-stored RTL when
    /// both use base-form characters (no presentation forms, no `/ReversedChars`).
    #[serde(skip_serializing_if = "is_false", default)]
    pub rtl_draw_logical: bool,
    /// True when the composed text rendering matrix has a negative
    /// determinant — the run is mirrored, so its glyph-up axis is the writing
    /// axis *reflected*, not rotated (`snap_run_rotation` likewise refuses to
    /// snap these so they are never confused with a clean rotation).
    /// [`Self::page_bbox`] reflects the across-axis for such runs instead of
    /// rotating it. Runtime metadata — deliberately not serialized.
    #[serde(skip)]
    pub mirrored: bool,
    /// Clockwise page `/Rotate` (`0`/`90`/`180`/`270`) already folded into
    /// [`Self::bbox`] by span postprocessing; `0` means `bbox` is raw user
    /// space. On a `/Rotate`d page rotated-content bboxes are rewritten into
    /// the displayed frame while `rotation_degrees` keeps the raw content
    /// angle (it is the marker the reading-order passes select on), so
    /// [`Self::page_bbox`] needs this to reconstruct the run frame without
    /// double-transforming (#806). Runtime metadata — deliberately not
    /// serialized.
    #[serde(skip)]
    pub page_rotation_applied: i32,
    /// Provenance of this span's Unicode text — which ISO 32000-1 §9.10.2
    /// mapping tier the font offered, as a
    /// [`MappingProvenance`](crate::fonts::MappingProvenance). `None` when the
    /// span was not produced by the extractor (synthetic/test spans). A
    /// [`Fallback`](crate::fonts::MappingProvenance::Fallback) value means the
    /// font carried no mapping resource, so the text is a fabricated glyph-index
    /// echo, not read from the file. Runtime metadata only — deliberately not
    /// serialized (kept out of span JSON so existing output is byte-identical);
    /// bindings surface it through explicit accessors.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provenance: Option<crate::fonts::MappingProvenance>,
}

/// serde skip helper: omit a `false` flag (the common case) from serialized output.
pub(crate) fn is_false(v: &bool) -> bool {
    !*v
}

/// serde skip helper: omit a `0` writing mode (horizontal, the common case)
/// from serialized output so existing fixtures stay unchanged.
pub(crate) fn is_zero_u8(v: &u8) -> bool {
    *v == 0
}

/// serde skip helper: omit a `0.0` rotation (the overwhelming common case) from
/// serialized output so existing fixtures stay unchanged.
pub(crate) fn is_zero_f32(v: &f32) -> bool {
    *v == 0.0
}

impl Default for TextSpan {
    fn default() -> Self {
        Self {
            text: String::new(),
            bbox: Rect::default(),
            font_name: "Helvetica".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence: 0,
            split_boundary_before: false,
            offset_semantic: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            primary_detected: false,
            artifact_type: None,
            char_widths: Vec::new(),
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: 0.0,
            wmode: 0,
            text_rise: 0.0,
            rtl_draw_logical: false,
            mirrored: false,
            page_rotation_applied: 0,
            provenance: None,
        }
    }
}

impl TextSpan {
    /// Where the run physically sits on the page.
    ///
    /// [`Self::bbox`] carries the run's own extents: `width` is the advance
    /// along the writing axis and `height` the font size, whatever the
    /// rotation. That is the right frame for measuring the text, and it is the
    /// frame every existing consumer already reads, so it is left alone — but
    /// it means a run drawn with `Tm [0 1 -1 0]` reports a wide, short box for
    /// text that is physically tall and narrow.
    ///
    /// This rotates those extents about the run origin into the same frame
    /// `bbox` lives in. At `rotation_degrees == 0` it returns `bbox`
    /// unchanged, so upright pages cannot move; free angles get the
    /// axis-aligned hull of the rotated box. On a `/Rotate`d page, where span
    /// postprocessing already rewrote `bbox` into the displayed frame
    /// ([`Self::page_rotation_applied`]), the hull is built in that displayed
    /// frame rather than re-rotating the already-mapped rect.
    ///
    /// Mirrors [`crate::elements::PathContent::rendered_bbox`]: a derived accessor
    /// rather than a second stored rectangle that could drift out of sync.
    pub fn page_bbox(&self) -> Rect {
        if self.rotation_degrees == 0.0 {
            return self.bbox;
        }
        // Conjugate the run frame by the clockwise page rotation already
        // applied to `bbox`: the run origin's image is a fixed, known corner
        // of the mapped rect; a rotation conjugates to itself, a mirror to
        // `θ - 2·rot`.
        let rot = self.page_rotation_applied.rem_euclid(360);
        let theta = if self.mirrored {
            self.rotation_degrees - 2.0 * rot as f32
        } else {
            self.rotation_degrees
        }
        .to_radians();
        let (sin, cos) = theta.sin_cos();
        // Writing axis and the across-line axis, the same pair the assembler
        // resolves displacements onto (ISO 32000-1 §9.4.4). A mirrored run
        // (negative determinant) carries its across-axis on the clockwise
        // side of the baseline: v is reflected, not rotated.
        let (ux, uy) = (cos, sin);
        let (vx, vy) = if self.mirrored {
            (sin, -cos)
        } else {
            (-sin, cos)
        };
        let b = self.bbox;
        // Image of the run origin under the applied page rotation — the
        // anchor the run frame pivots about in `bbox`'s current frame.
        let (px, py) = match rot {
            90 => (b.x, b.y + b.height),
            180 => (b.x + b.width, b.y + b.height),
            270 => (b.x + b.width, b.y),
            _ => (b.x, b.y),
        };
        let corners = [
            (b.x, b.y),
            (b.x + b.width, b.y),
            (b.x, b.y + b.height),
            (b.x + b.width, b.y + b.height),
        ]
        .map(|(cx, cy)| {
            let (dx, dy) = (cx - px, cy - py);
            (px + dx * ux + dy * vx, py + dx * uy + dy * vy)
        });

        let min_x = corners.iter().map(|c| c.0).fold(f32::INFINITY, f32::min);
        let max_x = corners
            .iter()
            .map(|c| c.0)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = corners.iter().map(|c| c.1).fold(f32::INFINITY, f32::min);
        let max_y = corners
            .iter()
            .map(|c| c.1)
            .fold(f32::NEG_INFINITY, f32::max);
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Decompose the span into individual characters.
    pub fn to_chars(&self) -> Vec<TextChar> {
        let char_count = self.text.chars().count();
        if char_count == 0 {
            return Vec::new();
        }

        // Preferred path: accurate per-glyph x-origins captured from the
        // spec-aligned char extractor (ISO 32000-1:2008 §9.4.3 / §9.4.4). These
        // carry the real text-space positions, so they do not accumulate the
        // TJ-kerning drift that prefix-summing `char_widths` from `bbox.x`
        // produces. Only taken when the offsets cover every glyph exactly.
        const OFFSET_BBOX_TOLERANCE: f32 = 0.5;
        let offsets_fit_bbox = self.char_x_offsets.iter().all(|&x| {
            x.is_finite()
                && x >= self.bbox.x - OFFSET_BBOX_TOLERANCE
                && x <= self.bbox.x + self.bbox.width + OFFSET_BBOX_TOLERANCE
        });
        if self.char_x_offsets.len() == char_count && offsets_fit_bbox {
            let has_widths = self.char_widths.len() == char_count;
            let offsets = &self.char_x_offsets;
            return self
                .text
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    let char_x = offsets[i];
                    // Width preference: the gap to the next offset, then
                    // nominal `char_widths`, then the remainder of the span
                    // bbox for the final glyph.
                    //
                    // The offsets are measured glyph origins, so the distance
                    // between two of them *is* the first one's advance.
                    // `char_widths` is meant to be the same quantity, and when
                    // the two disagree it is the bookkeeping that has drifted:
                    // on a table-of-contents line containing an em dash the
                    // widths ran two entries out of step with the text, pairing
                    // `C` with a 2.67 pt advance where its own is 5.83 while
                    // its offset was exactly right. Those phantom gaps are what
                    // the word-gap clusterer splits on, and
                    // `Coast Guard, Department` came out as
                    // `C|o|ast G|u|ard, D|e|partm|ent`.
                    //
                    // A non-increasing pair is not an advance — visually-stored
                    // RTL and mirrored runs backtrack — so those fall through to
                    // the nominal width unchanged.
                    //
                    // The distance to the next origin is the first glyph's
                    // advance only while the two glyphs are consecutive on one
                    // run. A `Tm` that repositions inside the text object puts
                    // the next origin an arbitrary distance away, and taking
                    // that as an advance makes the glyph before it as wide as
                    // the jump — which closes the gap the word clusterer splits
                    // on. Two footer words set 40 pt apart came out as one word.
                    //
                    // ISO 32000-1:2008 §9.4.4 makes the advance the glyph's own
                    // displacement, and a glyph's displacement is bounded by its
                    // design width: an em is the widest ordinary advance, and
                    // even a stretched inter-word space on justified text stays
                    // well inside `MAX_ADVANCE_EM`. Beyond it the distance is
                    // not an advance, so the nominal width stands.
                    const MAX_ADVANCE_EM: f32 = 1.5;
                    let advance_ceiling = self.font_size * MAX_ADVANCE_EM;
                    let next_gap = if i + 1 < char_count {
                        let d = offsets[i + 1] - char_x;
                        (d.is_finite() && d > 0.0 && d <= advance_ceiling).then_some(d)
                    } else {
                        None
                    };
                    let w = match (next_gap, has_widths) {
                        (Some(d), _) => d,
                        (None, true) => self.char_widths[i],
                        (None, false) => (self.bbox.x + self.bbox.width - char_x).max(0.0),
                    };
                    TextChar {
                        char: c,
                        bbox: Rect::new(char_x, self.bbox.y, w, self.bbox.height),
                        font_name: self.font_name.clone(),
                        font_size: self.font_size,
                        font_weight: self.font_weight,
                        is_italic: self.is_italic,
                        is_monospace: self.is_monospace,
                        color: self.color,
                        mcid: self.mcid,
                        origin_x: char_x,
                        origin_y: self.bbox.y,
                        rotation_degrees: self.rotation_degrees,
                        advance_width: w,
                        rendered_advance: w,
                        ascent: 0.95 * self.font_size,
                        descent: -0.35 * self.font_size,
                        matrix: Some([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
                    }
                })
                .collect();
        }

        // Use per-character widths when available and matching text length;
        // otherwise fall back to uniform division (backward compatible).
        if self.char_widths.len() == char_count {
            let mut x = self.bbox.x;
            self.text
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    let w = self.char_widths[i];
                    let char_x = x;
                    x += w;
                    TextChar {
                        char: c,
                        bbox: Rect::new(char_x, self.bbox.y, w, self.bbox.height),
                        font_name: self.font_name.clone(),
                        font_size: self.font_size,
                        font_weight: self.font_weight,
                        is_italic: self.is_italic,
                        is_monospace: self.is_monospace,
                        color: self.color,
                        mcid: self.mcid,
                        origin_x: char_x,
                        origin_y: self.bbox.y,
                        rotation_degrees: self.rotation_degrees,
                        advance_width: w,
                        rendered_advance: w,
                        ascent: 0.95 * self.font_size,
                        descent: -0.35 * self.font_size,
                        matrix: Some([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
                    }
                })
                .collect()
        } else {
            let char_width = self.bbox.width / (char_count as f32);
            self.text
                .chars()
                .enumerate()
                .map(|(i, c)| TextChar {
                    char: c,
                    bbox: Rect::new(
                        self.bbox.x + (i as f32) * char_width,
                        self.bbox.y,
                        char_width,
                        self.bbox.height,
                    ),
                    font_name: self.font_name.clone(),
                    font_size: self.font_size,
                    font_weight: self.font_weight,
                    is_italic: self.is_italic,
                    is_monospace: self.is_monospace,
                    color: self.color,
                    mcid: self.mcid,
                    origin_x: self.bbox.x + (i as f32) * char_width,
                    origin_y: self.bbox.y,
                    rotation_degrees: self.rotation_degrees,
                    advance_width: char_width,
                    rendered_advance: char_width,
                    ascent: 0.95 * self.font_size,
                    descent: -0.35 * self.font_size,
                    matrix: Some([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
                })
                .collect()
        }
    }
}

/// A single character with its position and styling.
///
/// NOTE: This is kept for backward compatibility and special use cases.
/// For normal text extraction, prefer TextSpan which represents complete
/// text strings as the PDF provides them.
///
/// ## Transformation Properties (v0.3.1+)
///
/// TextChar now includes transformation information for precise text positioning:
/// - `origin_x`, `origin_y`: Baseline position (where the character sits)
/// - `rotation_degrees`: Text rotation angle
/// - `advance_width`: Horizontal distance to next character
/// - `matrix`: Full 6-element transformation matrix for advanced use cases
///
/// These properties match industry standards.
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct TextChar {
    /// The character itself
    pub char: char,
    /// Bounding box of the character
    pub bbox: Rect,
    /// Font name/family
    pub font_name: String,
    /// Font size in points
    pub font_size: f32,
    /// Font weight (normal or bold)
    pub font_weight: FontWeight,
    /// Font style: italic or normal
    pub is_italic: bool,
    /// Whether the font is monospaced (from PDF font descriptor FixedPitch flag)
    pub is_monospace: bool,
    /// Text color
    pub color: Color,
    /// Marked Content ID (for Tagged PDFs)
    ///
    /// This field stores the MCID if this character was extracted within
    /// a marked content sequence in a Tagged PDF.
    pub mcid: Option<u32>,

    // === Transformation properties (v0.3.1, Issue #27) ===
    /// Baseline origin X coordinate.
    ///
    /// This is the X position where the character's baseline starts,
    /// which is the standard reference point for text positioning in PDFs.
    /// Unlike bbox.x which is the left edge of the glyph, origin_x is
    /// the typographic origin point.
    pub origin_x: f32,

    /// Baseline origin Y coordinate.
    ///
    /// This is the Y position of the character's baseline. For horizontal
    /// text, this is where the bottom of letters like 'a', 'x' sit, while
    /// letters with descenders like 'g', 'y' extend below this line.
    pub origin_y: f32,

    /// Rotation angle in degrees (0-360, clockwise from horizontal).
    ///
    /// Calculated from the text transformation matrix using atan2(b, a).
    /// - 0° = normal horizontal text (left to right)
    /// - 90° = vertical text (top to bottom)
    /// - 180° = upside down text
    /// - 270° = vertical text (bottom to top)
    pub rotation_degrees: f32,

    /// Horizontal advance width (distance to next character position).
    ///
    /// Glyph advance width from font metrics (device space).
    ///
    /// This is the advance for the glyph shape only — it does **not** include
    /// character spacing (Tc), word spacing (Tw), or TJ array adjustments.
    /// For word-boundary detection and the full cursor advance including all
    /// spacing, use [`Self::rendered_advance`].
    pub advance_width: f32,

    /// Actual rendered advance to the next character's origin (device space).
    ///
    /// This is the per-glyph cursor advance including character spacing (Tc)
    /// and word spacing (Tw for U+0020), per the PDF spec Tx formula:
    /// `(w0 × Tfs / 1000 + Tc + Tw) × Th` converted to device space.
    ///
    /// TJ array adjustments between strings are **not** folded into this
    /// field.  They are emitted as separate synthetic-space [`TextChar`]s
    /// inserted between the glyphs they affect, so the overall cursor
    /// displacement is correctly represented by walking the full char list.
    ///
    /// Equivalent to Poppler's `dx` argument in `drawChar`.
    ///
    /// For the last character on a line this falls back to `advance_width`.
    /// Use this field (not `advance_width`) to detect word boundaries:
    /// a gap `next.origin_x − (this.origin_x + this.rendered_advance) > threshold`
    /// reliably identifies inter-word spacing.
    pub rendered_advance: f32,

    /// Distance from the baseline to the top of the typographic glyph box (device space).
    ///
    /// From the font descriptor `/Ascent`; falls back to Adobe AFM values for the 14
    /// standard PDF fonts, then to 0.95 × font_size (Poppler's default).
    ///
    /// `bbox.height` is the full em square and does not reflect the font's actual cap
    /// height. Use `origin_y + ascent` for the glyph's true top edge.
    pub ascent: f32,

    /// Distance from the baseline to the bottom of the typographic glyph box (device space, negative).
    ///
    /// From the font descriptor `/Descent`; falls back to Adobe AFM values for the 14
    /// standard PDF fonts, then to −0.35 × font_size (Poppler's default).
    ///
    /// `bbox` does not represent the descender region at all (its origin is the
    /// baseline). Use `origin_y + descent` for the glyph's true bottom edge.
    pub descent: f32,

    /// Full transformation matrix [a, b, c, d, e, f].
    ///
    /// The composed text matrix (CTM × Tm) that transforms this character
    /// from text space to device space. Provides complete transformation
    /// info for advanced use cases like re-rendering or precise positioning.
    ///
    /// Matrix layout:
    /// ```text
    /// [ a  b  0 ]
    /// [ c  d  0 ]
    /// [ e  f  1 ]
    /// ```
    /// Where (a,d) = scaling, (b,c) = rotation/skew, (e,f) = translation.
    pub matrix: Option<[f32; 6]>,
}

impl Default for TextChar {
    fn default() -> Self {
        Self {
            char: ' ',
            bbox: Rect::default(),
            font_name: "Helvetica".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            origin_x: 0.0,
            origin_y: 0.0,
            rotation_degrees: 0.0,
            advance_width: 0.0,
            rendered_advance: 0.0,
            ascent: 0.95 * 12.0,
            descent: -0.35 * 12.0,
            matrix: Some([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        }
    }
}

impl TextChar {
    /// Get the rotation angle in radians.
    pub fn rotation_radians(&self) -> f32 {
        self.rotation_degrees.to_radians()
    }

    /// Check if this character is rotated (non-zero rotation).
    pub fn is_rotated(&self) -> bool {
        self.rotation_degrees.abs() > 0.01
    }

    /// Set the transformation matrix and update derived values.
    ///
    /// This method sets the full transformation matrix and automatically
    /// calculates the rotation angle and origin from the matrix components.
    ///
    /// # Arguments
    ///
    /// * `matrix` - A 6-element transformation matrix [a, b, c, d, e, f]
    pub fn with_matrix(mut self, matrix: [f32; 6]) -> Self {
        self.matrix = Some(matrix);
        // Extract origin from translation components
        self.origin_x = matrix[4];
        self.origin_y = matrix[5];
        // Calculate rotation from matrix: atan2(b, a)
        self.rotation_degrees = matrix[1].atan2(matrix[0]).to_degrees();
        self
    }

    /// Get the transformation matrix, computing from basic values if not stored.
    ///
    /// If the matrix was stored during extraction, returns it directly.
    /// Otherwise, reconstructs a basic matrix from origin and rotation.
    ///
    /// # Returns
    ///
    /// A 6-element transformation matrix [a, b, c, d, e, f]
    pub fn get_matrix(&self) -> [f32; 6] {
        if let Some(m) = self.matrix {
            m
        } else {
            // Reconstruct matrix from rotation and origin
            let rad = self.rotation_radians();
            let cos_r = rad.cos();
            let sin_r = rad.sin();
            [cos_r, sin_r, -sin_r, cos_r, self.origin_x, self.origin_y]
        }
    }

    /// Create a simple TextChar with default transformation values.
    ///
    /// This is a convenience constructor for creating TextChar instances
    /// when transformation data is not available (e.g., programmatic creation).
    /// The origin defaults to the bbox position, rotation to 0, and
    /// advance_width to the bbox width.
    pub fn simple(char: char, bbox: Rect, font_name: String, font_size: f32) -> Self {
        Self {
            char,
            bbox,
            font_name,
            font_size,
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            origin_x: bbox.x,
            origin_y: bbox.y,
            rotation_degrees: 0.0,
            advance_width: bbox.width,
            rendered_advance: bbox.width,
            ascent: 0.95 * font_size,
            descent: -0.35 * font_size,
            matrix: None,
        }
    }
}

/// Font weight classification following PDF spec numeric scale.
///
/// PDF Spec: ISO 32000-1:2008, Table 122 - FontDescriptor
/// Values: 100-900 where 400 = normal, 700 = bold
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, Default)]
#[repr(u16)]
pub enum FontWeight {
    /// Thin (100)
    Thin = 100,
    /// Extra Light (200)
    ExtraLight = 200,
    /// Light (300)
    Light = 300,
    /// Normal (400) - default weight
    #[default]
    Normal = 400,
    /// Medium (500)
    Medium = 500,
    /// Semi Bold (600)
    SemiBold = 600,
    /// Bold (700) - standard bold weight
    Bold = 700,
    /// Extra Bold (800)
    ExtraBold = 800,
    /// Black (900) - heaviest weight
    Black = 900,
}

impl FontWeight {
    /// Check if this weight is considered bold (>= 600).
    ///
    /// Per PDF spec, weights 600+ are semi-bold or bolder.
    pub fn is_bold(&self) -> bool {
        *self as u16 >= 600
    }

    /// Create FontWeight from PDF numeric value.
    ///
    /// Rounds to nearest standard weight value.
    pub fn from_pdf_value(value: i32) -> Self {
        match value {
            ..=150 => FontWeight::Thin,
            151..=250 => FontWeight::ExtraLight,
            251..=350 => FontWeight::Light,
            351..=450 => FontWeight::Normal,
            451..=550 => FontWeight::Medium,
            551..=650 => FontWeight::SemiBold,
            651..=750 => FontWeight::Bold,
            751..=850 => FontWeight::ExtraBold,
            851.. => FontWeight::Black,
        }
    }

    /// Get the numeric PDF value for this weight.
    pub fn to_pdf_value(&self) -> u16 {
        *self as u16
    }
}

/// RGB color representation.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, Default)]
pub struct Color {
    /// Red channel (0.0 - 1.0)
    pub r: f32,
    /// Green channel (0.0 - 1.0)
    pub g: f32,
    /// Blue channel (0.0 - 1.0)
    pub b: f32,
}

impl Color {
    /// Create a new color.
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// Create a black color.
    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Create a white color.
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

/// Complete text extraction result for a single page.
///
/// Single-call API that provides spans, per-character data, and page dimensions.
/// The `chars` field is derived from spans via `TextSpan::to_chars()`, using
/// font-metric widths when available for accurate per-glyph bounding boxes.
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct PageText {
    /// Text spans in reading order.
    pub spans: Vec<TextSpan>,
    /// Per-character data derived from spans (uses font metric widths when available).
    pub chars: Vec<TextChar>,
    /// Page width in PDF points.
    pub page_width: f32,
    /// Page height in PDF points.
    pub page_height: f32,
}

/// A text block (word, line, or paragraph).
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct TextBlock {
    /// Characters in this block
    pub chars: Vec<TextChar>,
    /// Bounding box of the entire block
    pub bbox: Rect,
    /// Text content
    pub text: String,
    /// Average font size
    pub avg_font_size: f32,
    /// Dominant font name
    pub dominant_font: String,
    /// Whether the block contains bold text
    pub is_bold: bool,
    /// Whether the block contains italic text
    pub is_italic: bool,
    /// Marked Content ID (for Tagged PDFs)
    pub mcid: Option<u32>,
    /// Content-stream emission order of the originating span(s) — lets
    /// callers tell "drawn consecutively in the content stream" apart
    /// from "merely spatially close" (e.g. table cells vs. overlays).
    /// `from_chars` has no span to draw this from, so it defaults to 0;
    /// word-assembly call sites that build a block from a single span
    /// set it explicitly from that span's `sequence`.
    pub sequence: usize,
    /// Rotation of the block's glyph run in degrees, snapped to a quadrant
    /// (`0` / `90` / `180` / `-90`), from the composed text-rendering
    /// matrix (ISO 32000-1:2008 §9.4.4). `90` means the text reads
    /// bottom-to-top on an unrotated page — a landscape table typeset on a
    /// portrait page. Lets consumers transform coordinates into the
    /// reading frame instead of guessing from bbox aspect.
    /// Derived in `from_chars` from the constituent chars.
    #[serde(skip_serializing_if = "is_zero_f32", default)]
    pub rotation_degrees: f32,
}

impl TextBlock {
    /// Create a text block from a collection of characters.
    ///
    /// This computes the bounding box, text content, average font size,
    /// and dominant font from the character data.
    ///
    /// # Panics
    ///
    /// Panics if the `chars` vector is empty.
    pub fn from_chars(chars: Vec<TextChar>) -> Self {
        assert!(!chars.is_empty(), "Cannot create TextBlock from empty chars");

        // Collect text directly
        let text: String = chars.iter().map(|c| c.char).collect();

        // Compute bounding box as union of all character bboxes
        let bbox = chars
            .iter()
            .map(|c| c.bbox)
            .fold(chars[0].bbox, |acc, r| acc.union(&r));

        let avg_font_size = chars.iter().map(|c| c.font_size).sum::<f32>() / chars.len() as f32;

        // Find dominant font (most common)
        let mut font_counts = HashMap::new();
        for c in &chars {
            *font_counts.entry(c.font_name.clone()).or_insert(0) += 1;
        }
        // Explicit tie-break, not `max_by_key`: it returns the LAST maximal
        // element, and over a `HashMap` that is the per-process-randomized
        // iteration order. A block whose two fonts carry the same character
        // count would otherwise pick a different dominant font per process.
        // Ties resolve to the lexicographically smaller font name.
        let dominant_font = font_counts
            .iter()
            .max_by(|(a_font, a_count), (b_font, b_count)| {
                a_count.cmp(b_count).then_with(|| b_font.cmp(a_font))
            })
            .map(|(font, _)| font.clone())
            .unwrap_or_default();

        let is_bold = chars.iter().any(|c| c.font_weight.is_bold());
        let is_italic = chars.iter().any(|c| c.is_italic);

        // Determine MCID for the block
        let mcid = chars
            .first()
            .and_then(|c| c.mcid)
            .filter(|&first_mcid| chars.iter().all(|c| c.mcid == Some(first_mcid)));

        // A word's glyphs share one text-rendering matrix in practice; take
        // the rotation all chars agree on, and fall back to upright when a
        // block mixes rotations (no single frame describes it).
        let rotation_degrees = chars
            .first()
            .map(|c| c.rotation_degrees)
            .filter(|&r| chars.iter().all(|c| c.rotation_degrees == r))
            .unwrap_or(0.0);

        Self {
            chars,
            bbox,
            text,
            avg_font_size,
            dominant_font,
            is_bold,
            is_italic,
            mcid,
            sequence: 0,
            rotation_degrees,
        }
    }

    /// Append `other`'s glyphs to this block, growing the bbox to their union
    /// and re-deriving the aggregate attributes.
    ///
    /// Incremental by design: O(len(other)) per call, so folding a run of `k`
    /// blocks costs O(total_chars) instead of the O(n²) of rebuilding through
    /// [`Self::from_chars`] at every step.
    ///
    /// Attribute rules, which are NOT the same as rebuilding from the
    /// concatenated chars: `avg_font_size` becomes the glyph-count-weighted
    /// mean, `dominant_font` follows the larger side, the style flags OR
    /// together, and a differing `mcid` collapses to `None` (the union no
    /// longer belongs to one marked-content sequence). `sequence` and
    /// `rotation_degrees` are left as this block's — callers fold blocks from
    /// a single run, where both already agree.
    pub(crate) fn absorb(&mut self, other: TextBlock) {
        let self_n = self.chars.len() as f32;
        let other_n = other.chars.len() as f32;
        self.bbox = self.bbox.union(&other.bbox);
        // Both sides can be chars-empty (several call sites build blocks with
        // no chars); a plain weighted mean would divide 0.0 by 0.0 and poison
        // the font size with NaN, which then propagates into every downstream
        // gap threshold.
        if self_n + other_n > 0.0 {
            self.avg_font_size =
                (self.avg_font_size * self_n + other.avg_font_size * other_n) / (self_n + other_n);
        }
        if other_n > self_n {
            self.dominant_font = other.dominant_font;
        }
        self.is_bold |= other.is_bold;
        self.is_italic |= other.is_italic;
        if self.mcid != other.mcid {
            self.mcid = None;
        }
        self.text.push_str(&other.text);
        self.chars.extend(other.chars);
    }

    /// Get the center point of the text block.
    pub fn center(&self) -> Point {
        self.bbox.center()
    }

    /// Check if this block is horizontally aligned with another block.
    pub fn is_horizontally_aligned(&self, other: &TextBlock, tolerance: f32) -> bool {
        (self.bbox.y - other.bbox.y).abs() < tolerance
    }

    /// Check if this block is vertically aligned with another block.
    pub fn is_vertically_aligned(&self, other: &TextBlock, tolerance: f32) -> bool {
        (self.bbox.x - other.bbox.x).abs() < tolerance
    }
}

/// A word is a semantic unit of text (alias for TextBlock).
pub type Word = TextBlock;

/// A line of text containing multiple words.
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
pub struct TextLine {
    /// Words in this line
    pub words: Vec<Word>,
    /// Bounding box of the entire line
    pub bbox: Rect,
    /// Complete text content of the line (words joined by spaces)
    pub text: String,
}

impl TextLine {
    /// Create a new TextLine from a list of words.
    ///
    /// # Panics
    ///
    /// Panics if the `words` vector is empty.
    pub fn new(words: Vec<Word>) -> Self {
        assert!(!words.is_empty(), "Cannot create TextLine from empty words");

        // Compute bounding box as union of all word bboxes
        let bbox = words
            .iter()
            .map(|w| w.bbox)
            .fold(words[0].bbox, |acc, r| acc.union(&r));

        // Join word text with spaces
        let text = words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        Self { words, bbox, text }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The provenance fact reaches every JSON/serde binding (WASM, Go, Ruby,
    // Java's structured extraction, ...) through span serialization: present as
    // a stable label when known, omitted when absent so existing output is
    // byte-identical.
    #[test]
    fn provenance_serializes_as_stable_label_and_omits_when_absent() {
        let mut span = TextSpan {
            text: "x".to_string(),
            ..TextSpan::default()
        };
        span.provenance = Some(crate::fonts::MappingProvenance::Fallback);
        let json = serde_json::to_string(&span).unwrap();
        assert!(json.contains("\"provenance\":\"fallback\""), "got {json}");

        let plain = TextSpan {
            text: "y".to_string(),
            ..TextSpan::default()
        };
        let json = serde_json::to_string(&plain).unwrap();
        assert!(!json.contains("provenance"), "absent provenance must be omitted: {json}");
    }

    fn mock_char(c: char, x: f32, y: f32) -> TextChar {
        let bbox = Rect::new(x, y, 10.0, 12.0);
        TextChar {
            char: c,
            bbox,
            font_name: "Times".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            origin_x: bbox.x,
            origin_y: bbox.y,
            rotation_degrees: 0.0,
            advance_width: bbox.width,
            rendered_advance: bbox.width,
            ascent: 0.95 * 12.0,
            descent: -0.35 * 12.0,
            matrix: None,
        }
    }

    // A block whose two fonts carry equal character counts must resolve to the
    // lexicographically smaller name, every run. `max_by_key` over the backing
    // `HashMap` returned whichever entry the per-process-randomized iteration
    // order visited last, so this assertion failed roughly half the time before
    // the explicit tie-break.
    #[test]
    fn dominant_font_tie_resolves_to_lexicographically_smaller_name() {
        let mut chars = Vec::new();
        for (i, c) in "abcd".chars().enumerate() {
            let mut ch = mock_char(c, i as f32 * 10.0, 0.0);
            ch.font_name = "Times".to_string();
            chars.push(ch);
        }
        for (i, c) in "efgh".chars().enumerate() {
            let mut ch = mock_char(c, 40.0 + i as f32 * 10.0, 0.0);
            ch.font_name = "Courier".to_string();
            chars.push(ch);
        }
        let block = TextBlock::from_chars(chars);
        assert_eq!(
            block.dominant_font, "Courier",
            "4 chars of Times vs 4 of Courier must resolve to the smaller name"
        );
    }

    // The tie-break must not disturb the ordinary case.
    #[test]
    fn dominant_font_unique_winner_is_unaffected_by_tie_break() {
        let mut chars = Vec::new();
        for (i, c) in "abcdef".chars().enumerate() {
            let mut ch = mock_char(c, i as f32 * 10.0, 0.0);
            ch.font_name = "Times".to_string();
            chars.push(ch);
        }
        for (i, c) in "gh".chars().enumerate() {
            let mut ch = mock_char(c, 60.0 + i as f32 * 10.0, 0.0);
            ch.font_name = "Courier".to_string();
            chars.push(ch);
        }
        let block = TextBlock::from_chars(chars);
        assert_eq!(block.dominant_font, "Times");
    }

    #[test]
    fn test_text_block_from_chars() {
        let chars = vec![
            mock_char('H', 0.0, 0.0),
            mock_char('e', 10.0, 0.0),
            mock_char('l', 20.0, 0.0),
            mock_char('l', 30.0, 0.0),
            mock_char('o', 40.0, 0.0),
        ];

        let block = TextBlock::from_chars(chars);
        assert_eq!(block.text, "Hello");
        assert_eq!(block.avg_font_size, 12.0);
    }

    #[test]
    fn test_text_span_is_monospace_default() {
        let span = TextSpan::default();
        assert!(!span.is_monospace, "Default spans should not be monospace");
    }

    #[test]
    fn test_text_span_is_monospace_set() {
        let span = TextSpan {
            is_monospace: true,
            text: "AB".to_string(),
            bbox: Rect::new(0.0, 0.0, 20.0, 12.0),
            ..TextSpan::default()
        };
        assert!(span.is_monospace);

        // to_chars should propagate is_monospace
        let chars = span.to_chars();
        for c in &chars {
            assert!(c.is_monospace, "TextChar should inherit is_monospace from span");
        }
    }

    #[test]
    fn test_text_char_is_monospace() {
        let c = TextChar {
            char: 'A',
            bbox: Rect::new(0.0, 0.0, 10.0, 12.0),
            font_name: "Courier".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: true,
            color: Color::black(),
            mcid: None,
            origin_x: 0.0,
            origin_y: 0.0,
            rotation_degrees: 0.0,
            advance_width: 10.0,
            rendered_advance: 10.0,
            ascent: 0.95 * 12.0,
            descent: -0.35 * 12.0,
            matrix: None,
        };
        assert!(c.is_monospace);
    }

    #[test]
    fn test_to_chars_uses_char_widths_when_available() {
        let span = TextSpan {
            text: "AB".to_string(),
            bbox: Rect::new(10.0, 20.0, 30.0, 12.0),
            char_widths: vec![10.0, 20.0],
            char_x_offsets: Vec::new(),
            ..TextSpan::default()
        };
        let chars = span.to_chars();
        assert_eq!(chars.len(), 2);
        // First char: x=10, width=10
        assert!((chars[0].bbox.x - 10.0).abs() < 0.001);
        assert!((chars[0].bbox.width - 10.0).abs() < 0.001);
        assert!((chars[0].advance_width - 10.0).abs() < 0.001);
        // Second char: x=20, width=20
        assert!((chars[1].bbox.x - 20.0).abs() < 0.001);
        assert!((chars[1].bbox.width - 20.0).abs() < 0.001);
        assert!((chars[1].advance_width - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_to_chars_falls_back_to_uniform_when_no_widths() {
        let span = TextSpan {
            text: "AB".to_string(),
            bbox: Rect::new(10.0, 20.0, 30.0, 12.0),
            // char_widths left empty (default)
            ..TextSpan::default()
        };
        let chars = span.to_chars();
        assert_eq!(chars.len(), 2);
        // Uniform division: 30.0 / 2 = 15.0 each
        assert!((chars[0].bbox.width - 15.0).abs() < 0.001);
        assert!((chars[1].bbox.width - 15.0).abs() < 0.001);
        assert!((chars[0].bbox.x - 10.0).abs() < 0.001);
        assert!((chars[1].bbox.x - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_to_chars_handles_mismatched_widths_gracefully() {
        let span = TextSpan {
            text: "ABC".to_string(),
            bbox: Rect::new(0.0, 0.0, 30.0, 12.0),
            char_widths: vec![5.0, 10.0], // only 2 widths for 3 chars
            ..TextSpan::default()
        };
        let chars = span.to_chars();
        assert_eq!(chars.len(), 3);
        // Should fall back to uniform: 30.0 / 3 = 10.0 each
        assert!((chars[0].bbox.width - 10.0).abs() < 0.001);
        assert!((chars[1].bbox.width - 10.0).abs() < 0.001);
        assert!((chars[2].bbox.width - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_to_chars_prefers_char_x_offsets_over_widths() {
        // char_x_offsets carry positions that DIVERGE from a prefix-sum of
        // char_widths (simulating TJ-kerning drift). to_chars honours the
        // offsets for origin_x / bbox.x — and now for the width too.
        //
        // This asserted that widths still came from `char_widths`. The
        // divergence in this very fixture is the argument against it: the
        // glyph at 10.0 is followed by one at 25.0, so it advances 15, whatever
        // `char_widths` records. Trusting the bookkeeping over the measurement
        // shifted widths off their glyphs wherever the two drifted, opening
        // phantom gaps that the word-gap clusterer split on.
        let span = TextSpan {
            text: "AB".to_string(),
            bbox: Rect::new(10.0, 20.0, 30.0, 12.0),
            char_widths: vec![10.0, 20.0],
            char_x_offsets: vec![10.0, 25.0], // NOT 10.0, 20.0 (prefix-sum)
            ..TextSpan::default()
        };
        let chars = span.to_chars();
        assert_eq!(chars.len(), 2);
        // Positions come from char_x_offsets, not the prefix-sum of widths.
        assert!((chars[0].origin_x - 10.0).abs() < 0.001);
        assert!((chars[0].bbox.x - 10.0).abs() < 0.001);
        assert!((chars[1].origin_x - 25.0).abs() < 0.001);
        assert!((chars[1].bbox.x - 25.0).abs() < 0.001);
        // The first glyph's width is the measured distance to the next origin.
        assert!((chars[0].bbox.width - 15.0).abs() < 0.001);
        // The last has no next origin, so it keeps its nominal width.
        assert!((chars[1].bbox.width - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_to_chars_empty_offsets_is_byte_identical_fallback() {
        // Empty char_x_offsets (the default) must produce exactly the legacy
        // char_widths path — same positions and widths.
        let with_offsets = TextSpan {
            text: "AB".to_string(),
            bbox: Rect::new(10.0, 20.0, 30.0, 12.0),
            char_widths: vec![10.0, 20.0],
            char_x_offsets: Vec::new(),
            ..TextSpan::default()
        };
        let legacy = TextSpan {
            text: "AB".to_string(),
            bbox: Rect::new(10.0, 20.0, 30.0, 12.0),
            char_widths: vec![10.0, 20.0],
            char_x_offsets: Vec::new(),
            ..TextSpan::default()
        };
        let a = with_offsets.to_chars();
        let b = legacy.to_chars();
        assert_eq!(a.len(), b.len());
        for (ca, cb) in a.iter().zip(b.iter()) {
            assert!((ca.origin_x - cb.origin_x).abs() < 1e-6);
            assert!((ca.bbox.x - cb.bbox.x).abs() < 1e-6);
            assert!((ca.bbox.width - cb.bbox.width).abs() < 1e-6);
        }
        // And it matches the documented legacy positions.
        assert!((a[0].bbox.x - 10.0).abs() < 0.001);
        assert!((a[1].bbox.x - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_to_chars_offset_count_mismatch_falls_back() {
        // Offsets that do not cover every glyph must be ignored (legacy path).
        let span = TextSpan {
            text: "ABC".to_string(),
            bbox: Rect::new(0.0, 0.0, 30.0, 12.0),
            char_widths: vec![10.0, 10.0, 10.0],
            char_x_offsets: vec![0.0, 15.0], // 2 offsets for 3 chars
            ..TextSpan::default()
        };
        let chars = span.to_chars();
        assert_eq!(chars.len(), 3);
        // Falls through to char_widths prefix-sum: 0, 10, 20.
        assert!((chars[0].bbox.x - 0.0).abs() < 0.001);
        assert!((chars[1].bbox.x - 10.0).abs() < 0.001);
        assert!((chars[2].bbox.x - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_to_chars_out_of_bounds_offsets_fall_back_to_span_geometry() {
        let span = TextSpan {
            text: "1".to_string(),
            bbox: Rect::new(216.0, 315.0, 4.0, 7.0),
            char_widths: vec![4.0],
            // A repeated digit elsewhere on the same baseline was incorrectly
            // stamped onto this superscript run.
            char_x_offsets: vec![252.0],
            ..TextSpan::default()
        };

        let chars = span.to_chars();

        assert_eq!(chars.len(), 1);
        assert!((chars[0].origin_x - 216.0).abs() < 0.001);
        assert!((chars[0].bbox.width - 4.0).abs() < 0.001);
    }

    /// A glyph's width comes from the distance to the next measured origin,
    /// not from `char_widths` — because the two can drift apart and the
    /// offsets are the measurement.
    ///
    /// On a table-of-contents line containing an em dash the widths ran two
    /// entries out of step with the text, so `C` was paired with a 2.67 pt
    /// advance where its own is 5.83 while its offset was exactly right. The
    /// phantom gaps that opens are what the word-gap clusterer splits on:
    /// `Coast Guard, Department` came out as `C|o|ast G|u|ard, D|e|partm|ent`.
    #[test]
    fn test_char_takes_its_width_from_the_next_offset() {
        // Offsets are correct and evenly 6 pt apart; the widths are nonsense.
        let span = TextSpan {
            text: "Cat".to_string(),
            bbox: Rect::new(100.0, 200.0, 18.0, 8.0),
            char_widths: vec![2.0, 2.0, 2.0],
            char_x_offsets: vec![100.0, 106.0, 112.0],
            font_size: 8.0,
            ..TextSpan::default()
        };

        let chars = span.to_chars();

        assert_eq!(chars.len(), 3);
        assert!(
            (chars[0].bbox.width - 6.0).abs() < 0.001,
            "expected the measured 6.0 pt advance, got {}",
            chars[0].bbox.width
        );
        assert!((chars[1].bbox.width - 6.0).abs() < 0.001);
        // The final glyph has no next offset, so it keeps its nominal width.
        assert!((chars[2].bbox.width - 2.0).abs() < 0.001);
    }

    /// A zero-length step is not an advance either. A combining mark shares
    /// its base glyph's origin — the case the previous comment called the
    /// "Indic-guarded model" — so those keep the nominal width rather than
    /// collapsing to zero.
    #[test]
    fn test_zero_offset_step_keeps_the_nominal_width() {
        let span = TextSpan {
            text: "ka".to_string(),
            bbox: Rect::new(100.0, 200.0, 6.0, 8.0),
            char_widths: vec![6.0, 0.0],
            char_x_offsets: vec![100.0, 100.0],
            font_size: 8.0,
            ..TextSpan::default()
        };

        let chars = span.to_chars();

        assert_eq!(chars.len(), 2);
        assert!(
            (chars[0].bbox.width - 6.0).abs() < 0.001,
            "a combining mark at the base glyph's origin must not zero its \
             width, got {}",
            chars[0].bbox.width
        );
    }

    /// A backward step is not an advance. Visually-stored RTL and mirrored
    /// runs place successive glyphs at decreasing x, and subtracting there
    /// would give a negative width — those keep the nominal value.
    #[test]
    fn test_backward_offset_step_keeps_the_nominal_width() {
        let span = TextSpan {
            text: "ab".to_string(),
            bbox: Rect::new(100.0, 200.0, 12.0, 8.0),
            char_widths: vec![5.0, 5.0],
            char_x_offsets: vec![106.0, 100.0],
            font_size: 8.0,
            ..TextSpan::default()
        };

        let chars = span.to_chars();

        assert_eq!(chars.len(), 2);
        assert!(
            (chars[0].bbox.width - 5.0).abs() < 0.001,
            "a decreasing offset pair must fall back to char_widths, got {}",
            chars[0].bbox.width
        );
    }

    #[test]
    fn text_rise_zero_serde_omitted() {
        // A default (on-baseline) span must NOT serialize a `text_rise` key, so
        // existing fixtures stay byte-identical now that the field exists.
        let span = TextSpan {
            text: "x".to_string(),
            ..TextSpan::default()
        };
        let json = serde_json::to_string(&span).unwrap();
        assert!(
            !json.contains("text_rise"),
            "zero text_rise must be omitted from serialized output: {json}"
        );

        // A non-zero rise IS serialized (the rejoin signal must survive a round-trip).
        let raised = TextSpan {
            text: "2".to_string(),
            text_rise: 0.33,
            ..TextSpan::default()
        };
        let json = serde_json::to_string(&raised).unwrap();
        assert!(json.contains("text_rise"), "non-zero text_rise must be serialized: {json}");
    }
}
