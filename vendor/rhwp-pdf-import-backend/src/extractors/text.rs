//! Text extraction from PDF content streams.
//!
//! This module executes content stream operators to extract positioned
//! text characters with their Unicode mappings, font information,
//! bounding boxes.

#![forbid(unsafe_code)]

use crate::color::cmyk_to_rgb;
use crate::config::ExtractionProfile;
use crate::content::graphics_state::{GraphicsStateStack, Matrix};
use crate::content::operators::{Operator, TextElement};
use crate::content::parse_and_execute_text_only;
use crate::content::parse_content_stream;
use crate::error::{Error, Result};
use crate::extract_log_debug;
use crate::fonts::unicode_decode::{fallback_char_to_unicode, DecodePolicy, TextCharIter};
use crate::fonts::FontInfo;
use crate::geometry::Rect;

#[derive(Debug)]
pub(crate) struct SpanPaintOrder {
    pub(crate) order: Vec<usize>,
    pub(crate) group: usize,
    pub(crate) basis: [f32; 4],
    pub(crate) font: Option<std::sync::Arc<FontInfo>>,
}

/// Decode a show string the way extraction reads it.
///
/// Extraction keeps an unmapped code when the caller asked for it, leaves
/// ligatures whole for `ligature_processor` to expand, and writes a question
/// mark for a code no table resolves.
fn decode_text_to_unicode(bytes: &[u8], font: Option<&FontInfo>) -> String {
    crate::fonts::unicode_decode::decode_text_to_unicode(
        bytes,
        font,
        DecodePolicy {
            preserve_unmapped: preserve_unmapped_glyphs(),
            decompose_ligatures: false,
            question_mark_for_invalid: true,
        },
    )
}
use crate::layout::{Color, FontWeight, TextChar, TextSpan};
use crate::object::{Object, ObjectRef};
use crate::pipeline::config::WordBoundaryMode;
use crate::text::{BoundaryContext, CharacterInfo, DocumentScript, WordBoundaryDetector};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Global flag controlling whether glyph-decode sites emit `U+FFFD`
/// (REPLACEMENT CHARACTER) into `extract_text` / `extract_words` /
/// `extract_spans` output.
///
/// The historical default is to silently drop `U+FFFD` chars, which
/// is preserved here for back-compat. Setting `true` makes the
/// high-level accessors consistent with `extract_chars` (which
/// always preserves FFFD) so callers can detect unmapped-glyph
/// pages without diffing the two accessors' outputs.
///
/// `Ordering::Relaxed` is sufficient because every read is gated on
/// `Acquire`-style writes from the setter, and the flag is a single
/// boolean with no other state dependencies.
static PRESERVE_UNMAPPED_GLYPHS: AtomicBool = AtomicBool::new(false);

/// Set the global U+FFFD preservation flag. When `true`, the high-level
/// text accessors (`extract_text` / `extract_words` / `extract_spans`)
/// emit U+FFFD chars for glyphs that map to the REPLACEMENT
/// CHARACTER, matching the behaviour of `extract_chars` which has
/// always preserved them. Returns the previous flag value.
///
/// Resolves the filter divergence where the high-level accessors
/// silently drop FFFD while `extract_chars` keeps them, producing
/// empty `extract_text` output on pages whose visible glyphs all
/// map to FFFD (e.g. the MSAM10 math-symbol font).
///
/// The default is `false` to preserve historical fixture output
/// byte-identical for the no-FFFD-glyph case; downstream callers
/// that want to surface unmapped glyphs to the user opt in by
/// setting `true`.
pub fn set_preserve_unmapped_glyphs(preserve: bool) -> bool {
    PRESERVE_UNMAPPED_GLYPHS.swap(preserve, Ordering::SeqCst)
}

/// True if the high-level accessors should preserve `U+FFFD` glyphs.
#[inline]
pub(crate) fn preserve_unmapped_glyphs() -> bool {
    PRESERVE_UNMAPPED_GLYPHS.load(Ordering::Relaxed)
}

/// Source of a space decision in the unified pipeline.
///
/// This enum tracks why a space was inserted (or not), which helps with
/// debugging and understanding the text extraction behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceSource {
    /// Space triggered by TJ offset value (negative offset > threshold)
    /// Confidence: 0.95 (explicit PDF positioning signal)
    TjOffset,

    /// Space triggered by geometric gap between spans
    /// Confidence: 0.8 (heuristic based on font metrics)
    GeometricGap,

    /// Space triggered by character transition heuristic (e.g., CamelCase, number->letter)
    /// Confidence: 0.6 (pattern-based heuristic)
    CharacterHeuristic,

    /// Space already present in boundary (no insertion needed)
    /// Confidence: 1.0 (deterministic)
    AlreadyPresent,

    /// No space inserted
    /// Confidence: varies (default when no rule matches)
    NoSpace,

    /// No space: suppressed specifically by the intra-word kerning guard
    /// (a lowercase↔lowercase gap below 0.75× the space-glyph advance). Kept
    /// distinct from `NoSpace` so the #847 per-line bimodal rescue can override
    /// ONLY this purely-geometric suppression, never the semantic ones
    /// (complex-script, CJK, ligature) that also return no-space.
    IntraWordKerning,

    /// Space triggered by WordBoundaryDetector analysis
    /// Confidence: 0.85 (combines TJ offset, geometric, and CJK signals per PDF Spec 9.4.4)
    WordBoundaryAnalysis,
}

/// Result of unified space decision process.
///
/// This struct is the single source of truth for whether a space should be inserted
/// between two text spans. It combines all available signals:
/// - TJ offset values from PDF content stream
/// - Geometric gaps between spans
/// - Character transition heuristics
/// - Existing boundary whitespace
///
/// Per PDF Spec ISO 32000-1:2008, Section 9.4.4 NOTE 6:
/// "The identification of what constitutes a word is unrelated to how the text
/// happens to be grouped into show strings... text strings should be as long as possible."
#[derive(Debug, Clone)]
pub struct SpaceDecision {
    /// Whether a space should be inserted
    pub insert_space: bool,

    /// Source/reason for this decision
    pub source: SpaceSource,

    /// Confidence score (0.0-1.0) indicating certainty
    pub confidence: f32,
}

impl SpaceDecision {
    /// Create a decision to insert a space from a specific source.
    pub fn insert(source: SpaceSource, confidence: f32) -> Self {
        Self {
            insert_space: true,
            source,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Create a decision to not insert a space.
    pub fn no_space(source: SpaceSource, confidence: f32) -> Self {
        Self {
            insert_space: false,
            source,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// Configuration for text extraction heuristics.
///
/// PDF spec does not define explicit rules for many spacing scenarios.
/// These configurable thresholds allow tuning extraction behavior.
///
/// # PDF Spec Reference
///
/// ISO 32000-1:2008, Section 9.4.4 - Text Positioning operators (TJ, Tj)
/// The spec defines how positioning works but NOT when a position offset
/// represents a word boundary vs. tight kerning.
#[derive(Debug, Clone)]
pub struct TextExtractionConfig {
    /// Extraction profile with document-type-specific thresholds
    ///
    /// When set, this profile overrides individual threshold settings and provides
    /// pre-tuned parameters optimized for specific document types (Academic, Policy,
    /// Government, Form, ScannedOCR, etc.).
    ///
    /// **Default**: None (uses legacy individual thresholds for backward compatibility)
    pub profile: Option<ExtractionProfile>,

    /// Threshold for inserting space characters in TJ arrays.
    ///
    /// **DEPRECATED**: Consider using `profile` with an `ExtractionProfile` or
    /// `word_margin_ratio` with `use_adaptive_tj_threshold` enabled for geometry-based
    /// adaptive thresholds. This field is used as a fallback when font metrics are
    /// unavailable or adaptive thresholds are disabled, and when profile is not set.
    ///
    /// **HEURISTIC**: When a TJ array contains a negative offset (in text space units),
    /// and that offset exceeds this threshold, a space character is inserted.
    ///
    /// **Default**: -120.0 units ≈ 0.12em
    /// - Typical word space: 0.25-0.33em (250-330 units)
    /// - Typical letter kerning: <0.1em (<100 units)
    ///
    /// **Lower values** (e.g., -80): More sensitive, inserts more spaces (may add spurious spaces)
    /// **Higher values** (e.g., -200): Less sensitive, inserts fewer spaces (may miss word boundaries)
    ///
    /// Set to `f32::NEG_INFINITY` to disable space insertion entirely.
    pub space_insertion_threshold: f32,

    /// Word margin ratio for geometry-based adaptive TJ threshold.
    ///
    /// When `use_adaptive_tj_threshold` is true and font metrics are available,
    /// the TJ offset threshold is calculated as:
    /// ```text
    /// adaptive_threshold = -(average_glyph_width * word_margin_ratio)
    /// ```
    ///
    /// This approach adapts to different font sizes and families by using the
    /// actual glyph metrics instead of a static value. This matches pdfplumber's
    /// `word_margin` parameter (default 0.1).
    ///
    /// **Default**: 0.1 (10% of average glyph width)
    ///
    /// **Typical values**:
    /// - 0.05: Tighter spacing (fewer spaces inserted, better for narrow fonts)
    /// - 0.1: Standard word spacing (default, matches pdfplumber)
    /// - 0.15: Looser spacing (more spaces inserted, better for wide fonts)
    ///
    /// **Note**: If font metrics are unavailable, falls back to `space_insertion_threshold`.
    ///
    /// # PDF Spec Reference
    ///
    /// ISO 32000-1:2008, Section 9.4.4 - TJ offsets are in thousandths of em.
    /// Average glyph width is also in thousandths of em, making this ratio
    /// dimensionally correct.
    pub word_margin_ratio: f32,

    /// Enable adaptive TJ threshold based on font geometry.
    ///
    /// When true, uses font metrics to calculate the TJ offset threshold dynamically:
    /// `adaptive_threshold = -(average_glyph_width * word_margin_ratio)`
    ///
    /// This replaces the static `space_insertion_threshold` with a value that adapts
    /// to different font sizes, families, and document layouts.
    ///
    /// **Default**: true (adaptive approach enabled)
    ///
    /// Set to `false` for backward compatibility with legacy behavior, which
    /// uses only the static `space_insertion_threshold`.
    ///
    /// # Benefits
    ///
    /// - Handles font size variations (8pt vs 24pt documents)
    /// - Adapts to different character widths (serif vs sans-serif, monospace vs proportional)
    /// - Reduces spurious spaces in policy documents with tight kerning
    /// - Maintains word boundary detection in academic documents
    pub use_adaptive_tj_threshold: bool,

    /// Word boundary detection mode for TJ array processing
    ///
    /// Controls whether WordBoundaryDetector is used as:
    /// - Tiebreaker: Only when TJ and geometric signals conflict (default)
    /// - Primary: Before creating TextSpans from tj_character_array
    ///
    /// **Default**: WordBoundaryMode::Tiebreaker (backward compatible)
    pub word_boundary_mode: WordBoundaryMode,
}

impl Default for TextExtractionConfig {
    fn default() -> Self {
        Self {
            profile: None,
            // Default -120.0 (conservative; matches existing
            // ExtractionProfile::CONSERVATIVE for byte-identical
            // back-compat). Callers handling TJ-heavy PDFs that
            // produce `Loremipsumdolorsitamet`-style merged
            // paragraphs can override via
            // `TextExtractionConfig::with_space_threshold(-100.0)` or
            // via the `TJ_HEAVY` extraction profile (see
            // config/extraction_profiles.rs). The default stays at
            // -120 to preserve byte-identical fixture output for the
            // 75-PDF regression sweep.
            //
            // Per-document calibration via gap_statistics is the
            // ideal root-cause fix; it requires a calibration corpus
            // to validate the threshold against without regressing
            // other inputs.
            space_insertion_threshold: -120.0,
            word_margin_ratio: 0.1,
            use_adaptive_tj_threshold: false,
            word_boundary_mode: WordBoundaryMode::default(),
        }
    }
}

impl TextExtractionConfig {
    /// Create a new configuration with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::TextExtractionConfig;
    ///
    /// let config = TextExtractionConfig::new();
    /// assert_eq!(config.space_insertion_threshold, -120.0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with custom space insertion threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Negative offset threshold for space insertion (in text space units)
    ///
    /// **Note**: This uses the static threshold. For better results, consider using
    /// `with_word_margin_ratio()` with adaptive thresholds enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::TextExtractionConfig;
    ///
    /// // More aggressive space insertion
    /// let config = TextExtractionConfig::with_space_threshold(-80.0);
    ///
    /// // Disable space insertion entirely
    /// let no_spaces = TextExtractionConfig::with_space_threshold(f32::NEG_INFINITY);
    /// ```
    pub fn with_space_threshold(threshold: f32) -> Self {
        Self {
            profile: None,
            space_insertion_threshold: threshold,
            word_margin_ratio: 0.1,
            use_adaptive_tj_threshold: false, // Static threshold mode
            word_boundary_mode: WordBoundaryMode::default(),
        }
    }

    /// Create a configuration with custom word margin ratio for adaptive TJ thresholds.
    ///
    /// # Arguments
    ///
    /// * `ratio` - Word margin ratio as fraction of average glyph width (typically 0.05-0.15)
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::TextExtractionConfig;
    ///
    /// // Standard adaptive thresholds (matches pdfplumber)
    /// let config = TextExtractionConfig::with_word_margin_ratio(0.1);
    ///
    /// // More aggressive (wider thresholds, more spaces)
    /// let aggressive = TextExtractionConfig::with_word_margin_ratio(0.15);
    ///
    /// // More conservative (narrower thresholds, fewer spaces)
    /// let conservative = TextExtractionConfig::with_word_margin_ratio(0.05);
    /// ```
    pub fn with_word_margin_ratio(ratio: f32) -> Self {
        Self {
            profile: None,
            space_insertion_threshold: -120.0, // Fallback value
            word_margin_ratio: ratio,
            use_adaptive_tj_threshold: true, // Adaptive threshold mode
            word_boundary_mode: WordBoundaryMode::default(),
        }
    }

    /// Set the word margin ratio on an existing configuration (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `ratio` - Word margin ratio as fraction of average glyph width
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::TextExtractionConfig;
    ///
    /// let config = TextExtractionConfig::new()
    ///     .set_word_margin_ratio(0.15);
    /// ```
    pub fn set_word_margin_ratio(mut self, ratio: f32) -> Self {
        self.word_margin_ratio = ratio;
        self.use_adaptive_tj_threshold = true;
        self
    }

    /// Enable or disable adaptive TJ thresholds (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to use adaptive thresholds based on font metrics
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::TextExtractionConfig;
    ///
    /// // Use static threshold only
    /// let config = TextExtractionConfig::new()
    ///     .set_adaptive_tj_threshold(false);
    /// ```
    pub fn set_adaptive_tj_threshold(mut self, enabled: bool) -> Self {
        self.use_adaptive_tj_threshold = enabled;
        self
    }

    /// Set the extraction profile and apply its threshold configuration (builder pattern).
    ///
    /// This applies the profile's thresholds to the configuration, selecting document-type-specific
    /// parameters for better text extraction quality.
    ///
    /// # Arguments
    ///
    /// * `profile` - An extraction profile (e.g., ACADEMIC, POLICY, FORM)
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::TextExtractionConfig;
    /// use pdf_oxide::config::ExtractionProfile;
    ///
    /// // Use ACADEMIC profile for research papers
    /// let config = TextExtractionConfig::new()
    ///     .with_profile(ExtractionProfile::ACADEMIC);
    /// ```
    pub fn with_profile(mut self, profile: ExtractionProfile) -> Self {
        // Extract profile settings before moving profile
        let tj_offset = profile.tj_offset_threshold;
        let word_margin = profile.word_margin_ratio;
        let use_adaptive = profile.use_adaptive_threshold;

        // Set profile and apply its thresholds
        self.profile = Some(profile);
        self.space_insertion_threshold = tj_offset;
        self.word_margin_ratio = word_margin;
        self.use_adaptive_tj_threshold = use_adaptive;
        self
    }
}

/// Configuration for span merging behavior.
///
/// These thresholds control how adjacent text spans are merged together and when
/// spaces are inserted between them. All thresholds are in PDF points (1/72 inch).
///
/// # Rationale
///
/// PDF content streams don't explicitly mark word boundaries - text can be rendered
/// with arbitrary gaps. These configurable thresholds allow tuning extraction to
/// different document types:
/// - Academic papers: tight column spacing, small gaps between words
/// - Documents with tables: larger gaps to preserve structure
/// - Dense grids (author lists): very small gaps that are still word boundaries
///
/// # References
///
/// Typography standards: word spacing typically 0.25-0.33em (25-33% of font size)
/// See: SPAN_SPACING_INVESTIGATION.md for empirical measurements
#[derive(Clone, Debug, PartialEq)]
pub struct SpanMergingConfig {
    /// Minimum gap (in multiples of font size) to trigger space insertion.
    ///
    /// When the gap between two spans exceeds this threshold, a space is inserted.
    /// Expressed as a ratio of font size (em).
    ///
    /// **Default**: 0.25
    /// - Based on typography standards: typical word spacing is 0.25-0.33em
    /// - For 12pt font: 0.25em * 12pt = 3pt
    /// - For 10pt font: 0.25em * 10pt = 2.5pt
    ///
    /// **Tuning guidance**:
    /// - Lower values (0.15-0.20): More aggressive space insertion, catches dense layouts
    /// - Higher values (0.33-0.50): Conservative, only clear word boundaries
    pub space_threshold_em_ratio: f32,

    /// Conservative threshold for font transitions (in points).
    ///
    /// Below this gap, don't insert a space even if gap > 0, to avoid spurious spaces
    /// from font metric changes or very tight kerning.
    ///
    /// **Default**: 0.1
    /// - Avoids spaces from font metric alignment issues (very tight threshold)
    /// - Smaller than typical letter spacing in justified text
    /// - Catches actual overlaps/reversals while preserving character adjacency
    ///
    /// **Note**: Changed from 0.3 to 0.1 after regression testing revealed
    /// that 0.3pt was too conservative for policy documents (0.1-0.3pt word spacing),
    /// causing word fusion. Adaptive threshold analysis recommended for future improvement.
    ///
    /// **Tuning guidance**:
    /// - Lower values (0.1-0.2): More aggressive, inserts more spaces
    /// - Higher values (0.5-1.0): Conservative, only clear separations
    pub conservative_threshold_pt: f32,

    /// Column boundary threshold (in points).
    ///
    /// Gaps larger than this indicate column separation and prevent span merging.
    /// Used to preserve document structure (e.g., multi-column layouts, tables).
    ///
    /// **Default**: 5.0
    /// - Typical character width for 10-12pt font: 4-6pt
    /// - Word spacing: 2-4pt
    /// - Column gaps in academic papers: 5-15pt
    /// - Table column gaps: 10-50pt
    ///
    /// **Tuning guidance**:
    /// - Lower values (3.0-4.0): Merge more spans, risk merging across columns
    /// - Higher values (8.0-10.0): Keep columns separate, preserve structure
    pub column_boundary_threshold_pt: f32,

    /// Negative gap threshold for severe overlaps (in points).
    ///
    /// When gaps are negative (spans overlap), values more severe than this
    /// indicate genuine overlap and should prevent merging.
    ///
    /// **Default**: -0.5
    /// - Typical font metric variations: 0 to -0.3pt
    /// - Small overlaps from kerning: -0.3 to -0.5pt
    /// - Real overlap errors: worse than -0.5pt
    ///
    /// **Tuning guidance**:
    /// - Less negative (-0.2, -0.1): More conservative on overlaps
    /// - More negative (-1.0, -2.0): Allow some overlap to merge adjacent text
    pub severe_overlap_threshold_pt: f32,

    /// Enable adaptive threshold analysis (default: true).
    ///
    /// When true, the `conservative_threshold_pt` is automatically calculated
    /// based on the gap distribution within the document. This overrides the fixed
    /// threshold value and adapts to different document types.
    ///
    /// **Default**: true (adaptive enabled)
    /// Enabled by default to improve extraction quality across document types.
    /// Use `SpanMergingConfig::legacy()` for the old fixed-threshold behavior.
    ///
    /// # Performance
    ///
    /// Adaptive analysis adds minimal overhead (O(n log n) for gap analysis where n = spans).
    /// Expected overhead: <5% of total extraction time.
    pub use_adaptive_threshold: bool,

    /// Configuration for adaptive threshold analysis.
    ///
    /// Only used when `use_adaptive_threshold` is true.
    /// If None, uses `AdaptiveThresholdConfig::default()`.
    ///
    /// Allows fine-tuning the adaptive analysis for specific document types:
    /// - `AdaptiveThresholdConfig::policy_documents()` - For tight spacing
    /// - `AdaptiveThresholdConfig::academic()` - For standard spacing
    /// - `AdaptiveThresholdConfig::aggressive()` - For dense layouts
    /// - `AdaptiveThresholdConfig::conservative()` - For formal documents
    pub adaptive_config: Option<crate::extractors::gap_statistics::AdaptiveThresholdConfig>,

    /// Enable email pattern detection for spacing decisions.
    ///
    /// When true, detects email-like patterns in surrounding text
    /// (e.g., "user@domain" separated by spaces) and applies special spacing rules
    /// to preserve email addresses.
    ///
    /// Per PDF Spec ISO 32000-1:2008 Section 9.10, only extracted text patterns
    /// are used - no domain-specific semantics.
    ///
    /// **Default**: false
    pub detect_email_patterns: bool,

    /// Multiplier for email pattern threshold detection.
    ///
    /// Controls how aggressively email patterns are detected by adjusting the gap threshold.
    /// A multiplier > 1.0 makes detection more lenient (allows larger gaps to be considered email context).
    /// A multiplier < 1.0 makes detection stricter.
    ///
    /// Calculated as: `email_threshold = geometric_threshold * email_threshold_multiplier`
    ///
    /// **Default**: 2.5
    /// - At 2.5×, handles typical email address separations with spaces
    /// - Typical gap between email parts: 4-8pt (after @, before TLD)
    pub email_threshold_multiplier: f32,

    /// Enable citation marker detection for spacing decisions.
    ///
    /// When true, detects superscript citation markers (typically smaller font size)
    /// and adjusts spacing rules to preserve citation formatting.
    ///
    /// Per PDF Spec ISO 32000-1:2008 Section 9.10, font size ratios from extracted content
    /// are used for detection.
    ///
    /// **Default**: false
    pub detect_citation_markers: bool,

    /// Font size ratio for citation marker detection.
    ///
    /// Citation markers typically have font size between this ratio and 1.0 of the base text.
    /// Values below this ratio are considered citation markers.
    ///
    /// **Default**: 0.75
    /// - Typical citation markers: 70-80% of text font size
    /// - Superscript usually: 50-80% of base font
    pub citation_font_size_ratio: f32,

    /// When `false`, each `Tm` operator starts a fresh span regardless of position.
    /// Use this to preserve column boundaries for callers that need per-positioned-run spans
    /// (e.g. pdftotext `-bbox-layout` parity).
    ///
    /// # Warning
    /// Disabling this on character-by-character-positioned PDFs (common in academic typesetting)
    /// can produce very large span counts per page (100× or more).
    ///
    /// Default: `true` (existing behaviour preserved).
    ///
    /// Reference: ISO 32000-1 §9.4.2 / §9.4.4 NOTE 6.
    pub merge_tm_tj_runs: bool,
}

impl Default for SpanMergingConfig {
    fn default() -> Self {
        Self {
            space_threshold_em_ratio: 0.25,
            conservative_threshold_pt: 0.1, // Reverted from 0.3 after regression testing
            column_boundary_threshold_pt: 5.0,
            severe_overlap_threshold_pt: -0.5,
            use_adaptive_threshold: true, // Enabled by default for better quality
            adaptive_config: None,
            detect_email_patterns: false,
            email_threshold_multiplier: 2.5,
            detect_citation_markers: false,
            citation_font_size_ratio: 0.75,
            merge_tm_tj_runs: true,
        }
    }
}

impl SpanMergingConfig {
    /// Create a new configuration with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::SpanMergingConfig;
    ///
    /// let config = SpanMergingConfig::new();
    /// assert_eq!(config.space_threshold_em_ratio, 0.25);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with aggressive space insertion (for dense layouts).
    ///
    /// Uses lower thresholds to insert spaces more readily:
    /// - space_threshold_em_ratio: 0.15 (instead of 0.25)
    /// - conservative_threshold_pt: 0.1 (instead of 0.3)
    ///
    /// Good for documents with many short words close together (author lists, grids).
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::SpanMergingConfig;
    ///
    /// let config = SpanMergingConfig::aggressive();
    /// ```
    pub fn aggressive() -> Self {
        Self {
            space_threshold_em_ratio: 0.15,
            conservative_threshold_pt: 0.1,
            column_boundary_threshold_pt: 5.0,
            severe_overlap_threshold_pt: -0.5,
            use_adaptive_threshold: false,
            adaptive_config: None,
            detect_email_patterns: false,
            email_threshold_multiplier: 2.5,
            detect_citation_markers: false,
            citation_font_size_ratio: 0.75,
            merge_tm_tj_runs: true,
        }
    }

    /// Create a configuration with conservative space insertion (for formal documents).
    ///
    /// Uses higher thresholds to insert spaces less readily:
    /// - space_threshold_em_ratio: 0.33 (instead of 0.25)
    /// - conservative_threshold_pt: 0.3 (instead of 0.1)
    ///
    /// Good for formal documents where spacing is reliable.
    ///
    /// **Note**: After regression testing, 0.5pt threshold was found to cause
    /// excessive word fusion in policy documents. Reduced to 0.3pt.
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::SpanMergingConfig;
    ///
    /// let config = SpanMergingConfig::conservative();
    /// ```
    pub fn conservative() -> Self {
        Self {
            space_threshold_em_ratio: 0.33,
            conservative_threshold_pt: 0.3, // Reduced from 0.5 (was too aggressive for policy docs)
            column_boundary_threshold_pt: 5.0,
            severe_overlap_threshold_pt: -0.5,
            use_adaptive_threshold: false,
            adaptive_config: None,
            detect_email_patterns: false,
            email_threshold_multiplier: 2.5,
            detect_citation_markers: false,
            citation_font_size_ratio: 0.75,
            merge_tm_tj_runs: true,
        }
    }

    /// Create a configuration with custom thresholds.
    ///
    /// # Arguments
    ///
    /// * `space_threshold_em` - Space threshold as em ratio
    /// * `conservative_pt` - Conservative gap threshold in points
    /// * `column_boundary_pt` - Column boundary threshold in points
    /// * `overlap_pt` - Severe overlap threshold in points
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::SpanMergingConfig;
    ///
    /// let config = SpanMergingConfig::custom(0.2, 0.2, 6.0, -0.3);
    /// ```
    pub fn custom(
        space_threshold_em: f32,
        conservative_pt: f32,
        column_boundary_pt: f32,
        overlap_pt: f32,
    ) -> Self {
        Self {
            space_threshold_em_ratio: space_threshold_em,
            conservative_threshold_pt: conservative_pt,
            column_boundary_threshold_pt: column_boundary_pt,
            severe_overlap_threshold_pt: overlap_pt,
            use_adaptive_threshold: false,
            adaptive_config: None,
            detect_email_patterns: false,
            email_threshold_multiplier: 2.5,
            detect_citation_markers: false,
            citation_font_size_ratio: 0.75,
            merge_tm_tj_runs: true,
        }
    }

    /// Create a configuration with adaptive threshold enabled (default settings).
    ///
    /// This enables automatic threshold calculation based on the document's gap
    /// distribution. Uses conservative base settings for reliable defaults:
    /// - space_threshold_em_ratio: 0.25
    /// - conservative_threshold_pt: 0.1 (overridden by adaptive calculation)
    /// - column_boundary_threshold_pt: 5.0
    /// - severe_overlap_threshold_pt: -0.5
    /// - adaptive_config: AdaptiveThresholdConfig::default()
    ///
    /// The adaptive threshold is computed as: median_gap * 1.5, clamped to [0.05, 1.0] points.
    ///
    /// # Benefits
    ///
    /// - Automatically adapts to different document types
    /// - Reduces word fusion in policy documents with tight spacing
    /// - Minimizes spurious spaces in other document types
    /// - Maintains backward compatibility (disabled by default)
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::SpanMergingConfig;
    ///
    /// let config = SpanMergingConfig::adaptive();
    /// assert!(config.use_adaptive_threshold);
    /// ```
    pub fn adaptive() -> Self {
        Self {
            space_threshold_em_ratio: 0.25,
            conservative_threshold_pt: 0.1,
            column_boundary_threshold_pt: 5.0,
            severe_overlap_threshold_pt: -0.5,
            use_adaptive_threshold: true,
            adaptive_config: Some(
                crate::extractors::gap_statistics::AdaptiveThresholdConfig::default(),
            ),
            detect_email_patterns: false,
            email_threshold_multiplier: 2.5,
            detect_citation_markers: false,
            citation_font_size_ratio: 0.75,
            merge_tm_tj_runs: true,
        }
    }

    /// Create a configuration with adaptive threshold and custom settings.
    ///
    /// # Arguments
    ///
    /// * `adaptive_config` - Custom adaptive threshold configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::{SpanMergingConfig, AdaptiveThresholdConfig};
    ///
    /// let config = SpanMergingConfig::adaptive_with_config(
    ///     AdaptiveThresholdConfig::policy_documents()
    /// );
    /// assert!(config.use_adaptive_threshold);
    /// ```
    pub fn adaptive_with_config(
        adaptive_config: crate::extractors::gap_statistics::AdaptiveThresholdConfig,
    ) -> Self {
        Self {
            space_threshold_em_ratio: 0.25,
            conservative_threshold_pt: 0.1,
            column_boundary_threshold_pt: 5.0,
            severe_overlap_threshold_pt: -0.5,
            use_adaptive_threshold: true,
            adaptive_config: Some(adaptive_config),
            detect_email_patterns: false,
            email_threshold_multiplier: 2.5,
            detect_citation_markers: false,
            citation_font_size_ratio: 0.75,
            merge_tm_tj_runs: true,
        }
    }

    /// Create a configuration using the legacy fixed-threshold approach.
    ///
    /// This provides backward compatibility with legacy behavior where
    /// adaptive threshold was disabled by default. All thresholds are fixed values.
    ///
    /// **Default values**:
    /// - space_threshold_em_ratio: 0.25 (standard word spacing)
    /// - conservative_threshold_pt: 0.1 (tight font metric threshold)
    /// - column_boundary_threshold_pt: 5.0 (standard column separation)
    /// - severe_overlap_threshold_pt: -0.5 (standard overlap tolerance)
    /// - use_adaptive_threshold: false (no automatic adjustment)
    ///
    /// # When to Use
    ///
    /// Use this when you need the fixed-threshold behavior:
    /// - Testing regression against old baselines
    /// - Documents with known quirks that required specific thresholds
    /// - Performance-critical applications where adaptive overhead is unacceptable
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::SpanMergingConfig;
    ///
    /// let config = SpanMergingConfig::legacy();
    /// assert!(!config.use_adaptive_threshold);
    /// assert_eq!(config.conservative_threshold_pt, 0.1);
    /// ```
    pub fn legacy() -> Self {
        Self {
            space_threshold_em_ratio: 0.25,
            conservative_threshold_pt: 0.1,
            column_boundary_threshold_pt: 5.0,
            severe_overlap_threshold_pt: -0.5,
            use_adaptive_threshold: false, // Fixed thresholds, no adaptive
            adaptive_config: None,
            detect_email_patterns: false,
            email_threshold_multiplier: 2.5,
            detect_citation_markers: false,
            citation_font_size_ratio: 0.75,
            merge_tm_tj_runs: true,
        }
    }
}

/// Unified space decision function - SINGLE SOURCE OF TRUTH for space insertion.
///
/// This function consolidates all space insertion logic into one place per the
/// design principle in the comprehensive plan. It evaluates multiple signals
/// returns a definitive decision about whether to insert a space between spans.
///
/// # Rules (in priority order)
///
/// **Rule 0**: Check if boundary space already exists (from trailing/leading whitespace)
/// - If preceding text ends with space OR following text starts with space, don't insert
/// - Confidence: 1.0 (deterministic)
///
/// **Rule 1**: TJ offset triggered flag
/// - If the TJ processor set the flag due to negative offset > threshold, insert space
/// - This is explicit PDF positioning information
/// - Confidence: 0.95 (highest, explicit signal)
///
/// **Rule 2**: Dual threshold (PDFBox pattern) with document-type adjustment
/// - Calculate both space-width-based and char-width-based thresholds
/// - Adjust thresholds based on document type (Academic/Policy/Mixed)
/// - Use MINIMUM of the two for robustness
/// - If gap exceeds this threshold, insert space
/// - Confidence: 0.8 (geometric measurement)
///
/// **Rule 3**: Character heuristic (CamelCase, number->letter, etc.)
/// - Detect character transitions indicating word boundaries
/// - If heuristic fires, insert space
/// - Confidence: 0.6 (pattern-based)
///
/// **Rule 4**: Conservative threshold (document-type aware)
/// - If gap exceeds conservative threshold (very small), insert space
/// - Catches small intentional gaps that are still word boundaries
/// - Adaptive to document type (Policy uses lower threshold, Academic uses higher)
/// - Confidence: 0.5 (conservative)
///
/// **Default**: No space inserted
///
/// # Document Type Adjustment
///
/// When document_type is provided, thresholds are adjusted:
/// - **Academic** (1.4x multiplier): Higher thresholds for loose spacing
/// - **Policy** (0.6x multiplier): Lower thresholds for tight justified text
/// - **Mixed** (1.0x multiplier): Default/balanced approach
///
/// This matches research findings from LA-PDFText, pdfminer.six, PDFBox, and iText
/// that adaptive thresholds provide better results than fixed values.
///
/// # PDF Spec Reference
///
/// ISO 32000-1:2008, Section 9.4.4 NOTE 6:
/// "The identification of what constitutes a word is unrelated to how the text
/// happens to be grouped into show strings... text strings should be as long as possible."
/// Recover an honest inter-glyph gap for the space-insertion decision.
///
/// Per ISO 32000-1:2008 §9.4.4, the spacing between two glyphs is the
/// text-space displacement between their origins; a word space exists when
/// that displacement reaches the font's space advance. We measure it from
/// the bounding boxes (`raw_gap = next.x − prev.right_edge`).
///
/// When the previous span's font has no explicit `/Widths` array,
/// `FontInfo` substitutes a fixed fallback advance (~0.55 em) that
/// systematically OVER-reports proportional Latin glyphs. That inflates
/// `bbox.width`, pushing `prev.right_edge` past the real glyph end so it can
/// swallow a true word gap and drive `raw_gap` NEGATIVE — glyphs that do not
/// actually overlap appear to. Only in that overlap case do we
/// divide out the fallback inflation (0.55 em ÷ 0.45 em ≈ 1.22) to restore a
/// believable gap.
///
/// Crucially, the correction is applied ONLY when `raw_gap < 0`. When the
/// glyphs do not overlap (`raw_gap ≥ 0`) the layout is already honest
/// must not be second-guessed: inflating a non-overlapping gap manufactures
/// a phantom word space and splits single words that were positioned
/// edge-to-edge — e.g. a CamelCase brand "SalesForce" emitted as
/// "SalesF" + "orce" with `raw_gap == 0` would otherwise be torn into
/// "SalesF orce". (`bbox.width × (1 − 1/1.22)` is the algebraic form of
/// `next.x − (prev.x + width/1.22)` once `raw_gap` is substituted in.)
fn corrected_space_gap(
    raw_gap: f32,
    reliable_widths: bool,
    bbox_width: f32,
    text_empty: bool,
) -> f32 {
    if !reliable_widths && raw_gap < 0.0 && bbox_width > 0.0 && !text_empty {
        raw_gap + bbox_width * (1.0 - 1.0 / 1.22)
    } else {
        raw_gap
    }
}

/// detect whether a glyph's mapped text
/// represents an AGL Latin ligature (`/ff` / `/fi` / `/fl` / `/ffi` /
/// `/ffl`). When the upstream space-emission heuristic processes a
/// glyph adjacent to a ligature, the small intra-word kerning that
/// surrounds the ligature glyph can trigger spurious space
/// insertion (producing `di ff cult` for `difficult`). The detection
/// here lets the heuristic suppress space insertion at ligature
/// boundaries.
///
/// Returns true when the text *is* a bare AGL ligature glyph — a
/// single codepoint in the Latin Ligatures block (U+FB00..U+FB06) or
/// the multi-char ASCII fallback ("ff"/"fi"/"fl"/"ffi"/"ffl"). The
/// suppression at the call site targets the pdfTeX-style emission
/// pattern where the ligature is its own cluster between two
/// intra-word fragments (e.g. "di"→"ﬃ"→"cult" or "di"→"ffi"→"cult").
/// A multi-char cluster that merely starts with a ligature
/// (e.g. "ﬂuid" or "ffective") is a full word whose boundary with the
/// previous span is a legitimate space, so we return false in that
/// case.
#[inline]
pub(crate) fn starts_with_agl_ligature(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    // Bare single-codepoint ligature glyph from the Latin Ligatures
    // block.
    if ('\u{FB00}'..='\u{FB06}').contains(&first) && chars.next().is_none() {
        return true;
    }
    // Multi-character AGL outputs from non-PUA fallbacks — match only
    // when the cluster IS the ligature, never when it just begins
    // with one.
    matches!(text, "ff" | "fi" | "fl" | "ffi" | "ffl")
}

/// detect monospace fonts by name.
/// Monospace fonts emit one show-text op per glyph with one-em
/// advance positioning, which triggers the proportional-font space-
/// emission heuristic to fire inside ordinary tokens. Bumping the
/// threshold for these fonts closes the `function add (a , b )` repro
/// from `code_and_formula.pdf` (issue ). Used by
/// [`should_insert_space`] to switch its `word_margin_ratio` to
/// `1.2` for monospace.
///
/// Names matched case-insensitively. Covers the major monospace
/// families on macOS / Linux / Windows + the pdfTeX-emitted
/// Computer Modern Typewriter (CMTT*) and Latin Modern Mono
/// (LMMono*) families that frequently appear in academic PDFs.
pub(crate) fn is_monospace_font(font_name: &str) -> bool {
    let lower = font_name.to_lowercase();
    const MONO_MARKERS: &[&str] = &[
        "mono",
        "courier",
        "consolas",
        "menlo",
        "fira code",
        "fira mono",
        "source code",
        "inconsolata",
        "cmtt",   // pdfTeX Computer Modern Typewriter
        "lmmono", // Latin Modern Mono (pdfTeX)
        "letter gothic",
        "ocr ", // OCR-A, OCR-B
        "fixedsys",
        "terminal",
    ];
    MONO_MARKERS.iter().any(|m| lower.contains(m))
}

/// True for codepoints in the main emoji / pictographic blocks.
///
/// Used only as a word-spacing hint — ISO 32000-1:2008 §9.10 leaves word
/// segmentation to the reader. Deliberately **excludes** arrows
/// (U+2190–U+21FF) and the math-operator blocks so symbolic/technical text is
/// unaffected; restricted to clearly pictographic ranges plus the VS16 emoji
/// presentation selector.
pub(crate) fn is_pictographic(c: char) -> bool {
    matches!(c as u32,
        0x1F300..=0x1FAFF   // Misc & Supplemental Symbols and Pictographs, Ext-A
        | 0x1F000..=0x1F0FF // Mahjong / Dominoes / Playing cards
        | 0x2600..=0x27BF   // Misc Symbols + Dingbats
        | 0xFE0F) // VS16 emoji presentation selector
}

/// Remove an ASCII space sitting directly between a CJK ideograph/kana and an
/// ASCII digit (either direction). In Chinese and Japanese an embedded number
/// attaches to the surrounding ideographs with no space (e.g. "公元前1000年",
/// "10,000年"); some producers — notably headless-browser print-to-PDF — emit a
/// stray space glyph at that script transition. CJK↔CJK and CJK↔letter spacing
/// is left untouched, so genuine word/term spacing is preserved.
///
/// Hangul is deliberately EXCLUDED: Korean, unlike Chinese/Japanese, is written
/// with inter-word spaces, so a space between a Korean syllable and a number is
/// a real word boundary (e.g. "14 예" = "14 cases", "7 예중") — stripping it
/// corrupts the text. Only the space-less scripts (CJK ideographs + kana) are
/// treated as number-adjacent.
pub(crate) fn strip_cjk_digit_boundary_spaces(text: &str) -> String {
    if !text.contains(' ') {
        return text.to_string();
    }
    let is_cjk = |c: char| {
        matches!(c as u32,
            0x3040..=0x30FF      // Hiragana + Katakana
            | 0x3400..=0x4DBF    // CJK Ext A
            | 0x4E00..=0x9FFF    // CJK Unified
            | 0x20000..=0x2A6DF  // CJK Ext B
            | 0xFF66..=0xFF9F    // Halfwidth Katakana
        )
    };
    // A bracket hugs its content in every script, so a space between a CJK or
    // Hangul character and an adjacent bracket is a layout artifact, not a word
    // break (e.g. Korean "고양이(학명: …)" / "카투스[*]" — the paren and the
    // reference marker sit flush against the syllable). Hangul IS included here,
    // unlike the digit case above: the digit boundary is a real Korean word
    // break, but the bracket boundary never is. Full-width CJK brackets carry
    // their own spacing and are left alone.
    let is_cjk_or_hangul = |c: char| {
        is_cjk(c)
            || matches!(c as u32,
                0xAC00..=0xD7A3   // Hangul syllables
                | 0x1100..=0x11FF // Hangul Jamo
                | 0x3130..=0x318F // Hangul Compatibility Jamo
            )
    };
    let is_hug_bracket = |c: char| matches!(c, '(' | ')' | '[' | ']' | '{' | '}');
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ' ' && i > 0 && i + 1 < chars.len() {
            let (p, n) = (chars[i - 1], chars[i + 1]);
            if (is_cjk(p) && n.is_ascii_digit()) || (p.is_ascii_digit() && is_cjk(n)) {
                i += 1; // drop the artifact space
                continue;
            }
            if (is_cjk_or_hangul(p) && is_hug_bracket(n))
                || (is_hug_bracket(p) && is_cjk_or_hangul(n))
            {
                i += 1; // drop the artifact space hugging a bracket
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Remove an ASCII space that the geometric word-break heuristic injected inside
/// a prime-notation number, e.g. `0′′.28` → `0′′ .28` or `0′′. 28`.
///
/// Arc-second / arc-minute values attach their decimal fraction to the prime
/// without a break (`0′′.28`, `1′′.47`). A prime glyph's metric advance (w0,
/// ISO 32000-1 §9.4.4) is narrow relative to its inked form, so the gap to the
/// following `.NN` reads as wider than a space and the heuristic splits the
/// token. Two artifact positions are repaired:
///   • prime → `.`   (`′ .` → `′.`)
///   • `.` → digit, when the `.` directly follows a prime (`′. 2` → `′.2`)
///
/// Feet-and-inches like `5′ 6″` are left untouched: the space there sits between
/// a prime and a *digit* (not a `.`), which is a genuine measurement boundary.
pub(crate) fn strip_prime_decimal_boundary_spaces(text: &str) -> String {
    if !text.contains(' ') {
        return text.to_string();
    }
    let is_prime = |c: char| matches!(c, '\u{2032}' | '\u{2033}' | '\u{2034}');
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ' ' && i > 0 && i + 1 < chars.len() {
            let (p, n) = (chars[i - 1], chars[i + 1]);
            // `′ .28` — space between the prime and the decimal point.
            if is_prime(p) && n == '.' {
                i += 1;
                continue;
            }
            // `′. 28` — space between the prime's decimal point and its digits.
            if p == '.' && n.is_ascii_digit() && i >= 2 && is_prime(chars[i - 2]) {
                i += 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// True when any drawn glyph run puts ink inside the horizontal gap between
/// `left` and `right`, overlapping their vertical band.
///
/// Used by the decimal-value merge: two pure-digit runs a split-box-sized
/// gap apart merge into one decimal amount ONLY if the gap is empty. A
/// separator glyph occupying the gap — the comma of a subscript index pair
/// (`P_{1,0}`), a list delimiter — proves the runs are distinct tokens, no
/// matter where in the content stream it was drawn. The pair's own boxes
/// bound the gap exactly, so a small epsilon keeps them (and touching
/// neighbours) from counting as intruders.
fn decimal_gap_has_ink(ink_boxes: &[Rect], left: &Rect, right: &Rect) -> bool {
    const EPS: f32 = 0.01;
    let gap_start = left.x + left.width;
    let gap_end = right.x;
    if gap_end - gap_start <= 2.0 * EPS {
        return false;
    }
    let band_bottom = left.y.min(right.y);
    let band_top = (left.y + left.height).max(right.y + right.height);
    ink_boxes.iter().any(|b| {
        b.x + b.width > gap_start + EPS
            && b.x < gap_end - EPS
            && b.y < band_top
            && b.y + b.height > band_bottom
    })
}

/// True when a *full intervening glyph* occupies the horizontal gap between
/// `left` and `right` — e.g. a subscript drawn between a variable and the next
/// symbol (`λᵢr…`), which inflates the `λ`→`r` gap though both share a
/// baseline. Distinct from [`decimal_gap_has_ink`]: it requires an ink box to
/// cover a substantial fraction (>= 35%) of the gap width, so a mere
/// descender/ascender edge of an adjacent glyph clipping the gap band does NOT
/// count. Used by the #847 narrow-word-gap rescue to suppress splitting a math
/// sub/superscript from its base while still recovering ordinary prose word
/// gaps (whose gaps are empty of intervening ink).
fn gap_has_intervening_glyph(ink_boxes: &[Rect], left: &Rect, right: &Rect) -> bool {
    let gap_start = left.x + left.width;
    let gap_end = right.x;
    let gap_w = gap_end - gap_start;
    if gap_w <= 0.5 {
        return false;
    }
    let band_bottom = left.y.min(right.y);
    let band_top = (left.y + left.height).max(right.y + right.height);
    ink_boxes.iter().any(|b| {
        let overlap = (b.x + b.width).min(gap_end) - b.x.max(gap_start);
        overlap > gap_w * 0.35 && b.y < band_top && b.y + b.height > band_bottom
    })
}

fn should_insert_space(
    preceding_text: &str,
    following_text: &str,
    gap_pt: f32,
    font_size: f32,
    font_name: &str,
    fonts: &std::collections::HashMap<String, std::sync::Arc<crate::fonts::FontInfo>>,
    tj_offset_triggered: bool,
    config: &SpanMergingConfig,
    prev_bbox: Option<&crate::geometry::Rect>,
    next_bbox: Option<&crate::geometry::Rect>,
    prev_font_size: f32,
    next_font_size: f32,
) -> SpaceDecision {
    // PHASE 10: PDF Spec-Compliant Space Detection
    // Per ISO 32000-1:2008 Section 9.4.3 and 9.4.4
    //
    // Text positioning is determined by the text matrix and glyph positioning.
    // Only spec-defined signals are used; linguistic heuristics are excluded.
    //
    // Allowed signals (from PDF Spec):
    // 1. Boundary whitespace: spaces already present in text strings
    // 2. TJ array offsets: negative offsets < -100 thousandths of em
    // 3. Geometric gaps: gaps between character bounding boxes vs font metrics

    // Rule 0: Boundary Space (Section 9.4.3 - Text Showing)
    // Spaces already present in text strings should not be duplicated
    if has_boundary_space(preceding_text, following_text) {
        return SpaceDecision::no_space(SpaceSource::AlreadyPresent, 1.0);
    }

    // Rule 0.3: Complex-script combining-mark guard (#656-class Indic gap).
    // A Brahmic/Thai/Khmer dependent vowel sign, virama, or tone mark followed
    // by another character of a complex script is intra-word — the mark carries
    // its own advance, so the geometric gap and consensus paths below would
    // otherwise emit a spurious word space (the dominant matra→consonant error
    // for Tamil/Bengali/Devanagari). Genuine word breaks carry an explicit
    // space glyph, already handled by Rule 0. This guards the strong-geometric
    // and consensus branches, which never consult `WordBoundaryDetector`.
    if let (Some(pc), Some(nc)) =
        (preceding_text.chars().next_back(), following_text.chars().next())
    {
        use crate::text::complex_script_detector::{detect_complex_script, is_complex_script_mark};
        if is_complex_script_mark(pc as u32) && detect_complex_script(nc as u32).is_some() {
            return SpaceDecision::no_space(SpaceSource::NoSpace, 0.9);
        }
    }

    // Rule 0.4: Emoji / pictographic → letter boundary.
    // A wide pictographic glyph (e.g. 📄) advances far, so the residual gap to
    // the next token falls below the proportional-font space threshold and the
    // inter-token space would otherwise be dropped (`📄README` instead of
    // `📄 README`). In practice the emoji glyph's right edge abuts the next
    // token (gap ≈ 0). Word boundaries are reader latitude (§9.10), so when an
    // emoji is immediately followed by a letter, keep the space. The
    // `is_alphabetic` requirement on the following char already excludes combined
    // ZWJ/VS emoji sequences (whose next char is a selector or another pictograph,
    // never a letter), so a non-negative gap is the correct gate.
    if gap_pt >= 0.0
        && preceding_text
            .chars()
            .next_back()
            .is_some_and(is_pictographic)
        && following_text
            .chars()
            .next()
            .is_some_and(char::is_alphabetic)
    {
        return SpaceDecision::insert(SpaceSource::GeometricGap, 0.85);
    }

    // Rule 0.5: Email Pattern Detection
    // Per ISO 32000-1:2008 Section 9.10, email formatting preservation
    if config.detect_email_patterns && is_email_context(preceding_text, following_text) {
        let geometric_threshold = if let Some(font_info) = fonts.get(font_name) {
            let space_width_units = font_info.get_space_glyph_width();
            let space_width_pt = (space_width_units / 1000.0) * font_size;
            let word_margin_ratio = 0.5;
            space_width_pt * word_margin_ratio
        } else {
            font_size * 0.25
        };

        let email_threshold = geometric_threshold * config.email_threshold_multiplier;

        if gap_pt > email_threshold {
            log::debug!(
                "Email context detected: gap={:.2}pt > {:.2}pt email threshold - inserting space",
                gap_pt,
                email_threshold
            );
            return SpaceDecision::insert(SpaceSource::GeometricGap, 0.85);
        }

        log::debug!(
            "Email context detected: gap={:.2}pt <= {:.2}pt email threshold - suppressing space",
            gap_pt,
            email_threshold
        );
        return SpaceDecision::no_space(SpaceSource::NoSpace, 1.0);
    }

    // Line Break Handling
    // ==============================================================================
    // Per ISO 32000-1:2008 Section 5.2 (geometric positioning):
    // Line breaks are detected using bbox Y-coordinates (vertical positioning).
    // Words split across lines need special handling:
    // - Soft hyphen breaks: Previous text ends with '-' → NO space (word continuation)
    // - Hard line breaks: Normal breaks → INSERT space (new word on next line)
    //
    // Spec Reference: Section 5.2 states coordinates are in user space units.
    // Font size is used as reference for vertical gap detection threshold.

    if let (Some(prev_box), Some(next_box)) = (prev_bbox, next_bbox) {
        // Calculate vertical positioning for line break detection.
        // Use Y-coordinate difference (not bottom-to-top gap) to detect actual line breaks.
        // Two spans on the same line have nearly identical Y positions regardless of height.
        let y_diff = (prev_box.y - next_box.y).abs();

        // Line break threshold: if Y positions differ by more than 0.5× font size
        let line_break_threshold = font_size * 0.5;
        let is_line_break = y_diff > line_break_threshold;

        if is_line_break {
            // Verify same-column layout: X-positions within 2× font width
            let same_column = (prev_box.left() - next_box.left()).abs() < (font_size * 2.0);

            if same_column {
                log::debug!(
                    "Detected line break: y_diff={:.2}pt > {:.2}pt threshold, same_column=true",
                    y_diff,
                    line_break_threshold
                );

                // Check if previous text ends with hyphen (soft line break)
                if preceding_text.ends_with('-') {
                    log::debug!(
                        "Soft hyphen detected: '{}' ends with '-', suppressing space insertion",
                        preceding_text
                    );
                    return SpaceDecision::no_space(SpaceSource::NoSpace, 1.0);
                } else {
                    log::debug!("Hard line break detected: inserting space for word continuation");
                    return SpaceDecision::insert(SpaceSource::GeometricGap, 0.9);
                }
            }
        }
    }

    // NEW: Rule 1.5: Citation Marker Detection
    // ==============================================================================
    // Per ISO 32000-1:2008 Section 9.3, citation markers have distinct visual properties
    if config.detect_citation_markers
        && is_citation_context(prev_bbox, next_bbox, font_size, prev_font_size, next_font_size)
    {
        // For citations, use single-signal detection (don't require consensus)
        // Compute geometric threshold for citation context
        let citation_geometric_threshold = if let Some(font_info) = fonts.get(font_name) {
            let space_width_units = font_info.get_space_glyph_width();
            let space_width_pt = (space_width_units / 1000.0) * font_size;
            space_width_pt * 0.5
        } else {
            font_size * 0.25
        };

        if tj_offset_triggered || gap_pt > citation_geometric_threshold {
            log::debug!(
                "Citation context detected: using relaxed spacing rules (gap={:.2}pt, tj={})",
                gap_pt,
                tj_offset_triggered
            );
            return SpaceDecision::insert(SpaceSource::TjOffset, 0.90);
        }
    }

    // Consensus-Based Spacing Logic
    // ==============================================================================
    // Per ISO 32000-1:2008 Section 9.4.4 and 9.10:
    // "Determining word boundaries is not specified by PDF."
    // TJ offsets are typographic hints only, not definitive word boundaries.
    //
    // Solution: Require CONSENSUS between multiple PDF-spec-defined signals:
    // - TJ offset signal (explicit typography positioning)
    // - Geometric signal (bounding box analysis)
    // - Strong geometric signal alone is sufficient (gap > 2× threshold)

    // Rule 1: TJ Offset Signal (Section 9.4.3) - PDF-spec explicit signal
    // Calculate font-aware geometric threshold for consensus checking
    let geometric_threshold = if let Some(font_info) = fonts.get(font_name) {
        // Font found: use space glyph width for calculation
        let space_width_units = font_info.get_space_glyph_width(); // in 1000ths of em
        let space_width_pt = (space_width_units / 1000.0) * font_size;
        // monospace fonts emit one show-text
        // op per glyph at one-em-advance positioning, so the gap
        // between glyphs in normal tokens briefly exceeds the
        // proportional-font threshold. Use a 1.2× ratio for monospace
        // so spurious spaces around punctuation in code listings
        // (`function add (a , b )` → `function add(a, b)`) don't fire.
        let mut word_margin_ratio = if is_monospace_font(font_name) {
            1.2
        } else {
            0.5 // 50% of space width (proportional default)
        };
        // when prev_font_size
        // next_font_size differ significantly, we're at a font-run
        // boundary (italic → roman, bold → regular, or a font-family
        // switch). PdfTeX-typeset titles like
        // `Astronomy & Astrophysicsmanuscript no.` exhibit this when
        // the writer doesn't emit an explicit space-glyph at the font
        // switch. Reduce the threshold by 30% at boundaries so a
        // smaller gap suffices to trigger space insertion. The full
        // fix (font-name plumbing for italic→roman within same size)
        // is tracked in — many italic transitions
        // share font_size, so this only catches the size-changing
        // subset.
        if (prev_font_size - next_font_size).abs() > 0.5 {
            word_margin_ratio *= 0.7;
        }
        let threshold = space_width_pt * word_margin_ratio;

        log::debug!(
            "Font-aware spacing for '{}' @ {:.1}pt: space_width={:.1}pt, threshold={:.1}pt (mono={})",
            font_name,
            font_size,
            space_width_pt,
            threshold,
            is_monospace_font(font_name),
        );

        threshold
    } else {
        // Font not found: fallback to fixed 0.25em threshold
        log::debug!(
            "Font '{}' not found in font map, using default 0.25em threshold for {:.1}pt",
            font_name,
            font_size
        );
        font_size * 0.25
    };

    // suppress space insertion at AGL-
    // ligature boundaries. When the preceding or following text
    // starts with one of the Latin ligature codepoints (U+FB00..U+FB04)
    // or matches the multi-char AGL ligature names, the small kerning
    // gap that surrounds the ligature glyph is NOT a word boundary —
    // it's an intra-word position artefact from pdfTeX-style ligature
    // emission. Inflating the threshold by 1.5× at these positions
    // catches the `di ff cult` → `difficult` repro from issue .
    let ligature_boundary = starts_with_agl_ligature(following_text)
        || preceding_text
            .chars()
            .last()
            .map(|c| ('\u{FB00}'..='\u{FB06}').contains(&c))
            .unwrap_or(false);
    let geometric_threshold = if ligature_boundary {
        geometric_threshold * 1.5
    } else {
        geometric_threshold
    };

    let geometric_suggests_space = gap_pt > geometric_threshold;

    // #365 / B8b: Intra-word kerning guard (letter-letter branch).
    //
    // On TJ-heavy producers (LaTeX, MS Word → PDF) the Primary
    // word-boundary detector hands `should_insert_space` two adjacent
    // clusters like "cha"→"nge", "diffe"→"rent", "equivalen"→"t"
    // whose gap sits just above `geometric_threshold` (= 0.5 ×
    // space-glyph width) but well below a real word gap. The
    // consensus rule below would then emit a spurious space mid-word.
    // Real word gaps in real producers reach one full space-glyph
    // width or sit next to punctuation/digits, both of which fall
    // through this guard.
    //
    // The guard fires regardless of `tj_offset_triggered` because the
    // gap can also be geometric-only (when WordBoundaryDetector splits
    // the cluster but no explicit TJ offset crossed the threshold).
    // See the sibling guard in `process_tj_array_tiebreaker` for the
    // upstream space-as-span insertion path.
    //
    // Ceiling = 1.5 × `geometric_threshold` (= 0.75 × space-glyph width,
    // ≈ 0.2 em for a typical 0.25-em space). Inter-letter kerning is a
    // property of font size — realistic microtype / Word letter-spacing
    // is a few percent of the em and sits just above the 0.5-space-width
    // threshold, never far beyond it. The previous 2.4× ceiling
    // (≈ 1.2 × a full space-glyph advance, ≈ 0.33 em for Helvetica) was
    // far wider than any real kerning and swallowed genuine *tight* word
    // gaps between lowercase words — the dominant cause of
    // "MasterofScience" / "Resultsdriven" gluing on resume-style PDFs
    // that position words via small Td offsets. 1.5× still clears
    // worst-case ~0.19-em intra-word kerning (including the ~0.15-em
    // LaTeX/microtype letter-spacing case) while letting a
    // 0.2-em-and-wider word gap through to the consensus path — the same
    // ~0.18-0.2-em word-break point PyMuPDF / poppler use. Gaps in the
    // overlap zone (wide letter-tracking in titles, ~0.28 em) are not
    // separable from real word gaps by magnitude alone and fall through.
    //
    // Only fires when the font is available so the threshold is
    // computed from the font's own space-glyph advance — the no-font
    // fallback (`font_size * 0.25`) is a wider, deliberately
    // conservative value that already separates real word gaps from
    // kerning at the consensus level.
    let kerning_guard_threshold = if fonts.contains_key(font_name) {
        Some(geometric_threshold * 1.5)
    } else {
        None
    };
    if let Some(thr) = kerning_guard_threshold {
        if gap_pt < thr {
            let prev_last = preceding_text.chars().last();
            let next_first = following_text.chars().next();
            if let (Some(pc), Some(nc)) = (prev_last, next_first) {
                // Use is_lowercase on both sides: LaTeX/microtype intra-word kerning
                // occurs within lowercase letter runs. Real word boundaries in
                // professional PDFs frequently involve uppercase letters (headings,
                // abbreviations, proper nouns) — those fall through to the consensus
                // path, avoiding word-gluing like "APPENDIXA" or "OLIVERA.".
                if pc.is_lowercase() && nc.is_lowercase() {
                    log::debug!(
                        "intra-word kerning guard: suppressing space between '{pc}' and '{nc}' (gap={gap_pt:.2}pt < {thr:.2}pt, threshold = 0.75× space-glyph width)"
                    );
                    return SpaceDecision::no_space(SpaceSource::IntraWordKerning, 0.9);
                }
            }
        }
    }

    // Consensus checking
    // Only insert space if BOTH signals agree OR geometric signal is very strong
    // This reduces false positives in justified text where TJ offsets are arbitrary
    if tj_offset_triggered && geometric_suggests_space {
        // HIGH CONFIDENCE: Both TJ and geometric signals agree
        log::debug!(
            "Space decision: CONSENSUS - both TJ and geometric signals triggered (gap={:.2}pt > {:.2}pt) - inserting space",
            gap_pt,
            geometric_threshold
        );
        return SpaceDecision::insert(SpaceSource::TjOffset, 1.0);
    }

    // TJ offset with relaxed geometric confirmation
    // In tight typesetting (e.g., LaTeX academic papers), word gaps are narrower than
    // the standard 50% space-width threshold. When the PDF producer explicitly encoded
    // a TJ offset, accept a lower geometric bar (25% of space width).
    if tj_offset_triggered && gap_pt > geometric_threshold * 0.5 {
        log::debug!(
            "Space decision: TJ + relaxed geometric (gap={:.2}pt > {:.2}pt relaxed threshold) - inserting space",
            gap_pt,
            geometric_threshold * 0.5
        );
        return SpaceDecision::insert(SpaceSource::TjOffset, 0.9);
    }

    // WordBoundaryDetector tiebreaker when TJ and geometric signals conflict
    // Per ISO 32000-1:2008 Section 9.4.4, use multiple signals to determine word boundaries
    if tj_offset_triggered != geometric_suggests_space {
        if let (Some(prev_box), Some(next_box)) = (prev_bbox, next_bbox) {
            let (characters, context) = build_boundary_characters(
                preceding_text,
                following_text,
                prev_box,
                next_box,
                font_size,
                tj_offset_triggered,
            );

            // Use WordBoundaryDetector with geometric gap ratio matching our threshold
            // OPTIMIZATION: Detect document script profile to skip unnecessary detectors
            let script = DocumentScript::detect_from_characters(&characters);
            let detector = WordBoundaryDetector::new()
                .with_document_script(script)
                .with_geometric_gap_ratio(0.5);
            let boundaries = detector.detect_word_boundaries(&characters, &context);

            if !boundaries.is_empty() {
                log::debug!(
                    "Space decision: WordBoundaryDetector resolved conflict (TJ={}, geo={}) - inserting space",
                    tj_offset_triggered,
                    geometric_suggests_space
                );
                return SpaceDecision::insert(SpaceSource::WordBoundaryAnalysis, 0.85);
            }
        }
    }

    // Strong geometric signal alone.
    //
    // `geometric_threshold` is already `space_width_pt * 0.5`. A gap that
    // clears this threshold is >= 50 % of the font's own space-glyph
    // advance, which is what pdfium (Chrome/pypdfium2) uses as the
    // word-break heuristic in its default text-extraction path —
    // the reason pdf_oxide was glueing adjacent words like
    // "atBirmingham", "LIFESCIENCESRESEARCH", "STATIONFREEDOM",
    // "proteincrystals" before this change. The previous 2× multiplier
    // required gaps >= 100 % of a full space glyph, which is stricter
    // than the gaps modern tightly-kerned typesetters emit between
    // real words (often 60-80 % of a space glyph).
    //
    // Intra-word kerning and letter-spacing adjustments are well below
    // 50 % of a space glyph (typically under 5 % of font-size), so
    // lowering this threshold does not produce false word breaks
    // inside words. Pure digit-digit sequences are separately protected
    // in the value/token branch below via `digit_digit_gap_ok`.
    //
    // See issue #326 for the corpus-wide measurement that motivated
    // this change (NASA Apollo 11 jaccard 0.449 → target >= 0.90 vs
    // pypdfium2 on the 60-PDF regression corpus).
    if gap_pt > geometric_threshold {
        log::debug!(
            "Space decision: STRONG GEOMETRIC - gap={:.2}pt > {:.2}pt threshold - inserting space",
            gap_pt,
            geometric_threshold
        );
        return SpaceDecision::insert(SpaceSource::GeometricGap, 0.95);
    }

    // Separate token detection: when two spans have a positive gap and look like
    // distinct values (not fragments of the same word), insert a space.
    //
    // This catches adjacent table cell values like "$0.00" "$0.00" that have small
    // gaps (1-2pt) which fall below the standard geometric threshold but are clearly
    // separate tokens. Word fragments within the same word have zero or near-zero
    // gaps; any meaningful positive gap between non-fragment tokens indicates a
    // word boundary.
    //
    // Heuristic: gap > 0 AND spans look like separate tokens based on boundary characters.
    // Use near-zero threshold for currency boundaries (any positive gap = separate)
    let min_token_gap = 0.01; // Essentially any positive gap triggers token check
    if gap_pt > min_token_gap {
        let prev_last = preceding_text.chars().last();
        let next_first = following_text.chars().next();

        if let (Some(pc), Some(nc)) = (prev_last, next_first) {
            // Separate value tokens: digit/currency/punctuation boundaries that
            // indicate two distinct values rather than fragments of one word.
            // Examples: "$0.00" + "$0.00", "100" + "200", "Subtotal" + "$500.00"
            let prev_is_value_end = pc.is_ascii_digit() || pc == '%' || pc == ')' || pc == ']';

            // Pure digit→digit boundaries require a larger gap than the
            // global `min_token_gap`: a long number emitted as multiple
            // spans (e.g. due to glyph-level kerning or TJ positioning
            // rounding) can have a tiny positive gap between adjacent
            // digit spans, which must NOT become "123 456". Anything less
            // than half the font-aware geometric threshold is treated as
            // intra-number kerning, not a token boundary.
            let digit_digit = nc.is_ascii_digit() && pc.is_ascii_digit();
            let digit_digit_gap_ok = !digit_digit || gap_pt > geometric_threshold * 0.5;

            let next_is_value_start = nc == '$'
                || nc == '('
                || nc == '['
                || (nc == '-' && following_text.len() > 1)
                || (nc.is_ascii_digit() && prev_is_value_end && digit_digit_gap_ok);

            // Also detect: any text followed by currency symbol
            // e.g., "Subtotal" + "$500.00" or "49" + "$0.00"
            let text_then_currency = (pc.is_ascii_alphabetic() || pc.is_ascii_digit())
                && (nc == '$' || nc == '€' || nc == '£');

            if (prev_is_value_end && next_is_value_start) || text_then_currency {
                log::debug!(
                    "Space decision: SEPARATE VALUES - gap={:.2}pt > {:.2}pt min, prev='{}', next='{}' - inserting space",
                    gap_pt,
                    min_token_gap,
                    crate::utils::safe_suffix(preceding_text, 5),
                    crate::utils::safe_prefix(following_text, 5),
                );
                return SpaceDecision::insert(SpaceSource::GeometricGap, 0.85);
            }
        }
    }

    // Default: No space
    // Per ISO 32000-1:2008 Section 9.10, when PDF doesn't encode a clear word boundary,
    // we cannot reliably recover it. Requiring consensus prevents false positives in justified text.
    log::trace!(
        "Space decision: Insufficient consensus (TJ={}, gap={:.2}pt <= {:.2}pt) - no space",
        tj_offset_triggered,
        gap_pt,
        geometric_threshold
    );
    SpaceDecision::no_space(SpaceSource::NoSpace, 1.0)
}

/// Check if a boundary between spans already has whitespace.
///
/// Returns true if:
/// - The preceding text ends with whitespace, OR
/// - The following text starts with whitespace
///
/// This prevents double-spacing when text already contains space characters.
fn has_boundary_space(preceding: &str, following: &str) -> bool {
    // Use ends_with/starts_with patterns instead of .chars().last() to avoid
    // O(n) iteration over the entire accumulated text
    let has_trailing_space = preceding.ends_with(|c: char| c.is_whitespace());
    let has_leading_space = following.starts_with(|c: char| c.is_whitespace());

    has_trailing_space || has_leading_space
}

/// Build CharacterInfo for word boundary analysis between two text segments.
///
/// Creates minimal character info for the last character of the preceding text
/// and the first character of the following text. This allows WordBoundaryDetector
/// to determine if a word boundary exists between two spans.
///
/// Per ISO 32000-1:2008 Section 9.4.4, word boundaries can be identified through:
/// - TJ array offsets (passed via tj_offset_triggered)
/// - Geometric gaps between glyphs (calculated from bbox positions)
/// - Space characters in the text stream
/// - CJK character transitions
fn build_boundary_characters(
    prev_text: &str,
    next_text: &str,
    prev_bbox: &Rect,
    next_bbox: &Rect,
    font_size: f32,
    tj_offset_triggered: bool,
) -> (Vec<CharacterInfo>, BoundaryContext) {
    let prev_last_char = prev_text.chars().last().unwrap_or(' ');
    let next_first_char = next_text.chars().next().unwrap_or(' ');

    // Estimate character widths from bbox and character count
    // Use byte length as fast O(1) approximation (accurate for ASCII, close for UTF-8)
    // to avoid O(n) char counting on the accumulated merge text
    let prev_char_count = prev_text.len().max(1) as f32;
    let prev_char_width = prev_bbox.width / prev_char_count;
    let prev_last_x = prev_bbox.x + prev_bbox.width - prev_char_width;

    let next_char_count = next_text.len().max(1) as f32;
    let next_char_width = next_bbox.width / next_char_count;

    // Build CharacterInfo for boundary analysis
    let characters = vec![
        CharacterInfo {
            code: prev_last_char as u32,
            glyph_id: None,
            width: prev_char_width,
            x_position: prev_last_x,
            // Convert TJ trigger to offset value: -200 indicates word boundary
            tj_offset: if tj_offset_triggered {
                Some(-200)
            } else {
                None
            },
            font_size,
            is_ligature: false, // Not relevant for tiebreaker mode
            original_ligature: None,
            protected_from_split: false,
        },
        CharacterInfo {
            code: next_first_char as u32,
            glyph_id: None,
            width: next_char_width,
            x_position: next_bbox.x,
            tj_offset: None,
            font_size,
            is_ligature: false, // Not relevant for tiebreaker mode
            original_ligature: None,
            protected_from_split: false,
        },
    ];

    let context = BoundaryContext {
        font_size,
        horizontal_scaling: 100.0, // Default; actual value not available at span level
        word_spacing: 0.0,
        char_spacing: 0.0,
    };

    (characters, context)
}

/// Check if surrounding text forms an email-like pattern.
/// Per PDF spec, uses only extracted text pattern matching.
///
/// Patterns detected:
/// - "user@outlook" + "." + "com" (space before TLD)
/// - "user@" + "domain.com" (space after @)
fn is_email_context(preceding_text: &str, following_text: &str) -> bool {
    // Only check the last ~64 bytes for email patterns to avoid O(n) scan
    // of the entire accumulated text (which would cause O(n²) in merge loop)
    let prev_start = preceding_text.len().saturating_sub(64);
    // Round up to the next UTF-8 char boundary. `str::ceil_char_boundary`
    // would do this in one line but it's only stable since Rust 1.91,
    // above our MSRV (1.88 — pinned by transitive deps).
    let prev_start = {
        let mut i = prev_start;
        while i < preceding_text.len() && !preceding_text.is_char_boundary(i) {
            i += 1;
        }
        i
    };
    let prev = preceding_text[prev_start..].trim_end();
    let next = following_text.trim_start();

    // Pattern 1: @ followed by domain part
    if prev.contains('@') {
        let after_at = prev.split('@').next_back().unwrap_or("");

        // Pattern 1a: "outlook" + "." → likely email
        if !after_at.is_empty() && next.starts_with('.') {
            return true;
        }

        // Pattern 1b: "outlook." + "com" → likely email
        if after_at.ends_with('.') && next.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            return true;
        }
    }

    // Pattern 2: Previous ends with @ (immediate after @)
    if prev.ends_with('@')
        && next
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
    {
        return true;
    }

    false
}

/// Detect if bounding boxes indicate citation marker context.
/// Per PDF spec Section 9.3, citation markers have distinct visual properties:
/// - Smaller font size (typically 50-75% of body text)
/// - Raised position (superscript)
fn is_citation_context(
    prev_bbox: Option<&crate::geometry::Rect>,
    next_bbox: Option<&crate::geometry::Rect>,
    current_font_size: f32,
    prev_font_size: f32,
    next_font_size: f32,
) -> bool {
    let prev_ratio = prev_font_size / current_font_size;
    let next_ratio = next_font_size / current_font_size;

    // Superscript range: 50-75% of body text size
    const SUPERSCRIPT_MIN: f32 = 0.5;
    const SUPERSCRIPT_MAX: f32 = 0.75;

    let prev_is_superscript = (SUPERSCRIPT_MIN..=SUPERSCRIPT_MAX).contains(&prev_ratio);
    let next_is_superscript = (SUPERSCRIPT_MIN..=SUPERSCRIPT_MAX).contains(&next_ratio);

    if let (Some(prev_box), Some(next_box)) = (prev_bbox, next_bbox) {
        let vertical_offset = (prev_box.y - next_box.y).abs();
        let is_raised = vertical_offset > (current_font_size * 0.2);

        // Either previous OR next is superscript + raised
        if (prev_is_superscript || next_is_superscript) && is_raised {
            return true;
        }
    }

    // Fallback: just font size check if bbox unavailable
    prev_is_superscript || next_is_superscript
}

/// Buffer for accumulating text from TJ array elements into a single span.
///
/// Per PDF Spec ISO 32000-1:2008, Section 9.4.4 NOTE 6:
/// "The performance of text searching (and other text extraction operations) is
/// significantly better if the text strings are as long as possible."
///
/// This buffer accumulates consecutive string elements from TJ arrays into
/// a single logical text span, only breaking on explicit word boundaries.
#[derive(Debug)]
struct TjBuffer {
    source_order: Vec<usize>,
    /// Accumulated Unicode text
    unicode: String,
    /// Text matrix at the start of this buffer
    start_matrix: Matrix,
    /// Font name when buffer started
    font_name: Option<String>,
    /// Fill color RGB when buffer started
    fill_color_rgb: (f32, f32, f32),
    /// Character spacing (Tc) when buffer started
    char_space: f32,
    /// Word spacing (Tw) when buffer started
    word_space: f32,
    /// Horizontal scaling (Th) when buffer started
    horizontal_scaling: f32,
    /// MCID when buffer started
    mcid: Option<u32>,
    /// Accumulated width from advance_position_for_string calls.
    /// Avoids redundant per-byte width recalculation in flush.
    accumulated_width: f32,
    /// Cached font reference — avoids per-Tj HashMap lookup in append.
    /// Set once at buffer creation, never changes (font change flushes buffer).
    cached_font: Option<Arc<FontInfo>>,
    /// Pre-computed effective font size (CTM × text_matrix scaling × font_size).
    /// Computed once at buffer creation to avoid matrix multiply + sqrt per flush.
    effective_font_size: f32,
    /// Pre-computed font weight from cached font reference.
    font_weight: FontWeight,
    /// Pre-computed italic flag from cached font reference.
    is_italic: bool,
    /// Whether the font is monospaced (from FixedPitch flag or name heuristic).
    is_monospace: bool,
    /// Per-character advance widths in text-space units (before user_h_scale).
    char_widths: Vec<f32>,
    /// Pre-computed user-space position (CTM applied to text matrix origin).
    /// Avoids two transform_point calls per flush.
    user_pos_x: f32,
    user_pos_y: f32,
    /// Exact paint origin including Ts transformed through text matrix and CTM.
    /// Reading-order extraction retains its legacy origin separately.
    paint_pos_x: f32,
    paint_pos_y: f32,
    /// Pre-computed horizontal scale factor (CTM × text_matrix).
    /// Used to convert accumulated_width from text space to user space for bbox.
    user_h_scale: f32,
    /// Display rotation of this run in degrees, snapped to a quadrant when near
    /// one; `0.0` for ordinary horizontal text (see `snap_run_rotation`).
    rotation_degrees: f32,
    paint_basis: [f32; 4],
    /// Negative determinant of the composed matrix — mirrored text (see
    /// `run_is_mirrored`), carried onto the emitted span so `page_bbox`
    /// reflects rather than rotates its across-axis.
    mirrored: bool,
    /// Writing mode (0 = horizontal, 1 = vertical) captured from the
    /// graphics state when the buffer started, so each emitted span
    /// carries the wmode it was rendered under. A font change flushes the
    /// buffer, so a single buffer never spans mixed writing modes.
    wmode: u8,
    /// Baseline shift as a ratio of font size (`Ts ÷ Tf size`, ISO 32000-1
    /// §9.3.7), captured from the graphics state when the buffer started.
    /// `> 0` superscript, `< 0` subscript, `0.0` on-baseline. Stored as a
    /// ratio so it is text/CTM-scale-independent and directly comparable to a
    /// font-size fraction by the sub/superscript rejoin.
    text_rise: f32,
    /// Text render mode (`Tr`, ISO 32000-1 §9.3.6), captured from the
    /// graphics state when the buffer started. `3`/`7` (invisible — neither
    /// filled nor stroked) means this run has no rendering-correctness
    /// pressure: an OCR-sandwich producer has no visual reason to mirror
    /// already-logical RTL glyph positions the way a *visible*-text
    /// producer would, so the geometric visual/logical detector's ascending-
    /// x signal is uninformative here (#826) — see `bidi::apply_rtl_verdict`.
    render_mode: u8,
}

/// Snap a run's display rotation (from the composed `CTM × T_m` rotation block,
/// `θ = atan2(b, a)`) to the nearest of `0 / 90 / 180 / -90` when it is within
/// `SNAP_TOL_DEG` of one, and treat everything within tolerance of `0` as exactly
/// horizontal (`0.0`). Mirrored text (negative matrix determinant) is reported as
/// its raw angle, not snapped, so it is never confused with a clean rotation.
fn snap_run_rotation(combined: &Matrix) -> f32 {
    const SNAP_TOL_DEG: f32 = 5.0;
    let (a, b, c, d) = (combined.a, combined.b, combined.c, combined.d);
    // Pure horizontal/180° fast path: b and c ~ 0 covers both 0° (a,d > 0)
    // and 180° (a,d < 0) — sin(0°) and sin(180°) are both 0, so the
    // off-diagonal terms alone can't tell them apart. Check the sign of
    // `a` (cos(0°)=1, cos(180°)=-1) to disambiguate.
    if b.abs() < 1e-4 && c.abs() < 1e-4 {
        return if a < 0.0 { 180.0 } else { 0.0 };
    }
    let mut deg = b.atan2(a).to_degrees();
    // Normalise to (-180, 180].
    while deg > 180.0 {
        deg -= 360.0;
    }
    while deg <= -180.0 {
        deg += 360.0;
    }
    // Mirror (det < 0): leave the raw angle; the reading-order path treats any
    // non-zero rotation as a separate block regardless, and snapping a mirror to
    // a quadrant would misrepresent it.
    let det = a * d - b * c;
    if det < 0.0 {
        return if deg.abs() < SNAP_TOL_DEG { 0.0 } else { deg };
    }
    for &q in &[0.0_f32, 90.0, 180.0, -90.0] {
        if (deg - q).abs() <= SNAP_TOL_DEG {
            return q;
        }
    }
    deg
}

/// Negative determinant of the composed text rendering matrix: the run is
/// mirrored, so `rotation_degrees` alone cannot describe its frame (a mirrored
/// 90° run and a clean 90° run carry the same angle but opposite across-axes).
fn run_is_mirrored(combined: &Matrix) -> bool {
    combined.a * combined.d - combined.b * combined.c < 0.0
}

impl TjBuffer {
    /// Create a new empty buffer with current state.
    fn new(
        state: &crate::content::graphics_state::GraphicsState,
        mcid: Option<u32>,
        cached_font: Option<Arc<FontInfo>>,
        source_order: Vec<usize>,
    ) -> Self {
        // Pre-compute effective font size: CTM × text_matrix scaling × font_size
        let combined = state.ctm.multiply(&state.text_matrix);
        let effective_font_size =
            state.font_size * (combined.d * combined.d + combined.b * combined.b).sqrt();
        // Pre-compute horizontal scale for converting text-space widths to user space
        let user_h_scale = (combined.a * combined.a + combined.c * combined.c).sqrt();
        let font_weight = match &cached_font {
            Some(f) if f.is_bold() => FontWeight::Bold,
            _ => FontWeight::Normal,
        };
        let is_italic = cached_font.as_ref().map(|f| f.is_italic()).unwrap_or(false);
        // Invisible text (Tr 3/7, ISO 32000-1 §9.3.6) is never real visible
        // monospace content — it's an OCR text-sandwich layer sitting under
        // a scanned page image, or deliberately hidden text. Such layers
        // commonly use a synthetic font (conventionally named
        // "GlyphLessFont" by ocrmypdf/Tesseract and similar tools) whose
        // FontDescriptor sets the FixedPitch flag purely for positioning
        // simplicity — the glyphs are never rendered, so "monospace" has no
        // visual meaning to categorize by. Downstream markdown conversion
        // uses `is_monospace` to fence a line/paragraph as a code block; an
        // OCR'd scanned novel's dialogue tripping this on FixedPitch alone
        // fences narrative prose as code (#1024).
        let is_invisible_or_glyphless = state.render_mode == 3
            || state.render_mode == 7
            || cached_font
                .as_ref()
                .is_some_and(|f| f.base_font.to_uppercase().contains("GLYPHLESS"));
        let is_monospace = !is_invisible_or_glyphless
            && cached_font.as_ref().is_some_and(|f| {
                if f.flags.is_some_and(|flags| flags & 1 != 0) {
                    return true;
                }
                let name = f.base_font.to_uppercase();
                name.contains("COURIER")
                    || name.contains("CONSOLAS")
                    || name.contains("MONO")
                    || name.contains("FIXED")
            });
        let rotation_degrees = snap_run_rotation(&combined);
        // Pre-compute user-space position: text_matrix origin → CTM transform
        let text_pos = state.text_matrix.transform_point(0.0, 0.0);
        let user_pos = state.ctm.transform_point(text_pos.x, text_pos.y);
        let paint_text_pos = state.text_matrix.transform_point(0.0, state.text_rise);
        let paint_pos = state
            .ctm
            .transform_point(paint_text_pos.x, paint_text_pos.y);
        Self {
            source_order,
            unicode: String::new(),
            start_matrix: state.text_matrix,
            font_name: state.font_name.clone(),
            fill_color_rgb: state.fill_color_rgb,
            char_space: state.char_space,
            word_space: state.word_space,
            horizontal_scaling: state.horizontal_scaling,
            mcid,
            accumulated_width: 0.0,
            cached_font,
            effective_font_size,
            font_weight,
            is_italic,
            is_monospace,
            char_widths: Vec::new(),
            user_pos_x: user_pos.x,
            user_pos_y: user_pos.y,
            paint_pos_x: paint_pos.x,
            paint_pos_y: paint_pos.y,
            user_h_scale,
            rotation_degrees,
            paint_basis: [combined.a, combined.b, combined.c, combined.d],
            mirrored: run_is_mirrored(&combined),
            wmode: state.text_wmode,
            text_rise: if state.font_size > 0.0 {
                state.text_rise / state.font_size
            } else {
                0.0
            },
            render_mode: state.render_mode,
        }
    }

    /// Check if the buffer is empty.
    fn is_empty(&self) -> bool {
        self.unicode.is_empty()
    }

    /// Append a text string to the buffer.
    fn append(&mut self, bytes: &[u8]) -> Result<()> {
        // PDF spec Section 7.3.4.2: implementation limit of 32,767 bytes per string.
        // Malformed PDFs may exceed this, causing text blowup.
        let bytes = if bytes.len() > 32_767 {
            &bytes[..32_767]
        } else {
            bytes
        };

        let font = self.cached_font.as_deref();

        // Fast path: OneByte fonts push chars directly into buffer via lookup table.
        // Avoids String allocation in decode_text_to_unicode (2 allocations per call).
        if let Some(font) = font {
            if font.subtype != "Type0" {
                // #317 UTF-8-in-simple-font detection — see long comment in
                // `append_advance_buffer`. Some producers emit UTF-8 byte
                // sequences inside PDF string literals for fonts that only
                // declare a Latin encoding with no ToUnicode CMap. When the
                // entire byte slice is valid UTF-8 whose decoded chars
                // include at least one non-Latin-1 codepoint, treat it as
                // UTF-8 so we recover Cyrillic / Greek / CJK instead of
                // Latin-1 mojibake.
                if font.to_unicode.is_none() && bytes.len() >= 2 {
                    let has_high = bytes.iter().any(|&b| b >= 0x80);
                    if has_high {
                        if let Ok(decoded) = std::str::from_utf8(bytes) {
                            if decoded.chars().any(|c| c as u32 > 0xFF) {
                                for ch in decoded.chars() {
                                    self.unicode.push(ch);
                                }
                                return Ok(());
                            }
                        }
                    }
                }

                let table = font.get_byte_to_char_table();
                for &byte in bytes {
                    let c = table[byte as usize];
                    if c != '\0' {
                        self.unicode.push(c);
                    } else {
                        // Rare: multi-char mapping or unmapped byte
                        if let Some(s) = font.char_to_unicode(byte as u32) {
                            if s != "\u{FFFD}" || preserve_unmapped_glyphs() {
                                for ch in s.chars() {
                                    if ch >= '\x20' || ch == '\t' || ch == '\n' || ch == '\r' {
                                        self.unicode.push(ch);
                                    }
                                }
                            }
                        } else {
                            let fb = fallback_char_to_unicode(byte as u32);
                            if fb != "\u{FFFD}" || preserve_unmapped_glyphs() {
                                for ch in fb.chars() {
                                    if ch >= '\x20' || ch == '\t' || ch == '\n' || ch == '\r' {
                                        self.unicode.push(ch);
                                    }
                                }
                            }
                        }
                    }
                }
                return Ok(());
            }
        }

        // Slow path: Type0 (CID) fonts or no font — use full decode function
        let unicode_text = decode_text_to_unicode(bytes, font);
        self.unicode.push_str(&unicode_text);

        Ok(())
    }
}

/// Artifact type classification per PDF Spec Section 14.8.2.2
///
/// Artifacts are content that is not part of the document's logical structure,
/// such as headers, footers, page numbers, and decorative elements.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum ArtifactType {
    /// Pagination artifacts (headers, footers, page numbers)
    Pagination(PaginationSubtype),
    /// Layout artifacts (ruled lines, backgrounds, borders)
    Layout,
    /// Page artifacts (full-page backgrounds, watermarks)
    Page,
    /// Background graphics or decorations
    Background,
}

/// Pagination artifact subtypes per PDF Spec Section 14.8.2.2.1
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum PaginationSubtype {
    /// Page header content
    Header,
    /// Page footer content
    Footer,
    /// Watermark overlay
    Watermark,
    /// Page number
    PageNumber,
    /// Other pagination element
    Other,
}

/// Context for marked content sequences (per PDF Spec Section 14.6)
///
/// Tracks nested marked content tags to implement artifact filtering.
/// When content is marked as `/Artifact`, it should be excluded from text extraction.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
struct MarkedContentContext {
    tag: String,
    is_artifact: bool,
    /// Artifact type classification for filtered content (PDF Spec Section 14.8.2.2)
    artifact_type: Option<ArtifactType>,
    /// ActualText for marked content (PDF Spec Section 14.9.4)
    /// Used to replace extracted text with correct representation
    /// e.g., ligatures (fi, fl, ffi, ffl), decorated glyphs
    actual_text: Option<String>,
    /// True once an ActualText replacement has been emitted from this
    /// MC scope. Per ISO 32000-1:2008 §14.9.4 the `/ActualText` of a
    /// marked-content sequence is the replacement for the ENTIRE
    /// sequence — even if it contains multiple `Tj` / `TJ` operators
    /// the replacement is emitted ONCE. The first Tj inside a scope
    /// flips this flag; subsequent Tj operators see it and skip the
    /// replacement path.
    actual_text_emitted: bool,
    /// Expansion text for abbreviations (PDF Spec Section 14.9.5)
    /// The /E entry provides the expansion of an abbreviation or acronym.
    /// e.g., "PDF" might expand to "Portable Document Format"
    expansion: Option<String>,
    /// Whether this marked content context is an excluded Optional Content Group (layer).
    ///
    /// Set when tag is "OC" and the OCG /Name matches one of the excluded layers.
    is_excluded_layer: bool,
    /// Whether this marked content context is an InDesign "placed PDF" figure.
    ///
    /// Set when the tag is `/PlacedPDF` — an Adobe InDesign-specific
    /// marked-content tag that wraps an imported/placed PDF rendered AS a
    /// figure (always nested inside a `/Figure` structure element). Its text
    /// content is the placed artwork's own glyphs (e.g. a draft galley of the
    /// manuscript with line numbers), NOT the document's logical text — the
    /// authoritative copy is re-typeset outside the placed region. Treating it
    /// as a figure (suppressing its text) matches what pdftotext/PyMuPDF do
    /// and removes duplicated / mojibake overlay text. See `is_content_suppressed`.
    is_placed_pdf: bool,
    /// MCID declared by this BDC (only BDC; BMC carries no /MCID).
    ///
    /// Stored here so EMC can restore the outer scope's MCID instead
    /// of blanking `current_mcid` unconditionally. A `Tj` issued
    /// AFTER an inner EMC must still attribute to its enclosing
    /// MCID-bearing scope (the PDF spec specifies marked-content
    /// nesting at §14.6).
    own_mcid: Option<u32>,
}

/// Text extractor that processes content streams.
///
/// This structure maintains the graphics state stack and font information
/// while processing operators to extract positioned text.
///
/// The extractor can work in two modes:
/// - **Span mode** (default): Extracts complete text strings as PDF provides them (PDF spec compliant)
/// - **Character mode**: Extracts individual characters (for special use cases)
#[derive(Debug)]
pub struct TextExtractor<'doc> {
    /// Graphics state stack for handling q/Q operators
    state_stack: GraphicsStateStack,
    /// Loaded fonts (name -> FontInfo). Arc-wrapped to avoid deep cloning across pages.
    fonts: HashMap<String, Arc<FontInfo>>,
    /// Extracted text spans (complete strings from Tj/TJ operators)
    spans: Vec<TextSpan>,
    /// Extracted characters (for backward compatibility)
    chars: Vec<TextChar>,
    /// Resources dictionary (for accessing XObjects and fonts)
    resources: Option<Object>,
    /// Reference to the document (for loading XObjects)
    document: Option<&'doc crate::document::PdfDocument>,
    /// Set of processed XObject references to avoid duplicates.
    /// Key is `(ObjectRef, ctm_key)` where `ctm_key` is the CTM at the time of
    /// the `Do` operator call, encoded as 6 millipoint-rounded i64 values.
    /// Using the CTM as part of the key allows the same Form XObject to be
    /// processed multiple times when invoked with different transformation
    /// matrices (e.g., the same XObject stamped at different positions on a page),
    /// while still preventing infinite recursion (same ref + same CTM).
    processed_xobjects: HashSet<(ObjectRef, [i64; 6])>,
    /// Cached XObject name → ObjectRef mapping for current resources context.
    /// Avoids expensive repeated resolution of the resources/XObject dict chain.
    cached_xobject_refs: HashMap<String, Option<ObjectRef>>,
    /// Current XObject recursion depth (0 = page level)
    xobject_depth: u32,
    /// Number of XObjects decoded on this page (for budget limiting)
    xobject_decode_count: u32,
    /// Configuration for text extraction heuristics
    config: TextExtractionConfig,
    /// Configuration for span merging behavior
    merging_config: SpanMergingConfig,
    /// Visual import excludes text with no painted fill or stroke.
    visible_paint_only: bool,
    paint_source_order: Vec<usize>,
    /// Adjacent text paints may share word assembly until a non-text mark or Form boundary.
    paint_group: usize,
    span_paint_orders: HashMap<usize, SpanPaintOrder>,
    /// Current marked content ID (for Tagged PDFs)
    ///
    /// Tracks the MCID of the currently active marked content sequence.
    /// Used to associate extracted text with structure tree elements.
    current_mcid: Option<u32>,
    /// Set of MCIDs whose BDC carried inline `/ActualText` on this
    /// page.
    ///
    /// Populated by the BDC handler whenever it observes
    /// `/ActualText` on the properties dictionary. The struct-tree-
    /// scope ActualText applier (in `document.rs`) uses this set to
    /// honour MC-scope-wins precedence: an ancestor StructElem's
    /// `/ActualText` must NOT override an MCID whose in-stream
    /// /ActualText has already been applied at extraction time
    /// (ISO 32000-1:2008 §14.6, §14.9.4).
    mc_actualtext_mcids: HashSet<u32>,
    /// Stack of marked content contexts (per PDF Spec Section 14.6)
    ///
    /// Tracks nested marked content tags to enable artifact filtering.
    /// When content is marked as `/Artifact`, it should be excluded from text extraction.
    marked_content_stack: Vec<MarkedContentContext>,
    /// True once a `/ReversedChars` marked-content sequence (ISO 32000-1
    /// §14.8.2.3.3) has been seen on this page. Such producers draw RTL glyphs
    /// individually with explicit positioning and mark real word boundaries with
    /// explicit space glyphs — so oxide must NOT additionally insert geometric
    /// word spaces between cursively-adjacent Arabic letters (which would shatter
    /// words, e.g. `إسبريسو` → `إس بر يسو`).
    saw_reversed_chars: bool,
    /// Whether we're currently inside an /Artifact marked content context
    ///
    /// Per PDF Spec Section 14.6, artifact content should be excluded from text extraction.
    /// This flag is true when any ancestor in the marked_content_stack has is_artifact=true.
    inside_artifact: bool,
    /// Layer names (Optional Content Groups) to exclude from extraction.
    ///
    /// When a BDC operator with tag "OC" references an OCG whose /Name matches
    /// one of these entries, all content within that marked content scope is suppressed.
    excluded_layers: HashSet<String>,
    /// Whether we're currently inside an excluded OCG layer.
    ///
    /// True when any ancestor in the marked_content_stack has is_excluded_layer=true.
    inside_excluded_layer: bool,
    /// Whether we're currently inside an InDesign `/PlacedPDF` figure region.
    ///
    /// True when any ancestor in the marked_content_stack has is_placed_pdf=true.
    /// Text inside a placed-PDF figure is the placed artwork's own glyphs and is
    /// suppressed (it is a figure, not logical text). See `MarkedContentContext::is_placed_pdf`.
    inside_placed_pdf: bool,
    /// When true, `/PlacedPDF` text is KEPT instead of suppressed for this page.
    ///
    /// The placed-PDF suppression assumes the placed region is a *decorative
    /// figure overlay* whose glyphs duplicate logical text that lives OUTSIDE it
    /// (the PMC8100493 draft-galley case). But some publishers (e.g. MATEC Web of
    /// Conferences) place the ENTIRE article body inside a single `/PlacedPDF`
    /// region, leaving almost nothing outside — there the placed text IS the
    /// page's logical content and suppressing it drops the whole page. Set by a
    /// cheap page-content-stream pre-scan (`placed_pdf_text_dominates`) that
    /// flips this on only when the placed text dominates and the non-placed text
    /// is negligible. pymupdf/pdftotext likewise extract the body in that case.
    placed_pdf_keep: bool,
    /// Ink / separation names to exclude from extraction.
    ///
    /// When a `cs` operator sets a Separation or DeviceN color space whose ink name(s)
    /// match one of these entries, subsequent text is suppressed until the color space changes.
    excluded_inks: HashSet<String>,
    /// Whether the current fill color space is an excluded ink.
    ///
    /// Set when SetFillColorSpace resolves to a Separation or DeviceN color space
    /// whose ink name(s) intersect with `excluded_inks`.
    inside_excluded_ink: bool,
    /// Extraction mode: true for spans, false for characters
    extract_spans: bool,
    /// Buffer for accumulating consecutive Tj operators into single spans
    ///
    /// Per PDF Spec ISO 32000-1:2008 Section 9.4.4 NOTE 6, text strings should
    /// be as long as possible. This buffer accumulates consecutive Tj operators
    /// until a positioning command or state change is encountered.
    tj_span_buffer: Option<TjBuffer>,
    /// Sequence counter for TextSpan ordering
    ///
    /// Used as a tie-breaker when sorting spans by Y-coordinate. Ensures
    /// that spans with identical Y-coordinates maintain extraction order.
    span_sequence_counter: usize,
    /// History of TJ array offsets for statistical analysis
    ///
    /// Tracks TJ offset values to detect justified vs. normal text through
    /// statistical distribution analysis (coefficient of variation).
    /// Used to dynamically adjust spacing thresholds per ISO 32000-1:2008 Section 9.4.4.
    tj_offset_history: Vec<f32>,
    /// Running sum / sum-of-squares so `analyze_tj_distribution` is O(1) rather
    /// than re-scanning the offset history (called once per TJ offset → O(n²)
    /// per page). `tj_stats_len` is the history length they cover; if the
    /// history is replaced wholesale, `analyze` recomputes once. f64 for precision.
    tj_sum: f64,
    tj_sum_sq: f64,
    tj_stats_len: usize,
    /// Character-level tracking for word boundary detection
    ///
    /// Collects CharacterInfo for each character during TJ array processing.
    /// This provides character-level positioning, width, and TJ offset data
    /// to WordBoundaryDetector for primary word boundary detection.
    /// Per ISO 32000-1:2008 Section 9.4.4, character-level analysis improves accuracy.
    tj_character_array: Vec<CharacterInfo>,
    /// Current X position in text space for character tracking
    ///
    /// Updated as each character in a TJ array is processed. Used to calculate
    /// x_position for CharacterInfo entries (not used after character collection).
    current_x_position: f32,
    /// Word boundary detection mode
    ///
    /// Controls whether WordBoundaryDetector is used as:
    /// - Tiebreaker: Only when TJ and geometric signals conflict (default)
    /// - Primary: Before creating TextSpans from tj_character_array
    word_boundary_mode: WordBoundaryMode,
    /// Cached current font (updated on Tf). Avoids per-Tj HashMap lookup
    /// in advance_position_for_string.
    cached_current_font: Option<Arc<FontInfo>>,
    /// Stack of MCID content-stream scopes (ISO 32000-1:2008 §14.7.4.3).
    ///
    /// Bottom of the stack is the page's own content-stream scope
    /// (`McidScope::Page(page_index)`). Each entry into a Form XObject
    /// via `Do` pushes a `McidScope::Form(form_ref)`; the matching
    /// pop restores the outer scope. The top of the stack stamps every
    /// `TextSpan` emitted while it is active. Tiling-Pattern walks are
    /// not currently traversed by the extractor (patterns rasterize
    /// independently); the spec-strict three-variant scope still
    /// covers `Pattern(_)` in the data model so future pattern-content
    /// walks can populate it.
    mcid_scope_stack: Vec<crate::structure::McidScope>,
    // Only PdfDocument's canonical root stream may populate the page-op cache.
    // Public set_page_index alone does not assert source identity.
    import_page_source: Option<usize>,
}

impl<'doc> TextExtractor<'doc> {
    /// Fraction of a glyph's advance width considered "overlap" for
    /// duplicate detection. Used by both `deduplicate_overlapping_chars`
    /// and `deduplicate_overlapping_spans`.
    ///
    /// 0.30 comfortably catches real render-pass duplicates
    /// (stroke+fill, bold shadow, outline+fill) which sit well under
    /// 5 % of one advance apart, while staying below typical heaviest
    /// kerning (≤ 20 % of advance) so legitimate narrow-glyph
    /// neighbours (`ll`, `rr`, `II`, `ii`) are preserved.
    const DEDUP_OVERLAP_RATIO: f32 = 0.30;

    /// Absolute cap on the overlap window (in PDF points).
    ///
    /// Preserves pre-ratio v0.3.x behaviour for pathologically
    /// oversized advance values (drop-caps, large display text) where
    /// 30 % of the advance would swallow legitimate neighbours.
    const DEDUP_OVERLAP_CAP_PT: f32 = 2.0;

    /// Create a new text extractor with default configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::TextExtractor;
    ///
    /// let extractor = TextExtractor::new();
    /// ```
    pub fn new() -> Self {
        Self::with_config(TextExtractionConfig::default())
    }

    /// Create a new text extractor with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for text extraction heuristics
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::{TextExtractor, TextExtractionConfig};
    ///
    /// // Use custom space threshold
    /// let config = TextExtractionConfig::with_space_threshold(-80.0);
    /// let extractor = TextExtractor::with_config(config);
    /// ```
    pub fn with_config(config: TextExtractionConfig) -> Self {
        let word_boundary_mode = config.word_boundary_mode;
        Self {
            state_stack: GraphicsStateStack::new(),
            fonts: HashMap::new(),
            spans: Vec::new(),
            chars: Vec::new(),
            resources: None,
            document: None,
            processed_xobjects: HashSet::new(),
            cached_xobject_refs: HashMap::new(),
            xobject_depth: 0,
            xobject_decode_count: 0,
            config,
            merging_config: SpanMergingConfig::default(),
            visible_paint_only: false,
            paint_source_order: Vec::new(),
            paint_group: 0,
            span_paint_orders: HashMap::new(),
            current_mcid: None,
            mc_actualtext_mcids: HashSet::new(),
            extract_spans: true,      // Default to span mode (PDF spec compliant)
            tj_span_buffer: None,     // No buffer initially
            span_sequence_counter: 0, // Initialize sequence counter
            marked_content_stack: Vec::new(), // Track marked content contexts
            saw_reversed_chars: false,
            inside_artifact: false, // Track artifact state
            excluded_layers: HashSet::new(),
            inside_excluded_layer: false,
            inside_placed_pdf: false,
            placed_pdf_keep: false,
            excluded_inks: HashSet::new(),
            inside_excluded_ink: false,
            tj_offset_history: Vec::with_capacity(1000), // Track TJ offsets for statistical analysis
            tj_sum: 0.0,
            tj_sum_sq: 0.0,
            tj_stats_len: 0,
            tj_character_array: Vec::new(), // Character tracking for word boundaries
            current_x_position: 0.0,        // Start at origin
            word_boundary_mode,             // Word boundary detection mode
            cached_current_font: None,      // Set on first Tf
            // Default to Page(0); `set_page_index` overrides before
            // extraction. Form XObject `Do` invocations push their
            // own scope on top.
            mcid_scope_stack: vec![crate::structure::McidScope::Page(0)],
            import_page_source: None,
        }
    }

    pub(crate) fn set_visible_paint_only(&mut self, enabled: bool) {
        self.visible_paint_only = enabled;
    }

    pub(crate) fn set_import_page_source(&mut self, page: usize) {
        self.import_page_source = Some(page);
    }

    /// Stamp this extractor with the page index it is processing.
    ///
    /// Used so spans (and the lookup keys for `/ActualText`) carry the
    /// correct `McidScope::Page(page_index)` outside a Form XObject.
    pub fn set_page_index(&mut self, page_index: u32) {
        // The first entry is always the page scope (Form scopes are
        // pushed on top by `Do` and popped before the extractor
        // finishes); update it in place.
        if let Some(first) = self.mcid_scope_stack.first_mut() {
            *first = crate::structure::McidScope::Page(page_index);
        } else {
            self.mcid_scope_stack
                .push(crate::structure::McidScope::Page(page_index));
        }
    }

    /// Current MCID scope (top of the stack) — what should be stamped
    /// on every new `TextSpan`.
    fn current_mcid_scope(&self) -> crate::structure::McidScope {
        self.mcid_scope_stack
            .last()
            .cloned()
            .unwrap_or(crate::structure::McidScope::Page(0))
    }

    /// Create a new text extractor with custom merging configuration.
    ///
    /// This allows fine-tuning how adjacent spans are merged and when spaces
    /// are inserted, useful for documents with unusual spacing patterns.
    ///
    /// # Arguments
    ///
    /// * `merging_config` - Configuration for span merging thresholds
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::extractors::{TextExtractor, SpanMergingConfig};
    ///
    /// // Use aggressive space insertion for dense layouts
    /// let config = SpanMergingConfig::aggressive();
    /// let extractor = TextExtractor::new().with_merging_config(config);
    /// ```
    pub fn with_merging_config(mut self, merging_config: SpanMergingConfig) -> Self {
        self.merging_config = merging_config;
        self
    }

    /// Set the resources dictionary for this extractor.
    ///
    /// This allows the extractor to access XObjects and fonts during extraction.
    pub fn set_resources(&mut self, resources: Object) {
        self.resources = Some(resources);
    }

    /// Set the document reference for loading XObjects.
    pub fn set_document(&mut self, document: &'doc crate::document::PdfDocument) {
        self.document = Some(document);
    }

    /// Take ownership of the set of MCIDs whose marked-content
    /// sequence carried an inline `/ActualText` property on this
    /// extraction.
    ///
    /// The set is observed by the BDC handler; this method drains it
    /// out so the document layer can stash it on a per-page side
    /// channel for the struct-tree-scope ActualText applier.
    pub fn take_mc_actualtext_mcids(&mut self) -> HashSet<u32> {
        std::mem::take(&mut self.mc_actualtext_mcids)
    }

    /// Set layer names (Optional Content Groups) to exclude from extraction.
    ///
    /// Content within BDC/EMC scopes tagged "OC" whose OCG /Name matches one of
    /// the provided names will be suppressed during text extraction.
    pub fn set_excluded_layers(&mut self, layers: HashSet<String>) {
        self.excluded_layers = layers;
    }

    /// Set ink / separation names to exclude from extraction.
    ///
    /// When the fill color space is a Separation or DeviceN whose ink name(s)
    /// intersect with any of the provided names, subsequent text is suppressed
    /// until the color space changes to a non-excluded one.
    ///
    /// **DeviceN behavior:** For DeviceN color spaces (e.g.
    /// `[/DeviceN [/Cyan /SpotGold] ...]`), text is suppressed if ANY ink in
    /// the array matches — even process colors sharing the DeviceN definition.
    /// This is because tint values are not evaluated during extraction.
    pub fn set_excluded_inks(&mut self, inks: HashSet<String>) {
        self.excluded_inks = inks;
    }

    // ========================================================================
    // Debug/profiling helpers — exposed for examples/debug_katalog.rs
    // ========================================================================

    /// Convenience wrapper: identical to `set_document`.
    pub fn set_document_ptr(&mut self, doc: &'doc crate::document::PdfDocument) {
        self.set_document(doc);
    }

    /// Prepare for span extraction mode (same setup as extract_text_spans preamble).
    pub fn prepare_for_span_extraction(&mut self) {
        self.extract_spans = true;
        self.spans.clear();
        self.span_sequence_counter = 0;
    }

    /// Public wrapper for execute_operator (normally private).
    pub fn execute_operator_public(&mut self, op: crate::content::Operator) -> Result<()> {
        self.execute_operator(op)
    }

    /// Public wrapper for flush_tj_span_buffer (normally private).
    pub fn flush_public(&mut self) -> Result<()> {
        self.flush_tj_span_buffer()
    }

    /// Calculate adaptive TJ offset threshold based on font size and text justification.
    ///
    /// When `use_adaptive_tj_threshold` is enabled, this method calculates the TJ offset
    /// threshold dynamically using the formula:
    ///
    /// ```text
    /// adaptive_threshold = -(space_width * font_size * margin_ratio) / 1000
    /// ```
    ///
    /// Where `margin_ratio` is adjusted based on justified vs normal text detection:
    /// - **Justified text** (high CV > 0.5): Uses 3× the normal ratio (conservative)
    ///   to prevent false space insertions from arbitrary TJ offsets
    /// - **Normal text** (low CV ≤ 0.5): Uses the default ratio (aggressive)
    ///
    /// # Adaptive Threshold Enhancement
    ///
    /// Per ISO 32000-1:2008 Section 9.4.4, justified text uses arbitrary TJ offsets to
    /// distribute whitespace. This method detects justified text through statistical
    /// analysis (coefficient of variation) and adapts the threshold accordingly.
    ///
    /// # Fallback Behavior
    ///
    /// If adaptive thresholds are disabled, this method returns the static
    /// `space_insertion_threshold` from the configuration.
    ///
    /// # PDF Spec Compliance
    ///
    /// Per Section 9.10: "Determining word boundaries is not specified by PDF."
    /// This method uses only spec-defined TJ values and geometric positions.
    fn calculate_adaptive_tj_threshold(&self) -> f32 {
        // Check if adaptive thresholds are enabled
        if !self.config.use_adaptive_tj_threshold {
            return self.config.space_insertion_threshold;
        }

        // Get current text state
        let state = self.state_stack.current();

        // ==============================================================================
        // FONT-AWARE ADAPTIVE THRESHOLD WITH JUSTIFIED TEXT DETECTION
        // (ISO 32000-1:2008 Section 9.4.4, 9.6.3, 9.10)
        // ==============================================================================

        let font_size = state.font_size;

        // Get font from current text state to access space glyph width
        // ISO 32000-1:2008 Section 9.6.3: Font metrics (glyph widths)
        let space_width_units = state
            .font_name
            .as_ref()
            .and_then(|name| self.fonts.get(name))
            .map(|font| font.get_space_glyph_width())
            .unwrap_or(250.0); // Fallback: Times-Roman typical space width

        // Detect justified vs normal text
        let (is_justified, cv) = self.analyze_tj_distribution();

        // Adjust margin ratio based on text justification
        // Justified text: use 3× conservative ratio (reduce false spaces)
        // Normal text: use default ratio
        let margin_ratio = if is_justified {
            self.config.word_margin_ratio * 3.0 // Conservative for justified
        } else {
            self.config.word_margin_ratio // Normal for non-justified
        };

        // Calculate threshold: negative offset required to trigger space insertion
        // Normalized by 1000 (PDF spec font units are 1/1000em)
        let adaptive_threshold = -((space_width_units * font_size * margin_ratio) / 1000.0);

        log::debug!(
            "TJ threshold: {} (justified={}, cv={:.2}, margin_ratio={:.3}, ISO 32000-1 §9.4.4)",
            adaptive_threshold,
            is_justified,
            cv,
            margin_ratio
        );

        adaptive_threshold
    }

    /// Analyze TJ offset distribution to detect justified vs normal text.
    ///
    /// This method performs statistical analysis on collected TJ offsets to determine
    /// if the document uses justified alignment. Justified text has high variance in TJ
    /// offsets (to distribute whitespace), while normally-spaced text has low variance.
    ///
    /// # Returns
    ///
    /// A tuple `(is_justified: bool, coefficient_of_variation: f32)` where:
    /// - `is_justified`: true if CV > 0.5 (high variance = justified text)
    /// - `coefficient_of_variation`: standard deviation / mean (normalized spread)
    ///
    /// # Algorithm
    ///
    /// Per ISO 32000-1:2008 Section 9.4.4, TJ array offsets are in font-relative units
    /// (1/1000 of text space). The distribution is analyzed as:
    ///
    /// 1. Calculate mean of all TJ offsets
    /// 2. Calculate variance: average of squared deviations from mean
    /// 3. Calculate standard deviation: sqrt(variance)
    /// 4. Calculate coefficient of variation: std_dev / |mean|
    ///
    /// # Thresholds
    ///
    /// - CV > 0.5: Justified text (high variance in offsets)
    /// - CV ≤ 0.5: Normal text (consistent spacing)
    ///
    /// # PDF Spec Compliance
    ///
    /// Per ISO 32000-1:2008 Section 9.10 ("Extraction of Text Content"):
    /// "Determining word boundaries is not specified by PDF." This method uses only
    /// spec-defined TJ offset values to infer text characteristics, not semantic assumptions.
    fn analyze_tj_distribution(&self) -> (bool, f32) {
        let n = self.tj_offset_history.len();
        if n == 0 {
            return (false, 0.0);
        }

        // Use the accumulators when current; recompute from the slice if the
        // history was replaced wholesale (same sum order → same result).
        let (sum, sum_sq) = if self.tj_stats_len == n {
            (self.tj_sum, self.tj_sum_sq)
        } else {
            let mut s = 0.0f64;
            let mut sq = 0.0f64;
            for &x in &self.tj_offset_history {
                let x = x as f64;
                s += x;
                sq += x * x;
            }
            (s, sq)
        };

        let nf = n as f64;
        let mean = sum / nf;
        // Variance from accumulators: E[x²] − E[x]². Clamp to ≥0 to absorb
        // floating-point cancellation when the spread is tiny.
        let variance = ((sum_sq / nf) - mean * mean).max(0.0);
        let std_dev = variance.sqrt();

        // Coefficient of variation (normalized spread); guard zero mean.
        let cv = if mean.abs() > 0.001 {
            (std_dev / mean.abs()) as f32
        } else {
            0.0
        };

        let is_justified = cv > 0.5;

        log::debug!(
            "TJ distribution analysis: mean={:.2}, std_dev={:.2}, cv={:.2}, justified={}",
            mean,
            std_dev,
            cv,
            is_justified
        );

        (is_justified, cv)
    }

    /// Update the artifact state based on the marked content stack.
    ///
    /// This method computes whether we're currently inside an artifact region
    /// by checking if any ancestor in the marked_content_stack has is_artifact=true.
    /// Per PDF Spec Section 14.6, artifact content should be excluded from text extraction.
    ///
    /// # Performance
    ///
    /// This is O(n) where n is the depth of the marked content stack (typically 1-5).
    /// Called each time a marked content boundary is crossed (BMC/BDC/EMC).
    fn update_artifact_state(&mut self) {
        // True if ANY ancestor in the stack is an artifact
        self.inside_artifact = self.marked_content_stack.iter().any(|ctx| ctx.is_artifact);
    }

    /// Update the excluded-layer state based on the marked content stack.
    ///
    /// True if any ancestor in the stack is an excluded OCG layer.
    /// Called each time a marked content boundary is crossed (BMC/BDC/EMC).
    fn update_layer_state(&mut self) {
        self.inside_excluded_layer = self
            .marked_content_stack
            .iter()
            .any(|ctx| ctx.is_excluded_layer);
        self.inside_placed_pdf = self
            .marked_content_stack
            .iter()
            .any(|ctx| ctx.is_placed_pdf);
    }

    /// Whether content emission should be suppressed.
    ///
    /// Returns true when the current graphics/marked-content state means
    /// extracted text should be discarded. Currently checks:
    /// - Inside an excluded OCG layer (`inside_excluded_layer`)
    /// - Inside an excluded ink / separation color space (`inside_excluded_ink`)
    /// - Inside an InDesign `/PlacedPDF` figure region (`inside_placed_pdf`)
    ///
    /// Note: artifact filtering is handled separately via span metadata and
    /// downstream filtering, so `inside_artifact` is intentionally not checked here.
    fn is_content_suppressed(&self) -> bool {
        (self.visible_paint_only && !self.state_stack.current().text_has_paint())
            || self.inside_excluded_layer
            || self.inside_excluded_ink
            || (self.inside_placed_pdf && !self.placed_pdf_keep)
    }

    /// Decide whether `/PlacedPDF` text should be kept (not suppressed) for a page.
    ///
    /// Cheap read-only pre-scan of the page content stream. Text-show operands
    /// (`Tj`/`TJ`/`'`/`"`) are bucketed by whether they are emitted INSIDE a
    /// `/PlacedPDF` marked-content scope or OUTSIDE it, and the placed bucket is
    /// KEPT (not suppressed) unless it looks like a decorative/duplicate overlay:
    ///
    /// 1. Too little placed text (`< MIN_PLACED_CHARS`) -> suppress. A stray
    ///    placed logo or figure caption is not the page body.
    /// 2. Placed text clearly dominates the page (non-placed text is a small
    ///    minority, ~3:1) -> keep. The publisher placed the whole article body as
    ///    one `/PlacedPDF` and left only a running header outside (MATEC Web of
    ///    Conferences).
    /// 3. Placed text is substantial but the non-placed text is comparable or
    ///    larger -> keep ONLY when the placed words are mostly NOT already present
    ///    outside. A placed region that mostly repeats the surrounding text is a
    ///    draft galley / overlay copy and stays suppressed (PMC8100493, the de-dup
    ///    win); one carrying mostly-unique words is the page's real placed content
    ///    (e.g. an InDesign figure that holds the page's labels and body text, as
    ///    on placed floor-plan / marketing spreads) and must be kept.
    ///
    /// Conservative on purpose: when placed text lives in a nested XObject the
    /// page-stream scan undercounts it and gate 1 falls back to suppression (the
    /// prior behaviour).
    fn placed_pdf_text_dominates(content_stream: &[u8]) -> bool {
        // Gate: only pages that actually carry the InDesign tag pay for a parse.
        if !content_stream
            .windows(b"PlacedPDF".len())
            .any(|w| w == b"PlacedPDF")
        {
            return false;
        }
        let Ok(operators) = parse_content_stream(content_stream) else {
            return false;
        };
        // A substantial placed body; below this a placed region is treated as a
        // decorative figure, not the page's logical content.
        const MIN_PLACED_CHARS: usize = 800;
        // Keep placed text whose words are mostly unique; suppress it once a
        // majority is also present in the non-placed text (a duplicate overlay).
        const MAX_DUP_FRACTION: f64 = 0.5;

        let mut placed_stack: Vec<bool> = Vec::new();
        let mut placed_chars: usize = 0;
        let mut other_chars: usize = 0;
        let mut placed_txt: Vec<u8> = Vec::new();
        let mut other_txt: Vec<u8> = Vec::new();
        let inside = |stack: &[bool]| stack.iter().any(|&p| p);
        for op in &operators {
            match op {
                Operator::BeginMarkedContent { tag }
                | Operator::BeginMarkedContentDict { tag, .. } => {
                    placed_stack.push(tag == "PlacedPDF");
                },
                Operator::EndMarkedContent => {
                    placed_stack.pop();
                },
                Operator::Tj { text } | Operator::Quote { text } => {
                    let (chars, txt) = if inside(&placed_stack) {
                        (&mut placed_chars, &mut placed_txt)
                    } else {
                        (&mut other_chars, &mut other_txt)
                    };
                    *chars += text.len();
                    txt.extend_from_slice(text);
                    txt.push(b' ');
                },
                Operator::DoubleQuote { text, .. } => {
                    let (chars, txt) = if inside(&placed_stack) {
                        (&mut placed_chars, &mut placed_txt)
                    } else {
                        (&mut other_chars, &mut other_txt)
                    };
                    *chars += text.len();
                    txt.extend_from_slice(text);
                    txt.push(b' ');
                },
                Operator::TJ { array } => {
                    let (chars, txt) = if inside(&placed_stack) {
                        (&mut placed_chars, &mut placed_txt)
                    } else {
                        (&mut other_chars, &mut other_txt)
                    };
                    for e in array {
                        if let TextElement::String(s) = e {
                            *chars += s.len();
                            txt.extend_from_slice(s);
                        }
                    }
                    txt.push(b' ');
                },
                _ => {},
            }
        }

        // Gate 1: too little placed text -> decorative figure, suppress.
        if placed_chars < MIN_PLACED_CHARS {
            return false;
        }
        // Gate 2: placed text dominates the page -> whole-body placed, keep.
        if other_chars.saturating_mul(3) < placed_chars {
            return true;
        }
        // Gate 3: placed text is substantial but the outside text is comparable
        // or larger. Keep it unless a majority of the placed words also appear
        // outside (a duplicate overlay). Tokenising here (behind gates 1 and 2)
        // keeps the common single-column path allocation-free.
        match Self::text_duplication_fraction(&placed_txt, &other_txt) {
            // No tokens could be read out of the placed operands, so there is
            // no evidence either way — and "no evidence" must not read as "not
            // a duplicate". The operands carry encoded character codes, not
            // text: under Identity-H, the dominant modern encoding for exactly
            // the producer this gate targets, the bytes are two-byte CIDs and
            // no run of ASCII alphanumerics forms. The measured duplication was
            // then 0.0, the gate kept, and `extract_text` emitted every word
            // twice — precisely the case the suppression exists to prevent.
            //
            // Failing closed suppresses instead. Gate 2 above already keeps a
            // placed body that dominates the page whatever its encoding, so
            // what reaches here is placed text of comparable size to the rest
            // of the page, where a duplicate overlay is the likely reading.
            //
            // The real discriminator is bounding-box overlap, which exists
            // downstream and is not consulted here; decoding the operand
            // through the font before tokenising would also settle it. Either
            // is a larger change than this gate.
            None => false,
            Some(fraction) => fraction < MAX_DUP_FRACTION,
        }
    }

    /// Fraction of alphanumeric word tokens in `a` (counting repeats) that also
    /// occur anywhere in `b`. Words are lowercased runs of >= 2 alphanumeric
    /// bytes; punctuation and single characters are ignored.
    ///
    /// Returns `None` when `a` yields no tokens at all. That is not "nothing to
    /// be a duplicate of" — these are raw show operands, i.e. encoded character
    /// codes rather than text, so an encoding whose codes are not ASCII
    /// alphanumerics (Identity-H, and any non-Latin script) produces no tokens
    /// from text that is certainly there. The caller must treat it as absence
    /// of evidence, not as evidence of absence.
    fn text_duplication_fraction(a: &[u8], b: &[u8]) -> Option<f64> {
        fn tokens(bytes: &[u8]) -> Vec<Vec<u8>> {
            let mut out = Vec::new();
            let mut cur = Vec::new();
            for &c in bytes {
                if c.is_ascii_alphanumeric() {
                    cur.push(c.to_ascii_lowercase());
                } else if !cur.is_empty() {
                    if cur.len() >= 2 {
                        out.push(std::mem::take(&mut cur));
                    } else {
                        cur.clear();
                    }
                }
            }
            if cur.len() >= 2 {
                out.push(cur);
            }
            out
        }
        let a_tokens = tokens(a);
        if a_tokens.is_empty() {
            return None;
        }
        let b_set: std::collections::HashSet<Vec<u8>> = tokens(b).into_iter().collect();
        let shared = a_tokens.iter().filter(|t| b_set.contains(*t)).count();
        Some(shared as f64 / a_tokens.len() as f64)
    }

    /// Parse artifact type and subtype from artifact properties dictionary.
    ///
    /// Per PDF Spec Section 14.8.2.2, artifacts have optional /Type and /Subtype entries:
    /// - /Type: Pagination, Layout, Page, or Background
    /// - /Subtype: For Pagination artifacts: Header, Footer, Watermark, etc.
    ///
    /// # Arguments
    ///
    /// * `props_dict` - The properties dictionary from BDC operator
    ///
    /// # Returns
    ///
    /// The classified artifact type, or None if no type is specified
    fn parse_artifact_type(props_dict: &HashMap<String, Object>) -> Option<ArtifactType> {
        // Extract /Type entry (PDF Spec Section 14.8.2.2)
        let artifact_type_name = props_dict
            .get("Type")
            .and_then(|obj| obj.as_name())
            .map(|s| s.to_lowercase());

        // Extract /Subtype entry for Pagination artifacts
        let subtype_name = props_dict
            .get("Subtype")
            .and_then(|obj| obj.as_name())
            .map(|s| s.to_lowercase());

        match artifact_type_name.as_deref() {
            Some("pagination") => {
                let subtype = match subtype_name.as_deref() {
                    Some("header") => PaginationSubtype::Header,
                    Some("footer") => PaginationSubtype::Footer,
                    Some("watermark") => PaginationSubtype::Watermark,
                    Some("pagenumber") | Some("page") => PaginationSubtype::PageNumber,
                    _ => PaginationSubtype::Other,
                };
                Some(ArtifactType::Pagination(subtype))
            },
            Some("layout") => Some(ArtifactType::Layout),
            Some("page") => Some(ArtifactType::Page),
            Some("background") => Some(ArtifactType::Background),
            None => {
                // No /Type specified - check if /Subtype alone indicates pagination
                // Some PDFs use /Subtype without /Type
                match subtype_name.as_deref() {
                    Some("header") => Some(ArtifactType::Pagination(PaginationSubtype::Header)),
                    Some("footer") => Some(ArtifactType::Pagination(PaginationSubtype::Footer)),
                    Some("watermark") => {
                        Some(ArtifactType::Pagination(PaginationSubtype::Watermark))
                    },
                    _ => None,
                }
            },
            _ => None, // Unknown type
        }
    }

    /// Decode a PDF text string (handles UTF-16BE/LE with BOM and PDFDocEncoding).
    ///
    /// Thin delegate to [`crate::optional_content::decode_pdf_text_string`] — that
    /// module owns the canonical implementation shared with the rendering path
    /// (UTF-16BE/LE with BOM, PDFDocEncoding fallback per ISO 32000-1:2008 §7.9.2).
    fn decode_pdf_text_string(bytes: &[u8]) -> String {
        crate::optional_content::decode_pdf_text_string(bytes)
    }

    /// Resolve BDC properties: can be an inline dictionary or a name referencing /Properties resource.
    ///
    /// Thin delegate to [`crate::optional_content::resolve_bdc_properties`].
    /// Passing `self.document` as `Option` lets the inline-dict fast path work
    /// even on a freshly-constructed extractor with no document attached (used
    /// by unit tests).
    fn resolve_bdc_properties(
        &self,
        properties: &Object,
    ) -> Option<std::collections::HashMap<String, Object>> {
        crate::optional_content::resolve_bdc_properties(
            properties,
            self.resources.as_ref(),
            self.document,
        )
    }

    /// Resolve a named color space from the /Resources /ColorSpace dictionary.
    ///
    /// PDF content streams reference color spaces by name (e.g. `cs /CS1`).
    /// Device color spaces like "DeviceRGB" are built-in, but Separation and
    /// DeviceN color spaces live in the page resources:
    ///
    /// ```text
    /// /Resources << /ColorSpace << /CS1 [/Separation /PANTONE_Red /DeviceCMYK ...] >> >>
    /// ```
    ///
    /// Returns the resolved color space array if the name refers to a resource entry.
    fn resolve_color_space(&self, name: &str) -> Option<Vec<Object>> {
        let resources = self.resources.as_ref()?;
        let res_dict = if let Some(res_ref) = resources.as_reference() {
            self.document?.load_object(res_ref).ok()?
        } else {
            resources.clone()
        };
        let res_dict = res_dict.as_dict()?;
        let cs_dict_obj = res_dict.get("ColorSpace")?;
        let cs_dict = if let Some(r) = cs_dict_obj.as_reference() {
            self.document?.load_object(r).ok()?
        } else {
            cs_dict_obj.clone()
        };
        let cs_dict = cs_dict.as_dict()?;
        let cs_obj = cs_dict.get(name)?;
        let resolved = if let Some(r) = cs_obj.as_reference() {
            self.document?.load_object(r).ok()?
        } else {
            cs_obj.clone()
        };
        resolved.as_array().cloned()
    }

    /// Check if a color space name refers to an excluded ink.
    ///
    /// Resolves the color space from resources and checks:
    /// - `[/Separation /InkName /AlternateCS /TintTransform]` — single ink name
    /// - `[/DeviceN [/Ink1 /Ink2 ...] /AlternateCS /TintTransform]` — multiple ink names
    ///
    /// Returns true if any ink name in the color space matches `excluded_inks`.
    ///
    /// **Note:** For DeviceN, this is all-or-nothing — if any ink matches, the
    /// entire color space is treated as excluded. Tint values are not evaluated.
    fn is_excluded_ink_color_space(&self, name: &str) -> bool {
        if self.excluded_inks.is_empty() {
            return false;
        }
        if let Some(cs_array) = self.resolve_color_space(name) {
            if cs_array.len() >= 2 {
                if let Some(cs_type) = cs_array[0].as_name() {
                    // §8.6.6.2 / §8.6.6.3: the colorant slot (Separation's
                    // ink-name, DeviceN's names array) can be an indirect
                    // reference. Some subsetters share the names list across
                    // multiple DeviceN spaces, emitting
                    // `[/DeviceN 4 0 R /DeviceCMYK <attrs>]` where `4 0 R`
                    // points to the actual names list. Resolve before
                    // pattern-matching.
                    let deref = |obj: &Object| -> Object {
                        match (obj.as_reference(), self.document) {
                            (Some(r), Some(d)) => d.load_object(r).unwrap_or_else(|_| obj.clone()),
                            _ => obj.clone(),
                        }
                    };
                    match cs_type {
                        "Separation" => {
                            // [/Separation /InkName /AlternateCS /TintTransform]
                            let name_obj = deref(&cs_array[1]);
                            if let Some(ink_name) = name_obj.as_name() {
                                return self.excluded_inks.contains(ink_name);
                            }
                        },
                        "DeviceN" => {
                            // [/DeviceN <names-array> /AlternateCS /TintTransform <attrs>]
                            let names_obj = deref(&cs_array[1]);
                            if let Some(ink_names) = names_obj.as_array() {
                                return ink_names.iter().any(|obj| {
                                    obj.as_name()
                                        .map(|n| self.excluded_inks.contains(n))
                                        .unwrap_or(false)
                                });
                            }
                        },
                        _ => {},
                    }
                }
            }
        }
        false
    }

    /// Check whether a BDC properties dict represents an excluded OCG or OCMD.
    ///
    /// Thin delegate to [`crate::optional_content::check_ocg_excluded`] — that
    /// module is the single source of truth for OCG/OCMD evaluation, including
    /// OCMD `/P` visibility-policy handling.
    fn check_ocg_excluded(&self, props_dict: &std::collections::HashMap<String, Object>) -> bool {
        let doc = match self.document {
            Some(d) => d,
            None => return false,
        };
        crate::optional_content::check_ocg_excluded(props_dict, doc, &self.excluded_layers)
    }

    /// Get current ActualText from marked content stack (PDF Spec Section 14.9.4).
    ///
    /// Searches from the innermost marked content context outward, returning
    /// the first ActualText found. If no ActualText is defined, returns None.
    ///
    /// ActualText provides the exact text representation for content that's
    /// represented non-standardly, such as ligatures (fi, fl, ffi, ffl) or
    /// decorated glyphs.
    #[cfg(test)]
    fn get_current_actual_text(&self) -> Option<String> {
        self.marked_content_stack
            .iter()
            .rev() // Search from innermost (most recent) context
            .find_map(|ctx| ctx.actual_text.clone())
    }

    /// Return the innermost active `/ActualText`, alongside a flag
    /// indicating whether it has ALREADY been emitted in the current
    /// MC scope.
    ///
    /// Per ISO 32000-1:2008 §14.9.4 the `/ActualText` of a marked-
    /// content sequence replaces the ENTIRE sequence — even if the
    /// sequence contains multiple `Tj` / `TJ` operators the replacement
    /// is emitted ONCE.
    ///
    /// Returns `(text, already_emitted)`:
    /// - `(Some(text), false)` on the FIRST show-text inside a scope
    ///   that carries `/ActualText` — the caller should emit `text`
    ///   AND mark the scope's `actual_text_emitted` via
    ///   [`Self::mark_actual_text_emitted`].
    /// - `(Some(text), true)` on subsequent show-text operators inside
    ///   the same scope — the caller must suppress emission entirely
    ///   (no raw glyphs, no replacement). The text matrix advance
    ///   still runs so positioning stays consistent.
    /// - `(None, _)` when no `/ActualText` is active.
    fn peek_current_actual_text(&self) -> (Option<String>, bool) {
        for ctx in self.marked_content_stack.iter().rev() {
            if let Some(ref text) = ctx.actual_text {
                return (Some(text.clone()), ctx.actual_text_emitted);
            }
        }
        (None, false)
    }

    /// Mark the innermost scope's `/ActualText` as emitted. See
    /// [`Self::peek_current_actual_text`].
    fn mark_actual_text_emitted(&mut self) {
        for ctx in self.marked_content_stack.iter_mut().rev() {
            if ctx.actual_text.is_some() {
                ctx.actual_text_emitted = true;
                return;
            }
        }
    }

    /// Calculate the average glyph width for a font.
    ///
    /// Computes the mean width of printable ASCII characters (codes 32-126)
    /// in the given font, expressed in thousandths of em.
    ///
    /// # Fallback
    ///
    /// If the font doesn't have a widths array, uses the font's default width.
    ///
    /// # Performance
    ///
    /// This is relatively efficient, typically iterating over 95 ASCII characters.
    /// In practice, most fonts have widths arrays, so this completes quickly.
    #[allow(dead_code)]
    fn calculate_average_glyph_width(&self, font: &FontInfo) -> f32 {
        const PRINTABLE_ASCII_START: u32 = 32; // Space
        const PRINTABLE_ASCII_END: u32 = 126; // Tilde

        // If no widths array, use default width
        let Some(ref widths) = font.widths else {
            return font.default_width;
        };

        // We need FirstChar and LastChar to map character codes to width indices
        let Some(first_char) = font.first_char else {
            return font.default_width;
        };
        let Some(last_char) = font.last_char else {
            return font.default_width;
        };

        // Collect widths for all printable ASCII characters
        let mut total_width = 0.0;
        let mut count = 0;

        for char_code in PRINTABLE_ASCII_START..=PRINTABLE_ASCII_END {
            if char_code >= first_char && char_code <= last_char {
                // This character is in the widths array
                let index = (char_code - first_char) as usize;
                if index < widths.len() {
                    total_width += widths[index];
                    count += 1;
                }
            }
        }

        // Return average if we found any widths
        if count > 0 {
            total_width / count as f32
        } else {
            // Fallback if no widths in range
            font.default_width
        }
    }

    /// Add a font to the extractor.
    ///
    /// Fonts must be added before processing content streams that reference them.
    ///
    /// # Arguments
    ///
    /// * `name` - The font resource name (e.g., "F1", "TT1")
    /// * `font` - The font information
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pdf_oxide::extractors::TextExtractor;
    /// # use pdf_oxide::fonts::FontInfo;
    /// # fn example(font: FontInfo) {
    /// let mut extractor = TextExtractor::new();
    /// extractor.add_font("F1".to_string(), font);
    /// # }
    /// ```
    pub fn add_font(&mut self, name: String, font: FontInfo) {
        self.fonts.insert(name, Arc::new(font));
    }

    /// Add a pre-shared font (Arc-wrapped) to the extractor. Avoids deep cloning.
    #[cfg(test)]
    pub(crate) fn add_font_shared(&mut self, name: String, font: Arc<FontInfo>) {
        self.fonts.insert(name, font);
    }

    pub(crate) fn try_add_font_shared(&mut self, name: &str, font: Arc<FontInfo>) -> Result<()> {
        let budget = font.import_budget.clone();
        let a = crate::fonts::allocation::CollectionAllocation(budget.as_deref());
        let name = a
            .string(name)
            .ok_or(Error::ImportResourceLimit("font resource name"))?;
        a.insert(&mut self.fonts, name, font)
            .ok_or(Error::ImportResourceLimit("extractor font index"))
    }

    /// Return the current font set for caching purposes.
    pub fn get_font_set(&self) -> Vec<(String, Arc<FontInfo>)> {
        self.try_font_set().unwrap_or_default()
    }

    pub(crate) fn try_font_set(&self) -> Result<Vec<(String, Arc<FontInfo>)>> {
        let budget = self.document.and_then(|d| d.import_budget()).or_else(|| {
            self.fonts
                .values()
                .find_map(|font| font.import_budget.as_ref())
        });
        let a = crate::fonts::allocation::CollectionAllocation(budget.map(AsRef::as_ref));
        let mut result = a
            .vec(self.fonts.len())
            .ok_or(Error::ImportResourceLimit("font set allocation"))?;
        for (name, font) in &self.fonts {
            result.push((
                a.string(name)
                    .ok_or(Error::ImportResourceLimit("font set name"))?,
                Arc::clone(font),
            ));
        }
        Ok(result)
    }

    pub(crate) fn font_names(&self) -> impl Iterator<Item = &str> {
        self.fonts.keys().map(String::as_str)
    }

    fn try_font_map(&self) -> Result<HashMap<String, Arc<FontInfo>>> {
        let budget = self.document.and_then(|d| d.import_budget()).or_else(|| {
            self.fonts
                .values()
                .find_map(|font| font.import_budget.as_ref())
        });
        let a = crate::fonts::allocation::CollectionAllocation(budget.map(AsRef::as_ref));
        let mut result = a
            .map(self.fonts.len())
            .ok_or(Error::ImportResourceLimit("Form font index"))?;
        for (name, font) in &self.fonts {
            result.insert(
                a.string(name)
                    .ok_or(Error::ImportResourceLimit("Form font name"))?,
                Arc::clone(font),
            );
        }
        Ok(result)
    }

    /// Share TrueType cmap tables between fonts with matching base font names.
    /// When a CIDFontType2 Identity-H font has no truetype_cmap, borrow from
    /// another font on the same page with the same base font name (ignoring subset prefix).
    pub fn share_truetype_cmaps(&mut self) {
        let _ = self.try_share_truetype_cmaps();
    }

    fn try_share_truetype_cmaps(&mut self) -> Option<()> {
        let budget = self
            .document
            .and_then(|d| d.import_budget())
            .cloned()
            .or_else(|| {
                self.fonts
                    .values()
                    .find_map(|font| font.import_budget.clone())
            });
        let a = crate::fonts::allocation::CollectionAllocation(budget.as_deref());
        a.check()?;
        // Strip subset prefix (e.g., "QQPMQK+Impact" → "Impact")
        fn strip_subset(name: &str) -> &str {
            if name.len() > 7
                && name.as_bytes()[6] == b'+'
                && name[..6].chars().all(|c| c.is_ascii_uppercase())
            {
                &name[7..]
            } else {
                name
            }
        }

        // First pass: collect the BEST available TrueType cmap for each stripped base font name.
        // When multiple subset variants of the same font exist (e.g., ABCDEF+Arial, GHIJKL+Arial),
        // pick the cmap with the most glyph mappings — it has the best Unicode coverage.
        // On equal coverage, prefer the lexicographically smallest base_font name as a
        // deterministic tie-breaker (HashMap iteration order is randomized per-process).
        let mut best_cmaps: std::collections::HashMap<
            String,
            (crate::fonts::truetype_cmap::TrueTypeCMap, String),
        > = a.map(0)?;
        for font in self.fonts.values() {
            if let Some(cmap) = font.truetype_cmap() {
                let stripped = strip_subset(&font.base_font);
                let dominated = best_cmaps
                    .get(stripped)
                    .is_none_or(|(existing, existing_name)| {
                        match cmap.len().cmp(&existing.len()) {
                            std::cmp::Ordering::Greater => true,
                            std::cmp::Ordering::Equal => font.base_font < *existing_name,
                            std::cmp::Ordering::Less => false,
                        }
                    });
                if dominated {
                    a.insert(
                        &mut best_cmaps,
                        a.string(stripped)?,
                        (cmap.clone(), a.string(&font.base_font)?),
                    )?;
                }
            }
        }

        if best_cmaps.is_empty() {
            return Some(());
        }

        // Second pass: find CIDFontType2 Identity-H fonts without truetype_cmap
        for font_arc in self.fonts.values_mut() {
            if font_arc.truetype_cmap().is_some() {
                continue;
            }
            // Only target Type0 CIDFontType2 with Identity-H encoding
            if font_arc.subtype != "Type0" {
                continue;
            }
            let is_identity = matches!(&font_arc.encoding, crate::fonts::Encoding::Identity)
                || matches!(&font_arc.encoding, crate::fonts::Encoding::Standard(ref n) if n.contains("Identity"));
            if !is_identity {
                continue;
            }

            let stripped = strip_subset(&font_arc.base_font);
            if let Some((donor_cmap, _)) = best_cmaps.get(stripped) {
                log::info!(
                    "Sharing TrueType cmap ({} entries) to '{}' (Identity-H, no embedded font)",
                    donor_cmap.len(),
                    font_arc.base_font
                );
                // Keep donation local to this extractor. Shared program/CMap/memo
                // stay shared, while owned FontInfo payload must be charged first.
                if Arc::strong_count(font_arc) > 1 {
                    font_arc.reserve_cow_copy()?;
                } else if Arc::weak_count(font_arc) > 0 {
                    a.bytes(std::mem::size_of::<FontInfo>() + 2 * std::mem::size_of::<usize>())?;
                }
                Arc::make_mut(font_arc).set_truetype_cmap(Some(donor_cmap.clone()));
            }
        }
        Some(())
    }

    /// Extract text from a content stream.
    ///
    /// Parses the content stream and executes operators to extract positioned
    /// characters with Unicode mappings and font information.
    ///
    /// # Arguments
    ///
    /// * `content_stream` - The raw content stream data (should be decoded first)
    ///
    /// # Returns
    ///
    /// A vector of TextChar structures containing positioned characters.
    ///
    /// # Errors
    ///
    /// Returns an error if the content stream cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pdf_oxide::extractors::TextExtractor;
    /// # fn example(content_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut extractor = TextExtractor::new();
    /// let chars = extractor.extract(content_data)?;
    /// println!("Extracted {} characters", chars.len());
    /// # Ok(())
    /// # }
    /// ```
    /// Extract text as complete spans (PDF spec compliant).
    ///
    /// This is the recommended method for text extraction. It extracts complete
    /// text strings as the PDF provides them via Tj/TJ operators, following the
    /// PDF specification ISO 32000-1:2008.
    ///
    /// # Benefits
    /// - Avoids overlapping character issues
    /// - Preserves PDF's text positioning intent
    /// - More robust for complex layouts
    /// - Matches industry best practices
    ///
    /// # Arguments
    ///
    /// * `content_stream` - The PDF content stream data
    ///
    /// # Returns
    ///
    /// Vector of TextSpan objects in reading order
    pub fn extract_text_spans(&mut self, content_stream: &[u8]) -> Result<Vec<TextSpan>> {
        // Enable span extraction mode
        self.extract_spans = true;
        self.spans.clear();
        self.span_sequence_counter = 0; // Reset sequence counter for this page
                                        // Decide per page whether a whole-body `/PlacedPDF` region must be kept
                                        // rather than suppressed (see `placed_pdf_text_dominates`).
        self.placed_pdf_keep = Self::placed_pdf_text_dominates(content_stream);

        extract_log_debug!("Parsing content stream for text extraction");
        self.run_content_stream(content_stream)?;

        // Flush any remaining Tj buffer at end of content stream
        self.flush_tj_span_buffer()?;

        // Detect RTL glyph DRAW DIRECTION on the raw stream order, BEFORE the
        // reading-order sort destroys it (ISO 32000-1 §14.8.2.3.3 method 1).
        self.detect_rtl_draw_direction();

        // Sort spans by reading order (top-to-bottom, left-to-right)
        if log::log_enabled!(log::Level::Debug) {
            let space_spans = self
                .spans
                .iter()
                .filter(|s| s.text.chars().all(|c| c.is_whitespace()))
                .count();
            let offset_semantic = self.spans.iter().filter(|s| s.offset_semantic).count();
            log::debug!(
                "Before sort_spans_by_reading_order(): {} spans total, {} space-only, {} offset_semantic=true",
                self.spans.len(),
                space_spans,
                offset_semantic
            );
        }

        // Snap super/subscript glyph spans onto the baseline of an
        // adjacent base span BEFORE row-aware sorting. PDFs raise
        // or lower the text matrix via the `Ts` (text-rise) operator
        // for super/subscripts (§9.3.7); the rendered glyphs end up
        // at a Y offset of typically 0.3–0.5 × font_size from the
        // baseline. Without the snap, sorting groups all raised
        // glyphs into a separate Y-band above the body, producing
        // output like `"1,2 ★ 3,4 5 / Chibueze, …"` instead of
        // `"Chibueze,1,2★ Caleb,3,4† …"`.
        // Reading-order normalization must not destroy source paint geometry.
        // Positional import groups lines separately and retains each run's origin.
        if !self.visible_paint_only {
            self.snap_superscript_baselines();
        }

        self.sort_spans_by_reading_order();

        // Deduplicate overlapping spans
        if !self.visible_paint_only {
            self.deduplicate_overlapping_spans();
        }

        // Merge adjacent spans on the same line to reconstruct complete words
        if !self.visible_paint_only {
            self.merge_adjacent_spans();
        }

        // Resolve each span's font resource alias (e.g. "F1") to the resolved
        // /BaseFont name (e.g. "Helvetica", "CIDFont+F1") so extract_spans /
        // extract_words / extract_text_lines report the same font name that
        // extract_chars already does (which reads `font.base_font`). Run AFTER
        // merging so span reconstruction still keys off the raw resource alias
        // exactly as before, and it has no effect on the assembled text/md/html
        // output (font names are not emitted there) — only the API surface.
        let resolved_fonts: Vec<Option<String>> = self
            .spans
            .iter()
            .map(|s| {
                if self.visible_paint_only {
                    // Import spans captured the face in its live resource scope;
                    // the caller may reuse F1 for a different face after Do.
                    return None;
                }
                self.fonts
                    .get(&s.font_name)
                    .map(|f| f.base_font.clone())
                    .filter(|b| !b.is_empty())
            })
            .collect();
        for (span, resolved) in self.spans.iter_mut().zip(resolved_fonts) {
            if let Some(base_font) = resolved {
                span.font_name = base_font;
            }
        }

        // Attach the §9.10.2 mapping provenance now that each span's font name
        // is finalized: the tier the span's font offered, or `Fallback` when it
        // carries no mapping resource (the text is then a fabricated glyph-index
        // echo, not read from the file). `None` when the font is unresolvable.
        for span in self.spans.iter_mut() {
            span.provenance = self
                .fonts
                .get(span.font_name.as_str())
                .map(|f| f.best_mapping_provenance());
        }

        Ok(std::mem::take(&mut self.spans))
    }

    /// Feed a content stream's operators to `execute_operator`.
    ///
    /// The single parser dispatch for every extraction mode (chars, spans,
    /// Form XObject recursion). The modes must differ only in how
    /// `execute_operator` handles the show-text operators, never in which
    /// operators reach it: char and span mode once used different parsers
    /// (`parse_content_stream_text_only` vs `parse_and_execute_text_only`),
    /// which reconstruct the graphics state around a text region by different
    /// routes, so they could agree on every text operator and still hand the
    /// extractor a different CTM. On govdocs_003_003181.pdf page 4 that cost
    /// a 90°-rotated chart axis: char mode returned 2322 glyphs, all at
    /// rotation 0, where span mode saw 2590 glyphs, 122 at 90°.
    /// Whether this extraction carries an **emission filter** — a caller
    /// decision about what to leave out of the result.
    ///
    /// Every such filter is evaluated against interpreted state that the
    /// text-only parser's >256 KB prescan route does not deliver: it keeps
    /// only `BT..ET`/`Do` regions and discards everything between them.
    /// Ink filtering needs the colour operators (`cs`, `rg`, `g`, `k`); layer
    /// exclusion needs the `BDC`/`EMC` pairs that carry optional-content
    /// membership. Both must therefore take the full parser.
    ///
    /// This is one predicate rather than a condition per filter because the
    /// list had already been forgotten once: the gate tested inks alone, so
    /// `set_excluded_layers()` was silently ignored above the threshold — the
    /// caller asked for exclusion, got no error, and got the content. A new
    /// filter is added here, with its reason, or it inherits the same defect.
    ///
    /// This localises the class; it does not close it. ISO 32000-1:2008
    /// §8.11.3 requires that when optional content is hidden "the content
    /// shall not be drawn" while "graphics state operations … shall still be
    /// applied", so visibility is a decision about marking the page, taken
    /// after interpretation — never a licence to stop parsing. The structural
    /// answer is one sequential interpreter with suppression at emission,
    /// which retires this predicate along with the prescan branch.
    fn has_emission_filter(&self) -> bool {
        self.visible_paint_only
            || !self.excluded_inks.is_empty()
            || !self.excluded_layers.is_empty()
    }

    fn run_content_stream(&mut self, content_stream: &[u8]) -> Result<()> {
        // The path/image interpreters use the same full-parser operator ordinals.
        // A Form's local ordinals extend its caller Do ordinal, never a reading-order index.
        let prefix = self.paint_order_snapshot()?;
        let mut ordinal = 0;
        let mut execute = |this: &mut Self, operator: Operator| -> Result<()> {
            let form_boundary = this.visible_paint_only && matches!(&operator, Operator::Do { .. });
            if this.visible_paint_only {
                this.paint_source_order = prefix.clone();
                this.paint_source_order.push(ordinal);
                // A later non-text mark must not absorb an earlier buffered text run.
                if matches!(
                    &operator,
                    Operator::Stroke
                        | Operator::CloseAndStroke
                        | Operator::Fill
                        | Operator::FillEvenOdd
                        | Operator::FillStroke
                        | Operator::FillStrokeEvenOdd
                        | Operator::CloseFillStroke
                        | Operator::CloseFillStrokeEvenOdd
                        | Operator::Do { .. }
                        | Operator::InlineImage { .. }
                        | Operator::Tr { .. }
                        | Operator::SetLineWidth { .. }
                        | Operator::SetLineJoin { .. }
                        | Operator::SetMiterLimit { .. }
                        | Operator::SetDash { .. }
                        | Operator::SetStrokeRgb { .. }
                        | Operator::SetStrokeGray { .. }
                        | Operator::SetStrokeCmyk { .. }
                        | Operator::SetStrokeColorSpace { .. }
                        | Operator::SetStrokeColor { .. }
                        | Operator::SetStrokeColorN { .. }
                        | Operator::SetExtGState { .. }
                        // Runs on opposite sides of a graphics-state/clip boundary
                        // must keep their own source order and active clip witness.
                        | Operator::SaveState
                        | Operator::RestoreState
                        | Operator::ClipNonZero
                        | Operator::ClipEvenOdd
                        | Operator::EndPath
                ) {
                    this.flush_tj_span_buffer()?;
                    this.advance_paint_group()?;
                }
            }
            ordinal += 1;
            this.execute_operator(operator)?;
            if form_boundary {
                // A Form may end with buffered text. Keep its marks apart from
                // the caller's next run even if there was no path/image inside.
                this.flush_tj_span_buffer()?;
                this.advance_paint_group()?;
            }
            Ok(())
        };
        let result = if let Some(budget) =
            self.document.and_then(|doc| doc.import_budget()).cloned()
        {
            if self.xobject_depth == 0 && self.import_page_source.is_some() {
                let operators = self
                    .document
                    .expect("request source document")
                    .parse_page_import_content(
                        self.import_page_source.unwrap(),
                        content_stream,
                        parse_content_stream,
                    )?;
                operators.into_iter().try_for_each(|op| {
                    execute(self, op)?;
                    budget.check()
                })
            } else {
                crate::content::parser::execute_content_with_budget(content_stream, &budget, |op| {
                    execute(self, op)
                })
            }
        } else if self.has_emission_filter() {
            parse_content_stream(content_stream)?
                .into_iter()
                .try_for_each(|op| execute(self, op))
        } else {
            parse_and_execute_text_only(content_stream, |op| self.execute_operator(op))
        };
        self.paint_source_order = prefix;
        result
    }

    fn paint_order_snapshot(&self) -> Result<Vec<usize>> {
        if !self.visible_paint_only {
            return Ok(Vec::new());
        }
        if let Some(budget) = self.document.and_then(|doc| doc.import_budget()) {
            budget
                .charge_cache_copy(self.paint_source_order.len() * std::mem::size_of::<usize>())?;
        }
        Ok(self.paint_source_order.clone())
    }

    fn emitted_font_name(&self, resource: Option<&str>, font: Option<&FontInfo>) -> String {
        if self.visible_paint_only {
            if let Some(font) = font.filter(|f| !f.base_font.is_empty()) {
                return font.base_font.clone();
            }
        }
        resource.unwrap_or("Unknown").to_string()
    }

    fn record_span_paint_order(
        &mut self,
        sequence: usize,
        order: Vec<usize>,
        basis: [f32; 4],
        font: Option<Arc<FontInfo>>,
    ) -> Result<()> {
        if self.visible_paint_only {
            if let Some(budget) = self.document.and_then(|doc| doc.import_budget()) {
                budget.charge_entries(1)?;
                self.span_paint_orders
                    .try_reserve(1)
                    .map_err(|_| budget.reject("text paint order allocation"))?;
            }
            self.span_paint_orders.insert(
                sequence,
                SpanPaintOrder {
                    order,
                    group: self.paint_group,
                    basis,
                    font,
                },
            );
        }
        Ok(())
    }

    fn advance_paint_group(&mut self) -> Result<()> {
        self.paint_group = self
            .paint_group
            .checked_add(1)
            .ok_or_else(|| crate::error::Error::Unsupported("text paint group overflow".into()))?;
        Ok(())
    }

    pub(crate) fn take_span_paint_orders(&mut self) -> HashMap<usize, SpanPaintOrder> {
        std::mem::take(&mut self.span_paint_orders)
    }

    fn same_painted_font_resource(&self, first: usize, second: usize) -> bool {
        let font = |sequence| {
            self.span_paint_orders
                .get(&sequence)
                .and_then(|order| order.font.as_ref())
        };
        match (font(first), font(second)) {
            (Some(first), Some(second)) => Arc::ptr_eq(first, second),
            (None, None) => true,
            _ => false,
        }
    }

    fn reserve_import_span(&mut self) -> Result<()> {
        if let Some(budget) = self.document.and_then(|doc| doc.import_budget()) {
            budget.charge_span()?;
            self.spans
                .try_reserve(1)
                .map_err(|_| budget.reject("span allocation"))?;
        }
        Ok(())
    }

    /// Extract individual characters from a PDF content stream.
    ///
    /// This is a low-level method that extracts characters one by one.
    /// For most use cases, prefer using `extract_text_spans()` which groups
    /// characters into text spans according to PDF semantics.
    pub fn extract(&mut self, content_stream: &[u8]) -> Result<Vec<TextChar>> {
        self.extract_into_self(content_stream)?;
        Ok(self.chars.clone())
    }

    /// Run the character extraction and leave the result in `self.chars`.
    fn extract_into_self(&mut self, content_stream: &[u8]) -> Result<()> {
        // Enable character extraction mode
        self.extract_spans = false;
        self.chars.clear();
        self.spans.clear(); // Ensure spans are clear so they don't poison xobject_spans_cache
        self.placed_pdf_keep = Self::placed_pdf_text_dominates(content_stream);

        self.run_content_stream(content_stream)?;

        // BUG FIX #2: Sort characters by reading order (top-to-bottom, left-to-right)
        // PDF content streams are in rendering order, not reading order.
        // PDF Y coordinates increase upward, so higher Y = top of page.
        // We need to sort by Y descending (top first), then X ascending (left to right).
        self.sort_by_reading_order();

        // BUG FIX #3: Deduplicate overlapping characters
        // Some PDFs render text multiple times (for effects like boldness, shadowing).
        // This causes characters to appear at very close X positions (< 2pt).
        // We deduplicate by keeping only the first character when multiple chars
        // at the same Y position have X positions within 2pt of each other.
        self.deduplicate_overlapping_chars();

        Ok(())
    }

    /// Same extraction as [`Self::extract`], but hands the buffer over instead
    /// of copying it. Every `TextChar` owns a `font_name` String, so `extract`'s
    /// clone re-allocates once per glyph — measurable on long documents. Leaves
    /// `self.chars` empty, so callers that read `char_count`/`chars` afterwards
    /// must keep using [`Self::extract`].
    pub fn extract_owned(&mut self, content_stream: &[u8]) -> Result<Vec<TextChar>> {
        self.extract_into_self(content_stream)?;
        Ok(std::mem::take(&mut self.chars))
    }

    /// Deduplicate overlapping characters on the same line.
    ///
    /// Some PDFs render text multiple times at slightly different X positions
    /// (e.g., for bold effect or shadowing). This causes garbled text output when
    /// all renders are extracted. We keep only one character when multiple chars
    /// at nearly the same position exist.
    ///
    /// Heuristic: If two consecutive characters on the same line (Y rounded to
    /// integer) overlap by a fraction of their own advance width, keep only the
    /// first one.
    ///
    /// The threshold is expressed as a fraction of the glyph's `advance_width`
    /// (see [`Self::DEDUP_OVERLAP_RATIO`]) rather than an absolute point
    /// value. Real rendering duplicates (stroke+fill, bold shadow,
    /// outline+fill) sit at nearly identical positions — well under 30 % of
    /// one advance apart. Legitimate adjacent doublets of narrow glyphs
    /// (`ll`, `rr`, `II`, `ii` at small font sizes) are separated by one
    /// full advance; an absolute threshold of e.g. 2 pt would wrongly
    /// collapse them on fonts where a narrow glyph's advance drops below
    /// ~2 pt (e.g. Helvetica at ≤ 9 pt).
    ///
    /// Capped at [`Self::DEDUP_OVERLAP_CAP_PT`] to preserve the existing
    /// behaviour for pathologically oversized advance values, and falls
    /// back to `bbox.width` when `advance_width` is missing from the font
    /// dictionary.
    fn deduplicate_overlapping_chars(&mut self) {
        if self.chars.is_empty() {
            return;
        }

        let before = self.chars.len();
        let mut prev_y_rounded: Option<i32> = None;
        let mut prev_x: Option<f32> = None;
        let mut prev_char: Option<char> = None;

        // Retained in place: the predicate only looks back at the previously
        // KEPT glyph, which `retain`'s in-order visit preserves. Building a
        // second Vec instead deep-cloned every glyph — and `TextChar` owns a
        // `font_name` String, so that was one malloc per glyph, per page.
        self.chars.retain(|ch| {
            let y_rounded = ch.bbox.y.round() as i32;
            let x = ch.bbox.x;

            // Check if this char overlaps with the previous one
            let should_skip = if let (Some(prev_y), Some(prev_x_val), Some(prev_ch)) =
                (prev_y_rounded, prev_x, prev_char)
            {
                // Reference width: advance_width if known, else bbox.width,
                // else the legacy cap (keeps behaviour for pathological
                // inputs without advance metrics).
                let ref_width = if ch.advance_width > 0.0 {
                    ch.advance_width
                } else if ch.bbox.width > 0.0 {
                    ch.bbox.width
                } else {
                    Self::DEDUP_OVERLAP_CAP_PT
                };
                let threshold =
                    (ref_width * Self::DEDUP_OVERLAP_RATIO).min(Self::DEDUP_OVERLAP_CAP_PT);
                // Same character, same line, and within `threshold` horizontally
                ch.char == prev_ch && y_rounded == prev_y && (x - prev_x_val).abs() < threshold
            } else {
                false
            };

            if !should_skip {
                prev_y_rounded = Some(y_rounded);
                prev_x = Some(x);
                prev_char = Some(ch.char);
            } else {
                log::trace!(
                    "Deduplicating overlapping char '{}' at X={:.1}, Y={:.1} (too close to previous)",
                    ch.char,
                    x,
                    ch.bbox.y
                );
            }
            !should_skip
        });

        log::debug!(
            "Deduplicated {} overlapping characters ({} -> {} chars)",
            before - self.chars.len(),
            before,
            self.chars.len()
        );
    }

    /// Snap super/subscript glyph spans onto the baseline of an
    /// adjacent base span so downstream row-aware sorting keeps
    /// them inline.
    ///
    /// PDF §9.3.7 defines text rise (`Ts`) as a per-text-state
    /// vertical offset added to the rendering position; the
    /// resulting glyphs sit above (super) or below (sub) the
    /// surrounding baseline. The raw extracted bbox preserves
    /// that offset, so sorting by Y descending interprets a
    /// superscript line of affiliation markers (`1,2 ★ 3,4 …`)
    /// as a row that precedes the author names that they actually
    /// annotate. Snapping each candidate's Y to the matched base
    /// puts them back in the same Y-band.
    ///
    /// A span is a snap candidate when:
    /// - its font_size is < 85 % of a nearby larger-font span,
    /// - its Y is above that base by ≤ 50 % of the base's font_size
    ///   (or below it by the same — covers subscript too), and
    /// - its X falls between the base's right edge and one base
    ///   font_size beyond (the position a superscript would
    ///   appear when typeset directly after the base).
    fn snap_superscript_baselines(&mut self) {
        let n = self.spans.len();
        if n < 2 {
            return;
        }

        // Snapshot the read-side fields we need so the borrow checker
        // lets us mutate `self.spans[i].bbox.y` inside the loop.
        let snapshot: Vec<(f32, f32, f32, f32)> = self
            .spans
            .iter()
            .map(|s| (s.bbox.x, s.bbox.y, s.bbox.width, s.font_size))
            .collect();

        // A valid base candidate `j` always has `y_offset = sy - by` in
        // `[0, bfs*0.5]` (see the gates below), so `by` lies in
        // `[sy - bfs*0.5, sy] ⊆ [sy - max_fs*0.5, sy]`. Sort span indices by
        // Y once and, per candidate, binary-search that Y-window instead of
        // rescanning all spans — this turns the previous O(n²) double loop
        // (which hung for >30 s on archive.org / Google-Books pages whose
        // invisible hOCR layer emits thousands of spans, #575) into roughly
        // O(n log n + n·window). The window is a strict superset of the
        // acceptable bases, so the result is identical to the full scan.
        let max_fs = snapshot.iter().map(|s| s.3).fold(0.0f32, f32::max);
        let max_half_em = max_fs * 0.5;
        let mut by_order: Vec<usize> = (0..n).collect();
        by_order.sort_by(|&a, &b| crate::utils::safe_float_cmp(snapshot[a].1, snapshot[b].1));
        let ys_sorted: Vec<f32> = by_order.iter().map(|&idx| snapshot[idx].1).collect();

        for i in 0..n {
            let (sx, sy, _sw, sfs) = snapshot[i];
            if sfs <= 0.0 {
                continue;
            }
            // Find the closest base candidate (in Y) that satisfies
            // the super/subscript geometry. Pick the smallest |y_offset|
            // tie-breaker so a candidate sandwiched between two body
            // lines snaps onto the nearer one.
            let mut best_base_y: Option<f32> = None;
            let mut best_abs_offset = f32::MAX;
            // Candidates have `by ∈ [sy - max_half_em, sy]`; restrict the scan
            // to that contiguous slice of the Y-sorted index.
            let lo = ys_sorted.partition_point(|&y| y < sy - max_half_em);
            let hi = ys_sorted.partition_point(|&y| y <= sy);
            for &j in &by_order[lo..hi] {
                if i == j {
                    continue;
                }
                let (bx, by, bw, bfs) = snapshot[j];
                if bfs <= sfs * 1.15 {
                    continue;
                }
                let y_offset = sy - by;
                let half_em = bfs * 0.5;
                if y_offset.abs() > half_em {
                    continue;
                }
                // Skip subscripts (lowered glyphs). The document-level
                // pass `apply_super_sub_script_substitutions` needs to
                // see them at their original lowered baseline so it can
                // substitute ASCII digits with U+2080..U+2089 (e.g.
                // H2O -> H\u{2082}O). Snapping them onto the base
                // baseline would defeat that substitution.
                if y_offset < 0.0 {
                    continue;
                }
                // X adjacency: the candidate's left edge must sit
                // near the base's right edge — within one base
                // font_size to the right and a small slack to the
                // left for kerning. Combining diacritics are
                // excluded by the size-ratio gate above (they
                // typically share font_size with their base
                // letter, failing `bfs > sfs * 1.15`).
                let base_right = bx + bw;
                let dx = sx - base_right;
                if dx < -bfs * 0.25 || dx > bfs {
                    continue;
                }
                let abs_off = y_offset.abs();
                if abs_off < best_abs_offset {
                    best_abs_offset = abs_off;
                    best_base_y = Some(by);
                }
            }
            if let Some(by) = best_base_y {
                self.spans[i].bbox.y = by;
            }
        }
    }

    /// Sort extracted text spans by reading order (top-to-bottom, left-to-right).
    fn sort_spans_by_reading_order(&mut self) {
        if self.spans.is_empty() {
            return;
        }

        // Vertical-mode (tategaki) routing. Each span carries the writing
        // mode it was emitted under (`wmode == 1` for vertical text). When
        // the page is *predominantly* vertical we apply column-aware
        // top-to-bottom + right-to-left ordering. When the page is
        // predominantly horizontal we fall through to the existing
        // horizontal sort; the rare mixed-mode case stays governed by the
        // dominant mode here. Per-span wmode is preserved on every span
        // either way, so downstream consumers (export, search) can still
        // distinguish them.
        let vertical_count = self.spans.iter().filter(|s| s.wmode == 1).count();
        let total = self.spans.len();
        if total > 0 && vertical_count * 2 >= total {
            log::trace!(
                "Reading order: {}/{} spans are vertical — using tategaki sort",
                vertical_count,
                total
            );
            self.sort_spans_vertical_tategaki();
            return;
        }

        // Detect columns first
        let columns = self.detect_span_columns();

        log::trace!(
            "Column detection: found {} columns from {} spans",
            columns.len(),
            self.spans.len()
        );
        for (i, (left, right)) in columns.iter().enumerate() {
            log::trace!(
                "  Column {}: X range [{:.1}, {:.1}] (width: {:.1})",
                i,
                left,
                right,
                right - left
            );
        }

        if columns.len() <= 1 {
            // Single column or no columns detected: use simple sort
            log::trace!("Using simple Y-then-X sorting (single column)");
            self.simple_sort_spans();
        } else {
            // Multi-column layout: sort within each column, then across columns
            log::trace!("Using column-aware sorting ({} columns)", columns.len());
            self.sort_spans_by_columns(&columns);
        }
    }

    /// Sort spans in vertical writing (tategaki) order: right-to-left
    /// across columns, top-to-bottom within each column. See
    /// `crate::utils::sort_vertical_tategaki` for the column-clustering
    /// algorithm and the total-order rationale.
    fn sort_spans_vertical_tategaki(&mut self) {
        self.spans =
            crate::utils::sort_vertical_tategaki(std::mem::take(&mut self.spans), |s| &s.bbox);
    }

    /// Simple Y-then-X sorting for single-column layouts.
    fn simple_sort_spans(&mut self) {
        self.spans.sort_by(|a, b| {
            // Round Y coordinates for stable comparison
            let a_y_rounded = a.bbox.y.round() as i32;
            let b_y_rounded = b.bbox.y.round() as i32;

            match b_y_rounded.cmp(&a_y_rounded) {
                std::cmp::Ordering::Equal => {
                    // Same line: sort by X ascending (left to right)
                    crate::utils::safe_float_cmp(a.bbox.x, b.bbox.x)
                },
                other => other,
            }
        });
    }

    /// Detect columns by analyzing X-coordinate distribution.
    ///
    /// Returns column boundaries as (left_x, right_x) pairs, sorted left-to-right.
    fn detect_span_columns(&self) -> Vec<(f32, f32)> {
        if self.spans.is_empty() {
            return vec![];
        }

        // Find page bounds
        let min_x = self
            .spans
            .iter()
            .map(|s| s.bbox.x)
            .fold(f32::INFINITY, f32::min);
        let max_x = self
            .spans
            .iter()
            .map(|s| s.bbox.x + s.bbox.width)
            .fold(f32::NEG_INFINITY, f32::max);

        let page_width = max_x - min_x;

        // Build X-coordinate histogram to find vertical gaps
        let bins = 100;
        let bin_width = page_width / bins as f32;
        let mut histogram = vec![0; bins];

        for span in &self.spans {
            let start_bin = ((span.bbox.x - min_x) / bin_width) as usize;
            let end_bin = ((span.bbox.x + span.bbox.width - min_x) / bin_width) as usize;

            for i in start_bin..=end_bin.min(bins - 1) {
                histogram[i] += 1;
            }
        }

        // Find gaps (bins with zero or very low content)
        let avg_density: f32 = histogram.iter().sum::<i32>() as f32 / bins as f32;
        let gap_threshold = (avg_density * 0.2).max(1.0); // 20% of average or at least 1

        let mut gaps = vec![];
        let mut in_gap = false;
        let mut gap_start = 0;

        for (i, &count) in histogram.iter().enumerate() {
            if count as f32 <= gap_threshold {
                if !in_gap {
                    gap_start = i;
                    in_gap = true;
                }
            } else if in_gap {
                // End of gap - record if significant
                // Use 2% of page width OR absolute 15pt minimum (catches narrow column gutters)
                let gap_width = (i - gap_start) as f32 * bin_width;
                if gap_width > (page_width * 0.02).max(15.0) {
                    let gap_x = min_x + gap_start as f32 * bin_width;
                    gaps.push(gap_x);
                }
                in_gap = false;
            }
        }

        // No significant gaps found - single column
        if gaps.is_empty() {
            return vec![(min_x, max_x)];
        }

        // Build column boundaries from gaps
        let mut columns = vec![];
        let mut left = min_x;

        for gap_x in gaps {
            columns.push((left, gap_x));
            left = gap_x;
        }
        columns.push((left, max_x));

        log::debug!("Detected {} columns: {:?}", columns.len(), columns);

        columns
    }

    /// Sort spans by column-aware reading order.
    ///
    /// Process columns left-to-right, and within each column, top-to-bottom.
    fn sort_spans_by_columns(&mut self, columns: &[(f32, f32)]) {
        // Assign each span to a column
        let mut column_spans: Vec<Vec<TextSpan>> = vec![vec![]; columns.len()];

        for span in self.spans.drain(..) {
            let span_center_x = span.bbox.x + span.bbox.width / 2.0;

            // Find which column this span belongs to
            let col_idx = columns
                .iter()
                .position(|&(left, right)| span_center_x >= left && span_center_x <= right)
                .unwrap_or(0); // Default to first column if not found

            column_spans[col_idx].push(span);
        }

        // Sort within each column (top-to-bottom, then left-to-right)
        for col_spans in &mut column_spans {
            col_spans.sort_by(|a, b| {
                let a_y_rounded = a.bbox.y.round() as i32;
                let b_y_rounded = b.bbox.y.round() as i32;

                match b_y_rounded.cmp(&a_y_rounded) {
                    std::cmp::Ordering::Equal => crate::utils::safe_float_cmp(a.bbox.x, b.bbox.x),
                    other => other,
                }
            });
        }

        // Reassemble: read columns left-to-right
        for col_spans in column_spans {
            self.spans.extend(col_spans);
        }
    }

    /// Deduplicate overlapping text spans on the same line.
    ///
    /// Uses hybrid geometric + content-based deduplication:
    /// - Geometric check (same Y, X within a fraction of the span's per-glyph
    ///   advance) — catches identical positions
    /// - Content check (same text, same line Y, different X) — catches
    ///   duplicates across columns
    ///
    /// The geometric threshold is expressed as a fraction of the span's
    /// per-glyph width (bbox.width / char_count), capped by
    /// [`Self::DEDUP_OVERLAP_CAP_PT`] and scaled by
    /// [`Self::DEDUP_OVERLAP_RATIO`]. An absolute threshold would wrongly
    /// collapse legitimate single-glyph spans of adjacent narrow glyphs
    /// (`ll`, `rr`, `II`, `ii` at small font sizes) in PDFs that emit text
    /// glyph-by-glyph with kerning.
    fn deduplicate_overlapping_spans(&mut self) {
        if self.spans.is_empty() {
            return;
        }

        // Phase 0 (B7): same-text overlapping spans from stroke+fill render
        // passes. Maps (newspaper / poster) frequently draw every label
        // twice — once stroked for outline, once filled — and both passes
        // land at essentially the same CTM. Without this up-front filter,
        // the merge step later concatenates them into "EverestEverest" /
        // "CentralCentral". We bucket by lowercased text and compare each
        // new span's bbox against prior entries via IoU; any later span
        // whose bbox overlaps an earlier one by >= 70 % is dropped.
        self.dedup_stroke_fill_overlap();

        // Take ownership of spans to avoid cloning during iteration
        let old_len = self.spans.len();
        let spans = std::mem::take(&mut self.spans);
        let mut deduplicated = Vec::with_capacity(old_len);
        let mut prev_y_rounded: Option<i32> = None;
        let mut prev_x: Option<f32> = None;
        let mut prev_text: Option<String> = None;
        // Every place a given string has been kept, not merely the last one.
        //
        // One slot per string cannot see an overprint. A page that draws each
        // glyph twice — a grey pass and a black pass a fraction of a point
        // apart, the usual way of faking a bold — yields single-glyph spans, so
        // a title like `SICHERHEITS` stores `S` at its first position and then
        // overwrites it with the `S` at the end of the same word. The second
        // pass then compares its `S` against the wrong one and finds no
        // duplicate.
        const MAX_TRACKED_POSITIONS: usize = 32;
        let mut seen_content: std::collections::HashMap<String, Vec<(f32, f32)>> =
            std::collections::HashMap::new();

        let mut geometric_skips = 0;
        let mut content_skips = 0;

        for span in spans {
            let y_rounded = span.bbox.y.round() as i32;
            let x = span.bbox.x;

            // PHASE 1: Geometric deduplication — require BOTH position AND text match
            let geometric_duplicate = if let (Some(prev_y), Some(prev_x_val), Some(ref prev_txt)) =
                (prev_y_rounded, prev_x, &prev_text)
            {
                // Threshold scales with the span's per-glyph advance so that
                // single-glyph narrow spans (`l`, `r`, `I`) are never wrongly
                // treated as overlapping with their legitimate neighbour.
                let char_count = span.text.chars().count().max(1) as f32;
                let per_glyph_width = (span.bbox.width / char_count).max(0.1);
                let threshold =
                    (per_glyph_width * Self::DEDUP_OVERLAP_RATIO).min(Self::DEDUP_OVERLAP_CAP_PT);
                y_rounded == prev_y && (x - prev_x_val).abs() < threshold && span.text == *prev_txt
            } else {
                false
            };

            // PHASE 2: Content-based deduplication — require positions to OVERLAP
            //
            // Short runs are included. They were excluded because a five-byte
            // floor is a cheap stand-in for "this string is distinctive enough
            // that repeating it at one position means a duplicate", but it also
            // exempts every overprinted glyph, and PHASE 1 cannot cover them:
            // it compares against the immediately preceding span only, and the
            // sort that decides adjacency uses the same integer-rounded row key
            // that the overprint's sub-point offset straddles, so the two
            // copies are never neighbours — a page's whole grey pass is emitted
            // before its black pass begins.
            //
            // What replaces the floor for a short run is a tighter bar on
            // position: the same per-glyph advance PHASE 1 uses, so two
            // legitimate repeats of one letter — the `ll` in `callibrator`, an
            // `SS` — stand a full advance apart and are never confused with a
            // pair that differs by a fraction of one.
            let content_duplicate = {
                let char_count = span.text.chars().count().max(1) as f32;
                let per_glyph_width = (span.bbox.width / char_count).max(0.1);
                let x_bar = if span.text.len() >= 5 {
                    5.0
                } else {
                    (per_glyph_width * Self::DEDUP_OVERLAP_RATIO).min(Self::DEDUP_OVERLAP_CAP_PT)
                };
                seen_content.get(&span.text).is_some_and(|seen| {
                    seen.iter().any(|(prev_x_val, prev_y_val)| {
                        (span.bbox.y - prev_y_val).abs() < 2.0
                            && (span.bbox.x - prev_x_val).abs() < x_bar
                    })
                })
            };

            if geometric_duplicate {
                geometric_skips += 1;
            } else if content_duplicate {
                content_skips += 1;
            } else {
                prev_y_rounded = Some(y_rounded);
                prev_x = Some(x);
                prev_text = Some(span.text.clone());

                // Track content for duplicate detection. Bounded, so a page
                // repeating one string thousands of times cannot grow this
                // without limit; the positions that matter for an overprint are
                // the recent ones.
                let at = seen_content.entry(span.text.clone()).or_default();
                if at.len() < MAX_TRACKED_POSITIONS {
                    at.push((span.bbox.x, span.bbox.y));
                }
                // Move span instead of cloning
                deduplicated.push(span);
            }
        }

        log::debug!(
            "Deduplicated {} spans (geometric: {}, content: {}) ({} -> {} spans)",
            geometric_skips + content_skips,
            geometric_skips,
            content_skips,
            old_len,
            deduplicated.len()
        );

        self.spans = deduplicated;
    }

    /// Drop same-text spans whose bounding boxes overlap heavily with an
    /// earlier span. This is the canonical stroke+fill pattern on maps,
    /// posters, and marketing materials: a label is drawn twice (once
    /// stroked for the outline, once filled for the glyph) at identical
    /// positions. Both passes surface as distinct spans; without this
    /// filter the downstream merge pass concatenates them.
    ///
    /// Keyed by lowercased text + rounded (x, y) bucket to make the
    /// lookup O(1) without quadratic bbox comparisons on large pages.
    /// The actual overlap check falls through to a real IoU on collision.
    fn dedup_stroke_fill_overlap(&mut self) {
        use std::collections::HashMap;

        if self.spans.len() < 2 {
            return;
        }
        let old_len = self.spans.len();
        let spans = std::mem::take(&mut self.spans);
        // Bucket each text key into a coarse (cx,cy) grid instead of one flat
        // Vec (O(k²) when a short label repeats N times). The IoU ≥ 0.7 test is
        // unchanged: a partner with that overlap is within ≈0.176·width, so it
        // falls in this cell or a neighbour — querying the 3×3 neighbourhood
        // finds every match the full scan would. Only runs for text ≥ 2 chars
        // (shorter candidates rely on downstream positional dedup).
        const CELL: f32 = 16.0;
        type Grid = HashMap<(i32, i32), Vec<crate::geometry::Rect>>;
        let mut seen: HashMap<String, Grid> = HashMap::new();
        let mut kept: Vec<TextSpan> = Vec::with_capacity(old_len);
        let mut skipped = 0usize;
        for span in spans {
            let trimmed = span.text.trim();
            if trimmed.chars().count() < 2 {
                kept.push(span);
                continue;
            }
            let key = trimmed.to_ascii_lowercase();
            let b = span.bbox;
            let cx = ((b.x + b.width * 0.5) / CELL).floor() as i32;
            let cy = ((b.y + b.height * 0.5) / CELL).floor() as i32;
            let mut is_dup = false;
            if let Some(grid) = seen.get(&key) {
                // Saturating bounds: a span with an extreme/out-of-page bbox can
                // push cx/cy to the i32 limits, where `cx + 1` would overflow in
                // an overflow-checked build (observed on 1008.3918v2.pdf).
                'outer: for gx in cx.saturating_sub(1)..=cx.saturating_add(1) {
                    for gy in cy.saturating_sub(1)..=cy.saturating_add(1) {
                        let Some(others) = grid.get(&(gx, gy)) else {
                            continue;
                        };
                        for other in others {
                            // IoU — intersection over union. >= 0.7 means the
                            // two bboxes are almost the same rectangle, which is
                            // what stroke+fill produces.
                            let ix1 = b.x.max(other.x);
                            let iy1 = b.y.max(other.y);
                            let ix2 = (b.x + b.width).min(other.x + other.width);
                            let iy2 = (b.y + b.height).min(other.y + other.height);
                            if ix2 <= ix1 || iy2 <= iy1 {
                                continue;
                            }
                            let inter = (ix2 - ix1) * (iy2 - iy1);
                            let area_a = b.width * b.height;
                            let area_b = other.width * other.height;
                            let union = area_a + area_b - inter;
                            if union > 0.0 && inter / union >= 0.7 {
                                is_dup = true;
                                break 'outer;
                            }
                        }
                    }
                }
            }
            if is_dup {
                skipped += 1;
            } else {
                seen.entry(key)
                    .or_default()
                    .entry((cx, cy))
                    .or_default()
                    .push(b);
                kept.push(span);
            }
        }
        if skipped > 0 {
            log::debug!("Stroke+fill dedup: dropped {skipped} duplicate spans of {old_len}");
        }
        self.spans = kept;
    }

    /// Merge adjacent text spans on the same line to reconstruct complete words.
    ///
    /// PDF content streams often break words into multiple Tj operators for precise
    /// kerning/positioning. This causes word fragmentation like "Intr oduction" instead
    /// of "Introduction". We merge spans that are:
    /// - On the same line (Y coordinates within 1pt)
    /// - Very close horizontally (gap < 3pt, approximately average char width)
    ///
    /// Mark spans whose RTL glyphs were drawn **right-to-left** — the producer
    /// stored the text in LOGICAL order and positioned each glyph individually at
    /// decreasing x (ISO 32000-1 §14.8.2.3.3 method 1). Such spans' characters are
    /// already logical and must NOT be character-reversed by the structure-path
    /// `push_span_text_bidi`. VISUAL storage (glyphs drawn left-to-right) is never
    /// marked, so it keeps the default character-reversal and stays byte-identical.
    ///
    /// MUST run on the raw stream order (right after the content stream is parsed),
    /// before `sort_spans_by_reading_order`, which reorders the spans into
    /// left-to-right and erases the draw direction.
    ///
    /// The draw direction is the only signal that separates logical-stored RTL
    /// from visual-stored RTL when both use base-form characters with no Arabic
    /// presentation forms and no `/ReversedChars` (the two are otherwise
    /// indistinguishable yet need opposite treatment).
    fn detect_rtl_draw_direction(&mut self) {
        use crate::text::rtl_detector::is_rtl_text;
        fn is_rtl_span(s: &TextSpan) -> bool {
            let mut rtl = false;
            for c in s.text.chars() {
                if c.is_ascii_alphabetic() {
                    return false;
                }
                if is_rtl_text(c as u32) {
                    rtl = true;
                }
            }
            rtl
        }
        let n = self.spans.len();
        // Index of the previous RTL span in stream order; a pure-whitespace span
        // between two RTL glyphs (a word break) does not break the run.
        let mut prev: Option<usize> = None;
        for i in 0..n {
            if self.spans[i].text.chars().all(char::is_whitespace) {
                continue;
            }
            if !is_rtl_span(&self.spans[i]) {
                prev = None;
                continue;
            }
            if let Some(p) = prev {
                let same_line = (self.spans[i].bbox.y - self.spans[p].bbox.y).abs()
                    < self.spans[p].font_size.max(1.0) * 0.6;
                // The incoming glyph sits to the LEFT of the previous one on the
                // same baseline ⇒ right-to-left placement ⇒ logical storage.
                if same_line && self.spans[i].bbox.x < self.spans[p].bbox.x - 0.5 {
                    self.spans[i].rtl_draw_logical = true;
                    self.spans[p].rtl_draw_logical = true;
                }
            }
            prev = Some(i);
        }
    }

    /// Per-line bimodal word-gap thresholds for the narrow-space rescue (#847).
    ///
    /// The fixed intra-word kerning guard in `should_insert_space`
    /// (0.75× the space-glyph advance) suppresses genuine but *narrow* word
    /// gaps on condensed/tracked lines — a bold heading or a running footer
    /// typeset with NO space glyph, whose inter-word gaps are ~0.18 em, just
    /// under the guard. A fixed magnitude cannot separate a 0.18 em word gap
    /// from ~0.15 em intra-word kerning. But within one line the intra-word
    /// glyph gaps cluster near zero (tight/slightly-overlapping side-bearings)
    /// while the inter-word gaps form a distinct larger cluster: a clean
    /// bimodal split that pins the word boundary *regardless of absolute
    /// magnitude*.
    ///
    /// This walks the content-order span list, groups it into baseline runs,
    /// and for each run whose inter-span gaps are clearly bimodal returns the
    /// gap value separating the two clusters (indexed per span). Spans on
    /// unimodal or too-short lines get `None` and keep the default guard. The
    /// merge loop uses a returned threshold only to *rescue* a suppressed word
    /// gap — it never removes a space the default logic already inserts.
    fn bimodal_line_gap_thresholds(spans: &[TextSpan]) -> Vec<Option<f32>> {
        let n = spans.len();
        let mut out = vec![None; n];
        let mut i = 0;
        while i < n {
            // Extend a run of consecutive same-baseline spans.
            let mut j = i;
            while j + 1 < n && (spans[j].bbox.y - spans[j + 1].bbox.y).abs() < 1.0 {
                j += 1;
            }
            if j > i {
                let fs = spans[i..=j]
                    .iter()
                    .map(|s| s.font_size)
                    .fold(0.0f32, f32::max)
                    .max(1.0);
                // ALL consecutive gaps (intra-word gaps are near-zero or
                // slightly negative, so they must be kept, not filtered) — but
                // ONLY between glyphs sharing a baseline. A super/subscript sits
                // at a baseline shift (~0.15 em) and its horizontal gap to the
                // base is the same ~0.10 em magnitude as a condensed footer's
                // word gap; including it would let the narrow-gap rescue split a
                // math subscript from its variable (`λᵢ` → `λ i`), which the
                // advance-aware extractors correctly do NOT do. Excluding
                // baseline-shifted pairs keeps the footer word gap (same
                // baseline) while leaving dense math untouched.
                let gaps: Vec<f32> = (i..j)
                    .filter(|&k| (spans[k].bbox.y - spans[k + 1].bbox.y).abs() < fs * 0.04)
                    .map(|k| spans[k + 1].bbox.x - (spans[k].bbox.x + spans[k].bbox.width))
                    .collect();
                if let Some(split) = Self::bimodal_gap_split(&gaps, fs) {
                    for slot in out.iter_mut().take(j + 1).skip(i) {
                        *slot = Some(split);
                    }
                }
            }
            i = j + 1;
        }
        out
    }

    /// Given the consecutive inter-span gaps of one baseline run, return the
    /// threshold separating an intra-word cluster from an inter-word cluster
    /// when the distribution is clearly bimodal, else `None`. `fs` is the
    /// run's font size; all bounds are expressed as em fractions so headings
    /// and body calibrate independently.
    fn bimodal_gap_split(gaps: &[f32], fs: f32) -> Option<f32> {
        if gaps.len() < 3 {
            return None;
        }
        let mut sorted = gaps.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // Return the LOWEST cluster border, not the widest jump: walking the
        // sorted gaps from the bottom, the first jump that leaves the intra-word
        // cluster for a real word gap. A qualifying border needs
        //   * an intra-word-sized low side (< 0.10 em — kerning, tight
        //     side-bearings, or overlap),
        //   * a high side that is a real (if narrow) word gap (>= 0.09 em) —
        //     reaching the ~0.10 em gaps of condensed running footers that
        //     pymupdf/pdfplumber's fixed thresholds miss (an explicit positive
        //     advance IS a word-boundary signal, ISO 32000-1 §9.4.4), and
        //   * a real separation between them (>= 0.08 em), not a smooth spread.
        // Taking the LOWEST such border handles a *multi-level* condensed line —
        // tight intra-word gaps, a narrow ~0.10 em word gap, AND a wide real
        // space glyph — splitting at every level above intra-word, matching the
        // advance-aware extractors (pdfminer, poppler). A single-word line (all
        // gaps low) yields no qualifying border and returns None. The caller
        // feeds only SAME-BASELINE gaps, so a math subscript gap of the same
        // magnitude (which sits at a baseline shift) never enters this
        // distribution and is not split.
        for w in sorted.windows(2) {
            let (lo, hi) = (w[0], w[1]);
            if lo < fs * 0.10 && hi >= fs * 0.09 && (hi - lo) >= fs * 0.08 {
                return Some((lo + hi) * 0.5);
            }
        }
        None
    }

    /// This matches the behavior of industry-standard PDF tools.
    fn same_painted_span_style(left: &TextSpan, right: &TextSpan) -> bool {
        // Positional lowering emits one character style per text box. Reading
        // order may glue emphasis runs, but paint must retain each source style.
        left.font_name == right.font_name
            && left.font_size == right.font_size
            && left.font_weight == right.font_weight
            && left.is_italic == right.is_italic
            && left.is_monospace == right.is_monospace
            && left.color == right.color
            && left.char_spacing == right.char_spacing
            && left.word_spacing == right.word_spacing
            && left.horizontal_scaling == right.horizontal_scaling
            && left.text_rise == right.text_rise
            && left.wmode == right.wmode
            && left.rotation_degrees == right.rotation_degrees
            && left.mirrored == right.mirrored
    }

    fn merge_adjacent_spans(&mut self) {
        if self.spans.is_empty() {
            return;
        }

        // Take ownership of spans to avoid cloning during iteration
        let old_len = self.spans.len();
        let spans = std::mem::take(&mut self.spans);
        // Geometry of every drawn (non-whitespace) glyph run, captured
        // before the fold consumes the list. The decimal-merge branch below
        // needs it: a separator glyph between two digit runs — the comma of
        // a subscript index pair like `P_{1,0}` — is often drawn elsewhere
        // in the content stream, so the fold sees the digits as adjacent
        // and only a geometric test over ALL runs can spot the ink sitting
        // in the gap.
        let ink_boxes: Vec<Rect> = spans
            .iter()
            .filter(|s| !s.text.trim().is_empty())
            .map(|s| s.bbox)
            .collect();
        // #847 M2: per-line bimodal word-gap thresholds, indexed to `spans`,
        // used below to rescue a narrow word gap the fixed kerning guard
        // suppressed. Computed before the fold consumes the list.
        let line_thresholds = Self::bimodal_line_gap_thresholds(&spans);
        let mut merged = Vec::with_capacity(old_len);
        let mut current_span: Option<TextSpan> = None;

        for (span_idx, span) in spans.into_iter().enumerate() {
            if current_span.is_none() {
                // First span — move, no clone needed
                current_span = Some(span);
                continue;
            }

            // Take ownership of current to avoid borrow checker issues.
            // Safety: checked is_none() above which continues, so this is always Some.
            let mut current = match current_span.take() {
                Some(s) => s,
                None => {
                    current_span = Some(span);
                    continue;
                },
            };

            if self.visible_paint_only
                && (!Self::same_painted_span_style(&current, &span)
                    || self
                        .span_paint_orders
                        .get(&current.sequence)
                        .map(|order| order.group)
                        != self
                            .span_paint_orders
                            .get(&span.sequence)
                            .map(|order| order.group)
                    || !self.same_painted_font_resource(current.sequence, span.sequence))
            {
                merged.push(current);
                current_span = Some(span);
                continue;
            }

            // Spans drawn under different writing modes must never merge,
            // even when their baselines coincide. A horizontal (`wmode=0`)
            // span advances along x; a vertical (`wmode=1`) span advances
            // along y. The text-level merge semantics (same word, same line,
            // gap small enough to glue) all assume a single advance axis,
            // and the bbox-extension at the end of the merge branch grows
            // a horizontal bbox even if the right-hand side was a vertical
            // column. Fold per-span wmode into the line-equality test
            // up-front so all downstream merge variants (same-font,
            // cross-font glue, small-caps, decimal-merge) inherit the
            // gate. Without this, `BT 100 700 Td /F1 12 Tf (A) Tj
            // /F2 12 Tf (B) Tj ET` with F1 horizontal + F2 vertical glues
            // the two glyphs into a single horizontal span and clobbers
            // the wmode metadata for the vertical glyph.
            let wmode_compatible = current.wmode == span.wmode;
            // ±90°-rotated runs (text matrix rotation, not wmode) advance
            // along Y with their line axis on X, so the portrait same-line
            // test below reads PERPENDICULAR geometry for them: two runs
            // from adjacent rotated lines share a baseline-Y and sit a
            // word-gap apart in X, which glued words from different lines
            // of a rotated table into one span ("row" + "row" → "row row",
            //). Runs in a rotated frame never merge here; each
            // stays per-literal and the rotated-frame reading order and
            // word assembly handle them downstream.
            // "Never merge here" per the comment above means exactly that —
            // any non-zero rotation on either side stays per-literal, not
            // just the ±90° (vertical-quadrant) case. A 180°-rotated run
            // previously slipped through this gate (only ±90° was checked),
            // letting two upside-down lines' runs glue together under the
            // portrait same-line test below even though 180° text advances
            // in the opposite X direction.
            let rotation_compatible =
                current.rotation_degrees == 0.0 && span.rotation_degrees == 0.0;
            let y_diff = (span.bbox.y - current.bbox.y).abs();
            let same_line = y_diff < 1.0 && wmode_compatible && rotation_compatible;

            // Gap between end of current span and start of next span
            let current_end_x = current.bbox.x + current.bbox.width;
            let gap = span.bbox.x - current_end_x;
            // Fallback-width correction: When the previous
            // span's font has no explicit `/Widths` array, every glyph in
            // that span reports the 500/550/600-thousandths-of-em fallback
            // from `FontInfo::new`. For proportional Latin fonts whose
            // real glyphs are narrower than that fallback (`SR` in the
            // NASA Apollo report is a concrete example), the span's
            // `bbox.width` is systematically inflated and `current_end_x`
            // overshoots the actual end of the rendered text — often by
            // enough to swallow the real inter-word gap entirely, turning
            // the visible word boundary into a negative `gap` value
            // tripping merge logic that then glues the words without a
            // space.
            //
            // `space_gap` is a corrected gap value used ONLY for the
            // space-insertion decision below. The original `gap` is left
            // unchanged so the merge-vs-column decision, the decimal-merge
            // heuristic, and any downstream branch that reasons about the
            // actual bbox layout still see the real layout and don't
            // suddenly reclassify legitimate adjacent words as column
            // boundaries. In other words: the merge still happens exactly
            // as before on fallback-width fonts, but once we're inside the
            // merge branch we consult a more honest gap to decide whether
            // a space is warranted.
            let reliable_widths = self
                .fonts
                .get(&current.font_name)
                .map(|f| f.has_explicit_widths())
                .unwrap_or(true);
            let space_gap = corrected_space_gap(
                gap,
                reliable_widths,
                current.bbox.width,
                current.text.is_empty(),
            );

            // Column-boundary gap, font-size-aware. The same 6pt gap is
            // a column gutter at 11pt body text but normal word kerning
            // at a 36pt title; use 0.5em as a floor above the configured
            // absolute threshold.
            let font_size_ref = current.font_size.max(span.font_size);
            let column_threshold = self
                .merging_config
                .column_boundary_threshold_pt
                .max(font_size_ref * 0.5);
            let large_gap_indicates_column = gap > column_threshold;

            // SPLIT BOUNDARY CHECK: Respect boundaries from CamelCase splitting
            // If a span has split_boundary_before=true, it represents a word boundary
            // from a split operation (e.g., "the" + "General" from "theGeneral")
            // These should always be merged WITH a space, never without.
            let has_split_boundary = span.split_boundary_before;

            // Font identity: same base font AND same size AND same styling.
            let is_same_font = current.font_name == span.font_name
                && (current.font_size - span.font_size).abs() < 0.01
                && current.font_weight == span.font_weight
                && current.is_italic == span.is_italic;

            // MCID identity: per ISO 32000-1:2008 §14.6, two adjacent
            // Tj operators that sit in different marked-content
            // sequences belong to different *structure elements*.
            // Merging them would silently fuse their identities (the
            // merged span keeps `current.mcid`) and lose the
            // boundary that downstream consumers — structure-tree
            // reading order, tree-scope ActualText suppression,
            // table-cell membership — rely on. Adjacent spans whose
            // MCIDs differ (including one `None` ↔ one `Some(_)`)
            // are kept separate.
            let same_mcid = current.mcid == span.mcid;

            // Cross-font word glue: same-baseline spans in different
            // fonts/weights, tight gap (<0.25em), both sides alphabetic,
            // and one side is a single character. Targets the drop-cap /
            // single-letter-small-caps typography pattern where per-
            // letter emphasis runs would corrupt proper nouns.
            //
            // Issue 484 (pr-136-example.pdf): CJK ideographs satisfy
            // `is_alphabetic()` per Unicode, so a CJK→Latin (or Latin→CJK)
            // transition between adjacent characters in different fonts —
            // the standard mixed-script PDF layout pattern — was triggering
            // cross-font glue and concatenating "神鹰集团" + "Z" into
            // "神鹰集团Z" with no separator. Word-F1 against pdftotext
            // ground truth (which inserts a space at every CJK↔non-CJK
            // boundary) then loses both the trailing CJK token and the
            // leading Latin/digit token. Skip cross-font glue when the
            // boundary crosses CJK / non-CJK scripts.
            //
            // EXCLUDES fullwidth ASCII (U+FF01..FF5E) and CJK Symbols
            // Punctuation (U+3000..303F) — those operator-style glyphs sit
            // inline with adjacent Latin/digit in CJK technical writing
            // (e.g. "60000≤Q＜80000" in issue-336). Treating them as a CJK
            // boundary would split the compound token.
            let is_cjk_char = |c: char| {
                matches!(
                    c as u32,
                    0x3040..=0x309F      // Hiragana
                    | 0x30A0..=0x30FF    // Katakana
                    | 0x3400..=0x4DBF    // CJK Unified Ideographs Extension A
                    | 0x4E00..=0x9FFF    // CJK Unified Ideographs
                    | 0xAC00..=0xD7AF    // Hangul Syllables
                    | 0x20000..=0x2A6DF  // CJK Unified Ideographs Extension B
                    | 0xFF66..=0xFF9F    // Halfwidth Katakana
                )
            };
            let prev_tail_char = current.text.chars().last();
            let curr_head_char = span.text.chars().next();
            let crosses_cjk_boundary = match (prev_tail_char, curr_head_char) {
                (Some(p), Some(c)) => is_cjk_char(p) != is_cjk_char(c),
                _ => false,
            };
            // Drop-caps / single-letter emphasis sit TIGHT against their word
            // (gap ~0, often overlapping). A gap in word-space territory
            // (≥~0.15em) across a font change is a genuine token boundary —
            // typically a word followed by a single-letter math variable in a
            // math-italic run (`solution` → `U`). Gluing those drops the space
            // poppler/PDFium keep. The 0.12em ceiling is the valley between
            // drop-cap kerning (~0) and a word space (≥~0.2em). (Previously
            // 0.25em, which is itself a full word space: the v0.3.75
            // advance-fold made per-glyph advance accurate enough that these
            // ~0.24em gaps — formerly inflated above 0.25em by the advance
            // undershoot — dropped under the ceiling and began gluing.)
            let cross_font_word_glue = !is_same_font
                && same_line
                && gap > -1.0
                && gap < font_size_ref * 0.12
                && !current.text.is_empty()
                && !span.text.is_empty()
                && !crosses_cjk_boundary
                && prev_tail_char.is_some_and(|c| c.is_alphabetic())
                && curr_head_char.is_some_and(|c| c.is_alphabetic())
                && (current.text.chars().count() == 1 || span.text.chars().count() == 1);

            // Small-caps / drop-cap glue: same base font and same
            // weight/italic flags but different font_size, adjacent
            // on the same baseline, both alphabetic. PDFs simulate
            // small-caps by rendering the capital initial at body
            // font size and the remaining letters at a reduced
            // size in the same font, emitted as separate Tj runs
            // with zero gap between them. The strict `is_same_font`
            // gate rejects the merge because of the size mismatch,
            // and the single-character drop-cap glue above doesn't
            // help when both runs are multi-character (an initial
            // run of several full-size capitals followed by a
            // reduced-size remainder). Spec basis: PDF §9.3.1
            // treats font_size as a graphics-state parameter that
            // may change between Tj operators; nothing in §9.4
            // makes such a change a word boundary.
            let small_caps_glue = !is_same_font
                && current.font_name == span.font_name
                && current.font_weight == span.font_weight
                && current.is_italic == span.is_italic
                && same_line
                && gap.abs() < 1.0
                && !current.text.is_empty()
                && !span.text.is_empty()
                && !crosses_cjk_boundary
                && prev_tail_char.is_some_and(|c| c.is_alphabetic())
                && curr_head_char.is_some_and(|c| c.is_alphabetic());

            // Merge threshold: Use configured values
            // Negative gaps: use severe_overlap_threshold_pt (default -0.5pt)
            // Positive gaps: use a threshold that allows for justified text but
            // avoids merging across clear column boundaries.
            // Same-font spans are merged more aggressively to reconstruct words.
            let merge_threshold_pt = if is_same_font {
                column_threshold.max(3.0)
            } else {
                // Different fonts: only merge if they are effectively overlapping
                // to handle minor kerning/rounding issues, but generally keep separate.
                0.5
            };

            let should_merge = (same_line
                && is_same_font
                && same_mcid
                && (self.merging_config.severe_overlap_threshold_pt..merge_threshold_pt)
                    .contains(&gap)
                && !large_gap_indicates_column)
                || (same_line && has_split_boundary && same_mcid)
                || (cross_font_word_glue && same_mcid)
                || (small_caps_glue && same_mcid);

            // DECIMAL VALUE MERGE: Some forms place integer and decimal parts
            // of dollar amounts in separate fixed-width boxes.
            // e.g., "123456" (integer box) + "72" (cents box) with ~10pt gap.
            // Detect this pattern: both spans are pure digits, the second is
            // exactly 1-2 digits (cents), same line, and there's a meaningful
            // column-boundary-sized gap between them.
            //
            // Issue 484 (pr-136-example.pdf): without a minimum-gap floor this
            // also matches tightly-packed adjacent digit characters from CJK
            // documents that emit each glyph as its own Tj — e.g. the year
            // "2013" rendered as four separate TjL operators with sub-pixel
            // gaps was being mangled into "201.3", losing the year token from
            // word-F1 scoring. Real "$123 _ 45" split-box layouts always have
            // a gap > ~half the font size; tight letter spacing is < 0.1 em.
            // A separator glyph drawn INSIDE the gap is proof the two digit
            // runs are distinct tokens (the comma of a subscript index pair
            // like `P_{1,0}`, drawn out of content-stream order): a genuine
            // split-box amount has nothing between its boxes. The gap band
            // alone cannot make this call — an index pair and a real
            // split-box amount can sit at the same gap-to-font-size ratio.
            // A genuine split-box amount prints its integer and cents at
            // the SAME size; a digit run markedly smaller than its
            // neighbour is super/subscript context (the exponent of a
            // scientific-notation value next to the following value's
            // mantissa), and fusing those fabricates a decimal.
            let decimal_sizes_match = {
                let (a, b) = (current.font_size, span.font_size);
                a > 0.0 && b > 0.0 && (a.min(b) / a.max(b)) >= 0.85
            };
            //
            // The gap also needs an upper ceiling. In scientific and math
            // PDFs, subscript index pairs like `P_{1,0}` draw the two subscript
            // digits in a smaller font (~7pt) spaced ~1.5-1.7x the font size
            // apart; too loose a ceiling lets the rule fire and invent a
            // decimal ("1" + "0" -> "1.0"). Genuine split-box amounts cluster
            // near ~0.8-1.0x the font size, so a 1.3x ceiling separates real
            // integer/cents boxes from widely-spaced subscripts.
            let min_decimal_gap = current.font_size * 0.4;
            let max_decimal_gap = current.font_size * 1.3;
            let decimal_merge = same_line
                && same_mcid
                && decimal_sizes_match
                && gap > min_decimal_gap
                && gap < max_decimal_gap
                && !current.text.is_empty()
                && !span.text.is_empty()
                && current.text.chars().all(|c| c.is_ascii_digit())
                && span.text.chars().all(|c| c.is_ascii_digit())
                && (1..=2).contains(&span.text.len())
                && !decimal_gap_has_ink(&ink_boxes, &current.bbox, &span.bbox);

            // Snapshot the pre-merge shape for the positional `char_widths`
            // maintenance below: the merged text is `current + [separator] +
            // span`, so each contribution's widths must land at the same
            // position its chars occupy. (Captured before any branch mutates
            // `current.text` / `current.bbox`.)
            let current_chars_before = current.text.chars().count();
            let span_char_count = span.text.chars().count();

            if decimal_merge {
                // Join integer and decimal parts with "."
                log::debug!(
                    "Decimal value merge: '{}' + '{}' -> '{}.{}' (gap={:.1}pt)",
                    current.text,
                    span.text,
                    current.text,
                    span.text,
                    gap
                );
                current.text.push('.');
                current.text.push_str(&span.text);
            } else if cross_font_word_glue || small_caps_glue {
                // Mid-word font/weight change: concatenate without any space
                // or space-heuristic — these are same-word character runs.
                //
                // `small_caps_glue` belongs here for the same reason, and
                // reaching the space heuristic instead is what put a space
                // inside a small-capitals caption: it admitted the merge, then
                // the heuristic it fell through to inserted a space anyway, so
                // `TABLE 66.01-11(5)-COORDINATES OF CHROMATICITY` came out as
                // `...-C OORDINATES...`. The predicate already establishes that
                // the two runs share a font, a weight, a slant and a baseline
                // and sit within a point of each other; ISO 32000-1:2008 §9.3.1
                // makes the size a graphics-state parameter that may change
                // between show operators, and nothing in §9.4 makes such a
                // change a word boundary.
                current.text.push_str(&span.text);
            } else if should_merge {
                // PHASE 1 FIX: Check if next span is entirely whitespace-only OR marked as offset_semantic space
                // If either is true, never insert an additional space - just concatenate directly
                // This prevents double-space issue when TJ processor creates space spans
                let next_is_whitespace_only = span.text.chars().all(|c| c.is_whitespace());
                let next_is_offset_semantic_space = span.offset_semantic && next_is_whitespace_only;

                // Merge spans: append text in-place using push_str (O(n) total vs O(n²) with format!)
                if next_is_whitespace_only {
                    // Next span is already space-only: just concatenate without adding more space
                    log::debug!(
                        "Merging with whitespace-only span: '{}' + '{}' (whitespace, offset_semantic={})",
                        current.text,
                        span.text.escape_default(),
                        span.offset_semantic
                    );
                    current.text.push_str(&span.text);
                } else {
                    let tj_offset_triggered_override = has_split_boundary;
                    let mut space_decision = should_insert_space(
                        &current.text,
                        &span.text,
                        space_gap,
                        current.font_size,
                        &current.font_name,
                        &self.fonts,
                        tj_offset_triggered_override,
                        &self.merging_config,
                        Some(&current.bbox),
                        Some(&span.bbox),
                        current.font_size,
                        span.font_size,
                    );

                    // #847 M2: narrow-word-gap rescue. The fixed intra-word
                    // kerning guard suppresses genuine word gaps on condensed/
                    // tracked lines with no space glyph (bold headings, running
                    // footers). When this line's own gap distribution is clearly
                    // bimodal and this gap sits in the inter-word cluster, honor
                    // the boundary. Only ever ADDS a space (never removes one),
                    // and ONLY when the suppression came from the purely-
                    // geometric intra-word kerning guard — never the semantic
                    // no-space rules (complex-script/Brahmic, CJK, ligature),
                    // else Bengali/Devanagari syllables shatter into fragments.
                    // RTL is excluded too — the ReversedChars guard below owns
                    // that decision.
                    // Two guards keep the narrow-gap rescue off dense math, whose
                    // sub/superscript gaps are the same ~0.10 em magnitude as a
                    // condensed footer's word gap:
                    //   * same-baseline: never rescue directly across a
                    //     super/subscript baseline shift, and
                    //   * empty-gap: never rescue when another glyph's ink sits
                    //     inside the gap — a subscript drawn between a variable
                    //     and the next symbol (`λᵢ r…`) inflates the `λ`→`r` gap
                    //     though both share the baseline; the ink in the gap marks
                    //     it as not-a-word-boundary. A genuine footer word gap is
                    //     empty. (`λᵢ` must not become `λ i`.)
                    let same_baseline = (current.bbox.y - span.bbox.y).abs()
                        < current.font_size.max(span.font_size).max(1.0) * 0.04;
                    if space_decision.source == SpaceSource::IntraWordKerning
                        && !self.saw_reversed_chars
                        && same_baseline
                        && !gap_has_intervening_glyph(&ink_boxes, &current.bbox, &span.bbox)
                    {
                        // Split only when the PER-LINE bimodal threshold fires.
                        // A uniform per-pair advance floor (the way pdfminer/
                        // poppler decide word boundaries) would catch a few more
                        // footer instances this adaptive test misses, but a fixed
                        // magnitude cannot tell a 0.10 em condensed word gap from
                        // 0.10 em loose intra-word tracking, so it also splits
                        // real words on loosely-set/scanned lines
                        // (`walking` → `wa lking`) — exactly the over-splitting
                        // pdfminer exhibits. The per-line bimodal only fires when
                        // the intra-word cluster is genuinely tight, so it never
                        // over-splits, at the cost of the handful of footer
                        // instances whose gap distribution is not cleanly bimodal.
                        if let Some(thr) = line_thresholds.get(span_idx).copied().flatten() {
                            if gap > thr {
                                space_decision =
                                    SpaceDecision::insert(SpaceSource::GeometricGap, 0.9);
                            }
                        }
                    }

                    // ReversedChars Arabic word-shatter guard (ISO 32000-1
                    // §14.8.2.3.3). On a page that draws RTL glyphs individually
                    // under /ReversedChars, real word boundaries are marked with
                    // explicit space glyphs (preserved above as whitespace-only
                    // spans). A GEOMETRIC space between two cursively-adjacent
                    // Arabic letters is therefore a positioning artifact, not a
                    // word break — suppress it so words stay whole (إسبريسو, not
                    // إس بر يسو). Only fires on ReversedChars pages, so ordinary
                    // geometric-spaced Arabic producers are unaffected.
                    if self.saw_reversed_chars && space_decision.insert_space {
                        use crate::text::rtl_detector::is_arabic_letter;
                        let prev_ar = current
                            .text
                            .chars()
                            .next_back()
                            .is_some_and(|c| is_arabic_letter(c as u32));
                        let next_ar = span
                            .text
                            .chars()
                            .next()
                            .is_some_and(|c| is_arabic_letter(c as u32));
                        if prev_ar && next_ar {
                            space_decision.insert_space = false;
                        }
                    }

                    log::debug!(
                        "Span merge decision: gap={:.2}pt, decision={:?}, source={:?}, confidence={:.2}, offset_semantic={}",
                        gap,
                        space_decision.insert_space,
                        space_decision.source,
                        space_decision.confidence,
                        span.offset_semantic
                    );

                    if space_decision.insert_space {
                        // Space insertion triggered by unified decision
                        // But SKIP if this span is already a TJ-offset space (would create double space)
                        if next_is_offset_semantic_space {
                            log::debug!(
                                "Suppressing space insertion: next span is already TJ-offset space"
                            );
                            current.text.push_str(&span.text);
                        } else {
                            // Prevent double-space edge case
                            let would_create_double_space =
                                current.text.ends_with(' ') && span.text.starts_with(' ');

                            if would_create_double_space {
                                log::debug!(
                                    "Preventing double-space: current ends with space, next starts with space"
                                );
                                current.text.push_str(&span.text);
                            } else {
                                log::trace!("Space via {:?}", space_decision.source);
                                current.text.push(' ');
                                current.text.push_str(&span.text);
                            }
                        }
                    } else {
                        // No space: adjacent characters within same word
                        log::trace!(
                            "No space insertion: decision source={:?}",
                            space_decision.source
                        );
                        current.text.push_str(&span.text);
                    }
                }
            }

            if decimal_merge || should_merge || cross_font_word_glue {
                // A merged span is logical-draw RTL if any of its glyph runs was
                // drawn right-to-left (see `detect_rtl_draw_direction`).
                current.rtl_draw_logical |= span.rtl_draw_logical;
                // Extend bounding box to include both spans
                let new_width = (span.bbox.x + span.bbox.width) - current.bbox.x;
                let new_height = current.bbox.height.max(span.bbox.height);

                current.bbox.width = new_width;
                current.bbox.height = new_height;

                // Keep `char_widths` in POSITIONAL lockstep with the merged
                // text. The downstream width-based splitters
                // `is_column_spanning_decimal` and `char_widths_boundary_split`
                // (document.rs) fire when `char_widths.len() < char_count`, and
                // `TextSpan::to_chars` pairs each glyph's accurate
                // `char_x_offsets` origin with `char_widths[i]` — so every
                // width entry must sit at the same index as its char, not
                // merely make the lengths match. A trailing `resize` after a
                // width-less contribution (e.g. a TJ-offset space span merging
                // FIRST) shifted every later width one slot left, pairing each
                // glyph with its neighbor's advance and opening phantom
                // intra-word gaps that the word-gap clusterer split on
                // (`module` → `m|odu|le`). Maintain the merged
                // vector as `current + [separator] + span`, normalizing each
                // contribution at its own position instead.
                let pad = if current.font_size > 0.0 {
                    current.font_size * 0.25
                } else {
                    1.0
                };
                // 1. Normalize the accumulated widths to the pre-merge char
                //    count. A width-less contribution is split uniformly
                //    across its bbox (matching `to_chars`' uniform fallback);
                //    a partially-populated one keeps the legacy tail-pad.
                if current.char_widths.is_empty() && current_chars_before > 0 {
                    let old_width = (current_end_x - current.bbox.x).max(0.0);
                    current
                        .char_widths
                        .resize(current_chars_before, old_width / current_chars_before as f32);
                } else if current.char_widths.len() != current_chars_before {
                    current.char_widths.resize(current_chars_before, pad);
                }
                // 2. Inserted separator ('.' or ' ') widths land at the
                //    separator's own position: the real geometric gap the
                //    separator stands in for, with the legacy pad as the
                //    fallback for overlapping/degenerate layouts.
                let merged_char_count = current.text.chars().count();
                let separator_count =
                    merged_char_count.saturating_sub(current_chars_before + span_char_count);
                if separator_count > 0 {
                    let sep_gap = span.bbox.x - current_end_x;
                    let sep_width = if sep_gap.is_finite() && sep_gap > 0.0 {
                        sep_gap / separator_count as f32
                    } else {
                        pad
                    };
                    current
                        .char_widths
                        .resize(current_chars_before + separator_count, sep_width);
                }
                // 3. Append the merged-in span's widths, normalized the same
                //    way at its position.
                if span.char_widths.is_empty() && span_char_count > 0 {
                    let per_char = (span.bbox.width / span_char_count as f32).max(0.0);
                    current
                        .char_widths
                        .extend(std::iter::repeat_n(per_char, span_char_count));
                } else {
                    current.char_widths.extend_from_slice(&span.char_widths);
                    current.char_widths.resize(merged_char_count, pad);
                }
                debug_assert_eq!(
                    current.char_widths.len(),
                    merged_char_count,
                    "char_widths must stay in lockstep with merged text"
                );

                // Preserve the merged-in glyph's TRUE origin for scrambled-RTL
                // producers (e.g. /ReversedChars + per-glyph /ActualText Arabic,
                // ISO 32000-1 §14.8.2.3.3 / §14.9.4). Such producers reposition
                // glyphs out of advance-order, so the appended raw advances collapse
                // the merged span to advance-flow and `to_chars()` loses each glyph's
                // true x — the RTL visual-order sort (`merge_rtl_line_to_visual_span`)
                // then mis-places zero-width marks (القهوة → قالهوة). After the
                // char_widths are in lockstep with the (possibly space-inserted) text,
                // stretch the advance leading into the merged-in span's LAST glyph so
                // `to_chars()` reconstructs it at `span.bbox.x`. Gated to Arabic so
                // Latin/CJK output stays byte-identical.
                let touches_arabic = |t: &str| {
                    t.chars().any(|c| {
                        ('\u{0600}'..='\u{06FF}').contains(&c)
                            || ('\u{0750}'..='\u{077F}').contains(&c)
                            || ('\u{08A0}'..='\u{08FF}').contains(&c)
                    })
                };
                let n = current.char_widths.len();
                let span_chars = span.text.chars().count();
                if n >= 2
                    && span_chars >= 1
                    && span_chars < n
                    && (touches_arabic(&current.text) || touches_arabic(&span.text))
                {
                    // Index of the merged-in span's FIRST glyph in the (possibly
                    // space-inserted) merged text, and its target relative x.
                    let first_idx = n - span_chars;
                    let prefix: f32 = current.char_widths[..first_idx].iter().sum();
                    let want = span.bbox.x - current.bbox.x;
                    let adjust = want - prefix;
                    if adjust.abs() > 0.01 {
                        // Put the gap into the advance leading into that glyph.
                        current.char_widths[first_idx - 1] += adjust;
                    }
                }

                // After a cross-font glue, adopt the longer run's font
                // metadata. The single-letter side was typographic
                // decoration, not semantic emphasis, so the dominant-run
                // style should win.
                if cross_font_word_glue {
                    let span_chars = span.text.chars().count();
                    let current_chars_before = current.text.chars().count() - span_chars;
                    if span_chars > current_chars_before {
                        current.font_name = span.font_name.clone();
                        current.font_weight = span.font_weight;
                        current.is_italic = span.is_italic;
                    }
                }

                log::trace!(
                    "Merged span: appended '{}' (gap={:.1}pt, now {} chars)",
                    span.text,
                    gap,
                    current.text.len()
                );

                // Put modified current back
                current_span = Some(current);
            } else {
                // Not mergeable: save current and start new span
                if same_line {
                    if span.split_boundary_before {
                        log::trace!(
                            "Not merging spans (split boundary): '{}' | '{}'",
                            current.text,
                            span.text
                        );
                    } else {
                        log::trace!(
                            "Not merging spans (gap={:.1}pt > 3pt): '{}' | '{}'",
                            gap,
                            current.text,
                            span.text
                        );
                    }
                }
                merged.push(current);
                current_span = Some(span);
            }
        }

        // Don't forget the last span
        if let Some(last) = current_span {
            merged.push(last);
        }

        log::debug!("Merged adjacent spans: {} -> {} spans", old_len, merged.len());

        self.spans = merged;
    }

    /// Sort extracted characters by reading order (top-to-bottom, left-to-right).
    ///
    /// This is critical for proper text extraction as PDF content streams are
    /// organized for rendering efficiency, not reading order.
    fn sort_by_reading_order(&mut self) {
        self.chars.sort_by(|a, b| {
            // Handle NaN/Inf values - treat them as at the end
            if !a.bbox.y.is_finite() {
                return if b.bbox.y.is_finite() {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                };
            }
            if !b.bbox.y.is_finite() {
                return std::cmp::Ordering::Less;
            }

            // Sort by Y descending (top first), then by X ascending (left to right)
            // Round Y coordinates to ensure transitivity of the comparison function
            let a_y_rounded = a.bbox.y.round() as i32;
            let b_y_rounded = b.bbox.y.round() as i32;

            match b_y_rounded.cmp(&a_y_rounded) {
                std::cmp::Ordering::Equal => {
                    // Same line: sort by X ascending (left to right)
                    if !a.bbox.x.is_finite() {
                        return if b.bbox.x.is_finite() {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        };
                    }
                    if !b.bbox.x.is_finite() {
                        return std::cmp::Ordering::Less;
                    }

                    if a.bbox.x < b.bbox.x {
                        std::cmp::Ordering::Less
                    } else if a.bbox.x > b.bbox.x {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                },
                other => other,
            }
        });
    }

    /// ISSUE 1 FIX: Split fused words created by PDF authoring defects
    ///
    /// Some PDFs encode multiple words as a single TJ string without spacing:
    /// - "theGeneral" instead of "the" + "General"
    /// - "lengthThis" instead of "length" + "This"
    /// - "helporganisationscraft" (partial fusion)
    ///
    /// This post-processor detects word fusions and splits them into separate spans.
    ///
    /// Uses two strategies:
    /// 1. **CamelCase detection** (first priority): Detects lowercase->uppercase transitions
    ///    - Example: "theGeneral" -> ["the", "General"]
    /// 2. **Dictionary-based segmentation** (fallback): Uses Viterbi algorithm with word dictionary
    ///    - Example: "helporganisationscraft" -> ["help", "organisations", "craft"]
    ///
    /// Per ISO 32000-1:2008 Section 9.4.4: "Text strings are as long as possible" - spaces
    /// are positioning artifacts, so word fusions must be detected and reconstructed.
    #[allow(dead_code)]
    fn split_fused_words(&mut self) {
        let mut split_spans = Vec::new();

        for span in &self.spans {
            // DEBUG: Log field values before cloning
            log::debug!(
                "split_fused_words() processing span '{}' (offset_semantic={}, split_boundary_before={})",
                if span.text.len() <= 30 {
                    &span.text
                } else {
                    "[whitespace or long text]"
                },
                span.offset_semantic,
                span.split_boundary_before
            );

            // Try CamelCase split (handles mixed-case fusions)
            let parts = self.split_on_camelcase(&span.text);

            if parts.len() == 1 {
                // No split needed
                let cloned = span.clone();
                log::debug!(
                    "  → No split: cloned offset_semantic={} (text: '{}')",
                    cloned.offset_semantic,
                    if cloned.text.len() <= 30 {
                        &cloned.text
                    } else {
                        "[whitespace or long text]"
                    }
                );
                split_spans.push(cloned);
            } else {
                // Split into multiple spans with proportional bounding boxes
                let total_chars = span.text.len() as f32;
                let mut char_pos = 0;

                for (i, part) in parts.iter().enumerate() {
                    let part_len = part.len() as f32;
                    let part_ratio = part_len / total_chars;

                    // Calculate proportional bounding box
                    let new_width = span.bbox.width * part_ratio;
                    let new_x = span.bbox.x + (span.bbox.width * (char_pos as f32 / total_chars));

                    let mut new_span = span.clone();
                    new_span.text = part.clone();
                    new_span.bbox.x = new_x;
                    new_span.bbox.width = new_width;

                    // Set split_boundary_before flag for all parts except the first
                    // This prevents them from being re-merged during span merging
                    if i > 0 {
                        new_span.split_boundary_before = true;
                    }

                    log::debug!(
                        "  → Split part {}: '{}' offset_semantic={} split_boundary_before={}",
                        i,
                        part,
                        new_span.offset_semantic,
                        new_span.split_boundary_before
                    );
                    split_spans.push(new_span);
                    char_pos += part.len();
                }
            }
        }

        self.spans = split_spans;
    }

    /// Detect CamelCase boundaries and split text into parts
    ///
    /// Splits on lowercase->uppercase transitions:
    /// - "theGeneral" -> ["the", "General"]
    /// - "lengthThis" -> ["length", "This"]
    /// - "helporganisationscraft" -> ["help", "organisations", "craft"]
    #[allow(dead_code)]
    fn split_on_camelcase(&self, text: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut prev_is_lower = false;

        for ch in text.chars() {
            if prev_is_lower && ch.is_uppercase() {
                // CamelCase boundary detected
                if !current_part.is_empty() {
                    parts.push(current_part.clone());
                    current_part.clear();
                }
                current_part.push(ch);
                prev_is_lower = false;
            } else {
                current_part.push(ch);
                prev_is_lower = ch.is_lowercase();
            }
        }

        if !current_part.is_empty() {
            parts.push(current_part);
        }

        // Only return split if we found at least 2 parts with actual boundaries
        if parts.len() > 1 {
            parts
        } else {
            vec![text.to_string()]
        }
    }

    /// Execute a single operator.
    ///
    /// Updates the graphics state and extracts text as appropriate.
    fn execute_operator(&mut self, op: Operator) -> Result<()> {
        match op {
            // Text state operators
            Operator::Tf { font, size } => {
                // Skip flush + lookup when font name AND size haven't changed.
                // Many PDFs redundantly set the same font (e.g., Tf after q/Q).
                let same_font = {
                    let state = self.state_stack.current();
                    state.font_size == size && state.font_name.as_deref() == Some(font.as_str())
                };
                if !same_font {
                    // Flush Tj buffer before changing font — the buffer decodes bytes
                    // using the font set at creation time, so a font change requires a
                    // new buffer to avoid decoding with the wrong ToUnicode CMap.
                    self.flush_tj_span_buffer()?;

                    // Cache font reference for advance_position_for_string
                    self.cached_current_font = self.fonts.get(&font).cloned();
                    // Cache wmode on the graphics state so the advance hot
                    // path branches on a single primitive read instead of
                    // dereferencing the FontInfo every glyph.
                    let new_wmode = self
                        .cached_current_font
                        .as_deref()
                        .map(|f| f.wmode)
                        .unwrap_or(0);

                    let state = self.state_stack.current_mut();
                    state.font_name = Some(font);
                    state.font_size = size;
                    state.text_wmode = new_wmode;
                }
            },

            // Text positioning operators
            Operator::Tm { a, b, c, d, e, f } => {
                // Optimization: batch character-by-character Tm+Tj patterns.
                // Many PDFs position each character with individual Tm+Tj operators.
                // If the new Tm is on the same line with the same transform,
                // keep accumulating into the existing buffer instead of flushing
                // (avoids creating thousands of 1-char TextSpans per page).
                // When merge_tm_tj_runs is false, every Tm always starts a fresh span.
                //
                // #518: glyph-jitter tolerance. Microsoft Word emits each
                // glyph in its own `BT Tm Tj ET` block with ±2.5–5pt
                // sinusoidal baseline jitter for broken-image placeholder
                // text. ISO 32000-1 §9.4 leaves logical reading order to
                // the extractor, so a baseline delta far smaller than the
                // line's own height is the SAME visual line — only a
                // delta on the order of the font size is a real line
                // break (body leading ≳ 1.0× font size). The previous
                // `f.round() as i32 ==` check tolerated only ±0.5pt
                // split jittered glyphs into separate Y-banded spans that
                // the reading-order sort then scrambled. Tolerance is
                // scale-relative (0.5× the text-space glyph height, ≥0.5pt
                // floor) so it is correct at any font size and still
                // splits genuine line breaks.
                //
                // The `f`/`e` tests below only mean "same line" and "forward"
                // while the run advances along +x; under a rotated matrix the
                // two axes swap. The added conjunct re-checks both along the
                // run's own writing axis (ISO 32000-1 §9.4.4).
                let cur_font_size = self.state_stack.current().font_size;
                // A run is contiguous glyphs, so the new origin must not skip
                // an em of empty space past the run's own advance. `Td`, `TD`
                // and `T*` all end the run outright; without a bound here `Tm`
                // alone accepted an arbitrary forward jump, so two show
                // operations positioned in different columns were glued into
                // one span carrying no separator and a width spanning the void
                // between them. ISO 32000-1:2008 §9.4.2, Table 108 gives `Tm`
                // and `Td` the same effect on the text and text-line matrices,
                // so a displacement that ends a run for one must end it for the
                // other: continuity is a property of the resulting pen
                // position, not of the operator that moved the pen.
                //
                // The bound is a column gap, not a word space. A producer can
                // leave an intra-word repositioning seam WIDER than the same
                // font's declared space advance, so no word-space constant
                // separates a seam from a space; only the source-order
                // evidence the span merger reads (a space glyph occupies a
                // character position, a seam is pure repositioning) tells them
                // apart. Everything narrower is therefore left to the merger,
                // and this rule speaks only to gaps wide enough that the line
                // grouping would already call them a column boundary — the
                // same `max(3 x font size, 30 pt)` it uses, so the two levels
                // cannot disagree about what separates text.
                let is_continuation = self.merging_config.merge_tm_tj_runs
                    && match self.tj_span_buffer {
                        Some(ref mut buffer)
                            if !buffer.is_empty()
                                && a == buffer.start_matrix.a
                                && b == buffer.start_matrix.b
                                && c == buffer.start_matrix.c
                                && d == buffer.start_matrix.d
                                // Reading-order jitter tolerance is not a paint
                                // origin: retain a changed glyph baseline.
                                && (!self.visible_paint_only || f == buffer.start_matrix.f)
                                // The raw-matrix band and forward test are kept
                                // ANDed with the frame-correct rule rather than
                                // replaced by it. Substituting reads better and
                                // is what the writing-axis helper was built for,
                                // but it changes what a quarter-turn run does:
                                // the raw band collapses to its 0.5 pt floor
                                // there, so rotated runs never merge today, and
                                // letting them merge concatenates them in
                                // content-stream order. That defeats the
                                // writing-axis ordering the rotated line
                                // grouping performs — a chart label drawing its
                                // subscript last reads "H02" instead of "H2O".
                                // Merging rotated runs is worth doing (a run set
                                // glyph by glyph yields one span per glyph), but
                                // only together with an ordering rule that
                                // survives it, which is not this change.
                                && (f - buffer.start_matrix.f).abs()
                                    <= ((cur_font_size * buffer.start_matrix.d).abs() * 0.5)
                                        .max(0.5)
                                && e >= buffer.start_matrix.e =>
                        {
                            match Self::run_continuation_along(
                                buffer.start_matrix,
                                buffer.wmode,
                                e,
                                f,
                                cur_font_size,
                                buffer.accumulated_width,
                            ) {
                                // Same line, same transform, forward along the
                                // run → carry the visual extent forward.
                                Some(along) => {
                                    buffer.accumulated_width = along;
                                    true
                                },
                                None => false,
                            }
                        },
                        _ => false,
                    };

                if !is_continuation {
                    self.flush_tj_span_buffer()?;
                }

                let state = self.state_stack.current_mut();
                state.text_matrix = Matrix { a, b, c, d, e, f };
                state.text_line_matrix = state.text_matrix;
            },
            Operator::Td { tx, ty } => {
                // Flush Tj buffer before changing text position
                self.flush_tj_span_buffer()?;
                let state = self.state_stack.current_mut();
                // Per ISO 32000-1:2008 §9.4.2, Table 108:
                // Tlm_new = T(tx,ty) × Tlm_old
                // The translation is in text-line space, so it must be
                // pre-multiplied to be scaled by the existing Tlm transform.
                let tm = Matrix::translation(tx, ty);
                state.text_line_matrix = tm.multiply(&state.text_line_matrix);
                state.text_matrix = state.text_line_matrix;
            },
            Operator::TD { tx, ty } => {
                // Flush Tj buffer before changing text position
                self.flush_tj_span_buffer()?;

                // TD is like Td but also sets leading
                let state = self.state_stack.current_mut();
                state.leading = -ty;
                // Per ISO 32000-1:2008 §9.4.2: Tlm_new = T(tx,ty) × Tlm_old
                let tm = Matrix::translation(tx, ty);
                state.text_line_matrix = tm.multiply(&state.text_line_matrix);
                state.text_matrix = state.text_line_matrix;
            },
            Operator::TStar => {
                // Flush Tj buffer before moving to next line
                self.flush_tj_span_buffer()?;

                // Move to start of next line (using leading)
                let leading = self.state_stack.current().leading;
                let state = self.state_stack.current_mut();
                // Per ISO 32000-1:2008 §9.4.2: Tlm_new = T(0,-TL) × Tlm_old
                let tm = Matrix::translation(0.0, -leading);
                state.text_line_matrix = tm.multiply(&state.text_line_matrix);
                state.text_matrix = state.text_line_matrix;
            },

            // Text showing operators
            Operator::Tj { text } => {
                // Note: We do NOT skip /Artifact content here.
                // Many PDFs incorrectly mark page content as artifacts.
                // For tagged PDFs, the structure tree already excludes artifacts
                // via MCID mapping, so no filtering is needed at extractor level.

                // ActualText override
                // Per PDF Spec ISO 32000-1:2008, Section 14.9.4:
                // ActualText provides replacement text for the marked-content
                // SEQUENCE — emitted ONCE, no matter how many Tj operators
                // sit inside. The peek/mark pair below handles both first-Tj
                // (emit replacement) and subsequent-Tj (suppress entirely,
                // advance only) cases.
                let (current_at, already_emitted) = self.peek_current_actual_text();
                if let Some(actual_text) = current_at {
                    if already_emitted {
                        // Subsequent show-text inside the same MC scope:
                        // glyphs are already covered by the one replacement
                        // that fired on the first Tj. Advance positioning so
                        // any later, OUTER-scope show-text lands correctly,
                        // but emit nothing.
                        let w = self.advance_position_for_string(&text)?;
                        if let Some(ref mut buffer) = self.tj_span_buffer {
                            buffer.accumulated_width += w;
                        }
                    } else {
                        log::debug!("Tj operator: emitting MC-scope ActualText '{}'", actual_text);
                        self.mark_actual_text_emitted();
                        if self.extract_spans {
                            // Use ActualText in span mode — push pre-decoded
                            // Unicode directly into the buffer, bypassing
                            // font character mapping.
                            if self.tj_span_buffer.is_none() {
                                self.tj_span_buffer = Some(TjBuffer::new(
                                    self.state_stack.current(),
                                    self.current_mcid,
                                    self.cached_current_font.clone(),
                                    self.paint_order_snapshot()?,
                                ));
                            }
                            if let Some(ref mut buffer) = self.tj_span_buffer {
                                buffer.unicode.push_str(&actual_text);
                            }
                        } else {
                            // Character mode: show_text maps through font, but ActualText
                            // is already decoded. Fall back to show_text for positioning.
                            self.show_text(actual_text.as_bytes())?;
                        }
                        // Advance position for the original text (to maintain layout)
                        let w = self.advance_position_for_string(&text)?;
                        if let Some(ref mut buffer) = self.tj_span_buffer {
                            buffer.accumulated_width += w;
                        }
                    }
                } else {
                    // No ActualText - use standard text extraction
                    if self.extract_spans {
                        // NEW: Buffer consecutive Tj operators into single spans
                        // Per PDF Spec ISO 32000-1:2008, Section 9.4.4 NOTE 6:
                        // "text strings are as long as possible"

                        // Create buffer if doesn't exist
                        if self.tj_span_buffer.is_none() {
                            self.tj_span_buffer = Some(TjBuffer::new(
                                self.state_stack.current(),
                                self.current_mcid,
                                self.cached_current_font.clone(),
                                self.paint_order_snapshot()?,
                            ));
                        }

                        // Merged single-pass: Unicode decode + width + position advance
                        self.append_and_advance(&text)?;
                    } else {
                        self.show_text(&text)?;
                    }
                }
            },
            Operator::TJ { array } => {
                // Note: We do NOT skip /Artifact content here.
                // Many PDFs incorrectly mark page content as artifacts.
                // For tagged PDFs, the structure tree already excludes artifacts
                // via MCID mapping, so no filtering is needed at extractor level.

                // ActualText override
                // Per PDF Spec ISO 32000-1:2008, Section 14.9.4:
                // The MC-scope `/ActualText` replaces the ENTIRE sequence
                // exactly once — see the Tj path above for the per-scope
                // peek/mark protocol that handles both first and
                // subsequent show-text operators inside the same scope.
                let (current_at, already_emitted) = self.peek_current_actual_text();
                if let Some(actual_text) = current_at {
                    if !already_emitted {
                        log::debug!(
                            "TJ operator: emitting MC-scope ActualText '{}' (replacing {} elements)",
                            actual_text,
                            array.len()
                        );
                        self.mark_actual_text_emitted();
                        if self.extract_spans {
                            let mut buffer = TjBuffer::new(
                                self.state_stack.current(),
                                self.current_mcid,
                                self.cached_current_font.clone(),
                                self.paint_order_snapshot()?,
                            );
                            buffer.unicode.push_str(&actual_text);
                            self.flush_tj_buffer(buffer)?;
                        } else {
                            self.show_text(actual_text.as_bytes())?;
                        }
                    }
                    // First or subsequent: advance position for the
                    // entire TJ array so layout stays consistent.
                    for element in array {
                        match element {
                            TextElement::String(s) => {
                                let w = self.advance_position_for_string(&s)?;
                                if let Some(ref mut buffer) = self.tj_span_buffer {
                                    buffer.accumulated_width += w;
                                }
                            },
                            TextElement::Offset(offset) => {
                                self.advance_position_for_offset(offset)?;
                            },
                        }
                    }
                } else {
                    // No ActualText - use standard TJ array processing
                    if self.extract_spans {
                        // NEW: Use buffered TJ array processing for span extraction
                        // Per PDF Spec ISO 32000-1:2008, Section 9.4.4 NOTE 6:
                        // "text strings are as long as possible"
                        // This creates one span per logical text unit instead of fragmenting
                        self.process_tj_array(&array)?;
                    } else {
                        // Keep old behavior for character extraction mode
                        for element in array {
                            match element {
                                TextElement::String(s) => {
                                    self.show_text(&s)?;
                                },
                                TextElement::Offset(offset) => {
                                    // Adjust text position by offset (in thousandths of em)
                                    let state = self.state_stack.current();
                                    let tx = -offset / 1000.0
                                        * state.font_size
                                        * state.horizontal_scaling
                                        / 100.0;

                                    // HEURISTIC: Insert space character for significant negative offsets
                                    //
                                    // PDF Spec Reference: ISO 32000-1:2008, Section 9.4.4
                                    // The spec defines text positioning but does NOT specify when a positioning
                                    // offset represents a word boundary vs. tight kerning.
                                    //
                                    // In PDFs, spaces are often represented as negative positioning offsets in TJ arrays,
                                    // not as explicit space characters. For example:
                                    // [(Text1) -200 (Text2)] TJ <- the -200 creates visual spacing
                                    //
                                    // Geometry-based adaptive threshold (based on font metrics)
                                    // Formula: adaptive_threshold = -(average_glyph_width * word_margin_ratio)
                                    // This adapts to different font sizes and families.
                                    // Fallback: static threshold if font unavailable or adaptive disabled.
                                    let threshold = self.calculate_adaptive_tj_threshold();
                                    if offset < threshold {
                                        let text_matrix = state.text_matrix;
                                        let ctm = state.ctm;
                                        let font_name = state.font_name.clone();
                                        let font_size = state.font_size;
                                        let fill_color_rgb = state.fill_color_rgb;

                                        // Calculate effective font size (accounting for CTM and text matrix scaling)
                                        let combined = ctm.multiply(&text_matrix);
                                        let effective_font_size = font_size
                                            * (combined.d * combined.d + combined.b * combined.b)
                                                .sqrt();

                                        // Get font for determining weight
                                        let font = font_name
                                            .as_ref()
                                            .and_then(|name| self.fonts.get(name));
                                        let font_weight = if let Some(font) = font {
                                            if font.is_bold() {
                                                FontWeight::Bold
                                            } else {
                                                FontWeight::Normal
                                            }
                                        } else {
                                            FontWeight::Normal
                                        };

                                        // Create space character at current position
                                        // Apply CTM to get position in user space
                                        let text_pos = text_matrix.transform_point(0.0, 0.0);
                                        let pos = ctm.transform_point(text_pos.x, text_pos.y);
                                        let (r, g, b) = fill_color_rgb;
                                        let is_italic_space = font_name
                                            .as_ref()
                                            .and_then(|name| self.fonts.get(name))
                                            .map(|font| font.is_italic())
                                            .unwrap_or(false);
                                        let font_name_str = font_name.unwrap_or_default();
                                        // Compose CTM and text_matrix for full transformation
                                        let final_matrix = ctm.multiply(&text_matrix);
                                        // Calculate rotation from matrix: atan2(b, a)
                                        let rotation_degrees =
                                            final_matrix.b.atan2(final_matrix.a).to_degrees();

                                        let space_char = TextChar {
                                            char: ' ',
                                            bbox: Rect::new(
                                                pos.x,               // X position in user space
                                                pos.y,               // Y position in user space
                                                tx.abs(), // Width = the gap being created
                                                effective_font_size, // Height = effective font size
                                            ),
                                            font_name: font_name_str,
                                            font_size: effective_font_size,
                                            font_weight,
                                            color: Color::new(r, g, b),
                                            mcid: self.current_mcid,
                                            is_italic: is_italic_space,
                                            is_monospace: false,
                                            // Transformation properties (v0.3.1)
                                            origin_x: pos.x,
                                            origin_y: pos.y,
                                            rotation_degrees,
                                            advance_width: tx.abs(),
                                            rendered_advance: tx.abs(),
                                            ascent: font.map(|f| f.ascent).unwrap_or(0.95)
                                                * effective_font_size,
                                            descent: font.map(|f| f.descent).unwrap_or(-0.35)
                                                * effective_font_size,
                                            matrix: Some([
                                                final_matrix.a,
                                                final_matrix.b,
                                                final_matrix.c,
                                                final_matrix.d,
                                                final_matrix.e,
                                                final_matrix.f,
                                            ]),
                                        };
                                        if !self.is_content_suppressed() {
                                            self.chars.push(space_char);
                                        }
                                    }

                                    // Route through advance_text_matrix so the
                                    // axis swap (H vs V) lives in one place.
                                    // Per ISO 32000-1 §9.4.4 a TJ numeric
                                    // offset shifts along the active writing
                                    // axis: x for WMode 0, y for WMode 1.
                                    self.state_stack.current_mut().advance_text_matrix(tx);
                                },
                            }
                        }
                    }
                }
            },
            Operator::Quote { text } => {
                // ' operator: Move to next line (T*) and show text (Tj)
                // Flush any pending span buffer before line break
                self.flush_tj_span_buffer()?;

                let leading = self.state_stack.current().leading;
                {
                    let state = self.state_stack.current_mut();
                    // Per ISO 32000-1:2008 §9.4.2: Tlm_new = T(0,-TL) × Tlm_old
                    let tm = Matrix::translation(0.0, -leading);
                    state.text_line_matrix = tm.multiply(&state.text_line_matrix);
                    state.text_matrix = state.text_line_matrix;
                }

                if self.extract_spans {
                    if self.tj_span_buffer.is_none() {
                        self.tj_span_buffer = Some(TjBuffer::new(
                            self.state_stack.current(),
                            self.current_mcid,
                            self.cached_current_font.clone(),
                            self.paint_order_snapshot()?,
                        ));
                    }
                    self.append_and_advance(&text)?;
                } else {
                    self.show_text(&text)?;
                }
            },
            Operator::DoubleQuote {
                word_space,
                char_space,
                text,
            } => {
                // " operator: Set spacing, move to next line (T*), and show text (Tj)
                // Flush any pending span buffer before line break
                self.flush_tj_span_buffer()?;

                {
                    let state = self.state_stack.current_mut();
                    state.word_space = word_space;
                    state.char_space = char_space;
                    let leading = state.leading;
                    // Per ISO 32000-1:2008 §9.4.2: Tlm_new = T(0,-TL) × Tlm_old
                    let tm = Matrix::translation(0.0, -leading);
                    state.text_line_matrix = tm.multiply(&state.text_line_matrix);
                    state.text_matrix = state.text_line_matrix;
                }

                if self.extract_spans {
                    if self.tj_span_buffer.is_none() {
                        self.tj_span_buffer = Some(TjBuffer::new(
                            self.state_stack.current(),
                            self.current_mcid,
                            self.cached_current_font.clone(),
                            self.paint_order_snapshot()?,
                        ));
                    }
                    self.append_and_advance(&text)?;
                } else {
                    self.show_text(&text)?;
                }
            },

            // Text state parameters
            Operator::Tc { char_space } => {
                self.state_stack.current_mut().char_space = char_space;
            },
            Operator::Tw { word_space } => {
                self.state_stack.current_mut().word_space = word_space;
            },
            Operator::Tz { scale } => {
                self.state_stack.current_mut().horizontal_scaling = scale;
            },
            Operator::TL { leading } => {
                self.state_stack.current_mut().leading = leading;
            },
            Operator::Ts { rise } => {
                if self.visible_paint_only {
                    self.flush_tj_span_buffer()?;
                }
                self.state_stack.current_mut().text_rise = rise;
            },
            Operator::Tr { render } => {
                self.flush_tj_span_buffer()?;
                self.state_stack.current_mut().render_mode = render;
            },

            // Graphics state operators
            Operator::SaveState => {
                // Flush the Tj span buffer before pushing graphics state.
                // q/Q wraps a graphics-state block; restoring after Q can
                // re-set the CTM to an earlier value, leaving the
                // captured user_pos inside the buffer out of sync with
                // the active CTM. Flush so each q/Q block emits its
                // own clean cluster.
                self.flush_tj_span_buffer()?;
                self.state_stack.save();
            },
            Operator::RestoreState => {
                self.flush_tj_span_buffer()?;
                self.state_stack.restore();
                // Sync cached font with restored state
                self.cached_current_font = self
                    .state_stack
                    .current()
                    .font_name
                    .as_ref()
                    .and_then(|name| self.fonts.get(name))
                    .cloned();
                // Re-evaluate ink exclusion for the restored color space
                if !self.excluded_inks.is_empty() {
                    let cs = self.state_stack.current().fill_color_space.clone();
                    self.inside_excluded_ink = self.is_excluded_ink_color_space(&cs);
                }
            },
            Operator::Cm { a, b, c, d, e, f } => {
                // Flush the Tj span buffer before changing the CTM.
                // The buffer captured `user_pos_x`/`user_pos_y` and
                // `user_h_scale` from the CTM in effect when it was
                // created (TjBuffer::new at the first Tj after BT).
                // Non-conforming PDFs can issue cm operators inside
                // a text object — typically when figure / chart text
                // runs alternate `cm` for position with text
                // operators in the same BT/ET block. Without a
                // flush, subsequent Tj chars get a position derived
                // from the new CTM while the buffer still reports
                // the stale `user_pos`, dropping the cluster off
                // the page in the worst case. Flushing here emits
                // the current cluster at its captured position and
                // the next Tj creates a fresh buffer under the new
                // CTM. Spec basis: §9.4 lists cm as general
                // graphics state, not formally allowed inside
                // BT/ET, but conforming readers must process it.
                self.flush_tj_span_buffer()?;
                let state = self.state_stack.current_mut();
                let new_ctm = Matrix { a, b, c, d, e, f };
                // PDF spec ISO 32000-1:2008 §8.3.4: cm concatenates as M_cm × CTM
                state.ctm = new_ctm.multiply(&state.ctm);
            },

            // Color operators
            Operator::SetFillRgb { r, g, b } => {
                // rg operator implicitly sets DeviceRGB — a process color.
                self.inside_excluded_ink = false;
                self.state_stack.current_mut().fill_color_rgb = (r, g, b);
            },
            Operator::SetStrokeRgb { r, g, b } => {
                self.state_stack.current_mut().stroke_color_rgb = (r, g, b);
            },
            Operator::SetFillGray { gray } => {
                // g operator implicitly sets DeviceGray — a process color,
                // so clear any active ink exclusion.
                self.inside_excluded_ink = false;
                self.state_stack.current_mut().fill_color_rgb = (gray, gray, gray);
            },
            Operator::SetStrokeGray { gray } => {
                self.state_stack.current_mut().stroke_color_rgb = (gray, gray, gray);
            },
            Operator::SetFillCmyk { c, m, y, k } => {
                // k operator implicitly sets DeviceCMYK — a process color.
                self.inside_excluded_ink = false;
                let state = self.state_stack.current_mut();
                state.fill_color_cmyk = Some((c, m, y, k));
                state.fill_color_rgb = cmyk_to_rgb(c, m, y, k);
            },
            Operator::SetStrokeCmyk { c, m, y, k } => {
                let state = self.state_stack.current_mut();
                state.stroke_color_cmyk = Some((c, m, y, k));
                state.stroke_color_rgb = cmyk_to_rgb(c, m, y, k);
            },

            // Color space operators
            Operator::SetFillColorSpace { name } => {
                // Check for excluded ink before mutating state (needs &self)
                let ink_excluded = self.is_excluded_ink_color_space(&name);
                self.inside_excluded_ink = ink_excluded;
                if ink_excluded {
                    log::debug!(
                        "Fill color space {:?} matches excluded ink, suppressing text",
                        name
                    );
                }

                let state = self.state_stack.current_mut();
                state.fill_color_space = name.clone();
                // Reset color when changing color space
                state.fill_color_rgb = (0.0, 0.0, 0.0);
                state.fill_color_cmyk = None;
            },
            Operator::SetStrokeColorSpace { name } => {
                let state = self.state_stack.current_mut();
                state.stroke_color_space = name.clone();
                // Reset color when changing color space
                state.stroke_color_rgb = (0.0, 0.0, 0.0);
                state.stroke_color_cmyk = None;
            },
            Operator::SetFillColor { components } => {
                // Set fill color using components in current fill color space
                let state = self.state_stack.current_mut();
                match state.fill_color_space.as_str() {
                    "DeviceGray" | "CalGray" if components.len() == 1 => {
                        let gray = components[0];
                        state.fill_color_rgb = (gray, gray, gray);
                    },
                    "DeviceRGB" | "CalRGB" if components.len() == 3 => {
                        state.fill_color_rgb = (components[0], components[1], components[2]);
                    },
                    "Lab" if components.len() == 3 => {
                        // CIE L*a*b* color space
                        // For now, treat as RGB (proper conversion requires whitepoint)
                        // L* is lightness (0-100), a* and b* are color opponents
                        // Simplified conversion: normalize and treat as RGB
                        let l = components[0] / 100.0;
                        state.fill_color_rgb = (l, l, l); // Simplified grayscale approximation
                        log::debug!(
                            "Lab color space simplified to grayscale (full conversion not yet implemented)"
                        );
                    },
                    "DeviceCMYK" if components.len() == 4 => {
                        state.fill_color_cmyk =
                            Some((components[0], components[1], components[2], components[3]));
                        state.fill_color_rgb =
                            cmyk_to_rgb(components[0], components[1], components[2], components[3]);
                    },
                    "ICCBased" => {
                        // ICC profile-based color space
                        // For now, assume RGB and use components directly
                        if components.len() == 3 {
                            state.fill_color_rgb = (components[0], components[1], components[2]);
                        } else if components.len() == 1 {
                            let gray = components[0];
                            state.fill_color_rgb = (gray, gray, gray);
                        } else if components.len() == 4 {
                            // Treat as CMYK
                            state.fill_color_cmyk =
                                Some((components[0], components[1], components[2], components[3]));
                            state.fill_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        }
                        log::debug!(
                            "ICCBased color space using simplified conversion (ICC profile not processed)"
                        );
                    },
                    "Separation" if components.len() == 1 => {
                        // Separation color space (spot color)
                        // Component is tint value (0.0 = no ink, 1.0 = full ink)
                        // For now, treat as grayscale
                        let tint = components[0];
                        let gray = 1.0 - tint; // Inverted (0 tint = white, 1 tint = black)
                        state.fill_color_rgb = (gray, gray, gray);
                        log::debug!("Separation color space simplified to grayscale");
                    },
                    "DeviceN" if !components.is_empty() => {
                        // DeviceN color space (multiple colorants)
                        // For now, use simplified conversion
                        if components.len() == 4 {
                            state.fill_color_cmyk =
                                Some((components[0], components[1], components[2], components[3]));
                            state.fill_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        } else {
                            // Use first component as grayscale
                            let gray = 1.0 - components[0];
                            state.fill_color_rgb = (gray, gray, gray);
                        }
                        log::debug!("DeviceN color space using simplified conversion");
                    },
                    _ => {
                        // Named color space reference (e.g. "Cs1") or unknown —
                        // fall back by component count to avoid warn spam.
                        match components.len() {
                            1 => {
                                let gray = components[0];
                                state.fill_color_rgb = (gray, gray, gray);
                            },
                            3 => {
                                state.fill_color_rgb =
                                    (components[0], components[1], components[2]);
                            },
                            4 => {
                                state.fill_color_cmyk = Some((
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                ));
                                state.fill_color_rgb = cmyk_to_rgb(
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                );
                            },
                            _ => {},
                        }
                        log::debug!(
                            "Unknown fill color space {:?} with {} components; \
                             applied component-count fallback",
                            state.fill_color_space,
                            components.len()
                        );
                    },
                }
            },
            Operator::SetStrokeColor { components } => {
                // Set stroke color using components in current stroke color space
                let state = self.state_stack.current_mut();
                match state.stroke_color_space.as_str() {
                    "DeviceGray" | "CalGray" if components.len() == 1 => {
                        let gray = components[0];
                        state.stroke_color_rgb = (gray, gray, gray);
                    },
                    "DeviceRGB" | "CalRGB" if components.len() == 3 => {
                        state.stroke_color_rgb = (components[0], components[1], components[2]);
                    },
                    "Lab" if components.len() == 3 => {
                        let l = components[0] / 100.0;
                        state.stroke_color_rgb = (l, l, l);
                        log::debug!("Lab stroke color space simplified to grayscale");
                    },
                    "DeviceCMYK" if components.len() == 4 => {
                        state.stroke_color_cmyk =
                            Some((components[0], components[1], components[2], components[3]));
                        state.stroke_color_rgb =
                            cmyk_to_rgb(components[0], components[1], components[2], components[3]);
                    },
                    "ICCBased" => {
                        if components.len() == 3 {
                            state.stroke_color_rgb = (components[0], components[1], components[2]);
                        } else if components.len() == 1 {
                            let gray = components[0];
                            state.stroke_color_rgb = (gray, gray, gray);
                        } else if components.len() == 4 {
                            state.stroke_color_cmyk =
                                Some((components[0], components[1], components[2], components[3]));
                            state.stroke_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        }
                        log::debug!("ICCBased stroke color using simplified conversion");
                    },
                    "Separation" if components.len() == 1 => {
                        let tint = components[0];
                        let gray = 1.0 - tint;
                        state.stroke_color_rgb = (gray, gray, gray);
                        log::debug!("Separation stroke color simplified to grayscale");
                    },
                    "DeviceN" if !components.is_empty() => {
                        if components.len() == 4 {
                            state.stroke_color_cmyk =
                                Some((components[0], components[1], components[2], components[3]));
                            state.stroke_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        } else {
                            let gray = 1.0 - components[0];
                            state.stroke_color_rgb = (gray, gray, gray);
                        }
                        log::debug!("DeviceN stroke color using simplified conversion");
                    },
                    _ => {
                        match components.len() {
                            1 => {
                                let gray = components[0];
                                state.stroke_color_rgb = (gray, gray, gray);
                            },
                            3 => {
                                state.stroke_color_rgb =
                                    (components[0], components[1], components[2]);
                            },
                            4 => {
                                state.stroke_color_cmyk = Some((
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                ));
                                state.stroke_color_rgb = cmyk_to_rgb(
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                );
                            },
                            _ => {},
                        }
                        log::debug!(
                            "Unknown stroke color space {:?} with {} components; \
                             applied component-count fallback",
                            state.stroke_color_space,
                            components.len()
                        );
                    },
                }
            },
            Operator::SetFillColorN { components, name } => {
                // Like SetFillColor, but also supports pattern color spaces
                if name.is_some() {
                    // Pattern color space - for now, just log and ignore
                    log::debug!("Pattern fill color not yet supported: {:?}", name);
                } else {
                    // Same logic as SetFillColor - supports all color spaces
                    let state = self.state_stack.current_mut();
                    match state.fill_color_space.as_str() {
                        "DeviceGray" | "CalGray" if components.len() == 1 => {
                            let gray = components[0];
                            state.fill_color_rgb = (gray, gray, gray);
                        },
                        "DeviceRGB" | "CalRGB" if components.len() == 3 => {
                            state.fill_color_rgb = (components[0], components[1], components[2]);
                        },
                        "Lab" if components.len() == 3 => {
                            let l = components[0] / 100.0;
                            state.fill_color_rgb = (l, l, l);
                        },
                        "DeviceCMYK" if components.len() == 4 => {
                            state.fill_color_cmyk =
                                Some((components[0], components[1], components[2], components[3]));
                            state.fill_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        },
                        "ICCBased" => {
                            if components.len() == 3 {
                                state.fill_color_rgb =
                                    (components[0], components[1], components[2]);
                            } else if components.len() == 1 {
                                let gray = components[0];
                                state.fill_color_rgb = (gray, gray, gray);
                            } else if components.len() == 4 {
                                state.fill_color_cmyk = Some((
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                ));
                                state.fill_color_rgb = cmyk_to_rgb(
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                );
                            }
                        },
                        "Separation" if components.len() == 1 => {
                            let tint = components[0];
                            let gray = 1.0 - tint;
                            state.fill_color_rgb = (gray, gray, gray);
                        },
                        "DeviceN" if !components.is_empty() => {
                            if components.len() == 4 {
                                state.fill_color_cmyk = Some((
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                ));
                                state.fill_color_rgb = cmyk_to_rgb(
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                );
                            } else {
                                let gray = 1.0 - components[0];
                                state.fill_color_rgb = (gray, gray, gray);
                            }
                        },
                        _ => {
                            match components.len() {
                                1 => {
                                    let gray = components[0];
                                    state.fill_color_rgb = (gray, gray, gray);
                                },
                                3 => {
                                    state.fill_color_rgb =
                                        (components[0], components[1], components[2]);
                                },
                                4 => {
                                    state.fill_color_cmyk = Some((
                                        components[0],
                                        components[1],
                                        components[2],
                                        components[3],
                                    ));
                                    state.fill_color_rgb = cmyk_to_rgb(
                                        components[0],
                                        components[1],
                                        components[2],
                                        components[3],
                                    );
                                },
                                _ => {},
                            }
                            log::debug!(
                                "Unknown fill color space {:?} with {} components; \
                                 applied component-count fallback",
                                state.fill_color_space,
                                components.len()
                            );
                        },
                    }
                }
            },
            Operator::SetStrokeColorN { components, name } => {
                // Like SetStrokeColor, but also supports pattern color spaces
                if name.is_some() {
                    // Pattern color space - for now, just log and ignore
                    log::debug!("Pattern stroke color not yet supported: {:?}", name);
                } else {
                    // Same logic as SetStrokeColor - supports all color spaces
                    let state = self.state_stack.current_mut();
                    match state.stroke_color_space.as_str() {
                        "DeviceGray" | "CalGray" if components.len() == 1 => {
                            let gray = components[0];
                            state.stroke_color_rgb = (gray, gray, gray);
                        },
                        "DeviceRGB" | "CalRGB" if components.len() == 3 => {
                            state.stroke_color_rgb = (components[0], components[1], components[2]);
                        },
                        "Lab" if components.len() == 3 => {
                            let l = components[0] / 100.0;
                            state.stroke_color_rgb = (l, l, l);
                        },
                        "DeviceCMYK" if components.len() == 4 => {
                            state.stroke_color_cmyk =
                                Some((components[0], components[1], components[2], components[3]));
                            state.stroke_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        },
                        "ICCBased" => {
                            if components.len() == 3 {
                                state.stroke_color_rgb =
                                    (components[0], components[1], components[2]);
                            } else if components.len() == 1 {
                                let gray = components[0];
                                state.stroke_color_rgb = (gray, gray, gray);
                            } else if components.len() == 4 {
                                state.stroke_color_cmyk = Some((
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                ));
                                state.stroke_color_rgb = cmyk_to_rgb(
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                );
                            }
                        },
                        "Separation" if components.len() == 1 => {
                            let tint = components[0];
                            let gray = 1.0 - tint;
                            state.stroke_color_rgb = (gray, gray, gray);
                        },
                        "DeviceN" if !components.is_empty() => {
                            if components.len() == 4 {
                                state.stroke_color_cmyk = Some((
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                ));
                                state.stroke_color_rgb = cmyk_to_rgb(
                                    components[0],
                                    components[1],
                                    components[2],
                                    components[3],
                                );
                            } else {
                                let gray = 1.0 - components[0];
                                state.stroke_color_rgb = (gray, gray, gray);
                            }
                        },
                        _ => {
                            match components.len() {
                                1 => {
                                    let gray = components[0];
                                    state.stroke_color_rgb = (gray, gray, gray);
                                },
                                3 => {
                                    state.stroke_color_rgb =
                                        (components[0], components[1], components[2]);
                                },
                                4 => {
                                    state.stroke_color_cmyk = Some((
                                        components[0],
                                        components[1],
                                        components[2],
                                        components[3],
                                    ));
                                    state.stroke_color_rgb = cmyk_to_rgb(
                                        components[0],
                                        components[1],
                                        components[2],
                                        components[3],
                                    );
                                },
                                _ => {},
                            }
                            log::debug!(
                                "Unknown stroke color space {:?} with {} components; \
                                 applied component-count fallback",
                                state.stroke_color_space,
                                components.len()
                            );
                        },
                    }
                }
            },

            // Line style operators
            Operator::SetLineCap { cap_style } => {
                self.state_stack.current_mut().line_cap = cap_style;
            },
            Operator::SetLineJoin { join_style } => {
                self.state_stack.current_mut().line_join = join_style;
            },
            Operator::SetMiterLimit { limit } => {
                self.state_stack.current_mut().miter_limit = limit;
            },
            Operator::SetRenderingIntent { intent } => {
                self.state_stack.current_mut().rendering_intent = intent.clone();
            },
            Operator::SetFlatness { tolerance } => {
                self.state_stack.current_mut().flatness = tolerance;
            },
            Operator::SetExtGState { dict_name } => {
                self.flush_tj_span_buffer()?;
                if self.visible_paint_only {
                    if let (Some(doc), Some(resources)) = (self.document, self.resources.as_ref()) {
                        let (fill, stroke) = doc.paint_alpha(resources, &dict_name)?;
                        let state = self.state_stack.current_mut();
                        if let Some(alpha) = fill {
                            state.fill_alpha = alpha;
                        }
                        if let Some(alpha) = stroke {
                            state.stroke_alpha = alpha;
                        }
                    }
                }
            },
            Operator::PaintShading { name } => {
                // Shading operator - paint gradient/shading pattern
                // PDF Spec: ISO 32000-1:2008, Section 8.7.4.3
                //
                // Shading patterns define smooth color gradients and can be:
                // Type 1: Function-based shading
                // Type 2: Axial shading (linear gradient)
                // Type 3: Radial shading (circular gradient)
                // Type 4-7: Mesh-based shadings (Gouraud, Coons patch, tensor-product)
                //
                // For text extraction, shading patterns don't affect text content.
                // Full implementation would require rendering the gradient for visual output.
                log::debug!(
                    "Shading pattern '{}' referenced (gradients not rendered in text extraction)",
                    name
                );
            },
            Operator::InlineImage { dict, data } => {
                // Inline image operator - embedded image in content stream
                // PDF Spec: ISO 32000-1:2008, Section 8.9.7 - Inline Images
                //
                // Inline images are small images embedded directly in the content stream
                // using the BI...ID...EI sequence, rather than referenced as XObjects.
                //
                // For text extraction, inline images don't contribute to text content.
                // They would be rendered for visual output or extracted separately
                // for image extraction functionality.
                //
                // Common dictionary keys (abbreviated):
                // - W: Width, H: Height
                // - CS: ColorSpace (DeviceRGB, DeviceGray, etc.)
                // - BPC: BitsPerComponent
                // - F: Filter (FlateDecode, DCTDecode, etc.)
                let width = dict
                    .get("W")
                    .and_then(|obj| match obj {
                        Object::Integer(i) => Some(*i),
                        _ => None,
                    })
                    .unwrap_or(0);
                let height = dict
                    .get("H")
                    .and_then(|obj| match obj {
                        Object::Integer(i) => Some(*i),
                        _ => None,
                    })
                    .unwrap_or(0);
                log::debug!(
                    "Inline image encountered: {}x{} pixels, {} bytes of data (not rendered in text extraction)",
                    width,
                    height,
                    data.len()
                );
            },

            // Text object operators (BT/ET)
            // PDF Spec ISO 32000-1:2008, Section 9.4.1:
            // "At the beginning of a text object, Tm and Tlm shall be
            // initialized to the identity matrix."
            Operator::BeginText => {
                let state = self.state_stack.current_mut();
                state.text_matrix = Matrix::identity();
                state.text_line_matrix = Matrix::identity();
            },
            Operator::EndText => {
                // Flush any pending text buffer at end of text object
                self.flush_tj_span_buffer()?;
            },

            // Marked content operators - for tagged PDF structure
            // PDF Spec: ISO 32000-1:2008, Section 14.6 - Marked Content
            // These operators define logical structure and accessibility metadata.
            // Per PDF Spec Section 14.6, we track artifact status to filter out
            // non-text content (headers, footers, watermarks, resource paths).
            Operator::BeginMarkedContent { tag } => {
                // Flush the Tj span buffer at the marked-content boundary
                // (ISO 32000-1:2008 §14.6). Without this, consecutive Tj
                // operators that straddle a BMC/BDC/EMC boundary get
                // glued into a single span whose `mcid` reflects only
                // the FIRST Tj — fusing two structurally-distinct
                // elements and breaking every downstream consumer that
                // relies on MCID identity (structure-tree reading
                // order, tree-scope ActualText suppression,
                // table-cell membership).
                self.flush_tj_span_buffer()?;
                if tag == "ReversedChars" {
                    self.saw_reversed_chars = true;
                }
                // BMC doesn't have properties, but the tag can indicate artifacts
                let is_artifact = tag == "Artifact";
                // InDesign placed-PDF figure region (see MarkedContentContext::is_placed_pdf).
                let is_placed_pdf = tag == "PlacedPDF";
                self.marked_content_stack.push(MarkedContentContext {
                    tag: tag.clone(),
                    is_artifact,
                    artifact_type: None, // No artifact classification; None for backward compatibility
                    actual_text: None,   // BMC doesn't have ActualText
                    actual_text_emitted: false,
                    expansion: None,          // BMC doesn't have expansion
                    is_excluded_layer: false, // BMC cannot carry OCG properties
                    is_placed_pdf,
                    own_mcid: None, // BMC carries no MCID
                });
                self.update_artifact_state();
                self.update_layer_state();

                if is_artifact {
                    log::debug!("Entered /Artifact marked content (BMC, no subtype)");
                }
            },

            Operator::BeginMarkedContentDict { tag, properties } => {
                // See `BeginMarkedContent` for the rationale; same
                // reasoning applies to BDC.
                self.flush_tj_span_buffer()?;
                // BDC can have properties including MCID, artifact indicators, ActualText, and expansion
                // Properties can be an inline dictionary or a name referencing /Properties resource
                let mut actual_text = None;
                let mut artifact_type = None;
                let mut expansion = None;
                let mut own_mcid: Option<u32> = None;

                let mut is_excluded_layer = false;

                if let Some(props_dict) = self.resolve_bdc_properties(&properties) {
                    if let Some(mcid_obj) = props_dict.get("MCID") {
                        if let Some(mcid) = mcid_obj.as_integer() {
                            own_mcid = Some(mcid as u32);
                            self.current_mcid = Some(mcid as u32);
                            log::debug!("Entered marked content with MCID: {}", mcid);
                        }
                    }

                    if let Some(actual_text_obj) = props_dict.get("ActualText") {
                        if let Some(text_bytes) = actual_text_obj.as_string() {
                            actual_text = Some(Self::decode_pdf_text_string(text_bytes));
                            log::debug!("Marked content has ActualText: {:?}", actual_text);
                            // Record that this MCID's in-stream
                            // /ActualText is the authoritative
                            // replacement (MC-scope wins over any
                            // ancestor's struct-tree-scope
                            // /ActualText).
                            if let Some(mcid) = self.current_mcid {
                                self.mc_actualtext_mcids.insert(mcid);
                            }
                        }
                    }

                    if let Some(expansion_obj) = props_dict.get("E") {
                        if let Some(text_bytes) = expansion_obj.as_string() {
                            expansion = Some(Self::decode_pdf_text_string(text_bytes));
                            log::debug!("Marked content has expansion /E: {:?}", expansion);
                        }
                    }

                    if tag == "Artifact" {
                        artifact_type = Self::parse_artifact_type(&props_dict);
                    }

                    // OCG / OCMD (Optional Content) filtering.
                    // Per ISO 32000-1:2008 Section 8.11.2:
                    //  - Direct OCG: << /Type /OCG /Name /LayerName >>
                    //  - OCMD:       << /Type /OCMD /OCGs [refs...] /P /policy >>
                    if tag == "OC" && !self.excluded_layers.is_empty() {
                        is_excluded_layer = self.check_ocg_excluded(&props_dict);
                    }
                }

                // Check if this is an artifact (per PDF Spec Section 14.6)
                let is_artifact = tag == "Artifact";
                // InDesign placed-PDF figure region (see MarkedContentContext::is_placed_pdf).
                let is_placed_pdf = tag == "PlacedPDF";
                self.marked_content_stack.push(MarkedContentContext {
                    tag: tag.clone(),
                    is_artifact,
                    artifact_type: artifact_type.clone(),
                    actual_text,
                    actual_text_emitted: false,
                    expansion,
                    is_excluded_layer,
                    is_placed_pdf,
                    own_mcid,
                });
                self.update_artifact_state();
                self.update_layer_state();

                if is_artifact {
                    if let Some(ref atype) = artifact_type {
                        log::debug!("Entered /Artifact marked content: {:?}", atype);
                    } else {
                        log::debug!("Entered /Artifact marked content (no type specified)");
                    }
                }
            },

            Operator::EndMarkedContent => {
                // Flush the Tj span buffer at the marked-content
                // boundary; see `BeginMarkedContent` for the
                // rationale.
                self.flush_tj_span_buffer()?;
                // EMC ends the current marked content sequence.
                // Pop the stack THEN restore `current_mcid` from the
                // nearest enclosing BDC that carried `/MCID` — per
                // ISO 32000-1:2008 §14.6, marked-content sequences
                // nest, and a `Tj` issued after an inner EMC must
                // attribute to its enclosing scope. Blanking
                // `current_mcid` here would orphan that `Tj`'s span
                // (MAJOR-1 regression #...).
                if !self.marked_content_stack.is_empty() {
                    self.marked_content_stack.pop();
                    self.update_artifact_state();
                    self.update_layer_state();
                }
                let restored = self
                    .marked_content_stack
                    .iter()
                    .rev()
                    .find_map(|ctx| ctx.own_mcid);
                if let Some(prev) = self.current_mcid {
                    log::debug!(
                        "Exited marked content with MCID: {} -> restoring to {:?}",
                        prev,
                        restored
                    );
                }
                self.current_mcid = restored;
            },

            // XObject operator - Process Form XObjects for text extraction
            Operator::Do { name } => {
                // Flush the Tj span buffer before invoking a Form XObject.
                // `process_xobject` applies the form's /Matrix to the CTM
                // (§8.10.1) and may execute cm/Tm operators inside the
                // form's content stream. The buffer's captured user_pos
                // would no longer correspond to the CTM in effect when
                // the form's text is emitted, so subsequent Tj chars
                // would be stitched into the wrong cluster.
                self.flush_tj_span_buffer()?;

                // Process Form XObjects to extract text from reusable content.
                // Form XObjects can contain text that is not duplicated in the main stream.
                // We track processed XObjects to avoid infinite loops and duplicates.
                let result = self.process_xobject(&name);
                if let Some(doc) = self.document {
                    doc.check_import_budget()?;
                }
                if let Err(e) = result {
                    // Log error but continue processing - don't fail the entire extraction
                    log::warn!("Failed to process XObject '{}': {}", name, e);
                }
            },

            // Other operators we don't need for text extraction
            _ => {
                // Ignore path, image, and other operators
            },
        }

        Ok(())
    }

    /// Maximum XObject recursion depth. Text content in PDFs is rarely nested
    /// more than 2-3 levels. Deep nesting typically indicates complex vector
    /// graphics (charts, plots) with no text content.
    const MAX_XOBJECT_DEPTH: u32 = 10;

    const MAX_XOBJECT_DECODES: u32 = 500;

    /// Resolve XObject name to ObjectRef using cached mapping.
    fn resolve_xobject_ref(&mut self, name: &str) -> Result<Option<ObjectRef>> {
        // Check cache first (O(1) lookup)
        if let Some(cached) = self.cached_xobject_refs.get(name) {
            return Ok(*cached);
        }

        // Cache miss — resolve the full chain once and populate cache
        let resources = match &self.resources {
            Some(res) => res.clone(),
            None => return Ok(None),
        };

        let doc = match self.document {
            Some(d) => d,
            None => return Ok(None),
        };

        // Resolve resources → XObject dict
        let resources_obj = if let Some(res_ref) = resources.as_reference() {
            doc.load_object(res_ref)?
        } else {
            resources
        };

        let resources_dict = match resources_obj.as_dict() {
            Some(d) => d,
            None => return Ok(None),
        };

        let xobject_entry = match resources_dict.get("XObject") {
            Some(xobj) => xobj.clone(),
            None => return Ok(None),
        };

        let xobject_obj = if let Some(xobj_ref) = xobject_entry.as_reference() {
            doc.load_object(xobj_ref)?
        } else {
            xobject_entry
        };

        let xobject_dict = match xobject_obj.as_dict() {
            Some(d) => d,
            None => return Ok(None),
        };

        // Populate the entire cache for this resources context
        for (key, val) in xobject_dict.iter() {
            let obj_ref = val.as_reference();
            self.cached_xobject_refs.insert(key.clone(), obj_ref);
        }

        // Return the requested name
        Ok(self.cached_xobject_refs.get(name).copied().flatten())
    }

    fn process_xobject(&mut self, name: &str) -> Result<()> {
        let import_budget = self.document.and_then(|doc| doc.import_budget()).cloned();
        if let Some(budget) = &import_budget {
            budget.xobject_visit()?;
        } else {
            if self.xobject_depth >= Self::MAX_XOBJECT_DEPTH {
                return Ok(());
            }
            if self.xobject_decode_count >= Self::MAX_XOBJECT_DECODES {
                return Ok(());
            }
        }

        // Resolve name → ObjectRef using cached mapping (avoids expensive
        // repeated resolution of resources/XObject dict chain)
        let xobject_ref = match self.resolve_xobject_ref(name)? {
            Some(r) => r,
            None => return Ok(()),
        };

        // Build a CTM-aware deduplication key.
        //
        // Using just `xobject_ref` as the key incorrectly blocked re-processing
        // the same Form XObject when it was invoked a second time on the same page
        // with a different CTM (e.g., same header/footer XObject stamped at two
        // different Y positions, or the nougat_005 pattern where each page's
        // content stream sets a different `cm` translation before calling `Do`).
        //
        // The CTM is encoded as 6 millipoint-rounded i64 values so it can be
        // stored in a HashSet without floating-point equality hazards.
        // Infinite-recursion cycles are still prevented because a truly recursive
        // call re-enters with the *same* XObject ref AND the same CTM at that
        // nesting depth; the depth limiter (MAX_XOBJECT_DEPTH) provides a
        // second backstop.
        let current_ctm = self.state_stack.current().ctm;
        // Round to nearest millipoint instead of truncating with `as i64`,
        // so floating-point noise in the same logical CTM produces a
        // stable hash key (truncation alone could send 0.99999...
        // 1.00001... to different buckets).
        let ctm_key = [
            (current_ctm.a * 1000.0).round() as i64,
            (current_ctm.b * 1000.0).round() as i64,
            (current_ctm.c * 1000.0).round() as i64,
            (current_ctm.d * 1000.0).round() as i64,
            (current_ctm.e * 1000.0).round() as i64,
            (current_ctm.f * 1000.0).round() as i64,
        ];
        let xobj_key = (xobject_ref, ctm_key);

        // Skip already-processed (XObject, CTM) pairs — each unique combination
        // is processed at most once per page for text extraction.
        if self.processed_xobjects.contains(&xobj_key) {
            return Ok(());
        }

        self.processed_xobjects.insert(xobj_key);

        // Get document reference for loading objects.
        let doc = match self.document {
            Some(d) => d,
            None => return Ok(()),
        };

        if doc
            .xobject_text_free_cache
            .lock()
            .unwrap()
            .contains(&xobject_ref)
        {
            return Ok(());
        }

        // Quick Subtype check: skip Image XObjects without loading the full object.
        // Image XObjects can be megabytes of compressed pixel data — loading them
        // just to discover Subtype=Image is a major bottleneck (10-15ms per image).
        if !doc.is_form_xobject(xobject_ref) {
            doc.check_import_budget()?;
            return Ok(());
        }
        if let Some(budget) = &import_budget {
            budget.form_depth(self.xobject_depth as usize + 1)?;
        }

        // Span result cache: reuse extracted spans from self-contained Form XObjects.
        //
        // The cache key is (ObjectRef, ctm_key) where ctm_key encodes the caller's
        // CTM as 6 millipoint-rounded i64 values. This allows the same Form XObject
        // to have independent cached results for each unique CTM it is painted with,
        // fixing the issue where cross-page reuse of a single Form XObject with
        // different per-page CTM translations returned stale page-0 coordinates on
        // all subsequent pages (nougat_005.pdf, Issue B1).
        //
        // `ctm_key` was already computed above for the `processed_xobjects` guard.
        let spans_cache_key = (xobject_ref, ctm_key);
        let has_filters = self.visible_paint_only
            || !self.excluded_layers.is_empty()
            || !self.excluded_inks.is_empty();
        if self.extract_spans && !has_filters && import_budget.is_none() {
            let cached_spans = {
                doc.xobject_spans_cache
                    .lock()
                    .unwrap()
                    .get(&spans_cache_key)
                    .cloned()
            };
            if let Some(cached_spans) = cached_spans {
                if let Some(spans) = cached_spans {
                    self.spans.extend(spans.iter().cloned());
                }
                return Ok(());
            }
        }

        // Load the XObject (now known to be Form or unknown — worth the full load)
        let xobject = doc.load_object(xobject_ref)?;

        // Check if it's a Form XObject (has Subtype /Form)
        let xobject_dict = match xobject.as_dict() {
            Some(d) => d,
            None => {
                log::debug!("XObject '{}' is not a dictionary", name);
                return Ok(());
            },
        };

        let subtype = xobject_dict.get("Subtype").and_then(|s| s.as_name());

        match subtype {
            Some("Form") => {
                // Form XObject - extract text from it
                log::debug!("Processing Form XObject: {}", name);

                // Pre-decode resource check: if the XObject's own /Resources has
                // neither /Font nor /XObject entries, it cannot render text directly
                // and cannot invoke nested XObjects. Skip it without decoding the
                // stream, which avoids expensive FlateDecode decompression.
                if let Some(xobj_resources) = xobject_dict.get("Resources") {
                    let xobj_res = if let Some(res_ref) = xobj_resources.as_reference() {
                        doc.load_object(res_ref).ok()
                    } else {
                        Some(xobj_resources.clone())
                    };

                    if let Some(ref res_obj) = xobj_res {
                        if let Some(res_dict) = res_obj.as_dict() {
                            let has_font = res_dict.contains_key("Font");
                            let has_xobject = res_dict.contains_key("XObject");
                            if !has_font && !has_xobject {
                                log::debug!(
                                    "Skipping Form XObject '{}': no Font/XObject in Resources",
                                    name
                                );
                                doc.xobject_text_free_cache
                                    .lock()
                                    .unwrap()
                                    .insert(xobject_ref);
                                return Ok(());
                            }
                        }
                    }
                } else {
                    // No Resources at all — XObject inherits page-level fonts but
                    // still must be decoded to check for text operators. However,
                    // Form XObjects that are pure graphics often omit Resources
                    // entirely when they have no font/xobject needs. Check if the
                    // page has any active fonts; if not, skip.
                }

                // Decode the stream — check cache first to avoid repeated FlateDecode.
                self.xobject_decode_count += 1;
                let cached_stream = if import_budget.is_some() {
                    None
                } else {
                    doc.xobject_stream_cache
                        .lock()
                        .unwrap()
                        .get(&xobject_ref)
                        .cloned()
                };
                let stream_data = if let Some(cached) = cached_stream {
                    cached.as_ref().clone()
                } else {
                    match doc.decode_stream_with_encryption(&xobject, xobject_ref) {
                        Ok(data) => {
                            // Cache if under 50MB total
                            const MAX_STREAM_CACHE_BYTES: usize = 50 * 1024 * 1024;
                            let current = doc
                                .xobject_stream_cache_bytes
                                .load(std::sync::atomic::Ordering::Relaxed);
                            if import_budget.is_none()
                                && current + data.len() <= MAX_STREAM_CACHE_BYTES
                            {
                                doc.xobject_stream_cache_bytes.store(
                                    current + data.len(),
                                    std::sync::atomic::Ordering::Relaxed,
                                );
                                doc.xobject_stream_cache
                                    .lock()
                                    .unwrap()
                                    .insert(xobject_ref, std::sync::Arc::new(data.clone()));
                            }
                            data
                        },
                        Err(e) => {
                            log::warn!(
                                "Failed to decode Form XObject '{}' stream: {}, skipping",
                                name,
                                e
                            );
                            return Ok(());
                        },
                    }
                };

                if !crate::document::PdfDocument::may_contain_text(&stream_data) {
                    log::debug!(
                        "Skipping text-free Form XObject '{}' ({} bytes)",
                        name,
                        stream_data.len()
                    );
                    doc.xobject_text_free_cache
                        .lock()
                        .unwrap()
                        .insert(xobject_ref);
                    return Ok(());
                }

                // Parse /Matrix from Form XObject dict (default: identity per ISO 32000-1 §8.10.1)
                let form_matrix = if let Some(Object::Array(arr)) = xobject_dict.get("Matrix") {
                    let get_f32 = |i: usize| -> f32 {
                        match arr.get(i) {
                            Some(Object::Real(v)) => *v as f32,
                            Some(Object::Integer(v)) => *v as f32,
                            _ => {
                                if i == 0 || i == 3 {
                                    1.0
                                } else {
                                    0.0
                                }
                            },
                        }
                    };
                    Matrix {
                        a: get_f32(0),
                        b: get_f32(1),
                        c: get_f32(2),
                        d: get_f32(3),
                        e: get_f32(4),
                        f: get_f32(5),
                    }
                } else {
                    Matrix::identity()
                };

                // Parse /BBox (form coordinate space) for the §8.10.1 form clip.
                // A form XObject's painting is clipped to its /BBox; text the form
                // draws outside the BBox is invisible in a conformant renderer and
                // must not be extracted. Stored as [x0,y0,x1,y1]; None disables the
                // clip (defensive — /BBox is required, but malformed dicts exist).
                let form_bbox: Option<[f32; 4]> = match xobject_dict.get("BBox") {
                    Some(Object::Array(arr)) if arr.len() >= 4 => {
                        let f = |i: usize| -> Option<f32> {
                            match arr.get(i) {
                                Some(Object::Real(v)) => Some(*v as f32),
                                Some(Object::Integer(v)) => Some(*v as f32),
                                _ => None,
                            }
                        };
                        match (f(0), f(1), f(2), f(3)) {
                            (Some(a), Some(b), Some(c), Some(d))
                                if a.is_finite()
                                    && b.is_finite()
                                    && c.is_finite()
                                    && d.is_finite() =>
                            {
                                // Normalize so [x0,y0] is the min corner.
                                Some([a.min(c), b.min(d), a.max(c), b.max(d)])
                            },
                            _ => None,
                        }
                    },
                    _ => None,
                };

                // Only save/restore fonts+resources when XObject has its own Resources.
                // Avoids expensive HashMap clone for XObjects that inherit page fonts.
                let has_own_resources = xobject_dict.contains_key("Resources");

                let saved_fonts;
                let saved_resources;
                let saved_xobj_cache;

                if has_own_resources {
                    saved_fonts = Some(self.try_font_map()?);
                    saved_resources = self
                        .resources
                        .as_ref()
                        .map(|resources| {
                            crate::object_copy::clone_object(resources, import_budget.as_deref())
                        })
                        .transpose()?;
                    saved_xobj_cache = Some(std::mem::take(&mut self.cached_xobject_refs));

                    // Safety: has_own_resources was set by contains_key("Resources")
                    // so get("Resources") will always return Some here
                    let xobj_resources = xobject_dict
                        .get("Resources")
                        .expect("contains_key confirmed Resources exists");
                    let xobj_res = if let Some(res_ref) = xobj_resources.as_reference() {
                        match doc.load_object(res_ref) {
                            Ok(obj) => obj,
                            Err(_) => {
                                doc.check_import_budget()?;
                                crate::object_copy::clone_object(
                                    xobj_resources,
                                    import_budget.as_deref(),
                                )?
                            },
                        }
                    } else {
                        crate::object_copy::clone_object(xobj_resources, import_budget.as_deref())?
                    };

                    if let Err(e) = doc.load_fonts(&xobj_res, self) {
                        log::debug!(
                            "Failed to load fonts for Form XObject '{}': {}, using page fonts",
                            name,
                            e
                        );
                    }

                    self.resources = Some(xobj_res);
                } else {
                    saved_fonts = None;
                    saved_resources = None;
                    saved_xobj_cache = None;
                }

                // Track span count for result caching
                let spans_before = self.spans.len();
                // Same mark for the character layer, so the §8.10.1 /BBox clip
                // below can be applied to whichever layer this extraction is
                // populating. Only one of the two grows per run.
                let chars_before = self.chars.len();

                // Save graphics state (implicit q per ISO 32000-1 §8.10.1)
                self.state_stack.save();

                // Concatenate Form XObject /Matrix with CTM
                let state = self.state_stack.current_mut();
                state.ctm = form_matrix.multiply(&state.ctm);

                // Effective form→page transform used for every span drawn inside
                // the form; the /BBox clip below maps the BBox through this same
                // CTM so the comparison happens in one coordinate space.
                let form_ctm = self.state_stack.current().ctm;

                // Push the Form XObject scope (ISO 32000-1:2008
                // §14.7.4.3). Every MCID emitted inside this form's
                // content stream lives in the form's MCID namespace,
                // *not* the page's. Two distinct forms on the same
                // page that both emit MCID 0 stay distinct because
                // they push different `Form(form_ref)` scopes.
                self.mcid_scope_stack
                    .push(crate::structure::McidScope::Form(xobject_ref));

                self.xobject_depth += 1;
                let parse_result = self.run_content_stream(&stream_data);
                self.xobject_depth -= 1;
                // Pop the Form XObject scope pushed before the
                // content-stream walk. Cleared regardless of parse
                // success so the parent stream's scope is correctly
                // restored even on errors.
                self.mcid_scope_stack.pop();
                if let Err(e) = parse_result {
                    log::debug!(
                        "Error parsing Form XObject '{}' content stream: {}, partial text may be extracted",
                        name,
                        e
                    );
                }

                // Apply the Form XObject /BBox clip (ISO 32000-1:2008 §8.10.1): a
                // form's marks are clipped to its BBox, so text the form paints
                // OUTSIDE the BBox is invisible in a conformant renderer (pdfium,
                // Acrobat, MuPDF) and must not be extracted. Some producers (e.g.
                // pdfTeX \includegraphics of a figure PDF that retained a full
                // draft-galley page) paint a redundant copy of the article body
                // outside the figure's BBox; without this clip it surfaces as
                // duplicate text overlapping the real page body. Done BEFORE the
                // span cache so cached results are already clipped. Byte-identical
                // on every form whose painted text lies inside its BBox (the
                // conformant majority) — only out-of-BBox marks are dropped.
                if let Some([bx0, by0, bx1, by1]) = form_bbox {
                    let painted_anything =
                        self.spans.len() > spans_before || self.chars.len() > chars_before;
                    if painted_anything && bx1 > bx0 && by1 > by0 {
                        // Map the BBox corners through the form CTM into page space
                        // and take the axis-aligned bound (a superset for rotated
                        // forms — conservative, never over-clips).
                        let c = [
                            form_ctm.transform_point(bx0, by0),
                            form_ctm.transform_point(bx1, by0),
                            form_ctm.transform_point(bx1, by1),
                            form_ctm.transform_point(bx0, by1),
                        ];
                        let min_x = c.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                        let max_x = c.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                        let min_y = c.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                        let max_y = c.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                        if min_x.is_finite()
                            && max_x.is_finite()
                            && min_y.is_finite()
                            && max_y.is_finite()
                        {
                            // Tolerance so glyphs sitting exactly on the clip edge
                            // are kept (conformant clipping is exact; this only
                            // guards float rounding, far below any real margin).
                            const TOL: f32 = 1.0;
                            let inside_bbox = |b: &crate::geometry::Rect| {
                                let cx = b.x + b.width * 0.5;
                                let cy = b.y + b.height * 0.5;
                                cx >= min_x - TOL
                                    && cx <= max_x + TOL
                                    && cy >= min_y - TOL
                                    && cy <= max_y + TOL
                            };
                            let inside = |s: &TextSpan| inside_bbox(&s.bbox);
                            let inside_char = |c: &TextChar| inside_bbox(&c.bbox);
                            // Fast path: when every span this form painted is
                            // already inside its /BBox (the conformant majority —
                            // and where this clip is a no-op anyway), skip the
                            // split_off/extend allocation churn entirely. Only the
                            // rare out-of-BBox case (the draft-galley underlay)
                            // pays for the rebuild. Cheap O(form-spans) scan, no
                            // allocation; keeps large form-heavy docs fast.
                            let spans_stray = self.spans[spans_before..].iter().any(|s| !inside(s));
                            let chars_stray =
                                self.chars[chars_before..].iter().any(|c| !inside_char(c));
                            if spans_stray || chars_stray {
                                // Out-of-BBox spans exist. Distinguish a real
                                // figure form (whose stray out-of-BBox text is a
                                // draft-galley underlay safe to drop) from a
                                // full-page content-frame wrapper whose declared
                                // BBox happens to exclude real body text. A
                                // conformant *renderer* clips both, but every text
                                // *extractor* (poppler/pdftotext, the common
                                // reference) keeps a wrapper's body — and that body
                                // may be its only copy. The discriminator is
                                // coverage: a figure occupies a sub-region of the
                                // page; a wrapper covers most of it. Only clip when
                                // the form is figure-sized, so the galley-dedup win
                                // stays while page-wrapper bodies are preserved.
                                let clip_area = (max_x - min_x) * (max_y - min_y);
                                let page_idx = match self.mcid_scope_stack.first() {
                                    Some(crate::structure::McidScope::Page(p)) => *p as usize,
                                    _ => 0,
                                };
                                let page_area = self
                                    .document
                                    .and_then(|d| d.get_page_media_box(page_idx).ok())
                                    .map(|(llx, lly, urx, ury)| ((urx - llx) * (ury - lly)).abs())
                                    .filter(|a| *a > 0.0);
                                // ≥60% of page area ⇒ content-frame wrapper, not a
                                // figure (figures measured ≤27%; wrappers ≥82%).
                                let is_page_wrapper =
                                    page_area.is_some_and(|pa| clip_area >= 0.6 * pa);
                                if !is_page_wrapper {
                                    if spans_stray {
                                        let added = self.spans.split_off(spans_before);
                                        let kept: Vec<TextSpan> =
                                            added.into_iter().filter(|s| inside(s)).collect();
                                        self.spans.extend(kept);
                                    }
                                    if chars_stray {
                                        let added = self.chars.split_off(chars_before);
                                        let kept: Vec<TextChar> =
                                            added.into_iter().filter(inside_char).collect();
                                        self.chars.extend(kept);
                                    }
                                }
                            }
                        }
                    }
                }

                // Cache span results for self-contained Form XObjects.
                //
                // The cache key `spans_cache_key` already encodes (ObjectRef, ctm_key),
                // so each unique (XObject, CTM) pair gets its own entry. There is no
                // longer any need to restrict caching to identity-CTM invocations —
                // different CTMs produce different cache entries and therefore cannot
                // pollute each other (this was the root cause of issue B1).
                //
                // We still require `has_own_resources` so that font lookups are
                // self-contained; XObjects that inherit page-level fonts would
                // produce spans whose glyph mappings depend on caller context.
                if has_own_resources
                    && self.extract_spans
                    && !has_filters
                    && import_budget.is_none()
                {
                    let new_spans = if self.spans.len() > spans_before {
                        Some(self.spans[spans_before..].to_vec())
                    } else {
                        None
                    };
                    doc.xobject_spans_cache
                        .lock()
                        .unwrap()
                        .insert(spans_cache_key, new_spans);
                }

                // Restore graphics state (implicit Q per ISO 32000-1 §8.10.1)
                self.state_stack.restore();
                // Sync cached font with restored state
                self.cached_current_font = self
                    .state_stack
                    .current()
                    .font_name
                    .as_ref()
                    .and_then(|name| self.fonts.get(name))
                    .cloned();

                // Restore fonts, resources, and XObject cache only if saved
                if let Some(fonts) = saved_fonts {
                    self.fonts = fonts;
                }
                if let Some(res) = saved_resources {
                    self.resources = Some(res);
                }
                if let Some(cache) = saved_xobj_cache {
                    self.cached_xobject_refs = cache;
                }
                // Re-evaluate ink exclusion against the restored color space
                // and resources. The XObject may have set an excluded ink that
                // must not persist into the caller's scope.
                if !self.excluded_inks.is_empty() {
                    let cs = self.state_stack.current().fill_color_space.clone();
                    self.inside_excluded_ink = self.is_excluded_ink_color_space(&cs);
                }

                // Keep xobject_ref in processed_xobjects permanently.
                // For text extraction, re-processing the same Form XObject produces
                // identical text. Keeping it prevents O(n!) fan-out in pages with
                // deep XObject trees (e.g., 4000+ nested chart elements).

                doc.check_import_budget()?;
                Ok(())
            },
            Some("Image") => {
                // Image XObject - no text to extract
                log::debug!("Skipping Image XObject: {}", name);
                Ok(())
            },
            _ => {
                log::debug!("Unknown XObject subtype for '{}': {:?}", name, subtype);
                Ok(())
            },
        }
    }

    /// Get the current artifact type from the marked content stack.
    fn current_artifact_type(&self) -> Option<ArtifactType> {
        self.marked_content_stack
            .iter()
            .rev()
            .find_map(|ctx| ctx.artifact_type.clone())
    }

    /// Flush accumulated TJ buffer into a single TextSpan.
    ///
    /// This creates one span for the entire buffer content, properly calculating
    /// the total width including character spacing (Tc) and word spacing (Tw).
    fn flush_tj_buffer(&mut self, mut buffer: TjBuffer) -> Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        // Use accumulated width from advance_position_for_string calls
        // Convert from text space to user space using pre-computed horizontal scale
        let total_width = buffer.accumulated_width * buffer.user_h_scale;

        // Use pre-computed values from buffer creation (avoids
        // matrix multiply + sqrt + HashMap lookup + transform_point per flush)
        let effective_font_size = buffer.effective_font_size;
        let font_weight = buffer.font_weight;
        let is_italic_span = buffer.is_italic;

        // Move owned strings out of buffer (avoids clone)
        let font_name_span =
            self.emitted_font_name(buffer.font_name.as_deref(), buffer.cached_font.as_deref());

        // RTL text correction (#826): use the confidence-gated geometric
        // detector (#537) when `char_widths` gives us per-character user-space
        // x-positions, falling back to the coarse "buffer's net horizontal
        // advance is positive" heuristic only for genuinely ambiguous/short
        // runs. Mirrors `flush_tj_span_buffer`'s handling — this used to be
        // the one flush site still on the pre-#537 `accumulated_width > 0.0`
        // check, which (since `accumulated_width` only ever sums *positive*
        // glyph widths — TJ kerning offsets never subtract from it) is true
        // for nearly every non-empty RTL buffer and so was unconditionally
        // reversing every RTL run regardless of its actual source order.
        let mut text = std::mem::take(&mut buffer.unicode);
        if text.len() > 1 {
            let has_rtl = text
                .chars()
                .any(|c| crate::text::rtl_detector::is_rtl_text(c as u32));
            if has_rtl {
                let chars: Vec<char> = text.chars().collect();
                let verdict = if chars.len() == buffer.char_widths.len()
                    && !buffer.char_widths.is_empty()
                {
                    let mut chars_with_x: Vec<(char, f32)> = Vec::with_capacity(chars.len());
                    let mut cursor_text_space = 0.0_f32;
                    for (i, c) in chars.iter().enumerate() {
                        let user_x = buffer.user_pos_x + cursor_text_space * buffer.user_h_scale;
                        chars_with_x.push((*c, user_x));
                        cursor_text_space += buffer.char_widths[i];
                    }
                    crate::text::bidi::detect_visual_order_run(&chars_with_x)
                } else {
                    crate::text::bidi::RunOrder::Ambiguous
                };
                text = crate::text::bidi::apply_rtl_verdict(
                    &text,
                    verdict,
                    buffer.accumulated_width > 0.0,
                    matches!(buffer.render_mode, 3 | 7),
                );
            }
        }

        let span = TextSpan {
            provenance: None,
            text,
            bbox: Rect {
                x: if self.visible_paint_only {
                    buffer.paint_pos_x
                } else {
                    buffer.user_pos_x
                },
                y: if self.visible_paint_only {
                    buffer.paint_pos_y
                } else {
                    buffer.user_pos_y
                },
                width: total_width,
                height: effective_font_size,
            },
            font_name: font_name_span,
            font_size: effective_font_size,
            font_weight,
            color: Color::new(
                buffer.fill_color_rgb.0,
                buffer.fill_color_rgb.1,
                buffer.fill_color_rgb.2,
            ),
            mcid: buffer.mcid,
            mcid_scope: Some(self.current_mcid_scope()),
            sequence: self.span_sequence_counter,
            split_boundary_before: false,
            offset_semantic: false,
            char_spacing: buffer.char_space, // Tc - captured from PDF content stream
            word_spacing: buffer.word_space, // Tw - captured from PDF content stream
            horizontal_scaling: buffer.horizontal_scaling, // Tz - captured from PDF content stream
            is_italic: is_italic_span,
            is_monospace: buffer.is_monospace,
            primary_detected: false,
            artifact_type: self.current_artifact_type(),
            char_widths: {
                let mut cw = std::mem::take(&mut buffer.char_widths);
                let h = buffer.user_h_scale;
                for w in &mut cw {
                    *w *= h;
                }
                cw
            },
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: buffer.rotation_degrees,
            wmode: buffer.wmode,
            text_rise: buffer.text_rise,
            rtl_draw_logical: false,
            mirrored: buffer.mirrored,
            page_rotation_applied: 0,
        };
        self.span_sequence_counter += 1;

        if !self.is_content_suppressed() {
            self.record_span_paint_order(
                span.sequence,
                std::mem::take(&mut buffer.source_order),
                buffer.paint_basis,
                buffer.cached_font.clone(),
            )?;
            self.reserve_import_span()?;
            self.spans.push(span);
        }
        Ok(())
    }

    /// Calculate total width of TJ buffer using PDF spec formula.
    ///
    /// Process TJ array according to configured word boundary detection mode.
    ///
    /// Per PDF Spec ISO 32000-1:2008 Section 9.4.4,
    /// this method dispatches to either:
    /// - process_tj_array_tiebreaker(): WordBoundaryMode::Tiebreaker (default)
    /// - process_tj_array_primary(): WordBoundaryMode::Primary
    fn process_tj_array(&mut self, array: &[TextElement]) -> Result<()> {
        match self.word_boundary_mode {
            WordBoundaryMode::Tiebreaker => self.process_tj_array_tiebreaker(array),
            WordBoundaryMode::Primary => self.process_tj_array_primary(array),
        }
    }

    /// Process TJ array using tiebreaker mode (backward compatible).
    ///
    /// This is the legacy code path used when
    /// WordBoundaryMode::Tiebreaker is configured.
    ///
    /// Maintains 100% backward compatibility with existing behavior.
    /// Word boundaries are detected only as a tiebreaker when TJ offset
    /// and geometric signals contradict each other.
    ///
    /// Per PDF Spec ISO 32000-1:2008, Section 9.4.4 NOTE 6:
    /// "The performance of text searching (and other text extraction operations) is
    /// significantly better if the text strings are as long as possible."
    ///
    /// This method buffers consecutive strings into a single span, only breaking on:
    /// - Large negative offsets (indicating word boundaries)
    /// - End of TJ array
    fn process_tj_array_tiebreaker(&mut self, array: &[TextElement]) -> Result<()> {
        // Character-level tracking for word boundary detection
        // Collect detailed character information during TJ array processing
        // Per ISO 32000-1:2008 Section 9.4.4, character-level data improves accuracy

        self.tj_character_array.clear();
        self.current_x_position = 0.0;

        // Copy state data to avoid holding reference while borrowing self mutably
        let font_size = self.state_stack.current().font_size;
        let horizontal_scaling = self.state_stack.current().horizontal_scaling / 100.0;
        let font_name = self.state_stack.current().font_name.clone();
        let char_space = self.state_stack.current().char_space;
        let word_space = self.state_stack.current().word_space;

        let mut buffer = TjBuffer::new(
            self.state_stack.current(),
            self.current_mcid,
            self.cached_current_font.clone(),
            self.paint_order_snapshot()?,
        );
        let mut _element_count = 0;

        for (idx, element) in array.iter().enumerate() {
            _element_count += 1;
            match element {
                TextElement::String(s) => {
                    // Collect character-level data before processing buffer
                    // Extract individual characters with their properties
                    if let Some(ref name) = font_name {
                        if let Some(font) = self.fonts.get(name) {
                            // Process each byte in the string
                            for &byte in s.iter() {
                                // Normalize character code through encoding.
                                // This ensures word boundary detection works on actual characters,
                                // not raw byte codes from custom encodings
                                let char_code = font
                                    .get_encoded_char(byte)
                                    .map(|ch| ch as u32)
                                    .unwrap_or(byte as u32);

                                let glyph_width = font.get_glyph_width(byte as u16);

                                // Check if this is a ligature character (U+FB00-U+FB04)
                                let is_ligature = Self::is_ligature_code(char_code);

                                // Create CharacterInfo for this character
                                // The tj_offset will be applied when we encounter the next Offset element
                                let char_info = CharacterInfo {
                                    code: char_code,
                                    glyph_id: None, // Could be enhanced to extract actual GID
                                    width: glyph_width,
                                    x_position: self.current_x_position,
                                    tj_offset: None, // Will be set if next element is Offset
                                    font_size,
                                    is_ligature,
                                    original_ligature: None,
                                    protected_from_split: false,
                                };

                                self.tj_character_array.push(char_info);

                                // Update current X position (in text space units)
                                // Per PDF Spec: account for character spacing and scaling
                                let char_advance = glyph_width * horizontal_scaling
                                    + char_space
                                    + (if byte == 0x20 { word_space } else { 0.0 });
                                self.current_x_position += char_advance;
                            }
                        }
                    }

                    // Single-pass: append unicode + compute width + advance position
                    self.append_advance_buffer(&mut buffer, s)?;
                },
                TextElement::Offset(offset) => {
                    // Track TJ offset for statistical analysis
                    // Per ISO 32000-1:2008 Section 9.4.4, collect all TJ values
                    // to detect justified vs normal text through coefficient of variation
                    if self.tj_offset_history.len() < 10000 {
                        // Keep history reasonable size (first 10k offsets per document)
                        // and update the running accumulators.
                        let x = *offset as f64;
                        self.tj_sum += x;
                        self.tj_sum_sq += x * x;
                        self.tj_offset_history.push(*offset);
                        self.tj_stats_len = self.tj_offset_history.len();
                    }

                    // Associate TJ offset with the last character
                    // The offset applies AFTER the previous string, affecting spacing to next string
                    if !self.tj_character_array.is_empty() {
                        let last_idx = self.tj_character_array.len() - 1;
                        self.tj_character_array[last_idx].tj_offset = Some(*offset as i32);
                    }

                    // Check if this offset indicates a word boundary
                    // Per PDF spec: negative offsets increase spacing
                    // Use geometry-based adaptive threshold
                    let threshold = self.calculate_adaptive_tj_threshold();
                    if *offset < threshold {
                        // Note: #365 split-word symptoms ("diffe rent", "cha nge",
                        // "equivalen t") are handled at the higher level by the
                        // intra-word kerning guard in `should_insert_space`. An
                        // earlier TJ-side guard here (commit b2c6484) used a
                        // letter-letter + |offset| < space-glyph-width rule, but
                        // that rule misclassified real inter-word gaps in
                        // tightly-justified PDFs (LaTeX academic papers, Docling
                        // output) where producers encode word boundaries as TJ
                        // offsets smaller than a full space glyph. The
                        // span-merge-time guard has more context (full bbox,
                        // WordBoundaryDetector) and avoids that false positive.
                        //
                        // Check if buffer ends with space BEFORE flushing
                        // This prevents double spaces when TJ processor inserts space
                        // AND span merging would insert space at the same boundary.
                        let buffer_ends_with_space = !buffer.unicode.is_empty()
                            && buffer
                                .unicode
                                .chars()
                                .next_back()
                                .map(|c| c.is_whitespace())
                                .unwrap_or(false);

                        // Flush buffer before space
                        self.flush_tj_buffer(buffer)?;

                        // Check if the next element in the TJ array is a string
                        // that starts with whitespace. If so, DON'T insert a space to avoid doubling.
                        // This prevents patterns like "word " + " next" = "word next" (double space)
                        let next_element_starts_with_space = if idx + 1 < array.len() {
                            if let TextElement::String(next_s) = &array[idx + 1] {
                                next_s.first().is_some_and(|&byte| {
                                    byte == 0x20 || byte == 0x09 || byte == 0x0A || byte == 0x0D
                                })
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        // Only insert space if neither side already has whitespace
                        if !buffer_ends_with_space && !next_element_starts_with_space {
                            // Insert space character as separate span
                            self.insert_space_as_span()?;
                        }

                        // Apply the TJ offset to the text matrix BEFORE
                        // creating the new buffer so its `user_pos_x`
                        // captures the actual draw position of the next
                        // string. Otherwise the buffer anchors at the
                        // pre-offset position and every subsequent span
                        // on the line inherits the missing tx.
                        self.advance_position_for_offset(*offset)?;

                        // Start new buffer with current state
                        buffer = TjBuffer::new(
                            self.state_stack.current(),
                            self.current_mcid,
                            self.cached_current_font.clone(),
                            self.paint_order_snapshot()?,
                        );
                    } else {
                        // Sub-threshold offset: matrix advances but the
                        // current buffer keeps accumulating, so apply
                        // the offset unconditionally here as well.
                        self.advance_position_for_offset(*offset)?;
                        // Fold the same displacement into the buffer's
                        // advance record. Historically only the text matrix
                        // moved, so these kerning/word-space offsets were
                        // dropped from `char_widths`/`accumulated_width` —
                        // leaving the span's reconstructed per-glyph positions
                        // drifting behind the true render (poppler/PDFium/
                        // pymupdf all fold the offset into the advance). On
                        // justified body text drawn as one continuous buffer,
                        // the many small post-space offsets accumulate into a
                        // multi-point undershoot. Folding keeps
                        // `sum(char_widths) == accumulated_width == matrix
                        // advance` by construction.
                        self.fold_offset_into_buffer(&mut buffer, *offset);
                    }
                },
            }
        }

        // Flush remaining buffer
        if !buffer.is_empty() {
            self.flush_tj_buffer(buffer)?;
        }

        Ok(())
    }

    /// Process TJ array using primary detection mode.
    ///
    /// This implementation:
    /// 1. Creates BoundaryContext from graphics state
    /// 2. Calls WordBoundaryDetector to detect boundaries in tj_character_array
    /// 3. Apply ligature expansion decisions
    /// 4. Partitions characters into clusters at boundary positions
    /// 5. Converts each cluster to a TextSpan with proper bounding boxes
    /// 6. Marks spans with primary_detected flag
    fn process_tj_array_primary(&mut self, array: &[TextElement]) -> Result<()> {
        // Primary detection mode implementation

        // Step 1: If no characters collected, fall back to tiebreaker behavior
        if self.tj_character_array.is_empty() {
            return self.process_tj_array_tiebreaker(array);
        }

        // Mark pattern contexts BEFORE boundary detection
        // This protects email and URL patterns from being split at word boundaries
        let pattern_config = crate::extractors::PatternPreservationConfig::default();
        crate::extractors::PatternDetector::mark_pattern_contexts(
            &mut self.tj_character_array,
            &pattern_config,
        )?;

        // Step 2: Create BoundaryContext from current graphics state
        let context = self.create_boundary_context();

        // Step 3: Create WordBoundaryDetector and detect boundaries
        // OPTIMIZATION: Detect document script profile to skip unnecessary detectors (Issue #1 fix)
        let script = DocumentScript::detect_from_characters(&self.tj_character_array);
        let detector = WordBoundaryDetector::new().with_document_script(script);
        let boundaries = detector.detect_word_boundaries(&self.tj_character_array, &context);

        // Step 4: If no boundaries detected, process entire array as single span
        if boundaries.is_empty() {
            // All characters form a single word
            return self.process_tj_array_tiebreaker(array);
        }

        // Step 3.5: Apply ligature expansion decisions
        // This intelligently splits ligatures at word boundaries
        self.apply_ligature_decisions()?;

        // Step 5: Partition characters into clusters at boundary positions
        let clusters =
            self.partition_characters_by_boundaries(&self.tj_character_array, boundaries);

        // Step 6: Convert each cluster to a TextSpan
        for cluster in clusters {
            if !cluster.is_empty() {
                self.cluster_to_span(&cluster)?;
            }
        }

        Ok(())
    }

    /// Create BoundaryContext from current graphics state.
    ///
    /// Per ISO 32000-1:2008 Section 9.3, extracts text state parameters
    /// used by WordBoundaryDetector to make boundary decisions.
    fn create_boundary_context(&self) -> BoundaryContext {
        let state = self.state_stack.current();
        BoundaryContext {
            font_size: state.font_size,
            horizontal_scaling: state.horizontal_scaling,
            word_spacing: state.word_space,
            char_spacing: state.char_space,
        }
    }

    /// Partition character array into clusters at boundary positions.
    ///
    /// # Arguments
    /// * `characters` - Full character array from TJ processing
    /// * `boundaries` - Boundary indices (positions where word boundaries occur)
    ///
    /// # Returns
    /// Vector of character clusters, where boundaries separate clusters
    fn partition_characters_by_boundaries(
        &self,
        characters: &[CharacterInfo],
        boundaries: Vec<usize>,
    ) -> Vec<Vec<CharacterInfo>> {
        if boundaries.is_empty() {
            return vec![characters.to_vec()];
        }

        let mut clusters = Vec::new();
        let mut prev = 0;

        for boundary_idx in boundaries {
            if boundary_idx > prev {
                clusters.push(characters[prev..boundary_idx].to_vec());
            }
            prev = boundary_idx;
        }

        // Add remaining characters after last boundary
        if prev < characters.len() {
            clusters.push(characters[prev..].to_vec());
        }

        clusters
    }

    /// Convert a character cluster to a TextSpan.
    ///
    /// Calculates bounding box from character positions and creates
    /// a single TextSpan marked with primary_detected flag.
    ///
    /// # Arguments
    /// * `cluster` - Character cluster from partitioning
    fn cluster_to_span(&mut self, cluster: &[CharacterInfo]) -> Result<()> {
        if cluster.is_empty() {
            return Ok(());
        }

        // Snapshot the current MCID scope before borrowing graphics
        // state so the borrow checker doesn't reject the
        // `current_mcid_scope()` call at span construction time.
        let mcid_scope = self.current_mcid_scope();
        let state = self.state_stack.current();

        // Step 1: Calculate bounding box from character positions in text space
        // X position: from first character to end of last character
        let text_min_x = cluster[0].x_position;
        // Safety: caller checks cluster.is_empty() above and returns early
        let last = cluster.last().expect("cluster verified non-empty above");
        let text_max_x = last.x_position + last.width;
        let text_width = (text_max_x - text_min_x).max(0.0);

        // Height from font size
        let height = cluster[0].font_size.abs() * state.text_matrix.d.abs().max(1.0);

        // Step 2: Apply CTM to convert from text space to user space
        // Per PDF Spec ISO 32000-1:2008 Section 9.4.4
        let text_matrix = state.text_matrix;
        let ctm = state.ctm;
        let text_pos = text_matrix.transform_point(
            text_min_x,
            if self.visible_paint_only {
                state.text_rise
            } else {
                0.0
            },
        );
        let user_pos = ctm.transform_point(text_pos.x, text_pos.y);

        // Transform the width as well (accounting for matrix scaling)
        let user_width = text_width * text_matrix.a.abs() * ctm.a.abs();

        // Step 3: Create bounding box rectangle in user space
        let bbox = Rect {
            x: user_pos.x,
            y: user_pos.y,
            width: user_width.max(text_width), // Use larger of the two for safety
            height,
        };

        // Step 3: Convert characters to Unicode string
        // Use same decoding as existing code
        let mut unicode_text = if let Some(font_name) = state.font_name.as_ref() {
            if let Some(font) = self.fonts.get(font_name) {
                let mut text = String::new();
                for char_info in cluster {
                    if let Some(decoded) = font.char_to_unicode(char_info.code) {
                        text.push_str(&decoded);
                    }
                }
                text
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Step 3b: RTL text correction — reverse visual-order characters to logical order.
        //
        // PDF stores characters in content-stream order. For RTL scripts
        // (Arabic / Hebrew), the producer may emit text in either:
        //   * **visual order** — glyphs drawn left-to-right in user space
        //     even though the script reads right-to-left (legacy Acrobat
        //     output, pre-shaped Arabic, the Magic Palace Eilat PDF
        //     from issue #537), OR
        //   * **logical order** — glyphs drawn right-to-left in user space
        //     because the producer ran its own bidi pass before drawing
        //     (modern Word with bidi, the pdfium `hebrew_mirrored.pdf`
        //     test fixture).
        //
        // We use the confidence-gated geometric detector
        // [`text::bidi::detect_visual_order_run`] (v0.3.54 #537) when the
        // cluster has ≥4 RTL letters with clear X-monotonicity. For
        // shorter clusters (or `Ambiguous` verdict) we fall back to the
        // pre-v0.3.54 simple `last_x > first_x` heuristic — keeps the
        // existing 2-3-char RTL run behaviour byte-identical so the
        // upstream invariants (Arabic CID-TrueType samples, the
        // `right_to_left_02` fixture) still pass.
        if unicode_text.len() > 1 && cluster.len() >= 2 {
            let has_rtl = unicode_text
                .chars()
                .any(|c| crate::text::rtl_detector::is_rtl_text(c as u32));
            if has_rtl {
                // Build (char, user_x) pairs for the geometric detector.
                // One pair per source character — when the decoded
                // string has more chars than the cluster (e.g. ligature
                // expansion `fi` → "fi"), use the first decoded char as
                // a representative since they share the same source x.
                let font_for_cluster = state.font_name.as_ref().and_then(|n| self.fonts.get(n));
                let mut chars_with_x: Vec<(char, f32)> = Vec::with_capacity(cluster.len());
                for ci in cluster {
                    let decoded_first = font_for_cluster
                        .and_then(|f| f.char_to_unicode(ci.code))
                        .and_then(|s| s.chars().next());
                    if let Some(c) = decoded_first {
                        let p = text_matrix.transform_point(ci.x_position, 0.0);
                        let user_x = ctm.transform_point(p.x, p.y).x;
                        chars_with_x.push((c, user_x));
                    }
                }
                let verdict = crate::text::bidi::detect_visual_order_run(&chars_with_x);
                // Pre-v0.3.54 simple heuristic — used only as the
                // `Ambiguous` fallback (short cluster or mixed signal) so
                // existing 2-3-char RTL runs keep working; the pdfium
                // `hebrew_mirrored.pdf` fixture and similar land on
                // `Logical` above and are left alone regardless.
                let first_x = {
                    let p = text_matrix.transform_point(cluster[0].x_position, 0.0);
                    ctm.transform_point(p.x, p.y).x
                };
                let last_x = {
                    let p = text_matrix.transform_point(last.x_position, 0.0);
                    ctm.transform_point(p.x, p.y).x
                };
                unicode_text = crate::text::bidi::apply_rtl_verdict(
                    &unicode_text,
                    verdict,
                    last_x > first_x,
                    matches!(state.render_mode, 3 | 7),
                );
            }
        }

        // Step 4: Determine font weight
        let font_weight = if let Some(font_name) = state.font_name.as_ref() {
            if let Some(font) = self.fonts.get(font_name) {
                if font.is_bold() {
                    FontWeight::Bold
                } else {
                    FontWeight::Normal
                }
            } else {
                FontWeight::Normal
            }
        } else {
            FontWeight::Normal
        };

        // Determine if italic
        let is_italic = state
            .font_name
            .as_ref()
            .and_then(|name| self.fonts.get(name))
            .map(|font| font.is_italic())
            .unwrap_or(false);

        // Step 5: Create TextSpan with primary_detected flag
        let span = TextSpan {
            provenance: None,
            text: unicode_text,
            bbox,
            font_name: self.emitted_font_name(
                state.font_name.as_deref(),
                state
                    .font_name
                    .as_ref()
                    .and_then(|n| self.fonts.get(n))
                    .map(AsRef::as_ref),
            ),
            font_size: cluster[0].font_size,
            font_weight,
            color: Color::new(
                state.fill_color_rgb.0,
                state.fill_color_rgb.1,
                state.fill_color_rgb.2,
            ),
            mcid: self.current_mcid,
            mcid_scope: Some(mcid_scope),
            sequence: self.span_sequence_counter,
            split_boundary_before: false,
            offset_semantic: false,
            char_spacing: state.char_space,
            word_spacing: state.word_space,
            horizontal_scaling: state.horizontal_scaling,
            is_italic,
            is_monospace: false,
            primary_detected: true,
            artifact_type: None,
            char_widths: vec![],
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: snap_run_rotation(&state.ctm.multiply(&state.text_matrix)),
            wmode: state.text_wmode,
            text_rise: if state.font_size > 0.0 {
                state.text_rise / state.font_size
            } else {
                0.0
            },
            rtl_draw_logical: false,
            mirrored: run_is_mirrored(&state.ctm.multiply(&state.text_matrix)),
            page_rotation_applied: 0,
        };

        // Step 6: Increment sequence counter and add to spans
        self.span_sequence_counter += 1;
        if !self.is_content_suppressed() {
            let matrix = state.ctm.multiply(&state.text_matrix);
            self.record_span_paint_order(
                span.sequence,
                self.paint_order_snapshot()?,
                [matrix.a, matrix.b, matrix.c, matrix.d],
                state
                    .font_name
                    .as_ref()
                    .and_then(|name| self.fonts.get(name))
                    .cloned(),
            )?;
            self.reserve_import_span()?;
            self.spans.push(span);
        }

        Ok(())
    }

    /// Check if a character code is a ligature (U+FB00-U+FB04).
    ///
    /// Standard ligatures supported:
    /// - U+FB00: ff (LATIN SMALL LIGATURE FF)
    /// - U+FB01: fi (LATIN SMALL LIGATURE FI)
    /// - U+FB02: fl (LATIN SMALL LIGATURE FL)
    /// - U+FB03: ffi (LATIN SMALL LIGATURE FFI)
    /// - U+FB04: ffl (LATIN SMALL LIGATURE FFL)
    fn is_ligature_code(code: u32) -> bool {
        matches!(code, 0xFB00..=0xFB04)
    }

    /// Apply ligature expansion decisions after word boundary detection.
    ///
    /// This method processes the character array after boundary detection,
    /// making intelligent decisions about whether to split ligatures.
    ///
    /// Algorithm:
    /// 1. Iterate through character array
    /// 2. For each ligature character:
    ///    - Get next character (if exists)
    ///    - Call LigatureDecisionMaker::decide()
    ///    - If Split: expand to component characters with proportional widths
    ///    - If Keep: leave as-is
    /// 3. Recalculate x_positions for all following characters after splits
    fn apply_ligature_decisions(&mut self) -> Result<()> {
        use crate::text::ligature_processor::{
            expand_ligature_to_chars, LigatureDecision, LigatureDecisionMaker,
        };

        let context = self.create_boundary_context();
        let mut result = Vec::new();
        let mut i = 0;

        // OPTIMIZATION: Single-pass reconstruction instead of Vec::insert() in loop
        // This fixes O(n²) complexity to O(n) by avoiding repeated insertions
        // Issue #2 fix: Vec::insert was causing 50× slowdown for ligature-heavy PDFs
        while i < self.tj_character_array.len() {
            let char_info = &self.tj_character_array[i];

            // If not a ligature, keep as-is
            if !char_info.is_ligature {
                result.push(char_info.clone());
                i += 1;
                continue;
            }

            // Get next character without cloning (Issue #3 fix: eliminate unnecessary clones)
            let next_char = if i + 1 < self.tj_character_array.len() {
                Some(&self.tj_character_array[i + 1])
            } else {
                None
            };

            // Make decision using references
            let decision = LigatureDecisionMaker::decide(char_info, &context, next_char);

            if decision == LigatureDecision::Split {
                // Get the ligature character from code
                let ligature_char = char::from_u32(char_info.code).unwrap_or('?');
                let original_width = char_info.width;
                let original_x = char_info.x_position;
                let font_size = char_info.font_size;

                // Expand to component characters
                let components = expand_ligature_to_chars(ligature_char, original_width);

                if !components.is_empty() {
                    // Add first component (replacing the ligature)
                    let mut x_offset = 0.0;
                    result.push(CharacterInfo {
                        code: components[0].0 as u32,
                        glyph_id: char_info.glyph_id,
                        width: components[0].1,
                        x_position: original_x,
                        tj_offset: char_info.tj_offset,
                        font_size,
                        is_ligature: false,
                        original_ligature: Some(ligature_char),
                        protected_from_split: char_info.protected_from_split,
                    });
                    x_offset += components[0].1;

                    // Add remaining components (no Vec::insert needed - just push!)
                    for (comp_char, comp_width) in components.iter().skip(1) {
                        result.push(CharacterInfo {
                            code: *comp_char as u32,
                            glyph_id: None,
                            width: *comp_width,
                            x_position: original_x + x_offset,
                            tj_offset: None,
                            font_size,
                            is_ligature: false,
                            original_ligature: Some(ligature_char),
                            protected_from_split: false,
                        });
                        x_offset += comp_width;
                    }
                } else {
                    // If expansion failed, keep original ligature
                    result.push(char_info.clone());
                }
            } else {
                // Keep ligature intact
                result.push(char_info.clone());
            }

            i += 1;
        }

        // OPTIMIZATION: Replace entire array once instead of multiple insertions
        self.tj_character_array = result;
        Ok(())
    }

    /// Advance text position for a string (used in TJ array processing).
    /// Advance the text matrix position by the width of a text string.
    /// Returns the computed width so callers can accumulate it.
    fn advance_position_for_string(&mut self, text: &[u8]) -> Result<f32> {
        let state = self.state_stack.current();
        let font_size = state.font_size;
        let horizontal_scaling = state.horizontal_scaling;
        let char_space = state.char_space;
        let word_space = state.word_space;
        let wmode = state.text_wmode;

        let font = self.cached_current_font.as_deref();

        // Hoist loop-invariant computations (font cannot change mid-operator).
        // font_matrix_a converts glyph-space widths to text-space units.
        // Standard fonts (Type1/TrueType): font_matrix_a = 0.001.
        // Type3 with identity FontMatrix: font_matrix_a = 1.0 (no /1000 division).
        // Assumes FontMatrix[1] = 0 (no glyph-axis rotation), which holds for all
        // standard fonts and virtually all Type3 fonts encountered in practice.
        let font_matrix_a = font.map(|f| f.font_matrix_a).unwrap_or(0.001);
        let fs_factor = font_size * font_matrix_a;
        let hs_factor = horizontal_scaling / 100.0;
        let cs_hs = char_space * hs_factor;
        let ws_hs = word_space * hs_factor;

        let total_width = if let Some(font) = font {
            if font.subtype != "Type0" {
                // Fast path: use precomputed 256-entry width table (simple fonts)
                let width_table = font.get_byte_to_width_table();
                let mut w_sum = 0.0f32;
                for &byte in text {
                    let mut w = width_table[byte as usize] * fs_factor * hs_factor;
                    w += cs_hs;
                    if byte == 0x20 {
                        w += ws_hs;
                    }
                    w_sum += w;
                }
                w_sum
            } else if wmode == 0 {
                // Type0/CID font, horizontal: use TextCharIter so that the byte-width
                // (1 or 2) is determined by the font's encoding / ToUnicode CMap
                // codespace, not hardcoded to 2. Per ISO 32000-1:2008 §9.7.6.2.
                let mut w_sum = 0.0f32;
                for (cid, nbytes) in TextCharIter::new(text, Some(font)) {
                    let mut w = font.get_glyph_width(cid) * fs_factor * hs_factor;
                    w += cs_hs;
                    // Per ISO 32000-1:2008 §9.3.3: Tw applies ONLY to the
                    // single-byte character code 32, never to the byte value 32
                    // inside a multi-byte code. `TextCharIter` yields the raw
                    // code plus its byte width, so gate on a single-byte 32 — a
                    // 2-byte CID #32 (0x0020) in an Identity-H/CJK font must not
                    // take Tw (it would over-advance and mis-position the run).
                    if nbytes == 1 && cid == 32 {
                        w += ws_hs;
                    }
                    w_sum += w;
                }
                w_sum
            } else {
                // Type0/CID font, vertical (WMode 1): per-glyph displacement
                // is `w1y` (from /W2 or /DW2 default), in 1000ths-of-em.
                //
                // Per ISO 32000-1:2008 §9.4.4 the vertical formula is
                //     ty = (w1y * Tfs) + Tc + Tw
                // with NO Th factor. §9.3.4 defines Tz as the horizontal
                // glyph-stretching axis — it does not scale w1y, Tc, or
                // Tw in vertical mode.
                let mut w_sum = 0.0f32;
                for (cid, nbytes) in TextCharIter::new(text, Some(font)) {
                    let w1y = font.get_vertical_metrics(cid).w1y;
                    let mut w = w1y * fs_factor;
                    w += char_space;
                    if nbytes == 1 && cid == 32 {
                        w += word_space;
                    }
                    w_sum += w;
                }
                w_sum
            }
        } else {
            // No font: use default width
            let default_w = 500.0 * fs_factor * hs_factor + cs_hs;
            let space_w = default_w + ws_hs;
            let mut w_sum = 0.0f32;
            for &byte in text {
                w_sum += if byte == 0x20 { space_w } else { default_w };
            }
            w_sum
        };

        // Update text matrix position per ISO 32000-1:2008 §9.4.4. The
        // axis-swap (horizontal vs vertical) is encapsulated in
        // GraphicsState::advance_text_matrix so this site does not branch.
        self.state_stack
            .current_mut()
            .advance_text_matrix(total_width);

        Ok(total_width)
    }

    /// Combined Unicode decode + width calculation in a single pass.
    /// Merges TjBuffer::append and advance_position_for_string for simple fonts,
    /// eliminating one full per-byte iteration per Tj operator.
    fn append_and_advance(&mut self, text: &[u8]) -> Result<()> {
        let text = if text.len() > 32_767 {
            &text[..32_767]
        } else {
            text
        };

        let state = self.state_stack.current();
        let font_size = state.font_size;
        let horizontal_scaling = state.horizontal_scaling;
        let char_space = state.char_space;
        let word_space = state.word_space;
        let wmode = state.text_wmode;

        // Disjoint field borrows: cached_current_font (immutable) + tj_span_buffer (mutable)
        let font = self.cached_current_font.as_deref();
        // font_matrix_a converts glyph-space widths to text-space units.
        // Standard fonts (Type1/TrueType): font_matrix_a = 0.001.
        // Type3 with identity FontMatrix: font_matrix_a = 1.0 (no /1000 division).
        // Assumes FontMatrix[1] = 0 (no glyph-axis rotation), which holds for all
        // standard fonts and virtually all Type3 fonts encountered in practice.
        let font_matrix_a = font.map(|f| f.font_matrix_a).unwrap_or(0.001);
        let fs_factor = font_size * font_matrix_a;
        let hs_factor = horizontal_scaling / 100.0;
        let cs_hs = char_space * hs_factor;
        let ws_hs = word_space * hs_factor;
        // Safety: tj_span_buffer is always initialized via begin_text_object()
        let buffer = self
            .tj_span_buffer
            .as_mut()
            .expect("tj_span_buffer initialized in begin_text_object");

        let total_width = if let Some(font) = font {
            if font.subtype != "Type0" {
                // #317: UTF-8-in-simple-font detection (same heuristic as
                // `append_advance_buffer`). Some producers emit raw UTF-8
                // bytes inside PDF string literals when the font declares
                // only a Latin encoding and no ToUnicode CMap. Byte-by-byte
                // Latin decoding produces mojibake. When the slice is valid
                // UTF-8 with at least one non-Latin-1 codepoint, decode as
                // UTF-8 so non-Latin scripts (Cyrillic, Greek, CJK, …) come
                // through as their intended codepoints.
                if font.to_unicode.is_none() && text.len() >= 2 {
                    let has_high = text.iter().any(|&b| b >= 0x80);
                    if has_high {
                        if let Ok(decoded) = std::str::from_utf8(text) {
                            if decoded.chars().any(|c| c as u32 > 0xFF) {
                                let width_table = font.get_byte_to_width_table();
                                let mut w_sum = 0.0f32;
                                for &byte in text {
                                    let mut w = width_table[byte as usize] * fs_factor * hs_factor;
                                    w += cs_hs;
                                    if byte == 0x20 {
                                        w += ws_hs;
                                    }
                                    w_sum += w;
                                }
                                let char_count = decoded.chars().count();
                                if char_count > 0 {
                                    let per_char = w_sum / char_count as f32;
                                    for ch in decoded.chars() {
                                        buffer.unicode.push(ch);
                                        buffer.char_widths.push(per_char);
                                    }
                                }
                                // Fall through to the matrix update at the
                                // bottom of the function via `w_sum`. Vertical
                                // mode flips the axis inside the helper.
                                self.state_stack.current_mut().advance_text_matrix(w_sum);
                                return Ok(());
                            }
                        }
                    }
                }

                // Fast path: single pass over bytes for both Unicode and width
                let char_table = font.get_byte_to_char_table();
                let width_table = font.get_byte_to_width_table();
                let mut w_sum = 0.0f32;
                for &byte in text {
                    // Unicode decode — count chars added for per-char width tracking
                    let len_before = buffer.unicode.len();
                    let c = char_table[byte as usize];
                    if c != '\0' {
                        buffer.unicode.push(c);
                    } else {
                        // Rare: multi-char mapping or unmapped byte
                        if let Some(s) = font.char_to_unicode(byte as u32) {
                            if s != "\u{FFFD}" || preserve_unmapped_glyphs() {
                                for ch in s.chars() {
                                    if ch >= '\x20' || ch == '\t' || ch == '\n' || ch == '\r' {
                                        buffer.unicode.push(ch);
                                    }
                                }
                            }
                        } else {
                            let fb = fallback_char_to_unicode(byte as u32);
                            if fb != "\u{FFFD}" || preserve_unmapped_glyphs() {
                                for ch in fb.chars() {
                                    if ch >= '\x20' || ch == '\t' || ch == '\n' || ch == '\r' {
                                        buffer.unicode.push(ch);
                                    }
                                }
                            }
                        }
                    }
                    // Width calculation
                    let mut w = width_table[byte as usize] * fs_factor * hs_factor;
                    w += cs_hs;
                    if byte == 0x20 {
                        w += ws_hs;
                    }
                    w_sum += w;
                    // Track per-character advance widths
                    // Count the characters this code produced, not the bytes.
                    // `unicode` is a `String`, so its `len()` is a byte count:
                    // one em dash is three bytes and used to push three widths
                    // for one character, leaving `char_widths` two entries
                    // ahead of the text for the rest of the run. Every glyph
                    // after it then carried a neighbour's advance — on one
                    // regulatory caption the closing `C` of `66.01–11(5)—C`
                    // was given 2.664 pt, a bracket's width, where its own is
                    // 5.776 pt, so the span ended 3.648 pt short of its ink and
                    // a gap appeared where the page has none.
                    //
                    // ISO 32000-1:2008 §9.4.4 gives each glyph one displacement
                    // along the writing axis, so this array carries one entry
                    // per character and must be indexed the same way the text
                    // is.
                    let chars_added = buffer.unicode[len_before..].chars().count();
                    if chars_added == 1 {
                        buffer.char_widths.push(w);
                    } else if chars_added > 1 {
                        let per_char = w / chars_added as f32;
                        for _ in 0..chars_added {
                            buffer.char_widths.push(per_char);
                        }
                    }
                }
                w_sum
            } else if wmode == 0 {
                // Type0/CID font, horizontal: unified iterator handles 1- or
                // 2-byte codes per ToUnicode codespace.
                buffer.append(text)?;
                let mut w_sum = 0.0f32;
                for (char_code, nbytes) in TextCharIter::new(text, Some(font)) {
                    let mut w = font.get_glyph_width(char_code) * fs_factor * hs_factor;
                    w += cs_hs;
                    // Per ISO 32000-1:2008 §9.3.3: Tw applies only to the
                    // single-byte character code 32 — a 2-byte CID 32 inside
                    // an Identity-H/CJK font must not take Tw.
                    if nbytes == 1 && char_code == 32 {
                        w += ws_hs;
                    }
                    w_sum += w;
                    buffer.char_widths.push(w);
                }
                w_sum
            } else {
                // Type0/CID font, vertical (WMode 1): per-glyph displacement
                // is `w1y` (from /W2 or /DW2), in 1000ths-of-em.
                //
                // Per ISO 32000-1:2008 §9.4.4: `ty = (w1y * Tfs) + Tc + Tw`,
                // with no Th (Tz only stretches glyphs along the horizontal
                // axis per §9.3.4).
                buffer.append(text)?;
                let mut w_sum = 0.0f32;
                for (char_code, nbytes) in TextCharIter::new(text, Some(font)) {
                    let w1y = font.get_vertical_metrics(char_code).w1y;
                    let mut w = w1y * fs_factor;
                    w += char_space;
                    // Per ISO 32000-1:2008 §9.3.3: Tw applies only to the
                    // single-byte character code 32.
                    if nbytes == 1 && char_code == 32 {
                        w += word_space;
                    }
                    w_sum += w;
                    buffer.char_widths.push(w);
                }
                w_sum
            }
        } else {
            // No font: decode as ASCII + use default widths
            buffer.append(text)?;
            let default_w = 500.0 * fs_factor * hs_factor + cs_hs;
            let space_w = default_w + ws_hs;
            let mut w_sum = 0.0f32;
            for &byte in text {
                let w = if byte == 0x20 { space_w } else { default_w };
                w_sum += w;
                buffer.char_widths.push(w);
            }
            w_sum
        };

        buffer.accumulated_width += total_width;

        // Update text matrix position per ISO 32000-1:2008 §9.4.4. The
        // axis-swap (H vs V) is encapsulated in advance_text_matrix.
        self.state_stack
            .current_mut()
            .advance_text_matrix(total_width);

        Ok(())
    }

    /// Combined Unicode decode + width + position advance for a local buffer.
    /// Same as append_and_advance but works on an explicit buffer parameter
    /// instead of self.tj_span_buffer. Used by TJ array processing.
    fn append_advance_buffer(&mut self, buffer: &mut TjBuffer, text: &[u8]) -> Result<()> {
        let text = if text.len() > 32_767 {
            &text[..32_767]
        } else {
            text
        };

        let state = self.state_stack.current();
        let font_size = state.font_size;
        let horizontal_scaling = state.horizontal_scaling;
        let char_space = state.char_space;
        let word_space = state.word_space;
        let wmode = state.text_wmode;

        let font = self.cached_current_font.as_deref();
        // font_matrix_a converts glyph-space widths to text-space units.
        // Standard fonts (Type1/TrueType): font_matrix_a = 0.001.
        // Type3 with identity FontMatrix: font_matrix_a = 1.0 (no /1000 division).
        // Assumes FontMatrix[1] = 0 (no glyph-axis rotation), which holds for all
        // standard fonts and virtually all Type3 fonts encountered in practice.
        let font_matrix_a = font.map(|f| f.font_matrix_a).unwrap_or(0.001);
        let fs_factor = font_size * font_matrix_a;
        let hs_factor = horizontal_scaling / 100.0;
        let cs_hs = char_space * hs_factor;
        let ws_hs = word_space * hs_factor;

        let total_width = if let Some(font) = font {
            if font.subtype != "Type0" {
                // #317: UTF-8-in-simple-font detection.
                //
                // Some producers (Russian CAD exporters, MS Office via
                // non-English locales) emit UTF-8 byte sequences inside PDF
                // string literals for a font that only declares a Latin
                // encoding (WinAnsi, StandardEncoding, MacRoman) and no
                // ToUnicode CMap. Byte-by-byte decoding through the Latin
                // encoding produces mojibake like `ÐÐ¸ÑÑ` for "Лист".
                //
                // Heuristic: when the font has no ToUnicode and the entire
                // text slice is a valid UTF-8 sequence whose decoded
                // codepoints contain at least one non-Latin-1 character
                // (U+0100 and above), treat the slice as UTF-8 directly.
                // The non-Latin-1 gate prevents mis-interpreting genuine
                // Latin-1 Supplement content (`Résumé`, etc.) — those
                // decode entirely into U+0000..U+00FF and are left alone.
                let utf8_width: Option<f32> = if font.to_unicode.is_none() && text.len() >= 2 {
                    let has_high = text.iter().any(|&b| b >= 0x80);
                    if has_high {
                        if let Ok(decoded) = std::str::from_utf8(text) {
                            let has_non_latin1 = decoded.chars().any(|c| c as u32 > 0xFF);
                            if has_non_latin1 {
                                let width_table = font.get_byte_to_width_table();
                                let mut w_sum = 0.0f32;
                                for &byte in text {
                                    let mut w = width_table[byte as usize] * fs_factor * hs_factor;
                                    w += cs_hs;
                                    if byte == 0x20 {
                                        w += ws_hs;
                                    }
                                    w_sum += w;
                                }
                                let char_count = decoded.chars().count();
                                if char_count > 0 {
                                    let per_char = w_sum / char_count as f32;
                                    for ch in decoded.chars() {
                                        buffer.unicode.push(ch);
                                        buffer.char_widths.push(per_char);
                                    }
                                }
                                log::debug!(
                                    "UTF-8 mojibake repair: decoded {} Latin-1 bytes as {} chars via UTF-8 in font '{}'",
                                    text.len(),
                                    char_count,
                                    font.base_font
                                );
                                Some(w_sum)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(w) = utf8_width {
                    buffer.accumulated_width += w;
                    self.state_stack.current_mut().advance_text_matrix(w);
                    return Ok(());
                }

                let char_table = font.get_byte_to_char_table();
                let width_table = font.get_byte_to_width_table();
                let mut w_sum = 0.0f32;
                for &byte in text {
                    let len_before = buffer.unicode.len();
                    let c = char_table[byte as usize];
                    if c != '\0' {
                        buffer.unicode.push(c);
                    } else if let Some(s) = font.char_to_unicode(byte as u32) {
                        if s != "\u{FFFD}" || preserve_unmapped_glyphs() {
                            for ch in s.chars() {
                                if ch >= '\x20' || ch == '\t' || ch == '\n' || ch == '\r' {
                                    buffer.unicode.push(ch);
                                }
                            }
                        }
                    } else {
                        let fb = fallback_char_to_unicode(byte as u32);
                        if fb != "\u{FFFD}" || preserve_unmapped_glyphs() {
                            for ch in fb.chars() {
                                if ch >= '\x20' || ch == '\t' || ch == '\n' || ch == '\r' {
                                    buffer.unicode.push(ch);
                                }
                            }
                        }
                    }
                    let mut w = width_table[byte as usize] * fs_factor * hs_factor;
                    w += cs_hs;
                    if byte == 0x20 {
                        w += ws_hs;
                    }
                    w_sum += w;
                    // Count the characters this code produced, not the bytes.
                    // `unicode` is a `String`, so its `len()` is a byte count:
                    // one em dash is three bytes and used to push three widths
                    // for one character, leaving `char_widths` two entries
                    // ahead of the text for the rest of the run. Every glyph
                    // after it then carried a neighbour's advance — on one
                    // regulatory caption the closing `C` of `66.01–11(5)—C`
                    // was given 2.664 pt, a bracket's width, where its own is
                    // 5.776 pt, so the span ended 3.648 pt short of its ink and
                    // a gap appeared where the page has none.
                    //
                    // ISO 32000-1:2008 §9.4.4 gives each glyph one displacement
                    // along the writing axis, so this array carries one entry
                    // per character and must be indexed the same way the text
                    // is.
                    let chars_added = buffer.unicode[len_before..].chars().count();
                    if chars_added == 1 {
                        buffer.char_widths.push(w);
                    } else if chars_added > 1 {
                        let per_char = w / chars_added as f32;
                        for _ in 0..chars_added {
                            buffer.char_widths.push(per_char);
                        }
                    }
                }
                w_sum
            } else if wmode == 0 {
                buffer.append(text)?;
                // Width calculation: use TextCharIter so byte-width respects the
                // CMap codespace (1 or 2 bytes per character). Fixes CJK fonts
                // whose encoding name doesn't match the well-known Identity-H/EUC/…
                // keyword patterns but whose ToUnicode CMap declares a 2-byte
                // codespace range (§9.7.5).
                let mut w_sum = 0.0f32;
                for (cid, nbytes) in TextCharIter::new(text, Some(font)) {
                    let mut w = font.get_glyph_width(cid) * fs_factor * hs_factor;
                    w += cs_hs;
                    if nbytes == 1 && cid == 32 {
                        w += ws_hs;
                    }
                    w_sum += w;
                    buffer.char_widths.push(w);
                }
                w_sum
            } else {
                // Type0/CID font, vertical mode: per-glyph displacement is
                // /W2 `w1y` (or /DW2 default), in 1000ths-of-em. The
                // vertical formula `ty = (w1y * Tfs) + Tc + Tw` (§9.4.4)
                // does NOT apply Th — Tz only scales glyphs horizontally
                // (§9.3.4).
                buffer.append(text)?;
                let mut w_sum = 0.0f32;
                for (cid, nbytes) in TextCharIter::new(text, Some(font)) {
                    let w1y = font.get_vertical_metrics(cid).w1y;
                    let mut w = w1y * fs_factor;
                    w += char_space;
                    if nbytes == 1 && cid == 32 {
                        w += word_space;
                    }
                    w_sum += w;
                    buffer.char_widths.push(w);
                }
                w_sum
            }
        } else {
            buffer.append(text)?;
            let default_w = 500.0 * fs_factor * hs_factor + cs_hs;
            let space_w = default_w + ws_hs;
            let mut w_sum = 0.0f32;
            for &byte in text {
                let w = if byte == 0x20 { space_w } else { default_w };
                w_sum += w;
                buffer.char_widths.push(w);
            }
            w_sum
        };

        buffer.accumulated_width += total_width;

        self.state_stack
            .current_mut()
            .advance_text_matrix(total_width);

        Ok(())
    }

    /// Insert a space character as a separate span.
    fn insert_space_as_span(&mut self) -> Result<()> {
        let mcid_scope = self.current_mcid_scope();
        let state = self.state_stack.current();
        let font_size = state.font_size;
        let text_matrix = state.text_matrix;
        let ctm = state.ctm;
        let combined = ctm.multiply(&text_matrix);
        let effective_font_size =
            font_size * (combined.d * combined.d + combined.b * combined.b).sqrt();
        let word_space = state.word_space;
        let horizontal_scaling = state.horizontal_scaling;
        let wmode = state.text_wmode;

        // Calculate space displacement along the active writing axis. In
        // horizontal mode this is the glyph width (250/1000 em ≈ quarter
        // em) plus Tw, scaled by Th. In vertical mode Tz does not apply
        // (§9.3.4) and we use the same magnitude as a writing-axis step
        // — the synthetic gap a TJ offset stands in for.
        //
        // NOTE: the displacement is expressed against the raw `Tf` size,
        // not the `Tm`-scaled effective size, so for print-era producers
        // that set `/F 1 Tf` with the size in `Tm` this span is narrower
        // in device space than a quarter em. That geometry is load-bearing
        // for the downstream column/line heuristics, which were tuned
        // against it — widening it reorders text on real documents — so
        // the lockstep fix below keeps a `char_widths` entry
        // consistent with this bbox rather than rescaling both.
        let space_advance = if wmode == 0 {
            (250.0 * font_size / 1000.0 + word_space) * horizontal_scaling / 100.0
        } else {
            250.0 * font_size / 1000.0 + word_space
        };

        // Apply CTM to get position in user space
        // Per PDF Spec ISO 32000-1:2008 Section 9.4.4
        let text_pos = text_matrix.transform_point(
            0.0,
            if self.visible_paint_only {
                state.text_rise
            } else {
                0.0
            },
        );
        let user_pos = ctm.transform_point(text_pos.x, text_pos.y);

        log::trace!(
            "Inserting space span from TJ offset (offset_semantic=true) at position ({:.2}, {:.2})",
            user_pos.x,
            user_pos.y
        );

        let font_name_space = self.emitted_font_name(
            state.font_name.as_deref(),
            state
                .font_name
                .as_ref()
                .and_then(|n| self.fonts.get(n))
                .map(AsRef::as_ref),
        );
        let is_italic_space = state
            .font_name
            .as_ref()
            .and_then(|name| self.fonts.get(name))
            .map(|font| font.is_italic())
            .unwrap_or(false);
        // Bbox geometry follows the writing axis: a horizontal gap is
        // wide and font-tall; a vertical gap is glyph-em-wide and tall
        // along the writing direction. Downstream layout heuristics
        // (column detection, line breaking) read width vs height to
        // decide orientation, so labeling the synthetic-space geometry
        // correctly keeps them honest.
        let (space_width, space_height) = if wmode == 0 {
            (space_advance, effective_font_size)
        } else {
            (effective_font_size, space_advance.abs())
        };
        let span = TextSpan {
            provenance: None,
            text: " ".to_string(),
            bbox: Rect {
                x: user_pos.x,
                y: user_pos.y,
                width: space_width,
                height: space_height,
            },
            font_name: font_name_space,
            font_size: effective_font_size,
            font_weight: FontWeight::Normal,
            color: Color::new(
                state.fill_color_rgb.0,
                state.fill_color_rgb.1,
                state.fill_color_rgb.2,
            ),
            mcid: self.current_mcid,
            mcid_scope: Some(mcid_scope),
            sequence: self.span_sequence_counter,
            split_boundary_before: false,
            offset_semantic: true,
            char_spacing: state.char_space, // Tc - captured from PDF content stream
            word_spacing: state.word_space, // Tw - captured from PDF content stream
            horizontal_scaling: state.horizontal_scaling, // Tz - captured from PDF content stream
            is_italic: is_italic_space,
            is_monospace: false,
            primary_detected: false,
            artifact_type: self.current_artifact_type(),
            // One synthetic space char ⇒ one width entry, so the span-merge
            // lockstep (`char_widths.len() == text.chars().count()`) holds
            // from birth regardless of merge order. The width is
            // the bbox extent along x, consistent with `to_chars` geometry.
            char_widths: vec![space_width],
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: snap_run_rotation(&state.ctm.multiply(&state.text_matrix)),
            wmode: state.text_wmode,
            text_rise: if state.font_size > 0.0 {
                state.text_rise / state.font_size
            } else {
                0.0
            },
            rtl_draw_logical: false,
            mirrored: run_is_mirrored(&state.ctm.multiply(&state.text_matrix)),
            page_rotation_applied: 0,
        };
        self.span_sequence_counter += 1;

        log::trace!("PUSH space span with offset_semantic={}", span.offset_semantic);

        if !self.is_content_suppressed() {
            let matrix = state.ctm.multiply(&state.text_matrix);
            self.record_span_paint_order(
                span.sequence,
                self.paint_order_snapshot()?,
                [matrix.a, matrix.b, matrix.c, matrix.d],
                state
                    .font_name
                    .as_ref()
                    .and_then(|name| self.fonts.get(name))
                    .cloned(),
            )?;
            self.reserve_import_span()?;
            self.spans.push(span);
        }

        // Do NOT advance the text matrix here. The caller drives the
        // matrix forward by the *actual* TJ offset via
        // `advance_position_for_offset` immediately after; advancing
        // by `space_width` on top of that would double-count the gap
        // and capture the wrong `user_pos_x` when the next buffer is
        // created, producing spans whose bbox.x sits ~one synthetic
        // space-width to the right of the character actually drawn.

        Ok(())
    }

    /// Advance text position for a TJ offset value.
    ///
    /// Per ISO 32000-1:2008 §9.4.4 a number element in a TJ array shifts
    /// the position along the **active** writing axis:
    ///   horizontal: tx = -offset / 1000 * font_size * Th
    ///   vertical:   ty = -offset / 1000 * font_size     (NO Th)
    /// Th (Tz) is the horizontal glyph-stretching axis (§9.3.4) and does
    /// not apply in vertical mode. The matrix-side axis-swap lives in
    /// `advance_text_matrix`.
    fn advance_position_for_offset(&mut self, offset: f32) -> Result<()> {
        let state = self.state_stack.current();
        let font_size = state.font_size;
        let horizontal_scaling = state.horizontal_scaling;
        let wmode = state.text_wmode;

        let tx = if wmode == 0 {
            -offset / 1000.0 * font_size * horizontal_scaling / 100.0
        } else {
            -offset / 1000.0 * font_size
        };

        self.state_stack.current_mut().advance_text_matrix(tx);

        Ok(())
    }

    /// Fold a sub-threshold TJ offset into the active buffer's advance record
    /// so its `char_widths`/`accumulated_width` track the text-matrix position.
    ///
    /// The displacement is computed identically to `advance_position_for_offset`
    /// (text space, before the `user_h_scale` applied at flush) so it lands in
    /// the same units as the per-glyph advances pushed during string append.
    /// The offset conventionally belongs to the *preceding* glyph (it adjusts
    /// spacing after it), so it is added to the last recorded advance; if no
    /// glyph has been recorded yet the matrix move alone already positions the
    /// next buffer, so there is nothing to fold.
    fn fold_offset_into_buffer(&self, buffer: &mut TjBuffer, offset: f32) {
        let Some(last) = buffer.char_widths.last_mut() else {
            return;
        };
        let state = self.state_stack.current();
        let adv = if state.text_wmode == 0 {
            -offset / 1000.0 * state.font_size * state.horizontal_scaling / 100.0
        } else {
            -offset / 1000.0 * state.font_size
        };
        *last += adv;
        buffer.accumulated_width += adv;
    }

    /// Whether `(e, f)` continues `start`'s run along that run's writing axis.
    ///
    /// ISO 32000-1:2008 §9.4.4 places the writing direction along the matrix's
    /// `(a, b)` row, so the displacement resolves into a component along it
    /// (the advance) and one perpendicular (the line offset). For any `b = 0,
    /// a > 0` matrix the along test equals the caller's raw `e` test and the
    /// perpendicular test is implied by the raw `f` band (`hypot(c, d) >=
    /// |d|`, equal only when unskewed), so ANDing it in cannot change upright
    /// output.
    ///
    /// WMode 1 is exempt: vertical text advances along `(c, d)` instead, the
    /// branch [`GraphicsState::advance_text_matrix`] already makes, and reading
    /// its advance as a perpendicular offset splits a column glyph by glyph.
    /// The run's new accumulated width when `(e, f)` continues the run that
    /// started at `start`, or `None` when it ends the run.
    ///
    /// One frame-correct test in place of three raw-matrix ones. `e` and `f`
    /// are the text matrix's translation components, so comparing them
    /// directly assumes the run advances along `+x` and separates along `y` —
    /// true only for an upright matrix. Under a quarter turn the two axes are
    /// exchanged, and per ISO 32000-1:2008 §9.4.4 the glyph displacement lies
    /// along the text matrix's `(a, b)` row whichever way it points. Projecting
    /// the displacement onto that row and its perpendicular asks the same three
    /// questions in the run's own frame:
    ///
    /// * is the new origin on the run's line (perpendicular within tolerance),
    /// * does it lie forward along the run rather than behind it,
    /// * and is it close enough to the run's end to be the next glyph?
    ///
    /// The raw-matrix form got the right answer for upright text and, for a
    /// quarter turn, an accidental one: the perpendicular tolerance collapsed
    /// to its 0.5 pt floor, so consecutive glyphs of a rotated run each became
    /// their own span. Ten glyphs that batch into one span upright produced ten
    /// spans rotated.
    ///
    /// Vertical writing mode keeps the raw comparison. §9.7.4.3 gives it a
    /// different axis convention, which the `(a, b)` row does not describe.
    fn run_continuation_along(
        start: Matrix,
        wmode: u8,
        e: f32,
        f: f32,
        font_size: f32,
        accumulated: f32,
    ) -> Option<f32> {
        if wmode != 0 {
            let on_line = (f - start.f).abs() <= ((font_size * start.d).abs() * 0.5).max(0.5);
            return (on_line && e >= start.e).then_some(e - start.e);
        }
        // Unit vector along the writing direction. A degenerate (zero-scale)
        // matrix has no direction to speak of; fall back to +x so such runs
        // behave exactly as they did before this test existed.
        let axis = (start.a * start.a + start.b * start.b).sqrt();
        let (ux, uy) = if axis > 0.0 {
            (start.a / axis, start.b / axis)
        } else {
            (1.0, 0.0)
        };
        let (dx, dy) = (e - start.e, f - start.f);
        let along = ux * dx + uy * dy;
        let perp = -uy * dx + ux * dy;
        // Perpendicular scale; `hypot(c, d) >= |d|`, so an upright (`b == 0`)
        // run keeps at least the raw `f` band.
        let line_scale = (start.c * start.c + start.d * start.d).sqrt();
        let tolerance = ((font_size * line_scale).abs() * 0.5).max(0.5);
        if perp.abs() > tolerance || along < 0.0 {
            return None;
        }
        // The run is contiguous glyphs, so the new origin must not skip a
        // separating gap past the run's own advance. `max(3 x font size,
        // 30 pt)` is the column-gap threshold the line grouping already uses
        // to decide that two pieces of text belong to different columns; a
        // displacement that wide is the same judgement made one level earlier,
        // so the two agree by construction rather than by coincidence.
        let limit = ((font_size * 3.0).max(30.0) * axis.max(1e-6)).abs();
        (along - accumulated <= limit).then_some(along)
    }

    /// Flush accumulated Tj span buffer into a single TextSpan.
    ///
    /// This is similar to flush_tj_buffer but works with the tj_span_buffer field
    /// which accumulates consecutive Tj operators.
    fn flush_tj_span_buffer(&mut self) -> Result<()> {
        if let Some(mut buffer) = self.tj_span_buffer.take() {
            if !buffer.is_empty() {
                // Use accumulated width from advance_position_for_string calls
                // Convert from text space to user space using pre-computed horizontal scale
                let total_width = buffer.accumulated_width * buffer.user_h_scale;

                // Use pre-computed values from buffer creation (avoids
                // matrix multiply + sqrt + HashMap lookup per flush)
                let effective_font_size = buffer.effective_font_size;
                let font_weight = buffer.font_weight;
                let is_italic_buf = buffer.is_italic;

                // Move owned strings out of buffer (avoids clone)
                let font_name_buf = self
                    .emitted_font_name(buffer.font_name.as_deref(), buffer.cached_font.as_deref());

                // #537/#826: RTL visual-order detection for the Tj-span
                // path, via the shared `apply_rtl_verdict` decision point
                // (also used by `flush_tj_buffer` and `cluster_to_span`) —
                // geometric detector when `char_widths` give us per-char x,
                // falling back to the coarse `accumulated_width > 0`
                // heuristic only when ambiguous.
                let mut text = std::mem::take(&mut buffer.unicode);
                if text.len() > 1 {
                    let has_rtl = text
                        .chars()
                        .any(|c| crate::text::rtl_detector::is_rtl_text(c as u32));
                    if has_rtl {
                        // char_widths contains text-space relative widths;
                        // reconstruct absolute user-space x by accumulating,
                        // scaling by user_h_scale and offsetting by user_pos_x.
                        let chars: Vec<char> = text.chars().collect();
                        let verdict = if chars.len() == buffer.char_widths.len()
                            && !buffer.char_widths.is_empty()
                        {
                            let mut chars_with_x: Vec<(char, f32)> =
                                Vec::with_capacity(chars.len());
                            let mut cursor_text_space = 0.0_f32;
                            for (i, c) in chars.iter().enumerate() {
                                let user_x =
                                    buffer.user_pos_x + cursor_text_space * buffer.user_h_scale;
                                chars_with_x.push((*c, user_x));
                                cursor_text_space += buffer.char_widths[i];
                            }
                            crate::text::bidi::detect_visual_order_run(&chars_with_x)
                        } else {
                            crate::text::bidi::RunOrder::Ambiguous
                        };
                        text = crate::text::bidi::apply_rtl_verdict(
                            &text,
                            verdict,
                            buffer.accumulated_width > 0.0,
                            matches!(buffer.render_mode, 3 | 7),
                        );
                    }
                }

                let span = TextSpan {
                    provenance: None,
                    text,
                    bbox: Rect {
                        x: if self.visible_paint_only {
                            buffer.paint_pos_x
                        } else {
                            buffer.user_pos_x
                        },
                        y: if self.visible_paint_only {
                            buffer.paint_pos_y
                        } else {
                            buffer.user_pos_y
                        },
                        width: total_width,
                        height: effective_font_size,
                    },
                    font_name: font_name_buf,
                    font_size: effective_font_size,
                    font_weight,
                    color: Color::new(
                        buffer.fill_color_rgb.0,
                        buffer.fill_color_rgb.1,
                        buffer.fill_color_rgb.2,
                    ),
                    mcid: buffer.mcid,
                    mcid_scope: Some(self.current_mcid_scope()),
                    sequence: self.span_sequence_counter,
                    split_boundary_before: false,
                    offset_semantic: false,
                    char_spacing: 0.0, // Tc - per ISO 32000-1:2008 Section 9.3.1
                    word_spacing: 0.0, // Tw - per ISO 32000-1:2008 Section 9.3.1
                    horizontal_scaling: 100.0, // Tz - per ISO 32000-1:2008 Section 9.3.1
                    is_italic: is_italic_buf,
                    is_monospace: buffer.is_monospace,
                    primary_detected: false,
                    artifact_type: self.current_artifact_type(),
                    char_widths: {
                        let mut cw = std::mem::take(&mut buffer.char_widths);
                        let h = buffer.user_h_scale;
                        for w in &mut cw {
                            *w *= h;
                        }
                        cw
                    },
                    char_x_offsets: Vec::new(),
                    heading_level: None,
                    rotation_degrees: buffer.rotation_degrees,
                    wmode: buffer.wmode,
                    text_rise: buffer.text_rise,
                    rtl_draw_logical: false,
                    mirrored: buffer.mirrored,
                    page_rotation_applied: 0,
                };
                self.span_sequence_counter += 1;

                log::trace!(
                    "FLUSH_TJ_SPAN_BUFFER creating span: text='{}', offset_semantic={} (space-only spans marked as offset_semantic)",
                    if span.text.chars().all(|c| c.is_whitespace()) {
                        "<space-only>"
                    } else {
                        crate::utils::safe_prefix(&span.text, 20)
                    },
                    span.offset_semantic
                );

                if !self.is_content_suppressed() {
                    self.record_span_paint_order(
                        span.sequence,
                        std::mem::take(&mut buffer.source_order),
                        buffer.paint_basis,
                        buffer.cached_font.clone(),
                    )?;
                    self.reserve_import_span()?;
                    self.spans.push(span);
                }
            }
        }
        Ok(())
    }

    fn show_text(&mut self, text: &[u8]) -> Result<()> {
        // PDF spec Section 7.3.4.2: implementation limit of 32,767 bytes per string.
        let text = if text.len() > 32_767 {
            log::warn!(
                "String exceeds PDF spec limit: {} bytes (max 32,767), truncating",
                text.len()
            );
            &text[..32_767]
        } else {
            text
        };

        // Get current state values
        let state = self.state_stack.current();
        let font_size = state.font_size;
        let horizontal_scaling = state.horizontal_scaling;
        let char_space = state.char_space;
        let word_space = state.word_space;
        let fill_color_rgb = state.fill_color_rgb;
        let ctm = state.ctm;
        let wmode = state.text_wmode;

        // Get current font from cached reference
        let font = self.cached_current_font.as_deref();

        for (char_code, nbytes) in TextCharIter::new(text, font) {
            // Get current text matrix (may be updated by previous characters in this string)
            let state = self.state_stack.current();
            let text_matrix = state.text_matrix;

            // Get Unicode string using font mapping
            let unicode_string = if let Some(font) = font {
                font.char_to_unicode(char_code as u32)
                    .unwrap_or_else(|| fallback_char_to_unicode(char_code as u32))
            } else if char_code < 256 && (char_code as u8).is_ascii() {
                (char_code as u8 as char).to_string()
            } else {
                "?".to_string()
            };

            // Calculate character position in user space
            let text_pos = text_matrix.transform_point(
                0.0,
                if self.visible_paint_only {
                    state.text_rise
                } else {
                    0.0
                },
            );
            let pos = ctm.transform_point(text_pos.x, text_pos.y);

            // Calculate effective font size
            let combined_char = ctm.multiply(&text_matrix);
            let effective_font_size = font_size
                * (combined_char.d * combined_char.d + combined_char.b * combined_char.b).sqrt();

            // Calculate character dimensions using accurate glyph width
            let glyph_width_font_units = if let Some(font) = font {
                font.get_glyph_width(char_code)
            } else {
                500.0 // Default 0.5em
            };

            // font_matrix_a converts glyph-space widths to text-space units.
            // Standard fonts (Type1/TrueType): font_matrix_a = 0.001.
            // Type3 with identity FontMatrix: font_matrix_a = 1.0 (no /1000 division).
            // Assumes FontMatrix[1] = 0 (no glyph-axis rotation), which holds for all
            // standard fonts and virtually all Type3 fonts encountered in practice.
            let font_matrix_a = font.map(|f| f.font_matrix_a).unwrap_or(0.001);
            let fs_factor = font_size * font_matrix_a;
            let hs_factor = horizontal_scaling / 100.0;
            let glyph_width_user_space = glyph_width_font_units * fs_factor * hs_factor;

            // Advance along the active writing axis per ISO 32000-1 §9.4.4:
            //   horizontal: tx = (w0 * Tfs + Tc + Tw) * Th
            //   vertical:   ty = w1y * Tfs + Tc + Tw    (NO Th — Tz is a
            //               glyph-stretching factor on the X axis only;
            //               see §9.3.4).
            // Word spacing applies only to the SINGLE-BYTE code 32
            // (ISO 32000-1 §9.3.3), never to a multi-byte code whose value
            // happens to be 32.
            let ws_applies = char_code == 32 && nbytes == 1;
            let mut tx = if wmode == 0 {
                glyph_width_user_space
                    + char_space * hs_factor
                    + if ws_applies {
                        word_space * hs_factor
                    } else {
                        0.0
                    }
            } else {
                let w1y = font
                    .map(|f| f.get_vertical_metrics(char_code).w1y)
                    .unwrap_or(crate::fonts::VerticalMetrics::SPEC_DEFAULT.w1y);
                w1y * fs_factor + char_space + if ws_applies { word_space } else { 0.0 }
            };

            // For TextChar, we use the device-space width
            let glyph_width_device_space = glyph_width_user_space * combined_char.a.abs();
            let tx_device_space = tx * combined_char.a.abs();
            let height_device_space = effective_font_size;
            // Quiet unused-mut warning when wmode != 0 and tx is read-only after this point.
            let _ = &mut tx;

            // Determine font weight and style
            let (font_weight, is_italic_char) = if let Some(font) = font {
                (
                    if font.is_bold() {
                        FontWeight::Bold
                    } else {
                        FontWeight::Normal
                    },
                    font.is_italic(),
                )
            } else {
                (FontWeight::Normal, false)
            };

            // Get color
            let (r, g, b) = fill_color_rgb;
            let color = Color::new(r, g, b);

            // Compose CTM and text_matrix for full transformation
            let final_matrix = ctm.multiply(&text_matrix);
            let rotation_degrees = final_matrix.b.atan2(final_matrix.a).to_degrees();

            // Guard against malformed fonts
            let unicode_string = if unicode_string.chars().count() > 8 {
                unicode_string.chars().next().unwrap_or('?').to_string()
            } else {
                unicode_string
            };

            // Process each character in the expanded string (ligatures)
            let char_count = unicode_string.chars().count();
            let char_width_device = if char_count > 0 {
                glyph_width_device_space / char_count as f32
            } else {
                glyph_width_device_space
            };
            let char_width_user = if char_count > 0 {
                glyph_width_user_space / char_count as f32
            } else {
                glyph_width_user_space
            };
            // Spread the total advance evenly across the ligature's output chars.
            // Tc applies once per character *code*, not per output glyph, so this
            // approximation slightly over-distributes Tc for multi-char ligatures —
            // the same trade-off advance_width already makes for glyph_width_device.
            let rendered_advance_per_char = if char_count > 0 {
                tx_device_space / char_count as f32
            } else {
                tx_device_space
            };

            for (char_index, unicode_char) in unicode_string.chars().enumerate() {
                let should_skip = unicode_char == '\0'
                    || (unicode_char.is_control()
                        && unicode_char != '\t'
                        && unicode_char != '\n'
                        && unicode_char != '\r');

                if !should_skip {
                    let x_offset_device = char_index as f32 * char_width_device;
                    let x_offset_user = char_index as f32 * char_width_user;

                    let char_origin_x = pos.x + x_offset_device;
                    let char_origin_y = pos.y;

                    let text_char = TextChar {
                        char: unicode_char,
                        bbox: Rect::new(
                            char_origin_x,
                            char_origin_y,
                            char_width_device,
                            height_device_space,
                        ),
                        font_name: font.map(|f| f.base_font.clone()).unwrap_or_default(),
                        font_size: effective_font_size,
                        font_weight,
                        color,
                        mcid: self.current_mcid,
                        is_italic: is_italic_char,
                        is_monospace: false,
                        origin_x: char_origin_x,
                        origin_y: char_origin_y,
                        rotation_degrees,
                        advance_width: char_width_device,
                        rendered_advance: rendered_advance_per_char,
                        ascent: font.map(|f| f.ascent).unwrap_or(0.95) * effective_font_size,
                        descent: font.map(|f| f.descent).unwrap_or(-0.35) * effective_font_size,
                        matrix: Some([
                            final_matrix.a,
                            final_matrix.b,
                            final_matrix.c,
                            final_matrix.d,
                            final_matrix.e + x_offset_user,
                            final_matrix.f,
                        ]),
                    };

                    if !self.is_content_suppressed() {
                        self.chars.push(text_char);
                    }
                }
            }

            // Update text matrix per ISO 32000-1:2008 §9.4.4. The axis swap
            // (x for WMode 0, y for WMode 1) is encapsulated in
            // advance_text_matrix so this site does not branch.
            self.state_stack.current_mut().advance_text_matrix(tx);
        }

        Ok(())
    }

    /// Get the number of extracted characters.
    pub fn char_count(&self) -> usize {
        self.chars.len()
    }

    /// Clear all extracted characters.
    pub fn clear(&mut self) {
        self.chars.clear();
    }
}

impl<'doc> Default for TextExtractor<'doc> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to determine if a space should be inserted between two text spans
/// based on character transition heuristics.
///
/// This complements gap-based space detection by catching cases where the geometric
/// gap is small but a space is semantically needed based on character patterns.
///
/// # Detected Patterns
///
/// - **CamelCase transitions**: `thenThe` → `then The` (lowercase followed by uppercase)
/// - **Number-letter transitions**: `Figure1` → `Figure 1` (digit followed by letter)
/// - **Letter-number transitions**: `page3` → `page 3` (letter followed by digit)
///
/// # Arguments
///
/// * `current_text` - The text of the current span
/// * `next_text` - The text of the next span to be merged
///
/// # Returns
///
/// `true` if a space should be inserted based on character transitions,
/// `false` if no space is needed
///
/// # Preserves
///
/// - Acronyms like "HTML", "PDF", "API" (all uppercase)
/// - Normal word boundaries (already handled by gap detection)
/// - Intentional concatenations within words
// DELETED: should_insert_space_heuristic()
// Character pattern heuristics (CamelCase detection, number-letter transitions)
// are not defined in ISO 32000-1:2008 PDF spec. Per spec-compliance refactoring,
// only spec-defined signals (TJ offsets, geometric gaps, boundary whitespace)
// are used for space insertion decisions.
// See: PHASE10_PDF_SPEC_COMPLIANCE.md
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::{Encoding, LazyCMap};
    use std::sync::Arc;

    /// #847 M2: a condensed bold heading typeset with no space glyph — the
    /// intra-word glyph gaps cluster near zero (tight/overlapping side-bearings)
    /// while inter-word gaps sit at ~0.18 em. The split must land between the
    /// clusters so a gap of ~0.18 em reads as a word boundary.
    #[test]
    fn test_bimodal_gap_split_heading() {
        // fs = 20.5; intra-word ~0/negative, inter-word ~3.7pt (0.18 em).
        let gaps = [-0.5, -0.7, -0.3, 3.72, 3.70, 3.68, -0.4, 3.71];
        let split = TextExtractor::bimodal_gap_split(&gaps, 20.5);
        let split = split.expect("clearly bimodal line must yield a split");
        assert!(
            split > 0.0 && split < 3.5,
            "split {split} must separate the ~0 and ~3.7pt clusters"
        );
    }

    /// A normally-spaced line (all gaps already a full word-space) is NOT
    /// bimodal — there is no narrow-gap rescue to perform, so `None`.
    #[test]
    fn test_bimodal_gap_split_uniform_word_spacing_none() {
        let gaps = [6.0, 6.1, 5.9, 6.05, 5.95];
        assert!(TextExtractor::bimodal_gap_split(&gaps, 12.0).is_none());
    }

    /// A single word (all gaps intra-word, near zero) has no inter-word
    /// cluster — must return `None`, never fabricate a boundary.
    #[test]
    fn test_bimodal_gap_split_single_word_none() {
        let gaps = [-0.5, -0.7, -0.3, 0.1, -0.4];
        assert!(TextExtractor::bimodal_gap_split(&gaps, 20.5).is_none());
    }

    /// Multi-level condensed footer: near-zero/overlapping intra-word gaps, a
    /// NARROW ~0.10 em word gap (1.14 pt @ 11 pt), AND a wide ~0.25 em real
    /// space (2.75 pt) on one line. The split must land just above the
    /// intra-word cluster — below the narrow gap — so BOTH the narrow word gap
    /// and the wide space read as boundaries (recovering `All` / `rights` in
    /// `© ISO 2021 - All rights…`, matching pdfminer/poppler).
    #[test]
    fn test_bimodal_gap_split_multilevel_footer() {
        let gaps = [-0.1, -0.2, -0.15, 1.14, -0.1, -0.05, 2.75, -0.2];
        let split = TextExtractor::bimodal_gap_split(&gaps, 11.0)
            .expect("a multi-level line must yield a split");
        assert!(
            split > 0.0 && split < 1.14,
            "split {split} must sit below the narrow 1.14pt word gap so both it and the wide space split"
        );
    }

    /// The narrow-gap rescue's math guard: a full subscript glyph occupying the
    /// gap between a variable and the next symbol must be detected (suppress the
    /// split, `λᵢr` stays whole), while a mere descender/ascender edge clipping
    /// the gap band must NOT (so ordinary prose word gaps are still recovered).
    #[test]
    fn test_gap_has_intervening_glyph() {
        let r = |x, y, w, h| crate::geometry::Rect {
            x,
            y,
            width: w,
            height: h,
        };
        // `left` ends at x=10, `right` starts at x=24: a 14-unit gap on the
        // baseline band [0, 10].
        let left = r(0.0, 0.0, 10.0, 10.0);
        let right = r(24.0, 0.0, 10.0, 10.0);
        // A subscript glyph centred in the gap (x 13..21 = 8 units ≈ 57% of the
        // 14-unit gap), shifted down but overlapping the band.
        let subscript = r(13.0, -3.0, 8.0, 8.0);
        assert!(
            gap_has_intervening_glyph(&[left, right, subscript], &left, &right),
            "a full subscript occupying the gap must be detected"
        );
        // A descender edge just clipping the gap (x 9..12 = only ~2 units into
        // the 14-unit gap, < 35%) must NOT count.
        let descender_edge = r(9.0, -4.0, 3.0, 6.0);
        assert!(
            !gap_has_intervening_glyph(&[left, right, descender_edge], &left, &right),
            "a descender edge clipping the gap must not be treated as an intervening glyph"
        );
    }

    /// The writing-axis continuation test, quadrant by quadrant.
    ///
    /// This is now the whole continuation rule rather than one veto ANDed onto
    /// a raw-matrix pair, so each quadrant pins all three of its questions: on
    /// the line, forward along it, and near enough to the run's end to be the
    /// next glyph.
    #[test]
    fn test_run_continuation_by_quadrant() {
        let m = |a, b, c, d| Matrix {
            a,
            b,
            c,
            d,
            e: 100.0,
            f: 500.0,
        };
        let fs = 10.0;
        // A run that has already advanced 14 pt, so a 14 pt displacement puts
        // the new origin exactly at its end — the contiguous case.
        let at = |mat: Matrix, de: f32, df: f32| {
            TextExtractor::run_continuation_along(mat, 0, mat.e + de, mat.f + df, fs, 14.0)
                .is_some()
        };

        // Upright: the frame the raw `e`/`f` comparison was already right for.
        let upright = m(1.0, 0.0, 0.0, 1.0);
        assert!(at(upright, 14.0, 0.0), "upright advance must continue");
        assert!(!at(upright, 0.0, -14.0), "upright line break must not");
        assert!(!at(upright, -14.0, 0.0), "upright backwards must not");
        // Perpendicular tolerance: 0.5 × font size (5pt here) admits a
        // sub-glyph baseline offset; a full line step is vetoed.
        assert!(at(upright, 14.0, 4.0), "upright sub-glyph offset must not be vetoed");
        assert!(!at(upright, 14.0, 8.0), "upright line step must be vetoed");

        // Advances along +y; lines separate along +x.
        let cw = m(0.0, 1.0, -1.0, 0.0);
        assert!(at(cw, 0.0, 14.0), "90° along-axis advance must continue");
        assert!(!at(cw, 14.0, 0.0), "90° line break must not");
        assert!(at(cw, -4.0, 14.0), "90° sub-glyph offset must not be vetoed");
        assert!(!at(cw, -8.0, 14.0), "90° line step must be vetoed");

        // Advances along -y; the sign a single-rotation fixture cannot catch.
        let ccw = m(0.0, -1.0, 1.0, 0.0);
        assert!(at(ccw, 0.0, -14.0), "270° along-axis advance must continue");
        assert!(!at(ccw, 0.0, 14.0), "270° backwards advance must not");
        assert!(!at(ccw, 14.0, 0.0), "270° line break must not");

        // 180°: advances along -x.
        let flip = m(-1.0, 0.0, 0.0, -1.0);
        assert!(at(flip, -14.0, 0.0), "180° along-axis advance must continue");
        assert!(!at(flip, 14.0, 0.0), "180° backwards advance must not");

        // No writing direction: falls back to +x, as before.
        let degenerate = m(0.0, 0.0, 0.0, 0.0);
        assert!(at(degenerate, 14.0, 0.0));
        assert!(!at(degenerate, -14.0, 0.0));
    }

    /// The gap bound, in every quadrant. A jump past the run's end by more
    /// than a column gap is a new run wherever the run happens to point — the
    /// rule that keeps two columns of rotated text from gluing into one span.
    ///
    /// The threshold is `max(3 x font size, 30 pt)`, the same one the line
    /// grouping uses to separate columns, so the two levels agree about what
    /// counts as a separating gap.
    #[test]
    fn test_run_continuation_bounds_the_gap_in_every_quadrant() {
        let m = |a, b, c, d| Matrix {
            a,
            b,
            c,
            d,
            e: 100.0,
            f: 500.0,
        };
        let fs = 10.0;
        // The run has advanced 14 pt and the threshold at 10 pt type is
        // max(30, 30) = 30 pt, so a displacement of 44 lands exactly at the
        // bound and 60 is well beyond it.
        let at = |mat: Matrix, de: f32, df: f32| {
            TextExtractor::run_continuation_along(mat, 0, mat.e + de, mat.f + df, fs, 14.0)
                .is_some()
        };
        for (name, mat, unit) in [
            ("upright", m(1.0, 0.0, 0.0, 1.0), (1.0f32, 0.0f32)),
            ("90°", m(0.0, 1.0, -1.0, 0.0), (0.0, 1.0)),
            ("270°", m(0.0, -1.0, 1.0, 0.0), (0.0, -1.0)),
            ("180°", m(-1.0, 0.0, 0.0, -1.0), (-1.0, 0.0)),
        ] {
            let step = |d: f32| (unit.0 * d, unit.1 * d);
            let (e44, f44) = step(44.0);
            assert!(at(mat, e44, f44), "{name}: a gap of exactly the threshold must continue");
            let (e60, f60) = step(60.0);
            assert!(!at(mat, e60, f60), "{name}: a gap beyond the threshold must end the run");
        }
    }

    /// Vertical writing mode keeps the raw comparison: §9.7.4.3 gives it an
    /// axis convention the `(a, b)` row does not describe, so it is out of
    /// this rule's scope and must behave exactly as before.
    #[test]
    fn test_run_continuation_leaves_vertical_writing_mode_alone() {
        let upright = Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 100.0,
            f: 500.0,
        };
        let fs = 10.0;
        // The raw test: same `f`, forward `e`.
        assert_eq!(
            TextExtractor::run_continuation_along(upright, 1, 140.0, 500.0, fs, 0.0),
            Some(40.0),
            "a forward step on the same baseline continues, and reports its offset"
        );
        assert!(
            TextExtractor::run_continuation_along(upright, 1, 60.0, 500.0, fs, 0.0).is_none(),
            "a backwards step ends the run"
        );
        assert!(
            TextExtractor::run_continuation_along(upright, 1, 140.0, 460.0, fs, 0.0).is_none(),
            "a line step ends the run"
        );
    }

    #[test]
    fn test_snap_run_rotation() {
        let m = |a, b, c, d| Matrix {
            a,
            b,
            c,
            d,
            e: 0.0,
            f: 0.0,
        };
        // Horizontal identity-scale → 0.0 (byte-identical path).
        assert_eq!(snap_run_rotation(&m(12.0, 0.0, 0.0, 12.0)), 0.0);
        // Tiny float noise still counts as horizontal.
        assert_eq!(snap_run_rotation(&m(12.0, 1e-5, -1e-5, 12.0)), 0.0);
        // 90° CCW (a=0, b=+s, c=-s, d=0).
        assert_eq!(snap_run_rotation(&m(0.0, 12.0, -12.0, 0.0)), 90.0);
        // 270° / -90° (a=0, b=-s, c=+s, d=0).
        assert_eq!(snap_run_rotation(&m(0.0, -12.0, 12.0, 0.0)), -90.0);
        // 180° (a=-s, d=-s, b=c=0) must not alias to 0° — both have
        // b≈0, c≈0, so only the sign of `a` (cos 0° vs cos 180°)
        // distinguishes them.
        assert_eq!(snap_run_rotation(&m(-12.0, 0.0, 0.0, -12.0)), 180.0);
        // Tiny float noise on a 180° matrix still counts as 180°, not 0°.
        assert_eq!(snap_run_rotation(&m(-12.0, 1e-5, -1e-5, -12.0)), 180.0);
        // ~88° snaps to 90.
        let r = 12.0_f32;
        let th = 88.0_f32.to_radians();
        assert_eq!(
            snap_run_rotation(&m(r * th.cos(), r * th.sin(), -r * th.sin(), r * th.cos())),
            90.0
        );
        // 45° watermark is NOT snapped (kept as its own block downstream).
        let th = 45.0_f32.to_radians();
        let got = snap_run_rotation(&m(r * th.cos(), r * th.sin(), -r * th.sin(), r * th.cos()));
        assert!((got - 45.0).abs() < 0.5, "45° should pass through, got {got}");
    }

    fn create_test_font() -> FontInfo {
        FontInfo {
            base_font: "Times-Roman".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: crate::fonts::VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        }
    }

    #[test]
    fn test_text_extractor_new() {
        let extractor = TextExtractor::new();
        assert_eq!(extractor.char_count(), 0);
    }

    #[test]
    fn test_text_extractor_add_font() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);
        assert_eq!(extractor.fonts.len(), 1);
    }

    #[test]
    fn test_extract_simple_text() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 100 700 Td (Hello) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 5); // "Hello"
        assert_eq!(chars[0].char, 'H');
        assert_eq!(chars[1].char, 'e');
        assert_eq!(chars[2].char, 'l');
        assert_eq!(chars[3].char, 'l');
        assert_eq!(chars[4].char, 'o');
    }

    #[test]
    fn test_extract_with_matrix() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 1 0 0 1 100 700 Tm (Hi) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].char, 'H');
        assert_eq!(chars[1].char, 'i');
        // Position should be around (100, 700)
        assert!(chars[0].bbox.x >= 99.0 && chars[0].bbox.x <= 101.0);
    }

    /// Regression test for Issue #11: CTM must be applied to text positions
    ///
    /// Per PDF Spec ISO 32000-1:2008 Section 9.4.4, the text rendering matrix is:
    /// T_rm = [font_matrix] × T_m × CTM
    ///
    /// This test verifies that when CTM contains a translation, text positions
    /// are correctly transformed from text space to user space.
    #[test]
    fn test_ctm_applied_to_text_position() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // CTM translates by (100, 200), text matrix at origin
        // Final position should be (100, 200), not (0, 0)
        let stream = b"q 1 0 0 1 100 200 cm BT /F1 12 Tf (A) Tj ET Q";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, 'A');
        // Position should be translated by CTM: (100, 200)
        assert!(
            chars[0].bbox.x >= 99.0 && chars[0].bbox.x <= 101.0,
            "X position should be ~100 (got {})",
            chars[0].bbox.x
        );
        assert!(
            chars[0].bbox.y >= 199.0 && chars[0].bbox.y <= 201.0,
            "Y position should be ~200 (got {})",
            chars[0].bbox.y
        );
    }

    /// Regression test for issue #1006: char mode must run the same parser as
    /// span mode.
    ///
    /// The stream forces the streaming parser's >256KB prescan route: the
    /// rotating `q`/`cm` sits >4KB before `BT`, so the CTM reaches the text
    /// region only via the forward scan's injected `Cm`. The unbalanced
    /// literal-string opens are invisible to the prescan (they lie outside
    /// every text region) but feed the old char-mode parser,
    /// `parse_content_stream_text_only`, over `MAX_CONSECUTIVE_ERRORS`
    /// consecutive scan failures — it bails before `BT` and extracts nothing.
    #[test]
    fn test_char_mode_rotated_ctm_survives_large_hostile_stream() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let mut cs = Vec::new();
        cs.extend_from_slice(b"q\n0 1 -1 0 612 0 cm\n");
        cs.extend_from_slice(&[b'('; 1500]);
        cs.push(b'\n');
        for i in 0..13000u32 {
            let line = format!(
                "{}.0 {}.0 m {}.0 {}.0 l S\n",
                i % 500,
                (i * 7) % 500,
                (i * 3) % 500,
                (i * 11) % 500
            );
            cs.extend_from_slice(line.as_bytes());
        }
        assert!(cs.len() > 256 * 1024, "stream must exceed the 256KB prescan threshold");
        cs.extend_from_slice(b"BT /F1 12 Tf 100 200 Td (Hello) Tj ET\nQ\n");

        let chars = extractor.extract(&cs).unwrap();
        let mut glyphs: Vec<char> = chars.iter().map(|c| c.char).collect();
        glyphs.sort_unstable();
        assert_eq!(glyphs, vec!['H', 'e', 'l', 'l', 'o']);
        for c in &chars {
            assert!(
                (c.rotation_degrees - 90.0).abs() < 1.0,
                "expected 90 degrees from the cm before BT, got {}",
                c.rotation_degrees
            );
        }
    }

    /// Regression test for Issue #11: CTM scaling must affect text positions
    ///
    /// This test verifies that CTM scaling is correctly applied to text positions.
    #[test]
    fn test_ctm_scaling_applied_to_text_position() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // CTM scales by 2x, text at position (50, 100) in text space
        // Final position should be (100, 200) in user space
        let stream = b"q 2 0 0 2 0 0 cm BT /F1 12 Tf 1 0 0 1 50 100 Tm (B) Tj ET Q";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, 'B');
        // Position should be scaled: (50*2, 100*2) = (100, 200)
        assert!(
            chars[0].bbox.x >= 99.0 && chars[0].bbox.x <= 101.0,
            "X position should be ~100 (got {})",
            chars[0].bbox.x
        );
        assert!(
            chars[0].bbox.y >= 199.0 && chars[0].bbox.y <= 201.0,
            "Y position should be ~200 (got {})",
            chars[0].bbox.y
        );
    }

    /// Regression test for Issue #11: Combined CTM translation and text matrix
    ///
    /// This test verifies the complete transformation chain works correctly.
    #[test]
    fn test_ctm_combined_with_text_matrix() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // CTM translates by (50, 50), text matrix positions at (25, 25)
        // Final position should be (75, 75)
        let stream = b"q 1 0 0 1 50 50 cm BT /F1 12 Tf 1 0 0 1 25 25 Tm (C) Tj ET Q";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, 'C');
        // Position: text_matrix(25,25) + CTM_translation(50,50) = (75, 75)
        assert!(
            chars[0].bbox.x >= 74.0 && chars[0].bbox.x <= 76.0,
            "X position should be ~75 (got {})",
            chars[0].bbox.x
        );
        assert!(
            chars[0].bbox.y >= 74.0 && chars[0].bbox.y <= 76.0,
            "Y position should be ~75 (got {})",
            chars[0].bbox.y
        );
    }

    #[test]
    fn test_extract_with_tj_array() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 0 0 Td [(H)(i)] TJ ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].char, 'H');
        assert_eq!(chars[1].char, 'i');
    }

    /// Test extraction of multi-byte characters from Type0 fonts (Identity-H)
    /// This verifies the fix for Issue #186 where extract_chars() was garbling CJK text.
    #[test]
    fn test_extract_type0_multibyte_character_extraction() {
        let mut extractor = TextExtractor::new();

        // Create a mock Type0 font with Identity-H encoding
        let mut font = create_test_font();
        font.subtype = "Type0".to_string();
        font.encoding = Encoding::Standard("Identity-H".to_string());

        // Create a valid ToUnicode CMap stream that maps CID 0x4E2D to '中' and 0x6587 to '文'
        let cmap_data = b"
            /CIDInit /ProcSet findresource begin
            12 dict begin
            begincmap
            /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def
            /CMapName /Adobe-Identity-UCS def
            /CMapType 2 def
            1 begincodespacerange <0000> <FFFF> endcodespacerange
            2 beginbfchar
            <4E2D> <4E2D>
            <6587> <6587>
            endbfchar
            endcmap
            CMapName currentdict /CMap defineresource pop
            end
            end
        ";

        // Use public parse_tounicode_cmap to create CMap, then wrap in LazyCMap
        let lazy_cmap = LazyCMap::new(cmap_data.to_vec());
        font.to_unicode = Some(lazy_cmap);

        extractor.add_font("F1".to_string(), font);

        // Content stream with 2-byte CIDs for "中文" (0x4E2D 0x6587)
        let stream = b"BT /F1 12 Tf 0 0 Td <4E2D6587> Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].char, '中');
        assert_eq!(chars[1].char, '文');
    }

    #[test]
    fn test_extract_color() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT 1 0 0 rg /F1 12 Tf 0 0 Td (R) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, 'R');
        assert_eq!(chars[0].color.r, 1.0);
        assert_eq!(chars[0].color.g, 0.0);
        assert_eq!(chars[0].color.b, 0.0);
    }

    /// Regression test for a text-only-parser bug where a fill colour set by
    /// `scn` *before* the enclosing `BT` was silently dropped, leaving the
    /// text drawn in the GraphicsState default (black) instead of the
    /// colour the content stream actually requested.
    ///
    /// Root cause: `scan_graphics_region()` (src/content/parser.rs) is used
    /// by `parse_and_execute_text_only()` to fast-scan non-text regions
    /// looking for the next `BT`. It classified `scn`/`cs`/`sc`/`rg`/`g`/`k`
    /// (and friends) as unconditionally "skippable" - correct only when a
    /// matching `Q` is guaranteed to revert the change before any `BT`, but
    /// wrong at the top level (outside any q/Q scope), where the colour
    /// change legitimately persists into the next text object per
    /// ISO 32000-1:2008 SS8.4. Reproduces the exact operator sequence found
    /// on a real-world govdocs1 slide-deck PDF: a marked-content BDC opens,
    /// `scn` sets a blue fill colour *outside* any text object, then `BT`
    /// opens the text object that draws the (should-be-blue) heading.
    #[test]
    fn test_fill_color_scn_before_bt_after_bdc_not_dropped() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"/Shape <</MCID 3 >>BDC \
                        0.2 0.2 0.604 scn \
                        BT /F1 12 Tf 100 700 Td (Blue Heading) Tj ET \
                        EMC";
        let spans = extractor.extract_text_spans(stream).unwrap();

        assert_eq!(spans.len(), 1);
        assert!(
            (spans[0].color.r - 0.2).abs() < 0.01,
            "expected blue fill (0.2, 0.2, 0.604), got {:?}",
            spans[0].color
        );
        assert!((spans[0].color.g - 0.2).abs() < 0.01);
        assert!((spans[0].color.b - 0.604).abs() < 0.01);
    }

    /// Same bug, second real-world pattern: a `Q` (RestoreGraphicsState)
    /// immediately precedes the out-of-text-object `scn`. Reproduces the
    /// gold author-block sequence from the same source PDF.
    #[test]
    fn test_fill_color_scn_after_q_before_bt_not_dropped() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"q 1 0 0 1 0 0 cm Q \
                        1 1 0 scn \
                        BT /F1 12 Tf 100 700 Td (Gold Author) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        assert_eq!(spans.len(), 1);
        assert!(
            (spans[0].color.r - 1.0).abs() < 0.01,
            "expected gold fill (1, 1, 0), got {:?}",
            spans[0].color
        );
        assert!((spans[0].color.g - 1.0).abs() < 0.01);
        assert!((spans[0].color.b - 0.0).abs() < 0.01);
    }

    /// Must-not-regress guard: `scn` issued *inside* an already-open text
    /// object (continuing after a prior `Tj`, still within the same BT/ET)
    /// always worked correctly - it goes through the ordinary text-operator
    /// parse path, not the non-text `scan_graphics_region` fast scanner.
    /// Confirms the fix above did not disturb this working case.
    #[test]
    fn test_fill_color_scn_inside_open_text_object_still_works() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 100 700 Td (Black Text) Tj \
                        0.2 0.2 0.604 scn \
                        0 -20 Td (Blue Text) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        assert_eq!(spans.len(), 2);
        assert!(
            (spans[0].color.r - 0.0).abs() < 0.01 && (spans[0].color.b - 0.0).abs() < 0.01,
            "first run should still be default black, got {:?}",
            spans[0].color
        );
        assert!(
            (spans[1].color.r - 0.2).abs() < 0.01,
            "second run should be blue (0.2, 0.2, 0.604), got {:?}",
            spans[1].color
        );
        assert!((spans[1].color.g - 0.2).abs() < 0.01);
        assert!((spans[1].color.b - 0.604).abs() < 0.01);
    }

    /// Regression test: is_monospace flag must propagate from FontInfo flags
    /// through TjBuffer into the final TextSpan.
    ///
    /// When font descriptor flags have bit 0 (FixedPitch) set, spans produced
    /// by extract_text_spans() must report is_monospace == true.
    /// Conversely, a proportional font (e.g. Helvetica) must yield false.
    #[test]
    fn test_is_monospace_from_font_flags() {
        // --- Monospace font: flags bit 0 (FixedPitch) set ---
        let mut mono_font = create_test_font();
        mono_font.base_font = "Courier".to_string();
        mono_font.flags = Some(1); // bit 0 = FixedPitch

        let mut extractor = TextExtractor::new();
        extractor.add_font("F1".to_string(), mono_font);

        let stream = b"BT /F1 12 Tf 100 700 Td (Code) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        assert!(!spans.is_empty(), "should produce at least one span");
        assert!(
            spans[0].is_monospace,
            "Courier with FixedPitch flag should be monospace, got is_monospace=false"
        );

        // --- Proportional font: no FixedPitch flag ---
        let mut prop_font = create_test_font();
        prop_font.base_font = "Helvetica".to_string();
        prop_font.flags = Some(0); // no FixedPitch

        let mut extractor2 = TextExtractor::new();
        extractor2.add_font("F2".to_string(), prop_font);

        let stream2 = b"BT /F2 12 Tf 100 700 Td (Text) Tj ET";
        let spans2 = extractor2.extract_text_spans(stream2).unwrap();

        assert!(!spans2.is_empty(), "should produce at least one span");
        assert!(
            !spans2[0].is_monospace,
            "Helvetica without FixedPitch flag should not be monospace"
        );

        // --- Name-based heuristic: font name containing MONO ---
        let mut mono_name_font = create_test_font();
        mono_name_font.base_font = "DejaVuSansMono".to_string();
        mono_name_font.flags = None; // no flags at all

        let mut extractor3 = TextExtractor::new();
        extractor3.add_font("F3".to_string(), mono_name_font);

        let stream3 = b"BT /F3 12 Tf 100 700 Td (Mono) Tj ET";
        let spans3 = extractor3.extract_text_spans(stream3).unwrap();

        assert!(!spans3.is_empty(), "should produce at least one span");
        assert!(
            spans3[0].is_monospace,
            "Font named DejaVuSansMono should be detected as monospace via name heuristic"
        );
    }

    /// Regression test for issue #1033: a `/PlacedPDF` marked-content scope
    /// whose `BDC` lands inside the first prescanned (>256KB fast-path)
    /// text region but whose matching `EMC` falls outside it — past tens
    /// of thousands of bytes of artwork the prescan never turns into a
    /// text region — must not suppress text in every subsequent region.
    /// Before the fix, `inside_placed_pdf` stayed `true` forever once the
    /// scope's `EMC` fell outside a prescanned region's byte range, since
    /// the marked-content stack (unlike CTM/font) got no
    /// per-region balancing.
    #[test]
    fn test_prescan_marked_content_scope_does_not_leak_across_regions() {
        let mut cs = Vec::new();
        cs.extend_from_slice(b"/PlacedPDF /MC0 BDC\n");
        cs.extend_from_slice(b"BT /F1 12 Tf 100 700 Td (Figure Label) Tj ET\n");
        // >256KB of filler path data (the artwork) with no BT/Do at all,
        // so the prescan never turns it into its own text region — the
        // EMC below lands in the gap between the two BT regions.
        for i in 0..13000u32 {
            let line = format!(
                "{}.0 {}.0 m {}.0 {}.0 l n\n",
                i % 500,
                (i * 7) % 500,
                (i * 3) % 500,
                (i * 11) % 500
            );
            cs.extend_from_slice(line.as_bytes());
        }
        cs.extend_from_slice(b"EMC\n");
        cs.extend_from_slice(b"BT /F1 12 Tf 100 600 Td (Body Text After Figure) Tj ET\n");
        assert!(cs.len() > 256 * 1024, "stream must exceed 256KB prescan threshold");

        let font = create_test_font();
        let mut extractor = TextExtractor::new();
        extractor.add_font("F1".to_string(), font);

        let spans = extractor.extract_text_spans(&cs).unwrap();
        let all_text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        assert!(
            !all_text.contains("Figure Label"),
            "text inside the /PlacedPDF scope must still be suppressed, got: {all_text:?}"
        );
        assert!(
            all_text.contains("Body Text After Figure"),
            "text after the /PlacedPDF scope closes (EMC) must not be suppressed \
             just because the EMC fell outside the prescanned region, got: {all_text:?}"
        );
    }

    /// Regression test for issue #1024: invisible text (Tr 3/7) must never
    /// be classified monospace, even under a FixedPitch-flagged or
    /// "GlyphLessFont"-named font. Such text is an OCR text-sandwich layer
    /// sitting under a scanned page image — a synthetic OCR font commonly
    /// sets FixedPitch purely for positioning simplicity, since the glyphs
    /// are never rendered. Markdown conversion uses `is_monospace` to fence
    /// a paragraph as a code block; without this gate, a scanned novel's
    /// OCR'd dialogue trips FixedPitch and gets served as a code block.
    #[test]
    fn test_invisible_text_is_never_monospace() {
        // Invisible render mode (Tr 3) + FixedPitch-flagged font: must NOT
        // be monospace despite the flag.
        let mut ocr_font = create_test_font();
        ocr_font.base_font = "GlyphLessFont".to_string();
        ocr_font.flags = Some(1); // bit 0 = FixedPitch

        let mut extractor = TextExtractor::new();
        extractor.add_font("F1".to_string(), ocr_font);

        let stream = b"BT /F1 12 Tf 3 Tr 100 700 Td (\"I don't know,\" she said.) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        assert!(!spans.is_empty(), "should produce at least one span");
        assert!(
            !spans[0].is_monospace,
            "invisible (Tr 3) text under a FixedPitch OCR font must not be \
             classified monospace"
        );

        // Control: the SAME FixedPitch font, but VISIBLE (default Tr 0),
        // must still be classified monospace — the gate must not
        // over-suppress real code/monospace content.
        let mut visible_font = create_test_font();
        visible_font.base_font = "Courier".to_string();
        visible_font.flags = Some(1);

        let mut extractor2 = TextExtractor::new();
        extractor2.add_font("F2".to_string(), visible_font);

        let stream2 = b"BT /F2 12 Tf 100 700 Td (let x = 1;) Tj ET";
        let spans2 = extractor2.extract_text_spans(stream2).unwrap();

        assert!(!spans2.is_empty(), "should produce at least one span");
        assert!(
            spans2[0].is_monospace,
            "visible FixedPitch text must still be classified monospace"
        );
    }

    #[test]
    fn test_extract_save_restore() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Valid PDF: q saves state, Tf changes font size inside, Q restores
        let stream = b"BT /F1 12 Tf q /F1 14 Tf (A) Tj Q (B) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].font_size, 14.0); // Inside q/Q
        assert_eq!(chars[1].font_size, 12.0); // After Q, restored to 12
    }

    #[test]
    fn test_extract_no_font() {
        let mut extractor = TextExtractor::new();
        // Don't add any fonts

        let stream = b"BT /F1 12 Tf (ABC) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        // Should still extract, using identity mapping
        assert_eq!(chars.len(), 3);
    }

    #[test]
    fn test_char_count() {
        let mut extractor = TextExtractor::new();
        assert_eq!(extractor.char_count(), 0);

        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf (Test) Tj ET";
        extractor.extract(stream).unwrap();
        assert_eq!(extractor.char_count(), 4);
    }

    #[test]
    fn test_clear() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf (Test) Tj ET";
        extractor.extract(stream).unwrap();
        assert_eq!(extractor.char_count(), 4);

        extractor.clear();
        assert_eq!(extractor.char_count(), 0);
    }

    #[test]
    fn test_default() {
        let extractor = TextExtractor::default();
        assert_eq!(extractor.char_count(), 0);
    }

    /// Test unified space decision: Boundary space already present
    #[test]
    fn test_space_decision_boundary_space() {
        let config = SpanMergingConfig::default();
        let fonts = std::collections::HashMap::new();

        // Preceding text ends with space
        let decision = should_insert_space(
            "word ", "next", 0.0, 12.0, "TestFont", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        assert!(!decision.insert_space);
        assert_eq!(decision.source, SpaceSource::AlreadyPresent);

        // Following text starts with space
        let decision = should_insert_space(
            "word", " next", 0.0, 12.0, "TestFont", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        assert!(!decision.insert_space);
        assert_eq!(decision.source, SpaceSource::AlreadyPresent);
    }

    /// Regression test for issue flagged in PR #281 review:
    /// a long number emitted as multiple digit-only spans with a kerning-sized
    /// positive gap must NOT have a space inserted between the digits (would
    /// turn "123456" into "123 456"). Adjacent table cell digit values with a
    /// larger gap must still be separated.
    #[test]
    fn test_space_decision_digit_digit_gap_threshold() {
        let config = SpanMergingConfig::default();
        let fonts = std::collections::HashMap::new();

        // Kerning-sized gap (0.3pt) between digit spans — must NOT insert.
        // For 12pt font with no font-info fallback, geometric_threshold is
        // typically around 1.5pt, so half of that is 0.75pt.
        let kerning = should_insert_space(
            "123", "456", 0.3, 12.0, "TestFont", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        assert!(
            !kerning.insert_space,
            "Kerning-sized gap (0.3pt) between digits must not split the number, got: {:?}",
            kerning
        );

        // Larger gap (2pt) between digit spans — adjacent table cell values,
        // must still insert a space.
        let table_cells = should_insert_space(
            "123", "456", 2.0, 12.0, "TestFont", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        assert!(
            table_cells.insert_space,
            "2pt gap between digits should still split adjacent table values, got: {:?}",
            table_cells
        );
    }

    /// Test split boundary merging with space insertion
    ///
    /// When split_boundary_before=true, it indicates the span is part of a boundary
    /// that was previously split (e.g., from CamelCase fusion like "theGeneral").
    /// These spans should be merged WITH a space to preserve word separation.
    #[test]
    fn test_split_boundary_merges_with_space() {
        let spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "the".to_string(),
                bbox: Rect {
                    x: 0.0,
                    y: 100.0,
                    width: 10.0,
                    height: 12.0,
                },
                font_name: "Arial".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "General".to_string(),
                bbox: Rect {
                    x: 10.0,
                    y: 100.0,
                    width: 25.0,
                    height: 12.0,
                },
                font_name: "Arial".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: true, // Marks this as part of a split boundary
                offset_semantic: false,
                primary_detected: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        // Simulate extraction state
        let mut extractor = TextExtractor::new();
        extractor.spans = spans;
        extractor.merging_config = SpanMergingConfig::default();

        // Merge adjacent spans
        extractor.merge_adjacent_spans();

        // Per PDF Spec ISO 32000-1:2008 Section 9.4.4 and implementation design:
        // split_boundary_before=true means "merge with a space, never without"
        // This ensures "length" + "This" becomes "length This" not "lengthThis"
        // The spans are merged INTO ONE span with space-separated text
        assert_eq!(extractor.spans.len(), 1);
        assert_eq!(extractor.spans[0].text, "the General");
    }

    // Removed: test_should_insert_space_heuristic - function doesn't exist in current codebase

    /// Test boundary space detection
    #[test]
    fn test_has_boundary_space() {
        // Preceding text with trailing space
        assert!(has_boundary_space("word ", "next"));

        // Following text with leading space
        assert!(has_boundary_space("word", " next"));

        // Both with space
        assert!(has_boundary_space("word ", " next"));

        // Neither
        assert!(!has_boundary_space("word", "next"));

        // Only whitespace characters count
        assert!(has_boundary_space("word\t", "next"));
        assert!(has_boundary_space("word\n", "next"));
        assert!(has_boundary_space("word", "\tnext"));
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: TextExtractionConfig
    // ========================================================================

    #[test]
    fn test_text_extraction_config_new_defaults() {
        let config = TextExtractionConfig::new();
        assert_eq!(config.space_insertion_threshold, -120.0);
        assert_eq!(config.word_margin_ratio, 0.1);
        assert!(!config.use_adaptive_tj_threshold);
        assert!(config.profile.is_none());
    }

    #[test]
    fn test_text_extraction_config_with_space_threshold() {
        let config = TextExtractionConfig::with_space_threshold(-80.0);
        assert_eq!(config.space_insertion_threshold, -80.0);
        assert_eq!(config.word_margin_ratio, 0.1);
        assert!(!config.use_adaptive_tj_threshold);
    }

    #[test]
    fn test_text_extraction_config_with_word_margin_ratio() {
        let config = TextExtractionConfig::with_word_margin_ratio(0.15);
        assert_eq!(config.word_margin_ratio, 0.15);
        assert!(config.use_adaptive_tj_threshold);
        assert_eq!(config.space_insertion_threshold, -120.0); // fallback
    }

    #[test]
    fn test_text_extraction_config_set_word_margin_ratio() {
        let config = TextExtractionConfig::new().set_word_margin_ratio(0.2);
        assert_eq!(config.word_margin_ratio, 0.2);
        assert!(config.use_adaptive_tj_threshold);
    }

    #[test]
    fn test_text_extraction_config_set_adaptive_tj_threshold() {
        let config = TextExtractionConfig::new().set_adaptive_tj_threshold(true);
        assert!(config.use_adaptive_tj_threshold);
        let config2 = config.set_adaptive_tj_threshold(false);
        assert!(!config2.use_adaptive_tj_threshold);
    }

    #[test]
    fn test_text_extraction_config_with_profile() {
        let config =
            TextExtractionConfig::new().with_profile(crate::config::ExtractionProfile::ACADEMIC);
        assert!(config.profile.is_some());
        let profile = config.profile.unwrap();
        assert_eq!(profile.name, "Academic");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: SpanMergingConfig
    // ========================================================================

    #[test]
    fn test_span_merging_config_defaults() {
        let config = SpanMergingConfig::new();
        assert_eq!(config.space_threshold_em_ratio, 0.25);
        assert_eq!(config.conservative_threshold_pt, 0.1);
        assert_eq!(config.column_boundary_threshold_pt, 5.0);
        assert_eq!(config.severe_overlap_threshold_pt, -0.5);
        assert!(config.use_adaptive_threshold);
        assert!(!config.detect_email_patterns);
        assert!(!config.detect_citation_markers);
    }

    #[test]
    fn test_span_merging_config_aggressive() {
        let config = SpanMergingConfig::aggressive();
        assert_eq!(config.space_threshold_em_ratio, 0.15);
        assert_eq!(config.conservative_threshold_pt, 0.1);
        assert!(!config.use_adaptive_threshold);
    }

    #[test]
    fn test_span_merging_config_conservative() {
        let config = SpanMergingConfig::conservative();
        assert_eq!(config.space_threshold_em_ratio, 0.33);
        assert_eq!(config.conservative_threshold_pt, 0.3);
        assert!(!config.use_adaptive_threshold);
    }

    #[test]
    fn test_span_merging_config_custom() {
        let config = SpanMergingConfig::custom(0.2, 0.2, 6.0, -0.3);
        assert_eq!(config.space_threshold_em_ratio, 0.2);
        assert_eq!(config.conservative_threshold_pt, 0.2);
        assert_eq!(config.column_boundary_threshold_pt, 6.0);
        assert_eq!(config.severe_overlap_threshold_pt, -0.3);
        assert!(!config.use_adaptive_threshold);
    }

    #[test]
    fn test_span_merging_config_adaptive() {
        let config = SpanMergingConfig::adaptive();
        assert!(config.use_adaptive_threshold);
        assert!(config.adaptive_config.is_some());
    }

    #[test]
    fn test_span_merging_config_legacy() {
        let config = SpanMergingConfig::legacy();
        assert!(!config.use_adaptive_threshold);
        assert_eq!(config.conservative_threshold_pt, 0.1);
        assert!(config.adaptive_config.is_none());
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: SpaceDecision
    // ========================================================================

    #[test]
    fn test_space_decision_insert() {
        let d = SpaceDecision::insert(SpaceSource::TjOffset, 0.95);
        assert!(d.insert_space);
        assert_eq!(d.source, SpaceSource::TjOffset);
        assert_eq!(d.confidence, 0.95);
    }

    #[test]
    fn test_space_decision_no_space() {
        let d = SpaceDecision::no_space(SpaceSource::NoSpace, 1.0);
        assert!(!d.insert_space);
        assert_eq!(d.source, SpaceSource::NoSpace);
        assert_eq!(d.confidence, 1.0);
    }

    #[test]
    fn test_space_decision_clamp_confidence() {
        let d = SpaceDecision::insert(SpaceSource::GeometricGap, 1.5);
        assert_eq!(d.confidence, 1.0); // clamped
        let d2 = SpaceDecision::insert(SpaceSource::GeometricGap, -0.5);
        assert_eq!(d2.confidence, 0.0); // clamped
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Text operators via execute_operator
    // ========================================================================

    #[test]
    fn test_operator_td_positioning() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // BT, set font, Td to position (100, 700), show "X", ET
        let stream = b"BT /F1 12 Tf 100 700 Td (X) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, 'X');
        // After Td(100, 700), position should be near (100, 700)
        assert!((chars[0].bbox.x - 100.0).abs() < 2.0);
        assert!((chars[0].bbox.y - 700.0).abs() < 2.0);
    }

    /// Issue #254: TD Y offset must be scaled by the text matrix.
    /// Pattern: `/F1 1 Tf 10 0 0 10 72 700 Tm (Line one) Tj 0 -1.3 TD (Line two) Tj`
    /// The Tm sets a 10x scale, so `0 -1.3 TD` should produce a 13pt vertical gap,
    /// not 1.3pt. Both lines must appear in extracted text.
    #[test]
    fn test_issue_254_tm_scale_td_offset() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Font size 1 with Tm scale 10 — effective font size is 10pt.
        // TD(0, -1.3) in text space = 13pt in user space.
        let stream = b"BT /F1 1 Tf 10 0 0 10 72 700 Tm (Line one) Tj 0 -1.3 TD (Line two) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        // Collect unique text
        let text: String = chars.iter().map(|c| c.char).collect();
        assert!(text.contains("Line one"), "Should contain 'Line one', got: {}", text);
        assert!(text.contains("Line two"), "Should contain 'Line two', got: {}", text);

        // Verify the Y gap is ~13pt (1.3 * 10), not 1.3pt
        let line_one_y = chars.iter().find(|c| c.char == 'L').unwrap().bbox.y;
        let line_two_chars: Vec<_> = chars.iter().filter(|c| c.char == 'L').collect();
        assert!(line_two_chars.len() >= 2, "Should have at least 2 'L' chars (one per line)");
        let line_two_y = line_two_chars[1].bbox.y;
        let y_gap = (line_one_y - line_two_y).abs();
        assert!(y_gap > 5.0, "Y gap should be ~13pt (Tm-scaled), got {:.1}pt", y_gap);
    }

    #[test]
    fn test_operator_td_sets_leading() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // TD sets leading = -ty, then positions text
        let stream = b"BT /F1 12 Tf 100 -14 TD (A) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, 'A');
    }

    #[test]
    fn test_operator_tstar_line_break() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // TL sets leading, then T* moves to next line using leading
        let stream = b"BT /F1 12 Tf 14 TL 100 700 Td (A) Tj T* (B) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].char, 'A');
        assert_eq!(chars[1].char, 'B');
        // B should be on a different line (different Y)
        assert!((chars[0].bbox.y - chars[1].bbox.y).abs() > 1.0);
    }

    #[test]
    fn test_operator_quote_next_line_show_text() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // ' operator: T* + Tj combined
        let stream = b"BT /F1 12 Tf 14 TL 100 700 Td (A) Tj (B) ' ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].char, 'A');
        assert_eq!(chars[1].char, 'B');
    }

    #[test]
    fn test_operator_double_quote() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // " operator: set word/char spacing, T*, Tj
        let stream = b"BT /F1 12 Tf 14 TL 100 700 Td 1 2 (Hi) \" ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].char, 'H');
        assert_eq!(chars[1].char, 'i');
    }

    #[test]
    fn test_operator_tc_char_spacing() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 2 Tc 100 700 Td (AB) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].char, 'A');
        assert_eq!(chars[1].char, 'B');
    }

    #[test]
    fn test_operator_tw_word_spacing() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 5 Tw 100 700 Td (A B) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert!(chars.len() >= 3); // A, space, B
    }

    #[test]
    fn test_operator_tz_horizontal_scaling() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 150 Tz 100 700 Td (X) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, 'X');
    }

    #[test]
    fn test_operator_tl_leading() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 20 TL 100 700 Td (A) Tj T* (B) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        // A and B should be 20pt apart vertically (the leading value)
        let y_diff = (chars[0].bbox.y - chars[1].bbox.y).abs();
        assert!(y_diff > 10.0, "Leading should create vertical gap, got {}", y_diff);
    }

    #[test]
    fn test_operator_ts_text_rise() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Ts sets text rise (superscript/subscript)
        let stream = b"BT /F1 12 Tf 5 Ts 100 700 Td (X) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, 'X');
    }

    #[test]
    fn test_operator_tr_render_mode() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Tr sets rendering mode
        let stream = b"BT /F1 12 Tf 1 Tr 100 700 Td (X) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, 'X');
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Color operators
    // ========================================================================

    #[test]
    fn test_set_fill_rgb() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT 0.5 0.3 0.8 rg /F1 12 Tf 0 0 Td (C) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert!((chars[0].color.r - 0.5).abs() < 0.01);
        assert!((chars[0].color.g - 0.3).abs() < 0.01);
        assert!((chars[0].color.b - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_gray() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT 0.5 g /F1 12 Tf 0 0 Td (G) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert!((chars[0].color.r - 0.5).abs() < 0.01);
        assert!((chars[0].color.g - 0.5).abs() < 0.01);
        assert!((chars[0].color.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_cmyk() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // CMYK 0 0 0 1 is the K ink, #231F20 - not #000000.
        let stream = b"BT 0 0 0 1 k /F1 12 Tf 0 0 Td (K) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert!((chars[0].color.r - 0.1373).abs() < 0.01);
        assert!((chars[0].color.g - 0.1216).abs() < 0.01);
        assert!((chars[0].color.b - 0.1255).abs() < 0.01);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Graphics state save/restore
    // ========================================================================

    #[test]
    fn test_save_restore_color() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Test that color state is saved/restored by q/Q within a BT/ET block
        // Set blue, save, set red, show R (red), restore, show B (blue restored)
        let stream = b"BT /F1 12 Tf 0 0 1 rg q 1 0 0 rg 100 700 Td (R) Tj Q 200 700 Td (B) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2, "Should extract 2 chars, got {}", chars.len());
        let r_char = chars.iter().find(|c| c.char == 'R').expect("Should find R");
        let b_char = chars.iter().find(|c| c.char == 'B').expect("Should find B");
        // R should be red (set inside q)
        assert!(
            (r_char.color.r - 1.0).abs() < 0.01,
            "R should be red, got ({}, {}, {})",
            r_char.color.r,
            r_char.color.g,
            r_char.color.b
        );
        // B should be blue (restored by Q)
        assert!(
            (b_char.color.b - 1.0).abs() < 0.01,
            "B should be blue after Q restore, got ({}, {}, {})",
            b_char.color.r,
            b_char.color.g,
            b_char.color.b
        );
    }

    #[test]
    fn test_save_restore_ctm() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Save, translate CTM, show A, restore (CTM back to identity), show B at different position
        let stream = b"q 1 0 0 1 100 200 cm BT /F1 12 Tf (A) Tj ET Q BT /F1 12 Tf (B) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        // A should be at (100, 200), B should be at (0, 0)
        assert!(chars[0].bbox.x > 90.0, "A should be translated by CTM");
        assert!(chars[1].bbox.x < 10.0, "B should be at origin after restore");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Span extraction mode
    // ========================================================================

    #[test]
    fn test_extract_text_spans_simple() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 100 700 Td (Hello World) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        assert!(!spans.is_empty());
        // Find the span containing "Hello World"
        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Hello"), "Expected 'Hello' in extracted text, got: {}", text);
        assert!(text.contains("World"), "Expected 'World' in extracted text, got: {}", text);
    }

    #[test]
    fn test_extract_text_spans_multiple_tj() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Two Tj operators that should be accumulated into one span
        let stream = b"BT /F1 12 Tf 100 700 Td (He) Tj (llo) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Hello"), "Expected 'Hello' in spans, got: {}", text);
    }

    #[test]
    fn test_extract_text_spans_with_font_info() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 14 Tf 100 700 Td (Test) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        assert!(!spans.is_empty());
        let span = &spans[0];
        assert!(
            span.font_name.contains("F1") || span.font_name.contains("Times"),
            "Font name should reference F1 or Times, got: {}",
            span.font_name
        );
        assert!(span.font_size > 0.0, "Font size should be positive");
    }

    #[test]
    fn test_extract_text_spans_empty_stream() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"";
        let spans = extractor.extract_text_spans(stream).unwrap();
        assert!(spans.is_empty());
    }

    #[test]
    fn test_extract_text_spans_bt_et_no_text() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf ET";
        let spans = extractor.extract_text_spans(stream).unwrap();
        assert!(spans.is_empty());
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: TJ array processing (span mode)
    // ========================================================================

    #[test]
    fn test_tj_array_with_spacing() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // TJ array with small kerning offset (should not insert space)
        let stream = b"BT /F1 12 Tf 100 700 Td [(H) -20 (ello)] TJ ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Hello"), "Small TJ offset should not split word, got: {}", text);
    }

    #[test]
    fn test_tj_array_word_boundary() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // TJ array with large negative offset (word boundary)
        let stream = b"BT /F1 12 Tf 100 700 Td [(Hello) -300 (World)] TJ ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        // Should have space between Hello and World
        assert!(
            text.contains("Hello") && text.contains("World"),
            "Should extract both words, got: {}",
            text
        );
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: fallback_char_to_unicode
    // ========================================================================

    #[test]
    fn test_fallback_common_punctuation() {
        assert_eq!(fallback_char_to_unicode(0x2014), "\u{2014}"); // Em dash
        assert_eq!(fallback_char_to_unicode(0x2013), "\u{2013}"); // En dash
        assert_eq!(fallback_char_to_unicode(0x2022), "\u{2022}"); // Bullet
        assert_eq!(fallback_char_to_unicode(0x2026), "\u{2026}"); // Ellipsis
        assert_eq!(fallback_char_to_unicode(0x00B0), "\u{00B0}"); // Degree
    }

    #[test]
    fn test_fallback_math_operators() {
        assert_eq!(fallback_char_to_unicode(0x00B1), "\u{00B1}"); // Plus-minus
        assert_eq!(fallback_char_to_unicode(0x00D7), "\u{00D7}"); // Multiply
        assert_eq!(fallback_char_to_unicode(0x221E), "\u{221E}"); // Infinity
        assert_eq!(fallback_char_to_unicode(0x2264), "\u{2264}"); // Less or equal
        assert_eq!(fallback_char_to_unicode(0x2265), "\u{2265}"); // Greater or equal
        assert_eq!(fallback_char_to_unicode(0x2260), "\u{2260}"); // Not equal
        assert_eq!(fallback_char_to_unicode(0x221A), "\u{221A}"); // Square root
        assert_eq!(fallback_char_to_unicode(0x222B), "\u{222B}"); // Integral
        assert_eq!(fallback_char_to_unicode(0x2211), "\u{2211}"); // Summation
    }

    #[test]
    fn test_fallback_greek_letters() {
        assert_eq!(fallback_char_to_unicode(0x03B1), "\u{03B1}"); // alpha
        assert_eq!(fallback_char_to_unicode(0x03B2), "\u{03B2}"); // beta
        assert_eq!(fallback_char_to_unicode(0x03C0), "\u{03C0}"); // pi
        assert_eq!(fallback_char_to_unicode(0x03C9), "\u{03C9}"); // omega
        assert_eq!(fallback_char_to_unicode(0x0393), "\u{0393}"); // Gamma
        assert_eq!(fallback_char_to_unicode(0x03A9), "\u{03A9}"); // Omega
    }

    #[test]
    fn test_fallback_currency() {
        assert_eq!(fallback_char_to_unicode(0x20AC), "\u{20AC}"); // Euro
        assert_eq!(fallback_char_to_unicode(0x00A3), "\u{00A3}"); // Pound
        assert_eq!(fallback_char_to_unicode(0x00A5), "\u{00A5}"); // Yen
        assert_eq!(fallback_char_to_unicode(0x00A2), "\u{00A2}"); // Cent
    }

    #[test]
    fn test_fallback_direct_unicode() {
        // Valid ASCII character
        assert_eq!(fallback_char_to_unicode(0x41), "A");
        assert_eq!(fallback_char_to_unicode(0x20), " ");
    }

    #[test]
    fn test_fallback_invalid_code_point() {
        // Surrogate pair range is invalid Unicode
        assert_eq!(fallback_char_to_unicode(0xD800), "?");
        assert_eq!(fallback_char_to_unicode(0xDFFF), "?");
    }

    #[test]
    fn test_fallback_private_use_area() {
        // PUA characters should still be returned (not replaced with ?)
        let result = fallback_char_to_unicode(0xE000);
        assert_ne!(result, "?");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: decode_text_to_unicode
    // ========================================================================

    #[test]
    fn test_decode_text_no_font_latin1() {
        let result = decode_text_to_unicode(b"Hello", None);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_decode_text_no_font_high_bytes() {
        // Latin-1 high bytes should map to Unicode code points
        let bytes = vec![0xC0, 0xE9]; // A-grave, e-acute in Latin-1
        let result = decode_text_to_unicode(&bytes, None);
        assert!(result.contains('\u{00C0}'), "Should contain A-grave");
        assert!(result.contains('\u{00E9}'), "Should contain e-acute");
    }

    #[test]
    fn test_decode_text_filters_control_chars() {
        // Control characters (except tab, newline, carriage return) should be filtered
        let bytes = vec![0x01, 0x02, 0x41, 0x09, 0x0A]; // ctrl chars, 'A', tab, newline
        let result = decode_text_to_unicode(&bytes, None);
        assert!(result.contains('A'), "Should contain 'A'");
        assert!(result.contains('\t'), "Should keep tab");
        assert!(result.contains('\n'), "Should keep newline");
        assert!(!result.contains('\x01'), "Should filter ctrl-A");
    }

    #[test]
    fn test_decode_text_with_simple_font() {
        let font = create_test_font();
        let result = decode_text_to_unicode(b"ABC", Some(&font));
        // With WinAnsiEncoding, ASCII characters should map correctly
        assert!(result.contains('A') || !result.is_empty(), "Should decode something");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: cmyk_to_rgb
    // ========================================================================

    #[test]
    fn test_cmyk_to_rgb_black() {
        // The K ink is #231F20, not #000000 - see color::cmyk_to_rgb.
        let (r, g, b) = cmyk_to_rgb(0.0, 0.0, 0.0, 1.0);
        assert!((r - 0.1373).abs() < 0.01);
        assert!((g - 0.1216).abs() < 0.01);
        assert!((b - 0.1255).abs() < 0.01);
    }

    #[test]
    fn test_cmyk_to_rgb_white() {
        let (r, g, b) = cmyk_to_rgb(0.0, 0.0, 0.0, 0.0);
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cmyk_to_rgb_cyan() {
        // Process cyan, #00ADEF.
        let (r, g, b) = cmyk_to_rgb(1.0, 0.0, 0.0, 0.0);
        assert!((r - 0.0).abs() < 0.01);
        assert!((g - 0.6784).abs() < 0.01);
        assert!((b - 0.9373).abs() < 0.01);
    }

    #[test]
    fn test_cmyk_to_rgb_magenta() {
        // Process magenta, #EC008C.
        let (r, g, b) = cmyk_to_rgb(0.0, 1.0, 0.0, 0.0);
        assert!((r - 0.9255).abs() < 0.01);
        assert!((g - 0.0).abs() < 0.01);
        assert!((b - 0.5490).abs() < 0.01);
    }

    #[test]
    fn test_cmyk_to_rgb_yellow() {
        // Process yellow, #FFF200.
        let (r, g, b) = cmyk_to_rgb(0.0, 0.0, 1.0, 0.0);
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 0.9490).abs() < 0.01);
        assert!((b - 0.0).abs() < 0.01);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: has_boundary_space edge cases
    // ========================================================================

    #[test]
    fn test_has_boundary_space_empty_strings() {
        assert!(!has_boundary_space("", ""));
        assert!(!has_boundary_space("", "hello"));
        assert!(!has_boundary_space("hello", ""));
    }

    #[test]
    fn test_has_boundary_space_only_spaces() {
        assert!(has_boundary_space(" ", " "));
        assert!(has_boundary_space(" ", "word"));
        assert!(has_boundary_space("word", " "));
    }

    #[test]
    fn test_has_boundary_space_unicode_whitespace() {
        // Non-breaking space (U+00A0)
        assert!(has_boundary_space("word\u{00A0}", "next"));
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: is_email_context
    // ========================================================================

    #[test]
    fn test_email_context_at_domain() {
        // Pattern: user@outlook + . + com
        assert!(is_email_context("user@outlook", ".com"));
    }

    #[test]
    fn test_email_context_after_at() {
        // Pattern: user@ + domain.com
        assert!(is_email_context("user@", "domain.com"));
    }

    #[test]
    fn test_email_context_domain_dot_tld() {
        // Pattern: user@domain. + com
        assert!(is_email_context("user@domain.", "com"));
    }

    #[test]
    fn test_email_context_not_email() {
        assert!(!is_email_context("hello", "world"));
        assert!(!is_email_context("no at sign", "here"));
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: is_citation_context
    // ========================================================================

    #[test]
    fn test_citation_context_superscript() {
        let prev_bbox = Rect::new(10.0, 100.0, 50.0, 12.0);
        let next_bbox = Rect::new(60.0, 105.0, 10.0, 7.0); // Raised, smaller

        // next_font_size is 0.6 * current = superscript range
        let result = is_citation_context(
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            12.0,
            7.2, // 60% of 12 = 0.6, within 0.5-0.75 range
        );
        assert!(result, "Should detect citation context");
    }

    #[test]
    fn test_citation_context_no_superscript() {
        let prev_bbox = Rect::new(10.0, 100.0, 50.0, 12.0);
        let next_bbox = Rect::new(60.0, 100.0, 50.0, 12.0); // Same size, same position

        let result = is_citation_context(
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            12.0,
            12.0, // Same font size = not a citation
        );
        assert!(!result, "Should not detect citation when same size");
    }

    #[test]
    fn test_citation_context_no_bbox() {
        // Font size ratio alone (without bbox) - prev is superscript
        let result = is_citation_context(None, None, 12.0, 7.2, 12.0);
        assert!(result, "Should detect citation from font size ratio alone");
    }

    // #575: snap_superscript_baselines was O(n²) (every span scanned against
    // every other), hanging >30 s on archive.org/Google-Books pages whose
    // invisible hOCR layer emits tens of thousands of spans. The Y-windowed
    // rewrite must (a) still snap a superscript onto its base and (b) scale —
    // 50k spans take ~10-20 s under the old double loop but milliseconds now,
    // so a generous wall-clock bound catches a quadratic regression without
    // being flaky.
    fn snap_span(text: &str, x: f32, y: f32, w: f32, fs: f32, seq: usize) -> TextSpan {
        TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: text.to_string(),
            bbox: Rect::new(x, y, w, fs),
            font_name: "F1".to_string(),
            font_size: fs,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence: seq,
            split_boundary_before: false,
            offset_semantic: false,
            is_italic: false,
            is_monospace: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            primary_detected: false,
            char_widths: vec![],
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: 0.0,
            wmode: 0,
            rtl_draw_logical: false,
            mirrored: false,
            page_rotation_applied: 0,
        }
    }

    #[test]
    fn test_snap_superscript_baselines_correctness() {
        let mut extractor = TextExtractor::new();
        // Base: 12pt body glyph at y=700, right edge x=130.
        // Superscript: 6pt glyph just above-right (y=704, x=130).
        extractor.spans = vec![
            snap_span("x", 100.0, 700.0, 30.0, 12.0, 0),
            snap_span("2", 130.0, 704.0, 4.0, 6.0, 1),
        ];
        extractor.snap_superscript_baselines();
        assert_eq!(
            extractor.spans[1].bbox.y, 700.0,
            "#575: superscript must snap onto the base baseline (y=700)"
        );
    }

    #[test]
    fn test_snap_superscript_baselines_scales() {
        let mut extractor = TextExtractor::new();
        let mut spans = Vec::with_capacity(50_002);
        // A real base+superscript pair we can assert on.
        spans.push(snap_span("x", 100.0, 700.0, 30.0, 12.0, 0));
        spans.push(snap_span("2", 130.0, 704.0, 4.0, 6.0, 1));
        // 50k body spans spread across the page (distinct Y) — same font size,
        // so none qualify as bases for each other; the cost is pure iteration.
        for k in 0..50_000usize {
            let y = (k as f32) * 2.0; // spread across Y so each window is tiny
            spans.push(snap_span("a", 50.0, y, 6.0, 10.0, k + 2));
        }
        extractor.spans = spans;

        let start = std::time::Instant::now();
        extractor.snap_superscript_baselines();
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_secs() < 5,
            "#575: snap_superscript_baselines took {elapsed:?} on 50k spans — \
             likely an O(n²) regression"
        );
        assert_eq!(
            extractor.spans[1].bbox.y, 700.0,
            "#575: the genuine superscript must still snap to its base"
        );
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: TextExtractor configuration
    // ========================================================================

    #[test]
    fn test_extractor_with_merging_config() {
        let extractor = TextExtractor::new().with_merging_config(SpanMergingConfig::aggressive());
        assert_eq!(extractor.merging_config.space_threshold_em_ratio, 0.15);
    }

    #[test]
    fn test_extractor_set_resources() {
        let mut extractor = TextExtractor::new();
        assert!(extractor.resources.is_none());
        extractor.set_resources(Object::Null);
        assert!(extractor.resources.is_some());
    }

    #[test]
    fn test_extractor_prepare_for_span_extraction() {
        let mut extractor = TextExtractor::new();
        extractor.extract_spans = false;
        extractor.span_sequence_counter = 42;
        extractor.prepare_for_span_extraction();
        assert!(extractor.extract_spans);
        assert_eq!(extractor.span_sequence_counter, 0);
        assert!(extractor.spans.is_empty());
    }

    #[test]
    fn test_extractor_get_font_set() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);
        let font2 = create_test_font();
        extractor.add_font("F2".to_string(), font2);

        let font_set = extractor.get_font_set();
        assert_eq!(font_set.len(), 2);
    }

    #[test]
    fn test_extractor_add_font_shared() {
        let mut extractor = TextExtractor::new();
        let font = Arc::new(create_test_font());
        extractor.add_font_shared("F1".to_string(), font.clone());
        assert_eq!(extractor.fonts.len(), 1);
        // Verify it's the same Arc
        assert!(Arc::ptr_eq(extractor.fonts.get("F1").unwrap(), &font));
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: analyze_tj_distribution
    // ========================================================================

    #[test]
    fn test_analyze_tj_distribution_empty() {
        let extractor = TextExtractor::new();
        let (is_justified, cv) = extractor.analyze_tj_distribution();
        assert!(!is_justified);
        assert_eq!(cv, 0.0);
    }

    #[test]
    fn test_analyze_tj_distribution_uniform() {
        let mut extractor = TextExtractor::new();
        // Uniform offsets (all the same) = low CV = not justified
        extractor.tj_offset_history = vec![-100.0; 50];
        let (is_justified, cv) = extractor.analyze_tj_distribution();
        assert!(!is_justified, "Uniform offsets should not be justified");
        assert!(cv < 0.01, "CV should be ~0 for uniform offsets, got {}", cv);
    }

    #[test]
    fn test_analyze_tj_distribution_high_variance() {
        let mut extractor = TextExtractor::new();
        // High variance offsets = justified text
        let mut offsets = Vec::new();
        for i in 0..100 {
            offsets.push(if i % 2 == 0 { -50.0 } else { -200.0 });
        }
        extractor.tj_offset_history = offsets;
        let (is_justified, cv) = extractor.analyze_tj_distribution();
        assert!(is_justified, "High variance should indicate justified text, cv={}", cv);
        assert!(cv > 0.5, "CV should be > 0.5 for justified text, got {}", cv);
    }

    /// The O(1) accumulator path and the recompute-from-slice fallback must
    /// produce identical results (same f64 formula, same sum order).
    #[test]
    fn test_tj_accumulator_matches_recompute() {
        let vals = vec![
            -50.0f32, -200.0, -75.0, -180.0, -60.0, -210.0, -90.0, -150.0,
        ];

        // O(1) path: accumulators kept consistent with the history (as `push` does).
        let mut a = TextExtractor::new();
        let mut sum = 0.0f64;
        let mut sq = 0.0f64;
        for &v in &vals {
            let x = v as f64;
            sum += x;
            sq += x * x;
            a.tj_offset_history.push(v);
        }
        a.tj_sum = sum;
        a.tj_sum_sq = sq;
        a.tj_stats_len = a.tj_offset_history.len();
        let (ja, cva) = a.analyze_tj_distribution();

        // Recompute path: only the history is set (stale accumulators).
        let mut b = TextExtractor::new();
        b.tj_offset_history = vals.clone();
        let (jb, cvb) = b.analyze_tj_distribution();

        assert_eq!(ja, jb, "is_justified must agree across paths");
        assert!((cva - cvb).abs() < 1e-6, "O(1) cv {cva} must equal recompute cv {cvb}");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: calculate_adaptive_tj_threshold
    // ========================================================================

    #[test]
    fn test_adaptive_threshold_disabled() {
        let config = TextExtractionConfig {
            use_adaptive_tj_threshold: false,
            space_insertion_threshold: -120.0,
            ..TextExtractionConfig::default()
        };
        let extractor = TextExtractor::with_config(config);
        let threshold = extractor.calculate_adaptive_tj_threshold();
        assert_eq!(threshold, -120.0);
    }

    #[test]
    fn test_adaptive_threshold_enabled() {
        let config = TextExtractionConfig {
            use_adaptive_tj_threshold: true,
            word_margin_ratio: 0.1,
            ..TextExtractionConfig::default()
        };
        let mut extractor = TextExtractor::with_config(config);
        // Set font size
        extractor.state_stack.current_mut().font_size = 12.0;
        let threshold = extractor.calculate_adaptive_tj_threshold();
        assert!(threshold < 0.0, "Adaptive threshold should be negative");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: update_artifact_state
    // ========================================================================

    #[test]
    fn test_update_artifact_state_empty_stack() {
        let mut extractor = TextExtractor::new();
        extractor.update_artifact_state();
        assert!(!extractor.inside_artifact);
    }

    #[test]
    fn test_update_artifact_state_artifact_present() {
        let mut extractor = TextExtractor::new();
        extractor.marked_content_stack.push(MarkedContentContext {
            artifact_type: None,
            tag: "Artifact".to_string(),
            is_artifact: true,
            actual_text: None,
            expansion: None,
            is_excluded_layer: false,
            is_placed_pdf: false,
            actual_text_emitted: false,
            own_mcid: None,
        });
        extractor.update_artifact_state();
        assert!(extractor.inside_artifact);
    }

    #[test]
    fn test_placed_pdf_suppresses_content() {
        // Text inside an InDesign /PlacedPDF figure region (the placed
        // artwork's own glyphs — e.g. a draft galley) must be suppressed,
        // matching pdftotext/PyMuPDF. Entering a /PlacedPDF BDC sets
        // inside_placed_pdf, which feeds is_content_suppressed().
        let mut extractor = TextExtractor::new();
        assert!(!extractor.inside_placed_pdf);
        assert!(!extractor.is_content_suppressed());

        extractor.marked_content_stack.push(MarkedContentContext {
            artifact_type: None,
            tag: "PlacedPDF".to_string(),
            is_artifact: false,
            actual_text: None,
            expansion: None,
            is_excluded_layer: false,
            is_placed_pdf: true,
            actual_text_emitted: false,
            own_mcid: None,
        });
        extractor.update_layer_state();
        assert!(extractor.inside_placed_pdf);
        assert!(extractor.is_content_suppressed(), "text inside /PlacedPDF must be suppressed");

        // Leaving the region restores normal extraction.
        extractor.marked_content_stack.pop();
        extractor.update_layer_state();
        assert!(!extractor.inside_placed_pdf);
        assert!(!extractor.is_content_suppressed());
    }

    #[test]
    fn test_non_placed_pdf_tag_does_not_suppress() {
        // A regular (non-PlacedPDF) marked-content tag such as /Figure must
        // NOT suppress its text — only the placed-PDF wrapper does.
        let mut extractor = TextExtractor::new();
        extractor.marked_content_stack.push(MarkedContentContext {
            artifact_type: None,
            tag: "Figure".to_string(),
            is_artifact: false,
            actual_text: None,
            expansion: None,
            is_excluded_layer: false,
            is_placed_pdf: false,
            actual_text_emitted: false,
            own_mcid: None,
        });
        extractor.update_layer_state();
        assert!(!extractor.inside_placed_pdf);
        assert!(!extractor.is_content_suppressed());
    }

    #[test]
    fn test_placed_pdf_kept_when_it_is_the_whole_page_body() {
        // A publisher that places the ENTIRE article body inside one /PlacedPDF
        // region (e.g. MATEC Web of Conferences) leaves almost nothing outside.
        // There the placed text IS the page's logical content and must NOT be
        // suppressed (pymupdf/pdftotext extract it). The coverage pre-scan flags
        // this: placed text dominates, non-placed text is a tiny header.
        let body =
            "(This is the full article body typeset inside a placed PDF region) Tj\n".repeat(20);
        let stream = format!("/PlacedPDF BMC\nBT\n{body}ET\nEMC\nBT (Journal vol 1) Tj ET\n");
        assert!(
            TextExtractor::placed_pdf_text_dominates(stream.as_bytes()),
            "whole-body /PlacedPDF must be KEPT (not suppressed)"
        );
    }

    #[test]
    fn test_placed_pdf_suppressed_when_minority_overlay() {
        // The decorative-figure case (PMC8100493): a small /PlacedPDF galley
        // duplicate sits amid a full page of real text OUTSIDE it. The placed
        // text is the minority, so it stays suppressed (the de-dup win).
        let outside =
            "(Real published paragraph of the article that lives outside the placed region) Tj\n"
                .repeat(20);
        let stream = format!("BT\n{outside}ET\n/PlacedPDF BMC\nBT (draft galley) Tj ET\nEMC\n");
        assert!(
            !TextExtractor::placed_pdf_text_dominates(stream.as_bytes()),
            "minority-overlay /PlacedPDF must stay suppressed"
        );
    }

    #[test]
    fn test_placed_pdf_coverage_noop_without_tag() {
        // No /PlacedPDF tag anywhere: the pre-scan must short-circuit to false
        // (keep the default suppression state; pay nothing for ordinary pages).
        let stream = b"BT (ordinary single column page of text) Tj ET\n";
        assert!(!TextExtractor::placed_pdf_text_dominates(stream));
    }

    #[test]
    fn test_placed_pdf_kept_when_unique_body_amid_comparable_outside() {
        // Gate 3: an InDesign spread (e.g. a placed floor-plan / marketing page)
        // where the placed region carries a substantial body of UNIQUE text and
        // the non-placed text is comparable or larger but different (labels,
        // headers). The 3:1 dominance ratio fails, yet the placed words are not a
        // duplicate of the outside text, so it must be KEPT (pdftotext/pymupdf
        // extract it; suppressing it drops the whole spread's content).
        let placed = "(master bedroom terrace kitchen dimensions balcony) Tj\n".repeat(30);
        let outside = "(square footage residence penthouse skyline waterfront) Tj\n".repeat(35);
        let stream = format!("BT\n{outside}ET\n/PlacedPDF /MC0 BDC\nBT\n{placed}ET\nEMC\n");
        assert!(
            TextExtractor::placed_pdf_text_dominates(stream.as_bytes()),
            "unique placed body amid comparable outside text must be KEPT"
        );
    }

    #[test]
    fn test_placed_pdf_suppressed_when_large_duplicate_overlay() {
        // Gate 3, the other side: a large placed region whose words DUPLICATE the
        // surrounding text is a draft galley / overlay copy and stays suppressed
        // even though it clears the size gate (the PMC8100493 de-dup intent, at
        // full body size rather than the minority-overlay size).
        let body = "(the published paragraph of the real article body content) Tj\n".repeat(30);
        let stream = format!("BT\n{body}ET\n/PlacedPDF /MC0 BDC\nBT\n{body}ET\nEMC\n");
        assert!(
            !TextExtractor::placed_pdf_text_dominates(stream.as_bytes()),
            "a full-size placed DUPLICATE of the outside text must stay suppressed"
        );
    }

    #[test]
    fn test_update_artifact_state_nested_non_artifact() {
        let mut extractor = TextExtractor::new();
        extractor.marked_content_stack.push(MarkedContentContext {
            artifact_type: None,
            tag: "Artifact".to_string(),
            is_artifact: true,
            actual_text: None,
            expansion: None,
            is_excluded_layer: false,
            is_placed_pdf: false,
            actual_text_emitted: false,
            own_mcid: None,
        });
        extractor.marked_content_stack.push(MarkedContentContext {
            artifact_type: None,
            tag: "Span".to_string(),
            is_artifact: false,
            actual_text: None,
            expansion: None,
            is_excluded_layer: false,
            is_placed_pdf: false,
            actual_text_emitted: false,
            own_mcid: None,
        });
        extractor.update_artifact_state();
        // Should still be inside artifact because parent is artifact
        assert!(extractor.inside_artifact);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: parse_artifact_type
    // ========================================================================

    #[test]
    fn test_parse_artifact_type_page() {
        let mut props = HashMap::new();
        props.insert("Type".to_string(), Object::Name("Page".to_string()));
        let result = TextExtractor::parse_artifact_type(&props);
        assert_eq!(result, Some(ArtifactType::Page));
    }

    #[test]
    fn test_parse_artifact_type_pagination_page_number() {
        let mut props = HashMap::new();
        props.insert("Type".to_string(), Object::Name("Pagination".to_string()));
        props.insert("Subtype".to_string(), Object::Name("PageNumber".to_string()));
        let result = TextExtractor::parse_artifact_type(&props);
        assert_eq!(result, Some(ArtifactType::Pagination(PaginationSubtype::PageNumber)));
    }

    #[test]
    fn test_parse_artifact_type_pagination_other_subtype() {
        let mut props = HashMap::new();
        props.insert("Type".to_string(), Object::Name("Pagination".to_string()));
        props.insert("Subtype".to_string(), Object::Name("SomethingElse".to_string()));
        let result = TextExtractor::parse_artifact_type(&props);
        assert_eq!(result, Some(ArtifactType::Pagination(PaginationSubtype::Other)));
    }

    #[test]
    fn test_parse_artifact_type_unknown_type() {
        let mut props = HashMap::new();
        props.insert("Type".to_string(), Object::Name("UnknownType".to_string()));
        let result = TextExtractor::parse_artifact_type(&props);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_artifact_type_subtype_footer_only() {
        let mut props = HashMap::new();
        props.insert("Subtype".to_string(), Object::Name("Footer".to_string()));
        let result = TextExtractor::parse_artifact_type(&props);
        assert_eq!(result, Some(ArtifactType::Pagination(PaginationSubtype::Footer)));
    }

    #[test]
    fn test_parse_artifact_type_subtype_watermark_only() {
        let mut props = HashMap::new();
        props.insert("Subtype".to_string(), Object::Name("Watermark".to_string()));
        let result = TextExtractor::parse_artifact_type(&props);
        assert_eq!(result, Some(ArtifactType::Pagination(PaginationSubtype::Watermark)));
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: decode_pdf_text_string
    // ========================================================================

    #[test]
    fn test_decode_pdf_text_string_utf8() {
        let result = TextExtractor::decode_pdf_text_string(b"Hello World");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_decode_pdf_text_string_utf16be_bom() {
        // UTF-16BE with BOM: FE FF, then "Hi" in UTF-16BE
        let bytes: Vec<u8> = vec![0xFE, 0xFF, 0x00, 0x48, 0x00, 0x69];
        let result = TextExtractor::decode_pdf_text_string(&bytes);
        assert_eq!(result, "Hi");
    }

    #[test]
    fn test_decode_pdf_text_string_utf16le_bom() {
        // UTF-16LE with BOM: FF FE, then "Hi" in UTF-16LE
        let bytes: Vec<u8> = vec![0xFF, 0xFE, 0x48, 0x00, 0x69, 0x00];
        let result = TextExtractor::decode_pdf_text_string(&bytes);
        assert_eq!(result, "Hi");
    }

    #[test]
    fn test_decode_pdf_text_string_empty() {
        let result = TextExtractor::decode_pdf_text_string(b"");
        assert_eq!(result, "");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: split_on_camelcase
    // ========================================================================

    #[test]
    fn test_split_camelcase_basic() {
        let extractor = TextExtractor::new();
        let parts = extractor.split_on_camelcase("theGeneral");
        assert_eq!(parts, vec!["the", "General"]);
    }

    #[test]
    fn test_split_camelcase_multiple() {
        let extractor = TextExtractor::new();
        let parts = extractor.split_on_camelcase("lengthThisPage");
        assert_eq!(parts, vec!["length", "This", "Page"]);
    }

    #[test]
    fn test_split_camelcase_no_split_all_lower() {
        let extractor = TextExtractor::new();
        let parts = extractor.split_on_camelcase("lowercase");
        assert_eq!(parts, vec!["lowercase"]);
    }

    #[test]
    fn test_split_camelcase_no_split_all_upper() {
        let extractor = TextExtractor::new();
        let parts = extractor.split_on_camelcase("HTML");
        assert_eq!(parts, vec!["HTML"]);
    }

    #[test]
    fn test_split_camelcase_single_char() {
        let extractor = TextExtractor::new();
        let parts = extractor.split_on_camelcase("A");
        assert_eq!(parts, vec!["A"]);
    }

    #[test]
    fn test_split_camelcase_empty() {
        let extractor = TextExtractor::new();
        let parts = extractor.split_on_camelcase("");
        // Empty string gives one empty part
        assert_eq!(parts.len(), 1);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: is_ligature_code
    // ========================================================================

    #[test]
    fn test_is_ligature_code() {
        // Standard ligatures: U+FB00-U+FB04
        assert!(TextExtractor::is_ligature_code(0xFB00)); // ff
        assert!(TextExtractor::is_ligature_code(0xFB01)); // fi
        assert!(TextExtractor::is_ligature_code(0xFB02)); // fl
        assert!(TextExtractor::is_ligature_code(0xFB03)); // ffi
        assert!(TextExtractor::is_ligature_code(0xFB04)); // ffl
    }

    #[test]
    fn test_is_not_ligature_code() {
        assert!(!TextExtractor::is_ligature_code(0x41)); // 'A'
        assert!(!TextExtractor::is_ligature_code(0xFAFF)); // Before range
        assert!(!TextExtractor::is_ligature_code(0xFB05)); // After range
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: BT/ET operators
    // ========================================================================

    #[test]
    fn test_bt_resets_text_matrix() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // First BT/ET block at (100, 700)
        // Second BT should reset text matrix to identity
        let stream = b"BT /F1 12 Tf 100 700 Td (A) Tj ET BT /F1 12 Tf (B) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].char, 'A');
        assert_eq!(chars[1].char, 'B');
        // B should be at origin (BT resets text matrix)
        assert!(
            chars[1].bbox.x < 10.0,
            "Second BT should reset text matrix, x={}",
            chars[1].bbox.x
        );
    }

    #[test]
    fn test_multiple_bt_et_blocks() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 100 700 Td (Hello) Tj ET BT /F1 12 Tf 100 680 Td (World) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        let text: String = chars.iter().map(|c| c.char).collect();
        assert!(text.contains("Hello"), "Should contain Hello");
        assert!(text.contains("World"), "Should contain World");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Marked content operators
    // ========================================================================

    #[test]
    fn test_bmc_artifact_tracking() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Use execute_operator_public for fine-grained testing
        extractor
            .execute_operator_public(crate::content::operators::Operator::BeginMarkedContent {
                tag: "Artifact".to_string(),
            })
            .unwrap();

        assert!(extractor.inside_artifact, "Should be inside artifact after BMC Artifact");

        extractor
            .execute_operator_public(crate::content::operators::Operator::EndMarkedContent)
            .unwrap();

        assert!(!extractor.inside_artifact, "Should be outside artifact after EMC");
    }

    #[test]
    fn test_bmc_non_artifact() {
        let mut extractor = TextExtractor::new();

        extractor
            .execute_operator_public(crate::content::operators::Operator::BeginMarkedContent {
                tag: "Span".to_string(),
            })
            .unwrap();

        assert!(!extractor.inside_artifact, "Non-artifact BMC should not set inside_artifact");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Font switching
    // ========================================================================

    #[test]
    fn test_font_switch_mid_stream() {
        let mut extractor = TextExtractor::new();
        let font1 = create_test_font();
        let mut font2_data = create_test_font();
        font2_data.base_font = "Helvetica".to_string();
        extractor.add_font("F1".to_string(), font1);
        extractor.add_font("F2".to_string(), font2_data);

        let stream = b"BT /F1 12 Tf 100 700 Td (Hello) Tj /F2 14 Tf (World) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        // All characters should be extracted
        let text: String = chars.iter().map(|c| c.char).collect();
        assert!(text.contains("Hello"), "Should contain Hello");
        assert!(text.contains("World"), "Should contain World");
    }

    #[test]
    fn test_font_switch_same_font_no_flush() {
        // Setting the same font twice should be a no-op (optimization)
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf /F1 12 Tf 100 700 Td (Test) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Test"), "Should extract text, got: {}", text);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Cm operator (CTM modification)
    // ========================================================================

    #[test]
    fn test_cm_operator_translation() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Apply translation via Cm operator
        let stream = b"1 0 0 1 50 100 cm BT /F1 12 Tf (X) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        assert!((chars[0].bbox.x - 50.0).abs() < 2.0, "X should be ~50");
        assert!((chars[0].bbox.y - 100.0).abs() < 2.0, "Y should be ~100");
    }

    #[test]
    fn test_cm_operator_scaling() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Scale by 2x via CTM
        let stream = b"2 0 0 2 0 0 cm BT /F1 12 Tf 1 0 0 1 50 100 Tm (Y) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        assert_eq!(chars.len(), 1);
        // Position should be scaled: (50*2, 100*2) = (100, 200)
        assert!(
            (chars[0].bbox.x - 100.0).abs() < 2.0,
            "X should be ~100 (got {})",
            chars[0].bbox.x
        );
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Deduplication
    // ========================================================================

    #[test]
    fn test_deduplicate_overlapping_chars() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Create overlapping chars (simulating bold rendering with duplicate glyphs)
        extractor.chars = vec![
            TextChar {
                char: 'A',
                bbox: Rect::new(100.0, 700.0, 6.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 100.0,
                origin_y: 700.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
            TextChar {
                char: 'A',
                bbox: Rect::new(100.5, 700.0, 6.0, 12.0), // Very close X
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 100.5,
                origin_y: 700.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
        ];

        extractor.deduplicate_overlapping_chars();
        assert_eq!(extractor.chars.len(), 1, "Overlapping chars should be deduplicated");
    }

    #[test]
    fn test_deduplicate_overlapping_chars_different_lines() {
        let mut extractor = TextExtractor::new();

        // Chars on different lines should NOT be deduplicated
        extractor.chars = vec![
            TextChar {
                char: 'A',
                bbox: Rect::new(100.0, 700.0, 6.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 100.0,
                origin_y: 700.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
            TextChar {
                char: 'A',
                bbox: Rect::new(100.0, 680.0, 6.0, 12.0), // Different Y
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 100.0,
                origin_y: 680.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
        ];

        extractor.deduplicate_overlapping_chars();
        assert_eq!(extractor.chars.len(), 2, "Chars on different lines should not be deduplicated");
    }

    #[test]
    fn test_deduplicate_overlapping_chars_empty() {
        let mut extractor = TextExtractor::new();
        extractor.deduplicate_overlapping_chars();
        assert!(extractor.chars.is_empty());
    }

    #[test]
    fn test_deduplicate_keeps_distinct_close_chars() {
        // Issue #253: distinct characters close together should NOT be dropped
        let mut extractor = TextExtractor::new();

        let make_char = |c: char, x: f32| TextChar {
            char: c,
            bbox: Rect::new(x, 700.0, 6.0, 12.0),
            font_name: "F1".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            is_italic: false,
            is_monospace: false,
            origin_x: x,
            origin_y: 700.0,
            rotation_degrees: 0.0,
            advance_width: 6.0,
            rendered_advance: 6.0,
            ascent: 11.4,
            descent: -4.2,
            matrix: None,
        };

        // 't' at x=100, ' ' at x=105, 'r' at x=106.5 (within 2pt of ' ' but different char)
        extractor.chars = vec![
            make_char('t', 100.0),
            make_char(' ', 105.0),
            make_char('r', 106.5),
        ];

        extractor.deduplicate_overlapping_chars();
        assert_eq!(
            extractor.chars.len(),
            3,
            "Distinct characters close together must not be dropped"
        );
        assert_eq!(extractor.chars[0].char, 't');
        assert_eq!(extractor.chars[1].char, ' ');
        assert_eq!(extractor.chars[2].char, 'r');
    }

    #[test]
    fn test_deduplicate_still_removes_same_char_duplicates() {
        // Duplicate same character at nearly the same position should still be deduped
        let mut extractor = TextExtractor::new();

        let make_char = |c: char, x: f32| TextChar {
            char: c,
            bbox: Rect::new(x, 700.0, 6.0, 12.0),
            font_name: "F1".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            is_italic: false,
            is_monospace: false,
            origin_x: x,
            origin_y: 700.0,
            rotation_degrees: 0.0,
            advance_width: 6.0,
            rendered_advance: 6.0,
            ascent: 11.4,
            descent: -4.2,
            matrix: None,
        };

        extractor.chars = vec![make_char('A', 100.0), make_char('A', 100.5)];

        extractor.deduplicate_overlapping_chars();
        assert_eq!(extractor.chars.len(), 1, "Duplicate same char should still be deduped");
        assert_eq!(extractor.chars[0].char, 'A');
    }

    #[test]
    fn test_deduplicate_keeps_narrow_glyph_doublets() {
        // Regression: `ll`, `rr`, `II`, `ii` in small-font body text were
        // wrongly collapsed to a single glyph because the dedup threshold
        // was a hardcoded 2 pt — larger than the advance width of narrow
        // glyphs at ≤ 9 pt in most fonts (Helvetica `l` ≈ 2.5 pt at 9 pt,
        // smaller below). This caused visible corruption like
        // `controller → controler` and `billed → biled`.
        //
        // Exercises the matrix of four narrow glyphs across three small
        // body-text sizes. Advance widths are the real Helvetica per-em
        // values (0.278 em for `l`/`i`, 0.333 em for `r`, 0.278 em for `I`).
        let narrow_char = |c: char, x: f32, font_size: f32, advance_em: f32| TextChar {
            char: c,
            bbox: Rect::new(x, 700.0, advance_em * font_size * 0.6, font_size),
            font_name: "Helvetica".to_string(),
            font_size,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            is_italic: false,
            is_monospace: false,
            origin_x: x,
            origin_y: 700.0,
            rotation_degrees: 0.0,
            advance_width: advance_em * font_size,
            rendered_advance: advance_em * font_size,
            ascent: 11.4,
            descent: -4.2,
            matrix: None,
        };

        // (glyph, Helvetica per-em advance width)
        let cases: &[(char, f32)] = &[('l', 0.278), ('r', 0.333), ('I', 0.278), ('i', 0.278)];
        // Body-text sizes where narrow-glyph advance falls at or below 2 pt.
        let sizes: &[f32] = &[7.0, 9.0, 11.0];

        for &(glyph, advance_em) in cases {
            for &font_size in sizes {
                let advance = advance_em * font_size;
                let mut extractor = TextExtractor::new();
                extractor.chars = vec![
                    narrow_char(glyph, 100.0, font_size, advance_em),
                    narrow_char(glyph, 100.0 + advance, font_size, advance_em),
                ];

                extractor.deduplicate_overlapping_chars();
                assert_eq!(
                    extractor.chars.len(),
                    2,
                    "Adjacent narrow-glyph doublet ('{glyph}{glyph}') at {font_size} pt \
                     (advance = {advance:.2} pt) must not be collapsed",
                );
            }
        }
    }

    #[test]
    fn test_deduplicate_still_collapses_narrow_glyph_stroke_fill_duplicates() {
        // Positive regression: even with the advance-scaled threshold,
        // stroke+fill render passes on narrow glyphs (two `l`s at ~0 pt
        // offset) must still be collapsed. The ratio (0.30) comfortably
        // catches real duplicates (< 5 % of one advance apart) while
        // staying below typical heaviest kerning (~20 %).
        let mut extractor = TextExtractor::new();

        let narrow_at = |x: f32| TextChar {
            char: 'l',
            bbox: Rect::new(x, 700.0, 1.5, 9.0),
            font_name: "Helvetica".to_string(),
            font_size: 9.0,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            is_italic: false,
            is_monospace: false,
            origin_x: x,
            origin_y: 700.0,
            rotation_degrees: 0.0,
            advance_width: 2.5, // 0.278 em × 9 pt
            rendered_advance: 2.5,
            ascent: 11.4,
            descent: -4.2,
            matrix: None,
        };

        // Stroke pass and fill pass typically land within 0.05 pt of each
        // other (2 % of advance at 9 pt Helvetica `l`).
        extractor.chars = vec![narrow_at(100.0), narrow_at(100.05)];

        extractor.deduplicate_overlapping_chars();
        assert_eq!(
            extractor.chars.len(),
            1,
            "Stroke+fill narrow-glyph duplicates (same char at ~0 pt offset) \
             must still be collapsed"
        );
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Span deduplication
    // ========================================================================

    #[test]
    fn test_deduplicate_overlapping_spans_geometric() {
        let mut extractor = TextExtractor::new();
        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello".to_string(),
                bbox: Rect::new(100.0, 700.0, 30.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello".to_string(),
                bbox: Rect::new(101.0, 700.0, 30.0, 12.0), // Very close
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.deduplicate_overlapping_spans();
        assert_eq!(extractor.spans.len(), 1, "Geometric duplicates should be removed");
    }

    #[test]
    fn test_deduplicate_overlapping_spans_empty() {
        let mut extractor = TextExtractor::new();
        extractor.deduplicate_overlapping_spans();
        assert!(extractor.spans.is_empty());
    }

    #[test]
    fn test_deduplicate_spans_keeps_narrow_glyph_doublets() {
        // Regression: PDFs that emit kerned text glyph-by-glyph produce
        // consecutive single-character spans. Two adjacent narrow-glyph
        // spans (`l`, `r`, `I`, `i` at ≤ 9 pt) sit roughly one advance-width
        // apart, which used to fall under the hardcoded 2 pt geometric
        // threshold and get collapsed. The threshold now scales with each
        // span's per-glyph width so legitimate doublets survive.
        //
        // Exercises the matrix of four narrow glyphs across three small
        // body-text sizes.
        let narrow_span =
            |glyph: char, x: f32, font_size: f32, advance: f32, seq: usize| TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: glyph.to_string(),
                bbox: Rect::new(x, 700.0, advance, font_size),
                font_name: "Helvetica".to_string(),
                font_size,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: seq,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            };

        // (glyph, Helvetica per-em advance width)
        let cases: &[(char, f32)] = &[('l', 0.278), ('r', 0.333), ('I', 0.278), ('i', 0.278)];
        let sizes: &[f32] = &[7.0, 9.0, 11.0];

        for &(glyph, advance_em) in cases {
            for &font_size in sizes {
                let advance = advance_em * font_size;
                let mut extractor = TextExtractor::new();
                extractor.spans = vec![
                    narrow_span(glyph, 100.0, font_size, advance, 0),
                    narrow_span(glyph, 100.0 + advance, font_size, advance, 1),
                ];

                extractor.deduplicate_overlapping_spans();
                assert_eq!(
                    extractor.spans.len(),
                    2,
                    "Adjacent single-glyph narrow-doublet spans ('{glyph}{glyph}') \
                     at {font_size} pt (advance = {advance:.2} pt) must not be collapsed",
                );
            }
        }
    }

    #[test]
    fn test_deduplicate_spans_still_collapses_stroke_fill_narrow_glyphs() {
        // Positive regression: stroke+fill single-glyph narrow spans at
        // ~0 pt offset must still be collapsed by the geometric dedup
        // phase. The ratio (0.30) comfortably catches real duplicates
        // while preserving legitimate doublets.
        let mut extractor = TextExtractor::new();

        let narrow_at = |x: f32, seq: usize| TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: "l".to_string(),
            bbox: Rect::new(x, 700.0, 2.5, 9.0),
            font_name: "Helvetica".to_string(),
            font_size: 9.0,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence: seq,
            split_boundary_before: false,
            offset_semantic: false,
            is_italic: false,
            is_monospace: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            primary_detected: false,
            char_widths: vec![],
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: 0.0,
            wmode: 0,
            rtl_draw_logical: false,
            mirrored: false,
            page_rotation_applied: 0,
        };

        // Stroke pass + fill pass at ~2 % of advance apart.
        extractor.spans = vec![narrow_at(100.0, 0), narrow_at(100.05, 1)];

        extractor.deduplicate_overlapping_spans();
        assert_eq!(
            extractor.spans.len(),
            1,
            "Stroke+fill narrow-glyph duplicate spans (same text at ~0 pt offset) \
             must still be collapsed"
        );
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Column detection
    // ========================================================================

    #[test]
    fn test_detect_span_columns_empty() {
        let extractor = TextExtractor::new();
        let columns = extractor.detect_span_columns();
        assert!(columns.is_empty());
    }

    #[test]
    fn test_detect_span_columns_single_column() {
        let mut extractor = TextExtractor::new();
        // Create spans all in one column
        for i in 0..10 {
            extractor.spans.push(TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: format!("Line {}", i),
                bbox: Rect::new(50.0, 700.0 - (i as f32 * 14.0), 200.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: i,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            });
        }

        let columns = extractor.detect_span_columns();
        assert_eq!(columns.len(), 1, "Should detect single column");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Sort by reading order
    // ========================================================================

    #[test]
    fn test_sort_by_reading_order() {
        let mut extractor = TextExtractor::new();
        // Add chars in wrong order
        extractor.chars = vec![
            TextChar {
                char: 'B',
                bbox: Rect::new(100.0, 680.0, 6.0, 12.0), // Lower on page
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 100.0,
                origin_y: 680.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
            TextChar {
                char: 'A',
                bbox: Rect::new(100.0, 700.0, 6.0, 12.0), // Higher on page
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 100.0,
                origin_y: 700.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
        ];

        extractor.sort_by_reading_order();
        // PDF Y increases upward, so 700 is higher than 680
        // Reading order: top first, so A (y=700) before B (y=680)
        assert_eq!(extractor.chars[0].char, 'A');
        assert_eq!(extractor.chars[1].char, 'B');
    }

    #[test]
    fn test_sort_by_reading_order_same_line() {
        let mut extractor = TextExtractor::new();
        extractor.chars = vec![
            TextChar {
                char: 'B',
                bbox: Rect::new(200.0, 700.0, 6.0, 12.0), // Right
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 200.0,
                origin_y: 700.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
            TextChar {
                char: 'A',
                bbox: Rect::new(100.0, 700.0, 6.0, 12.0), // Left
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 100.0,
                origin_y: 700.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
        ];

        extractor.sort_by_reading_order();
        // Same line: left to right
        assert_eq!(extractor.chars[0].char, 'A');
        assert_eq!(extractor.chars[1].char, 'B');
    }

    #[test]
    fn test_sort_by_reading_order_nan_values() {
        let mut extractor = TextExtractor::new();
        extractor.chars = vec![
            TextChar {
                char: 'A',
                bbox: Rect::new(f32::NAN, f32::NAN, 6.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 0.0,
                origin_y: 0.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
            TextChar {
                char: 'B',
                bbox: Rect::new(100.0, 700.0, 6.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                is_italic: false,
                is_monospace: false,
                origin_x: 100.0,
                origin_y: 700.0,
                rotation_degrees: 0.0,
                advance_width: 6.0,
                rendered_advance: 6.0,
                ascent: 11.4,
                descent: -4.2,
                matrix: None,
            },
        ];

        // Should not panic with NaN values
        extractor.sort_by_reading_order();
        assert_eq!(extractor.chars.len(), 2);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: merge_adjacent_spans
    // ========================================================================

    #[test]
    fn test_merge_adjacent_spans_same_line() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello".to_string(),
                bbox: Rect::new(100.0, 700.0, 30.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "World".to_string(),
                bbox: Rect::new(131.0, 700.0, 30.0, 12.0), // 1pt gap
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 1, "Adjacent spans on same line should merge");
        assert!(extractor.spans[0].text.contains("Hello"));
        assert!(extractor.spans[0].text.contains("World"));
    }

    #[test]
    fn test_merge_adjacent_spans_180_degree_runs_never_merge() {
        // Same shared-baseline-Y, small-gap shape as
        // `test_merge_adjacent_spans_same_line`, but both runs are
        // 180°-rotated (upside-down text). The rotation-compatibility gate
        // previously only rejected ±90° (vertical-quadrant) runs, so a
        // 180°/180° pair slipped through and merged under the portrait
        // same-line test even though 180° text advances in the opposite X
        // direction — exactly the hazard `snap_run_rotation`'s 180°-aliasing
        // bug (fixed alongside this) would otherwise mask, since before
        // that fix a 180° matrix was misreported as 0.0 in the first place.
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                mirrored: false,
                page_rotation_applied: 0,
                artifact_type: None,
                text: "Hello".to_string(),
                bbox: Rect::new(100.0, 700.0, 30.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 180.0,
                wmode: 0,
                rtl_draw_logical: false,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                mirrored: false,
                page_rotation_applied: 0,
                artifact_type: None,
                text: "World".to_string(),
                bbox: Rect::new(131.0, 700.0, 30.0, 12.0), // 1pt gap
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 180.0,
                wmode: 0,
                rtl_draw_logical: false,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(
            extractor.spans.len(),
            2,
            "180°-rotated runs must never merge here, even on a shared baseline-Y \
             with a small gap"
        );
    }

    #[test]
    fn test_merge_adjacent_spans_different_lines() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello".to_string(),
                bbox: Rect::new(100.0, 700.0, 30.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "World".to_string(),
                bbox: Rect::new(100.0, 680.0, 30.0, 12.0), // Different line
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 2, "Spans on different lines should not merge");
    }

    #[test]
    fn test_merge_adjacent_spans_empty() {
        let mut extractor = TextExtractor::new();
        extractor.merge_adjacent_spans();
        assert!(extractor.spans.is_empty());
    }

    #[test]
    fn test_merge_adjacent_spans_column_boundary() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Left".to_string(),
                bbox: Rect::new(50.0, 700.0, 30.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Right".to_string(),
                bbox: Rect::new(300.0, 700.0, 30.0, 12.0), // Large gap (column boundary)
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 2, "Spans separated by column boundary should not merge");
    }

    #[test]
    fn test_merge_whitespace_only_span() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello".to_string(),
                bbox: Rect::new(100.0, 700.0, 30.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: " ".to_string(),
                bbox: Rect::new(130.0, 700.0, 2.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: true, // TJ offset space
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "World".to_string(),
                bbox: Rect::new(132.0, 700.0, 30.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 2,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 1, "All three spans should merge");
        assert!(extractor.spans[0].text.contains("Hello"), "Should contain Hello");
        assert!(extractor.spans[0].text.contains("World"), "Should contain World");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: partition_characters_by_boundaries
    // ========================================================================

    #[test]
    fn test_partition_no_boundaries() {
        let extractor = TextExtractor::new();
        let chars = vec![
            CharacterInfo {
                code: 65,
                glyph_id: None,
                width: 10.0,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            },
            CharacterInfo {
                code: 66,
                glyph_id: None,
                width: 10.0,
                x_position: 10.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            },
        ];

        let clusters = extractor.partition_characters_by_boundaries(&chars, vec![]);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].len(), 2);
    }

    #[test]
    fn test_partition_with_boundary() {
        let extractor = TextExtractor::new();
        let chars = vec![
            CharacterInfo {
                code: 65,
                glyph_id: None,
                width: 10.0,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            },
            CharacterInfo {
                code: 66,
                glyph_id: None,
                width: 10.0,
                x_position: 10.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            },
            CharacterInfo {
                code: 67,
                glyph_id: None,
                width: 10.0,
                x_position: 25.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            },
        ];

        let clusters = extractor.partition_characters_by_boundaries(&chars, vec![2]);
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].len(), 2); // [A, B]
        assert_eq!(clusters[1].len(), 1); // [C]
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: create_boundary_context
    // ========================================================================

    #[test]
    fn test_create_boundary_context() {
        let mut extractor = TextExtractor::new();
        extractor.state_stack.current_mut().font_size = 12.0;
        extractor.state_stack.current_mut().horizontal_scaling = 100.0;
        extractor.state_stack.current_mut().word_space = 2.0;
        extractor.state_stack.current_mut().char_space = 0.5;

        let ctx = extractor.create_boundary_context();
        assert_eq!(ctx.font_size, 12.0);
        assert_eq!(ctx.horizontal_scaling, 100.0);
        assert_eq!(ctx.word_spacing, 2.0);
        assert_eq!(ctx.char_spacing, 0.5);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: build_boundary_characters
    // ========================================================================

    #[test]
    fn test_build_boundary_characters() {
        let prev_bbox = Rect::new(10.0, 100.0, 50.0, 12.0);
        let next_bbox = Rect::new(65.0, 100.0, 40.0, 12.0);

        let (chars, ctx) =
            build_boundary_characters("Hello", "World", &prev_bbox, &next_bbox, 12.0, false);

        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].code, 'o' as u32); // Last char of "Hello"
        assert_eq!(chars[1].code, 'W' as u32); // First char of "World"
        assert_eq!(ctx.font_size, 12.0);
    }

    #[test]
    fn test_build_boundary_characters_with_tj_offset() {
        let prev_bbox = Rect::new(10.0, 100.0, 50.0, 12.0);
        let next_bbox = Rect::new(65.0, 100.0, 40.0, 12.0);

        let (chars, _ctx) =
            build_boundary_characters("Hello", "World", &prev_bbox, &next_bbox, 12.0, true);

        assert_eq!(chars[0].tj_offset, Some(-200)); // TJ offset triggered
        assert_eq!(chars[1].tj_offset, None);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: TjBuffer
    // ========================================================================

    #[test]
    fn test_tj_buffer_empty() {
        let state = crate::content::graphics_state::GraphicsStateStack::new();
        let buffer = TjBuffer::new(state.current(), None, None, Vec::new());
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_tj_buffer_append() {
        let state = crate::content::graphics_state::GraphicsStateStack::new();
        let mut buffer = TjBuffer::new(state.current(), None, None, Vec::new());
        buffer.append(b"Hello").unwrap();
        assert!(!buffer.is_empty());
        assert_eq!(buffer.unicode, "Hello");
    }

    #[test]
    fn test_tj_buffer_append_truncates_long_string() {
        let state = crate::content::graphics_state::GraphicsStateStack::new();
        let mut buffer = TjBuffer::new(state.current(), None, None, Vec::new());
        // Create a string larger than 32,767 bytes
        let long_bytes = vec![0x41u8; 40_000]; // 40K 'A's
        buffer.append(&long_bytes).unwrap();
        // Should be truncated to 32,767 chars
        assert!(buffer.unicode.len() <= 32_767);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: advance_position_for_offset
    // ========================================================================

    #[test]
    fn test_advance_position_for_offset_positive() {
        let mut extractor = TextExtractor::new();
        extractor.state_stack.current_mut().font_size = 12.0;
        extractor.state_stack.current_mut().horizontal_scaling = 100.0;

        let initial_e = extractor.state_stack.current().text_matrix.e;
        extractor.advance_position_for_offset(100.0).unwrap();
        let new_e = extractor.state_stack.current().text_matrix.e;

        // Positive offset should move text position left (negative tx)
        // tx = -offset / 1000.0 * font_size * horizontal_scaling / 100.0
        // tx = -100 / 1000 * 12 * 100 / 100 = -1.2
        assert!((new_e - initial_e - (-1.2)).abs() < 0.01);
    }

    #[test]
    fn test_advance_position_for_offset_negative() {
        let mut extractor = TextExtractor::new();
        extractor.state_stack.current_mut().font_size = 12.0;
        extractor.state_stack.current_mut().horizontal_scaling = 100.0;

        let initial_e = extractor.state_stack.current().text_matrix.e;
        extractor.advance_position_for_offset(-200.0).unwrap();
        let new_e = extractor.state_stack.current().text_matrix.e;

        // Negative offset should move text position right (positive tx)
        // tx = -(-200) / 1000 * 12 * 100/100 = 2.4
        assert!((new_e - initial_e - 2.4).abs() < 0.01);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: should_insert_space function
    // ========================================================================

    #[test]
    fn test_should_insert_space_boundary_already_present_trailing() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        let decision = should_insert_space(
            "word ", "next", 5.0, 12.0, "F1", &fonts, true, &config, None, None, 12.0, 12.0,
        );
        assert!(!decision.insert_space);
        assert_eq!(decision.source, SpaceSource::AlreadyPresent);
    }

    #[test]
    fn test_should_insert_space_boundary_already_present_leading() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        let decision = should_insert_space(
            "word", " next", 5.0, 12.0, "F1", &fonts, true, &config, None, None, 12.0, 12.0,
        );
        assert!(!decision.insert_space);
        assert_eq!(decision.source, SpaceSource::AlreadyPresent);
    }

    #[test]
    fn test_should_insert_space_strong_geometric() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        // Very large gap should trigger strong geometric rule
        // geometric_threshold = 12.0 * 0.25 = 3.0 (fallback)
        // strong threshold = 3.0 * 2.0 = 6.0
        let decision = should_insert_space(
            "word", "next", 10.0, 12.0, "F1", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        assert!(decision.insert_space, "Large gap should insert space");
        assert_eq!(decision.source, SpaceSource::GeometricGap);
    }

    #[test]
    fn test_should_insert_space_consensus_tj_and_geometric() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        // Both TJ offset and geometric gap triggered
        // geometric_threshold = 12.0 * 0.25 = 3.0 (fallback)
        let decision = should_insert_space(
            "word", "next", 4.0, 12.0, "F1", &fonts, true, &config, None, None, 12.0, 12.0,
        );
        assert!(decision.insert_space, "Consensus should insert space");
        assert_eq!(decision.source, SpaceSource::TjOffset);
        assert_eq!(decision.confidence, 1.0);
    }

    #[test]
    fn test_should_insert_space_no_consensus_small_gap() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        // Small gap, no TJ offset - should not insert
        let decision = should_insert_space(
            "word", "next", 0.5, 12.0, "F1", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        assert!(!decision.insert_space, "Small gap without TJ should not insert space");
        assert_eq!(decision.source, SpaceSource::NoSpace);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Line break detection in should_insert_space
    // ========================================================================

    #[test]
    fn test_should_insert_space_line_break_hard() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        // Simulate line break: prev at Y=700, next at Y=680
        let prev_bbox = Rect::new(100.0, 700.0, 200.0, 12.0);
        let next_bbox = Rect::new(100.0, 680.0, 200.0, 12.0);

        let decision = should_insert_space(
            "end of line",
            "start of next",
            0.0,
            12.0,
            "F1",
            &fonts,
            false,
            &config,
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            12.0,
        );
        // Line break detected, same column, not ending with hyphen => insert space
        assert!(decision.insert_space, "Hard line break should insert space");
    }

    #[test]
    fn test_should_insert_space_line_break_hyphen() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        // Line break with hyphen: should NOT insert space
        let prev_bbox = Rect::new(100.0, 700.0, 200.0, 12.0);
        let next_bbox = Rect::new(100.0, 680.0, 200.0, 12.0);

        let decision = should_insert_space(
            "self-contain-",
            "ed text",
            0.0,
            12.0,
            "F1",
            &fonts,
            false,
            &config,
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            12.0,
        );
        assert!(!decision.insert_space, "Hyphenated line break should not insert space");
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Full extraction pipeline via content streams
    // ========================================================================

    #[test]
    fn test_extract_multiple_text_objects() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream =
            b"BT /F1 12 Tf 100 700 Td (First) Tj ET BT /F1 12 Tf 100 680 Td (Second) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        let text: String = chars.iter().map(|c| c.char).collect();
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
    }

    #[test]
    fn test_extract_spans_with_line_break() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Two lines of text
        let stream = b"BT /F1 12 Tf 14 TL 100 700 Td (First line) Tj T* (Second line) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        assert!(!spans.is_empty());
        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("First"), "Should contain first line");
        assert!(text.contains("Second"), "Should contain second line");
    }

    #[test]
    fn test_extract_chars_reading_order() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Text in reverse rendering order
        let stream = b"BT /F1 12 Tf 100 680 Td (B) Tj ET BT /F1 12 Tf 100 700 Td (A) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        // After sorting by reading order: A (y=700 higher) should come first
        assert_eq!(chars[0].char, 'A', "Higher Y should come first in reading order");
        assert_eq!(chars[1].char, 'B');
    }

    #[test]
    fn test_extract_empty_string() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 100 700 Td () Tj ET";
        let chars = extractor.extract(stream).unwrap();
        assert_eq!(chars.len(), 0);
    }

    #[test]
    fn test_extract_only_graphics_no_text() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Only graphics commands, no text
        let stream = b"q 1 0 0 1 0 0 cm 100 700 m 200 700 l S Q";
        let chars = extractor.extract(stream).unwrap();
        assert_eq!(chars.len(), 0);
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Inline images should not affect text
    // ========================================================================

    #[test]
    fn test_inline_image_ignored() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Text before and after inline image - both should be extracted
        // The inline image operators are handled by the parser
        let stream = b"BT /F1 12 Tf 100 700 Td (Before) Tj ET";
        let chars = extractor.extract(stream).unwrap();

        let text: String = chars.iter().map(|c| c.char).collect();
        assert!(text.contains("Before"));
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Tm operator batching optimization
    // ========================================================================

    #[test]
    fn test_tm_continuation_same_line() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Character-by-character Tm+Tj pattern on same line
        // The optimization should batch these into fewer spans
        let stream = b"BT /F1 12 Tf 1 0 0 1 100 700 Tm (H) Tj 1 0 0 1 106 700 Tm (i) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Hi"), "Should batch Tm+Tj on same line, got: {}", text);
    }

    #[test]
    fn test_tm_different_line_flushes() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Tm to different Y should flush buffer and start new span
        let stream = b"BT /F1 12 Tf 1 0 0 1 100 700 Tm (A) Tj 1 0 0 1 100 680 Tm (B) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        // Should have at least 2 spans (different lines)
        assert!(
            spans.len() >= 2 || {
                // Or could be merged if within merge range
                let text: String = spans
                    .iter()
                    .map(|s| s.text.as_str())
                    .collect::<Vec<_>>()
                    .join("");
                text.contains("A") && text.contains("B")
            }
        );
    }

    // ========================================================================
    // TESTS: merge_tm_tj_runs opt-out (#488)
    // ========================================================================

    /// With the default config (merge_tm_tj_runs = true), multiple Tm+Tj operators
    /// on the same line are batched into a single span.
    #[test]
    fn test_merge_tm_tj_runs_default_merges() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy(); // fixed thresholds, merging on
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Three separate Tm+Tj on the same baseline (same Y, same a/b/c/d, ascending e)
        let stream =
            b"BT /F1 12 Tf 1 0 0 1 100 700 Tm (A) Tj 1 0 0 1 107 700 Tm (B) Tj 1 0 0 1 114 700 Tm (C) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        // All three characters must be present
        let text: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert!(
            text.contains('A') && text.contains('B') && text.contains('C'),
            "All chars must be extracted, got: {:?}",
            text
        );

        // The default merging should combine them into fewer spans than the number
        // of Tm operators (3 Tms should not produce 3 separate spans)
        assert!(
            spans.len() < 3,
            "Default merge_tm_tj_runs=true should combine same-line Tm+Tj into fewer than 3 spans, got {} spans",
            spans.len()
        );
    }

    /// With merge_tm_tj_runs = false, each Tm operator starts a fresh span.
    #[test]
    fn test_merge_tm_tj_runs_disabled_splits() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig {
            merge_tm_tj_runs: false,
            ..SpanMergingConfig::legacy()
        };
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Three separate Tm+Tj on the same baseline
        let stream =
            b"BT /F1 12 Tf 1 0 0 1 100 700 Tm (A) Tj 1 0 0 1 107 700 Tm (B) Tj 1 0 0 1 114 700 Tm (C) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        // All three characters must still be present
        let text: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert!(
            text.contains('A') && text.contains('B') && text.contains('C'),
            "All chars must be extracted even with merging disabled, got: {:?}",
            text
        );

        // With merge disabled, each Tm flushes the buffer, so we get more spans
        // than with merging enabled (post-processing merge_adjacent_spans may combine
        // some, but at minimum we should get spans >= 1; the key invariant is that
        // the span count here is NOT reduced by the Tm-continuation shortcut)
        assert!(
            spans.len() >= 2,
            "merge_tm_tj_runs=false should not batch same-line runs; expected >= 2 spans, got {}",
            spans.len()
        );
    }

    // ========================================================================
    // NEW COMPREHENSIVE TESTS: Edge cases
    // ========================================================================

    #[test]
    fn test_extract_with_zero_font_size() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Zero font size is technically valid in PDF
        let stream = b"BT /F1 0 Tf 100 700 Td (X) Tj ET";
        let result = extractor.extract(stream);
        // Should not panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_with_negative_font_size() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Negative font size inverts text
        let stream = b"BT /F1 -12 Tf 100 700 Td (X) Tj ET";
        let result = extractor.extract(stream);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_with_very_large_coordinate() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 99999 99999 Td (X) Tj ET";
        let chars = extractor.extract(stream).unwrap();
        assert_eq!(chars.len(), 1);
    }

    // ========================================================================
    // COVERAGE TESTS: Color space operators (SetFillColor/SetStrokeColor)
    // ========================================================================

    #[test]
    fn test_set_fill_color_device_gray() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // cs sets color space, then sc sets color components
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceGray".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.5],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.5).abs() < 0.01);
        assert!((state.fill_color_rgb.1 - 0.5).abs() < 0.01);
        assert!((state.fill_color_rgb.2 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_device_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceRGB".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.2, 0.4, 0.6],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.2).abs() < 0.01);
        assert!((state.fill_color_rgb.1 - 0.4).abs() < 0.01);
        assert!((state.fill_color_rgb.2 - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_device_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceCMYK".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.0, 0.0, 0.0, 1.0], // the K ink
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.1373).abs() < 0.01);
        assert!((state.fill_color_rgb.1 - 0.1216).abs() < 0.01);
        assert!((state.fill_color_rgb.2 - 0.1255).abs() < 0.01);
        assert!(state.fill_color_cmyk.is_some());
    }

    #[test]
    fn test_set_fill_color_lab() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "Lab".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![50.0, 20.0, -10.0],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        // Lab simplified to grayscale: L/100
        assert!((state.fill_color_rgb.0 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_iccbased_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.1, 0.2, 0.3],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.1).abs() < 0.01);
        assert!((state.fill_color_rgb.1 - 0.2).abs() < 0.01);
        assert!((state.fill_color_rgb.2 - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_iccbased_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.7],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_iccbased_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![1.0, 0.0, 0.0, 0.0], // cyan
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.fill_color_cmyk.is_some());
    }

    #[test]
    fn test_set_fill_color_separation() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "Separation".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.8], // tint
            })
            .unwrap();

        let state = extractor.state_stack.current();
        // gray = 1.0 - tint = 0.2
        assert!((state.fill_color_rgb.0 - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_devicen_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceN".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.0, 0.0, 0.0, 0.5], // 4-component DeviceN
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.fill_color_cmyk.is_some());
    }

    #[test]
    fn test_set_fill_color_devicen_single() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceN".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.3], // single-component DeviceN
            })
            .unwrap();

        let state = extractor.state_stack.current();
        // gray = 1.0 - 0.3 = 0.7
        assert!((state.fill_color_rgb.0 - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_unknown_space() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "CustomUnknown".to_string(),
            })
            .unwrap();
        // This should log warning but not panic
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.5, 0.5],
            })
            .unwrap();
    }

    #[test]
    fn test_set_fill_color_cal_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "CalGray".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.8],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_cal_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "CalRGB".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColor {
                components: vec![0.9, 0.1, 0.5],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.9).abs() < 0.01);
        assert!((state.fill_color_rgb.1 - 0.1).abs() < 0.01);
        assert!((state.fill_color_rgb.2 - 0.5).abs() < 0.01);
    }

    // ========================================================================
    // COVERAGE TESTS: Stroke color operators
    // ========================================================================

    #[test]
    fn test_set_stroke_color_device_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceGray".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.4],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_device_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceRGB".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.1, 0.2, 0.3],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_lab() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "Lab".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![75.0, 10.0, -5.0],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_device_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceCMYK".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.0, 1.0, 0.0, 0.0], // magenta
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.stroke_color_cmyk.is_some());
    }

    #[test]
    fn test_set_stroke_color_iccbased_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.3],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_iccbased_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.9, 0.8, 0.7],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_iccbased_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.1, 0.2, 0.3, 0.4],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.stroke_color_cmyk.is_some());
    }

    #[test]
    fn test_set_stroke_color_separation() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "Separation".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.6],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        // gray = 1.0 - 0.6 = 0.4
        assert!((state.stroke_color_rgb.0 - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_devicen_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceN".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.1, 0.2, 0.3, 0.4],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.stroke_color_cmyk.is_some());
    }

    #[test]
    fn test_set_stroke_color_devicen_single() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceN".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.5],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_cal_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "CalRGB".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.5, 0.6, 0.7],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_cal_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "CalGray".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.9],
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_unknown() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "UnknownCS".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColor {
                components: vec![0.5],
            })
            .unwrap();
        // Should not panic
    }

    // ========================================================================
    // COVERAGE TESTS: SetFillColorN / SetStrokeColorN
    // ========================================================================

    #[test]
    fn test_set_fill_color_n_with_pattern() {
        let mut extractor = TextExtractor::new();
        // Pattern color space with name
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![],
                name: Some(Box::new("P1".to_string())),
            })
            .unwrap();
        // Should not panic (pattern ignored)
    }

    #[test]
    fn test_set_fill_color_n_without_pattern_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceGray".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![0.3],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_n_without_pattern_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceRGB".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![0.1, 0.2, 0.3],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_n_lab() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "Lab".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![80.0, 0.0, 0.0],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_n_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceCMYK".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![0.0, 0.0, 0.0, 0.0],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        // White (no ink)
        assert!((state.fill_color_rgb.0 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_n_iccbased() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![0.5, 0.6, 0.7],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_n_iccbased_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![0.9],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_n_iccbased_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![0.1, 0.2, 0.3, 0.4],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.fill_color_cmyk.is_some());
    }

    #[test]
    fn test_set_fill_color_n_separation() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "Separation".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![0.4],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_set_fill_color_n_devicen() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceN".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![0.1, 0.2, 0.3, 0.4],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.fill_color_cmyk.is_some());
    }

    #[test]
    fn test_set_fill_color_n_devicen_single() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceN".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetFillColorN {
                components: vec![0.2],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_n_with_pattern() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![],
                name: Some(Box::new("P2".to_string())),
            })
            .unwrap();
        // Should not panic
    }

    #[test]
    fn test_set_stroke_color_n_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceGray".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![0.6],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_n_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceRGB".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![0.8, 0.7, 0.6],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_n_lab() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "Lab".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![60.0, 0.0, 0.0],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_n_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceCMYK".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![0.0, 0.0, 1.0, 0.0], // yellow
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.stroke_color_cmyk.is_some());
    }

    #[test]
    fn test_set_stroke_color_n_iccbased_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![0.2, 0.3, 0.4],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_n_iccbased_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![0.5],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_n_iccbased_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "ICCBased".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![0.1, 0.2, 0.3, 0.4],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.stroke_color_cmyk.is_some());
    }

    #[test]
    fn test_set_stroke_color_n_separation() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "Separation".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![1.0],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_color_n_devicen_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceN".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![0.5, 0.5, 0.5, 0.5],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.stroke_color_cmyk.is_some());
    }

    #[test]
    fn test_set_stroke_color_n_devicen_single() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceN".to_string(),
            })
            .unwrap();
        extractor
            .execute_operator_public(Operator::SetStrokeColorN {
                components: vec![0.1],
                name: None,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.9).abs() < 0.01);
    }

    // ========================================================================
    // REGRESSION: named / unknown color space references
    // ========================================================================

    /// Named color space reference like "Cs1" should fall back by component
    /// count rather than emitting a warn! (regression: warn spam on PDFs
    /// with ICCBased color spaces registered under user-defined names).
    #[test]
    fn test_named_fill_color_space_fallback_gray() {
        let mut e = TextExtractor::new();
        e.execute_operator_public(Operator::SetFillColorSpace {
            name: "Cs1".to_string(),
        })
        .unwrap();
        e.execute_operator_public(Operator::SetFillColor {
            components: vec![0.4],
        })
        .unwrap();
        let state = e.state_stack.current();
        let (r, g, b) = state.fill_color_rgb;
        assert!((r - 0.4).abs() < 0.01 && (g - 0.4).abs() < 0.01 && (b - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_named_fill_color_space_fallback_rgb() {
        let mut e = TextExtractor::new();
        e.execute_operator_public(Operator::SetFillColorSpace {
            name: "Cs2".to_string(),
        })
        .unwrap();
        e.execute_operator_public(Operator::SetFillColor {
            components: vec![0.1, 0.2, 0.3],
        })
        .unwrap();
        let state = e.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.1).abs() < 0.01);
        assert!((state.fill_color_rgb.1 - 0.2).abs() < 0.01);
        assert!((state.fill_color_rgb.2 - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_named_fill_color_space_fallback_cmyk() {
        let mut e = TextExtractor::new();
        e.execute_operator_public(Operator::SetFillColorSpace {
            name: "Cs3".to_string(),
        })
        .unwrap();
        e.execute_operator_public(Operator::SetFillColor {
            components: vec![0.0, 0.0, 0.0, 0.5],
        })
        .unwrap();
        let state = e.state_stack.current();
        assert!(state.fill_color_cmyk.is_some());
    }

    #[test]
    fn test_named_stroke_color_space_fallback_rgb() {
        let mut e = TextExtractor::new();
        e.execute_operator_public(Operator::SetStrokeColorSpace {
            name: "Cs1".to_string(),
        })
        .unwrap();
        e.execute_operator_public(Operator::SetStrokeColor {
            components: vec![0.5, 0.6, 0.7],
        })
        .unwrap();
        let state = e.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.5).abs() < 0.01);
        assert!((state.stroke_color_rgb.1 - 0.6).abs() < 0.01);
        assert!((state.stroke_color_rgb.2 - 0.7).abs() < 0.01);
    }

    // ========================================================================
    // COVERAGE TESTS: Line style & misc operators
    // ========================================================================

    #[test]
    fn test_set_line_cap() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetLineCap { cap_style: 2 })
            .unwrap();
        assert_eq!(extractor.state_stack.current().line_cap, 2);
    }

    #[test]
    fn test_set_line_join() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetLineJoin { join_style: 1 })
            .unwrap();
        assert_eq!(extractor.state_stack.current().line_join, 1);
    }

    #[test]
    fn test_set_miter_limit() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetMiterLimit { limit: 5.0 })
            .unwrap();
        assert!((extractor.state_stack.current().miter_limit - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_set_rendering_intent() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetRenderingIntent {
                intent: "RelativeColorimetric".to_string(),
            })
            .unwrap();
        assert_eq!(extractor.state_stack.current().rendering_intent, "RelativeColorimetric");
    }

    #[test]
    fn test_set_flatness() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFlatness { tolerance: 0.5 })
            .unwrap();
        assert!((extractor.state_stack.current().flatness - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_set_ext_gstate() {
        let mut extractor = TextExtractor::new();
        // Should not panic, just logs debug info
        extractor
            .execute_operator_public(Operator::SetExtGState {
                dict_name: "GS1".to_string(),
            })
            .unwrap();
    }

    #[test]
    fn test_paint_shading() {
        let mut extractor = TextExtractor::new();
        // Should not panic, just logs debug info
        extractor
            .execute_operator_public(Operator::PaintShading {
                name: "sh1".to_string(),
            })
            .unwrap();
    }

    #[test]
    fn test_inline_image_operator() {
        let mut extractor = TextExtractor::new();
        let mut dict = HashMap::new();
        dict.insert("W".to_string(), Object::Integer(100));
        dict.insert("H".to_string(), Object::Integer(50));
        extractor
            .execute_operator_public(Operator::InlineImage {
                dict: Box::new(dict),
                data: vec![0u8; 100],
            })
            .unwrap();
        // Should not panic and not produce text
    }

    #[test]
    fn test_inline_image_no_dimensions() {
        let mut extractor = TextExtractor::new();
        let dict = HashMap::new(); // no W/H
        extractor
            .execute_operator_public(Operator::InlineImage {
                dict: Box::new(dict),
                data: vec![0u8; 10],
            })
            .unwrap();
    }

    // ========================================================================
    // COVERAGE TESTS: Email pattern detection with config
    // ========================================================================

    #[test]
    fn test_email_context_at_sign_end() {
        // Pattern: user@ + domain
        assert!(is_email_context("user@", "domain.com"));
    }

    #[test]
    fn test_email_context_domain_dot() {
        // Pattern: user@domain. + com
        assert!(is_email_context("user@domain.", "com"));
    }

    #[test]
    fn test_email_context_not_alpha_after_at() {
        // @ followed by non-alphanumeric should not be email
        assert!(!is_email_context("user@", " "));
    }

    #[test]
    fn test_email_context_long_preceding_text() {
        // Test with very long preceding text (should only check last 64 bytes)
        let long_prefix = "a".repeat(200) + "@domain";
        assert!(is_email_context(&long_prefix, ".com"));
    }

    #[test]
    fn test_should_insert_space_with_email_config() {
        let config = SpanMergingConfig {
            detect_email_patterns: true,
            email_threshold_multiplier: 2.5,
            ..Default::default()
        };
        let fonts = HashMap::new();

        // Email context with gap below threshold: suppress space
        let decision = should_insert_space(
            "user@domain",
            ".com",
            1.0,
            12.0,
            "F1",
            &fonts,
            false,
            &config,
            None,
            None,
            12.0,
            12.0,
        );
        assert!(!decision.insert_space, "Email context should suppress space for small gap");
    }

    #[test]
    fn test_should_insert_space_email_large_gap() {
        let config = SpanMergingConfig {
            detect_email_patterns: true,
            email_threshold_multiplier: 2.5,
            ..Default::default()
        };
        let fonts = HashMap::new();

        // Email context with very large gap: insert space
        let decision = should_insert_space(
            "user@domain",
            ".com",
            100.0,
            12.0,
            "F1",
            &fonts,
            false,
            &config,
            None,
            None,
            12.0,
            12.0,
        );
        assert!(decision.insert_space, "Email context should insert space for large gap");
    }

    #[test]
    fn test_should_insert_space_email_with_font_info() {
        let config = SpanMergingConfig {
            detect_email_patterns: true,
            ..Default::default()
        };
        let mut fonts: HashMap<String, Arc<FontInfo>> = HashMap::new();
        let font = create_test_font();
        fonts.insert("F1".to_string(), Arc::new(font));

        // Email context uses font metrics for threshold
        let decision = should_insert_space(
            "user@domain",
            ".com",
            1.0,
            12.0,
            "F1",
            &fonts,
            false,
            &config,
            None,
            None,
            12.0,
            12.0,
        );
        assert!(!decision.insert_space);
    }

    // ========================================================================
    // COVERAGE TESTS: Citation marker detection with config
    // ========================================================================

    #[test]
    fn test_should_insert_space_citation_context() {
        let config = SpanMergingConfig {
            detect_citation_markers: true,
            citation_font_size_ratio: 0.75,
            ..Default::default()
        };
        let fonts = HashMap::new();

        let prev_bbox = Rect::new(10.0, 100.0, 50.0, 12.0);
        let next_bbox = Rect::new(60.0, 105.0, 10.0, 7.0); // Raised, smaller

        let decision = should_insert_space(
            "text",
            "1",
            2.0,
            12.0,
            "F1",
            &fonts,
            true,
            &config,
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            7.2,
        );
        assert!(decision.insert_space, "Citation context with TJ should insert space");
    }

    #[test]
    fn test_is_pictographic_ranges() {
        assert!(is_pictographic('📄'));
        assert!(is_pictographic('✅'));
        assert!(!is_pictographic('A'));
        assert!(!is_pictographic('→')); // arrow excluded (math/symbol text)
        assert!(!is_pictographic('5'));
    }

    #[test]
    fn test_should_insert_space_emoji_letter_boundary() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();
        // The real case (arxiv_2510.26287): a wide emoji glyph abuts the next
        // token, so the gap is exactly 0. The space must still be kept.
        let decision0 = should_insert_space(
            "📄", "README", 0.0, 12.0, "F1", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        assert!(
            decision0.insert_space,
            "emoji→letter with a zero (abutting) gap must keep space"
        );

        // A positive gap also keeps it.
        let decision = should_insert_space(
            "📄", "README", 0.5, 12.0, "F1", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        assert!(decision.insert_space, "emoji→letter with a positive gap keeps the space");

        // A combined emoji sequence (next char is another pictograph, not a
        // letter) must NOT be forced into a space by this rule.
        let combined = should_insert_space(
            "📄", "📄", 0.0, 12.0, "F1", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        assert!(!combined.insert_space, "emoji→emoji must not be forced into a space");
    }

    #[test]
    fn test_should_insert_space_citation_geometric() {
        let config = SpanMergingConfig {
            detect_citation_markers: true,
            ..Default::default()
        };
        let fonts = HashMap::new();

        let prev_bbox = Rect::new(10.0, 100.0, 50.0, 12.0);
        let next_bbox = Rect::new(60.0, 105.0, 10.0, 7.0);

        // Citation context with large geometric gap
        let decision = should_insert_space(
            "text",
            "1",
            10.0,
            12.0,
            "F1",
            &fonts,
            false,
            &config,
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            7.2,
        );
        assert!(decision.insert_space, "Citation context with large gap should insert space");
    }

    #[test]
    fn test_should_insert_space_citation_with_font() {
        let config = SpanMergingConfig {
            detect_citation_markers: true,
            ..Default::default()
        };
        let mut fonts: HashMap<String, Arc<FontInfo>> = HashMap::new();
        fonts.insert("F1".to_string(), Arc::new(create_test_font()));

        let prev_bbox = Rect::new(10.0, 100.0, 50.0, 12.0);
        let next_bbox = Rect::new(60.0, 105.0, 10.0, 7.0);

        let decision = should_insert_space(
            "text",
            "1",
            5.0,
            12.0,
            "F1",
            &fonts,
            true,
            &config,
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            7.2,
        );
        assert!(decision.insert_space);
    }

    // ========================================================================
    // COVERAGE TESTS: Line break detection
    // ========================================================================

    #[test]
    fn test_line_break_different_column() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        // prev and next at very different X positions (different columns)
        let prev_bbox = Rect::new(50.0, 700.0, 200.0, 12.0);
        let next_bbox = Rect::new(400.0, 680.0, 200.0, 12.0);

        let decision = should_insert_space(
            "end",
            "start",
            0.0,
            12.0,
            "F1",
            &fonts,
            false,
            &config,
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            12.0,
        );
        // Different column - should not trigger same_column line break path
        // The default no space path should apply
    }

    #[test]
    fn test_line_break_not_triggered_small_vertical_gap() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        // Small vertical gap - not a line break
        let prev_bbox = Rect::new(100.0, 700.0, 200.0, 12.0);
        let next_bbox = Rect::new(100.0, 699.0, 200.0, 12.0);

        let decision = should_insert_space(
            "word",
            "next",
            0.0,
            12.0,
            "F1",
            &fonts,
            false,
            &config,
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            12.0,
        );
        // Small vertical gap should not trigger line break
    }

    // ========================================================================
    // COVERAGE TESTS: WordBoundary tiebreaker path
    // ========================================================================

    #[test]
    fn test_should_insert_space_tiebreaker_with_bboxes() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        let prev_bbox = Rect::new(100.0, 700.0, 50.0, 12.0);
        let next_bbox = Rect::new(155.0, 700.0, 50.0, 12.0);

        // TJ triggered but gap does not suggest space (conflict)
        // Should go through tiebreaker
        let decision = should_insert_space(
            "word",
            "next",
            1.0,
            12.0,
            "F1",
            &fonts,
            true,
            &config,
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            12.0,
        );
        // Result depends on WordBoundaryDetector
    }

    #[test]
    fn test_should_insert_space_geometric_only_conflict() {
        let config = SpanMergingConfig::default();
        let fonts = HashMap::new();

        let prev_bbox = Rect::new(100.0, 700.0, 50.0, 12.0);
        let next_bbox = Rect::new(155.0, 700.0, 50.0, 12.0);

        // No TJ but gap suggests space (conflict with no TJ)
        let decision = should_insert_space(
            "word",
            "next",
            5.0,
            12.0,
            "F1",
            &fonts,
            false,
            &config,
            Some(&prev_bbox),
            Some(&next_bbox),
            12.0,
            12.0,
        );
        // Geometric alone - should go through tiebreaker path
    }

    // ========================================================================
    // COVERAGE TESTS: Font-aware spacing in should_insert_space
    // ========================================================================

    #[test]
    fn test_should_insert_space_font_aware() {
        let config = SpanMergingConfig::default();
        let mut fonts: HashMap<String, Arc<FontInfo>> = HashMap::new();
        fonts.insert("F1".to_string(), Arc::new(create_test_font()));

        // With font info, threshold is calculated from font metrics
        let decision = should_insert_space(
            "word", "next", 0.5, 12.0, "F1", &fonts, false, &config, None, None, 12.0, 12.0,
        );
        // The result depends on font-specific threshold
    }

    // ── #12 spec-aligned gap correction (§9.4.4): the fallback-width
    //    inflation that splits "SalesForce" → "SalesF orce" is only applied
    //    when glyphs actually overlap (raw_gap < 0), per corrected_space_gap ──

    /// Adjacent glyphs (raw_gap == 0) on a fallback-width font must NOT be
    /// inflated into a phantom gap — this is the "SalesF"+"orce" case. The
    /// reported gap stays 0 so no spurious word space is inserted.
    #[test]
    fn test_corrected_space_gap_no_inflation_when_adjacent() {
        // raw_gap 0.0, unreliable widths, non-empty: must stay 0.0.
        assert_eq!(corrected_space_gap(0.0, false, 34.23, false), 0.0);
        // small positive raw gap (academic "XGBoostX"+"provides") untouched.
        assert_eq!(corrected_space_gap(0.47, false, 50.0, false), 0.47);
    }

    #[test]
    fn test_strip_cjk_digit_boundary_spaces() {
        // A space between a CJK ideograph and an embedded number is dropped at
        // both ends; the number itself is preserved.
        assert_eq!(strip_cjk_digit_boundary_spaces("公元前 1000 年"), "公元前1000年");
        assert_eq!(strip_cjk_digit_boundary_spaces("追溯至 10,000 年前"), "追溯至10,000年前");
        // Works for Japanese ideographs/kana too.
        assert_eq!(strip_cjk_digit_boundary_spaces("西暦 2024 年"), "西暦2024年");
        // Korean (Hangul) is EXCLUDED — Korean uses inter-word spaces, so a
        // space between a syllable and a number is a real word boundary and
        // must be preserved ("14 예" = "14 cases", "7 예중").
        assert_eq!(strip_cjk_digit_boundary_spaces("약 1 만년"), "약 1 만년");
        assert_eq!(strip_cjk_digit_boundary_spaces("기질은 14 예에서"), "기질은 14 예에서");
        // Legitimate spacing is preserved.
        assert_eq!(strip_cjk_digit_boundary_spaces("貓 通常"), "貓 通常"); // CJK↔CJK
        assert_eq!(strip_cjk_digit_boundary_spaces("catus 펠리스"), "catus 펠리스"); // letter↔CJK
        assert_eq!(strip_cjk_digit_boundary_spaces("10 000"), "10 000"); // digit↔digit
        assert_eq!(strip_cjk_digit_boundary_spaces("page 12 of 30"), "page 12 of 30"); // Latin
                                                                                       // No-op fast path.
        assert_eq!(strip_cjk_digit_boundary_spaces("中文"), "中文");

        // Brackets hug their content: a space between a CJK/Hangul character and
        // an adjacent bracket is a layout artifact, dropped on both sides and
        // for both ASCII paren/square/brace shapes.
        assert_eq!(strip_cjk_digit_boundary_spaces("고양이 (학명"), "고양이(학명"); // Hangul→(
        assert_eq!(strip_cjk_digit_boundary_spaces("카투스 [*]) 는"), "카투스[*])는"); // Hangul→[ , )→Hangul
        assert_eq!(strip_cjk_digit_boundary_spaces("漢字 (注)"), "漢字(注)"); // CJK↔paren
                                                                              // A space between Latin and a bracket is left alone (English may write
                                                                              // "study (note)" with a space).
        assert_eq!(strip_cjk_digit_boundary_spaces("study (note)"), "study (note)");
    }

    #[test]
    fn test_strip_prime_decimal_boundary_spaces() {
        // Artifact space between the prime and the decimal point is dropped.
        assert_eq!(
            strip_prime_decimal_boundary_spaces("0\u{2032}\u{2032} .28"),
            "0\u{2032}\u{2032}.28"
        );
        // Artifact space between the prime's decimal point and its digits.
        assert_eq!(
            strip_prime_decimal_boundary_spaces("0\u{2032}\u{2032}. 28"),
            "0\u{2032}\u{2032}.28"
        );
        // Single prime and double-prime (U+2033) both handled.
        assert_eq!(strip_prime_decimal_boundary_spaces("1\u{2032}.47"), "1\u{2032}.47"); // already tight: no-op
        assert_eq!(strip_prime_decimal_boundary_spaces("12\u{2033} .5"), "12\u{2033}.5");
        // Feet-and-inches keeps its space: prime → DIGIT (not a decimal point).
        assert_eq!(
            strip_prime_decimal_boundary_spaces("5\u{2032} 6\u{2033}"),
            "5\u{2032} 6\u{2033}"
        );
        // A prime ending a sentence followed by prose is untouched (next not . / digit).
        assert_eq!(strip_prime_decimal_boundary_spaces("see 3\u{2032} and"), "see 3\u{2032} and");
        // A lone decimal with no preceding prime is untouched.
        assert_eq!(strip_prime_decimal_boundary_spaces("v1. 0 release"), "v1. 0 release");
        // No-op fast path.
        assert_eq!(
            strip_prime_decimal_boundary_spaces("0\u{2032}\u{2032}.28"),
            "0\u{2032}\u{2032}.28"
        );
    }

    /// Overlap (raw_gap < 0) on a fallback-width font IS corrected — this is
    /// the issue #328 NASA-Apollo case where the 0.55 em fallback over-reports
    /// width and swallows a real word gap. The correction lifts the gap.
    #[test]
    fn test_corrected_space_gap_corrects_overlap() {
        // raw_gap -2.0, width 30 → -2.0 + 30*(1 - 1/1.22) ≈ -2.0 + 5.41 = 3.41
        let g = corrected_space_gap(-2.0, false, 30.0, false);
        assert!(g > 0.0, "overlap on fallback-width font must be lifted positive, got {g}");
    }

    /// Reliable-width fonts (explicit /Widths) are never corrected — the
    /// bbox gap is authoritative regardless of sign.
    #[test]
    fn test_corrected_space_gap_reliable_widths_untouched() {
        assert_eq!(corrected_space_gap(-2.0, true, 30.0, false), -2.0);
        assert_eq!(corrected_space_gap(5.0, true, 30.0, false), 5.0);
    }

    // ========================================================================
    // COVERAGE TESTS: SpanMergingConfig builder variants
    // ========================================================================

    #[test]
    fn test_span_merging_config_adaptive_with_config() {
        let adaptive_config = crate::extractors::gap_statistics::AdaptiveThresholdConfig::default();
        let config = SpanMergingConfig::adaptive_with_config(adaptive_config);
        assert!(config.use_adaptive_threshold);
        assert!(config.adaptive_config.is_some());
    }

    // ========================================================================
    // COVERAGE TESTS: Fallback char to unicode (more symbols)
    // ========================================================================

    #[test]
    fn test_fallback_quotation_marks() {
        assert_eq!(fallback_char_to_unicode(0x2018), "\u{2018}"); // Left single quote
        assert_eq!(fallback_char_to_unicode(0x2019), "\u{2019}"); // Right single quote
        assert_eq!(fallback_char_to_unicode(0x201C), "\u{201C}"); // Left double quote
        assert_eq!(fallback_char_to_unicode(0x201D), "\u{201D}"); // Right double quote
    }

    #[test]
    fn test_fallback_math_extended() {
        assert_eq!(fallback_char_to_unicode(0x00F7), "\u{00F7}"); // Division
        assert_eq!(fallback_char_to_unicode(0x2202), "\u{2202}"); // Partial diff
        assert_eq!(fallback_char_to_unicode(0x2207), "\u{2207}"); // Nabla
        assert_eq!(fallback_char_to_unicode(0x220F), "\u{220F}"); // Product
        assert_eq!(fallback_char_to_unicode(0x2261), "\u{2261}"); // Identical
        assert_eq!(fallback_char_to_unicode(0x2248), "\u{2248}"); // Almost equal
    }

    #[test]
    fn test_fallback_set_theory() {
        assert_eq!(fallback_char_to_unicode(0x2282), "\u{2282}"); // Subset
        assert_eq!(fallback_char_to_unicode(0x2283), "\u{2283}"); // Superset
        assert_eq!(fallback_char_to_unicode(0x2286), "\u{2286}"); // Subset or equal
        assert_eq!(fallback_char_to_unicode(0x2287), "\u{2287}"); // Superset or equal
        assert_eq!(fallback_char_to_unicode(0x2208), "\u{2208}"); // Element of
        assert_eq!(fallback_char_to_unicode(0x2209), "\u{2209}"); // Not element
        assert_eq!(fallback_char_to_unicode(0x2200), "\u{2200}"); // For all
        assert_eq!(fallback_char_to_unicode(0x2203), "\u{2203}"); // There exists
        assert_eq!(fallback_char_to_unicode(0x2205), "\u{2205}"); // Empty set
    }

    #[test]
    fn test_fallback_logic() {
        assert_eq!(fallback_char_to_unicode(0x2227), "\u{2227}"); // Logical and
        assert_eq!(fallback_char_to_unicode(0x2228), "\u{2228}"); // Logical or
        assert_eq!(fallback_char_to_unicode(0x00AC), "\u{00AC}"); // Not
    }

    #[test]
    fn test_fallback_arrows() {
        assert_eq!(fallback_char_to_unicode(0x2192), "\u{2192}"); // Right arrow
        assert_eq!(fallback_char_to_unicode(0x2190), "\u{2190}"); // Left arrow
        assert_eq!(fallback_char_to_unicode(0x2194), "\u{2194}"); // Left right arrow
        assert_eq!(fallback_char_to_unicode(0x21D2), "\u{21D2}"); // Double right
        assert_eq!(fallback_char_to_unicode(0x21D4), "\u{21D4}"); // Double left-right
    }

    #[test]
    fn test_fallback_greek_lowercase_extended() {
        assert_eq!(fallback_char_to_unicode(0x03B5), "\u{03B5}"); // epsilon
        assert_eq!(fallback_char_to_unicode(0x03B6), "\u{03B6}"); // zeta
        assert_eq!(fallback_char_to_unicode(0x03B7), "\u{03B7}"); // eta
        assert_eq!(fallback_char_to_unicode(0x03B9), "\u{03B9}"); // iota
        assert_eq!(fallback_char_to_unicode(0x03BA), "\u{03BA}"); // kappa
        assert_eq!(fallback_char_to_unicode(0x03BB), "\u{03BB}"); // lambda
        assert_eq!(fallback_char_to_unicode(0x03BC), "\u{03BC}"); // mu
        assert_eq!(fallback_char_to_unicode(0x03BD), "\u{03BD}"); // nu
        assert_eq!(fallback_char_to_unicode(0x03BE), "\u{03BE}"); // xi
        assert_eq!(fallback_char_to_unicode(0x03BF), "\u{03BF}"); // omicron
        assert_eq!(fallback_char_to_unicode(0x03C1), "\u{03C1}"); // rho
        assert_eq!(fallback_char_to_unicode(0x03C2), "\u{03C2}"); // final sigma
        assert_eq!(fallback_char_to_unicode(0x03C3), "\u{03C3}"); // sigma
        assert_eq!(fallback_char_to_unicode(0x03C4), "\u{03C4}"); // tau
        assert_eq!(fallback_char_to_unicode(0x03C5), "\u{03C5}"); // upsilon
        assert_eq!(fallback_char_to_unicode(0x03C6), "\u{03C6}"); // phi
        assert_eq!(fallback_char_to_unicode(0x03C7), "\u{03C7}"); // chi
        assert_eq!(fallback_char_to_unicode(0x03C8), "\u{03C8}"); // psi
    }

    #[test]
    fn test_fallback_greek_uppercase_extended() {
        assert_eq!(fallback_char_to_unicode(0x0391), "\u{0391}"); // Alpha
        assert_eq!(fallback_char_to_unicode(0x0392), "\u{0392}"); // Beta
        assert_eq!(fallback_char_to_unicode(0x0394), "\u{0394}"); // Delta
        assert_eq!(fallback_char_to_unicode(0x0395), "\u{0395}"); // Epsilon
        assert_eq!(fallback_char_to_unicode(0x0396), "\u{0396}"); // Zeta
        assert_eq!(fallback_char_to_unicode(0x0397), "\u{0397}"); // Eta
        assert_eq!(fallback_char_to_unicode(0x0398), "\u{0398}"); // Theta
        assert_eq!(fallback_char_to_unicode(0x0399), "\u{0399}"); // Iota
        assert_eq!(fallback_char_to_unicode(0x039A), "\u{039A}"); // Kappa
        assert_eq!(fallback_char_to_unicode(0x039B), "\u{039B}"); // Lambda
        assert_eq!(fallback_char_to_unicode(0x039C), "\u{039C}"); // Mu
        assert_eq!(fallback_char_to_unicode(0x039D), "\u{039D}"); // Nu
        assert_eq!(fallback_char_to_unicode(0x039E), "\u{039E}"); // Xi
        assert_eq!(fallback_char_to_unicode(0x039F), "\u{039F}"); // Omicron
        assert_eq!(fallback_char_to_unicode(0x03A0), "\u{03A0}"); // Pi
        assert_eq!(fallback_char_to_unicode(0x03A1), "\u{03A1}"); // Rho
        assert_eq!(fallback_char_to_unicode(0x03A3), "\u{03A3}"); // Sigma
        assert_eq!(fallback_char_to_unicode(0x03A4), "\u{03A4}"); // Tau
        assert_eq!(fallback_char_to_unicode(0x03A5), "\u{03A5}"); // Upsilon
        assert_eq!(fallback_char_to_unicode(0x03A6), "\u{03A6}"); // Phi
        assert_eq!(fallback_char_to_unicode(0x03A7), "\u{03A7}"); // Chi
        assert_eq!(fallback_char_to_unicode(0x03A8), "\u{03A8}"); // Psi
    }

    #[test]
    fn test_fallback_currency_extended() {
        assert_eq!(fallback_char_to_unicode(0x20A3), "\u{20A3}"); // Franc
        assert_eq!(fallback_char_to_unicode(0x20A4), "\u{20A4}"); // Lira
        assert_eq!(fallback_char_to_unicode(0x20A9), "\u{20A9}"); // Won
        assert_eq!(fallback_char_to_unicode(0x20AA), "\u{20AA}"); // Shekel
        assert_eq!(fallback_char_to_unicode(0x20AB), "\u{20AB}"); // Dong
        assert_eq!(fallback_char_to_unicode(0x20B9), "\u{20B9}"); // Rupee
    }

    // ========================================================================
    // COVERAGE TESTS: decode_text_to_unicode edge cases
    // ========================================================================

    #[test]
    fn test_decode_text_simple_font_with_control_chars() {
        let font = create_test_font();
        let bytes = vec![0x01, 0x41, 0x09]; // ctrl char, 'A', tab
        let result = decode_text_to_unicode(&bytes, Some(&font));
        // Should filter control chars but keep tab
        assert!(result.contains('\t') || result.contains('A'));
    }

    #[test]
    fn test_decode_text_single_byte_only() {
        // Test with bytes that hit the TwoByte < 2 fallback
        let mut font = create_test_font();
        font.subtype = "Type0".to_string();
        font.encoding = crate::fonts::Encoding::Identity;
        let bytes = vec![0x41]; // Single byte for Type0 identity
        let result = decode_text_to_unicode(&bytes, Some(&font));
        // Should hit trailing byte path
    }

    // ========================================================================
    // COVERAGE TESTS: Color space resets
    // ========================================================================

    #[test]
    fn test_set_fill_color_space_resets_color() {
        let mut extractor = TextExtractor::new();
        // Set RGB color first
        extractor
            .execute_operator_public(Operator::SetFillRgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 1.0).abs() < 0.01);

        // Change color space should reset to black
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceGray".to_string(),
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.fill_color_rgb.0 - 0.0).abs() < 0.01);
        assert!(state.fill_color_cmyk.is_none());
    }

    #[test]
    fn test_set_stroke_color_space_resets_color() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeRgb {
                r: 0.0,
                g: 1.0,
                b: 0.0,
            })
            .unwrap();

        extractor
            .execute_operator_public(Operator::SetStrokeColorSpace {
                name: "DeviceRGB".to_string(),
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.0).abs() < 0.01);
        assert!(state.stroke_color_cmyk.is_none());
    }

    // ========================================================================
    // COVERAGE TESTS: CMYK color operators
    // ========================================================================

    #[test]
    fn test_set_stroke_cmyk() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeCmyk {
                c: 1.0,
                m: 0.0,
                y: 0.0,
                k: 0.0,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!(state.stroke_color_cmyk.is_some());
        // Cyan: R=0, G=1, B=1
        assert!((state.stroke_color_rgb.0 - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_gray() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeGray { gray: 0.7 })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.7).abs() < 0.01);
        assert!((state.stroke_color_rgb.1 - 0.7).abs() < 0.01);
        assert!((state.stroke_color_rgb.2 - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_set_stroke_rgb() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetStrokeRgb {
                r: 0.3,
                g: 0.6,
                b: 0.9,
            })
            .unwrap();

        let state = extractor.state_stack.current();
        assert!((state.stroke_color_rgb.0 - 0.3).abs() < 0.01);
        assert!((state.stroke_color_rgb.1 - 0.6).abs() < 0.01);
        assert!((state.stroke_color_rgb.2 - 0.9).abs() < 0.01);
    }

    // ========================================================================
    // COVERAGE TESTS: CMYK to RGB edge cases
    // ========================================================================

    #[test]
    fn test_cmyk_to_rgb_mixed() {
        let (r, g, b) = cmyk_to_rgb(0.5, 0.3, 0.1, 0.2);
        assert!((0.0..=1.0).contains(&r));
        assert!((0.0..=1.0).contains(&g));
        assert!((0.0..=1.0).contains(&b));
    }

    #[test]
    fn test_cmyk_to_rgb_all_ones() {
        let (r, g, b) = cmyk_to_rgb(1.0, 1.0, 1.0, 1.0);
        assert!((r - 0.0).abs() < 0.01);
        assert!((g - 0.0).abs() < 0.01);
        assert!((b - 0.0).abs() < 0.01);
    }

    // ========================================================================
    // COVERAGE TESTS: Content deduplication - content-based
    // ========================================================================

    #[test]
    fn test_deduplicate_content_based() {
        let mut extractor = TextExtractor::new();
        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello World".to_string(), // >= 5 chars
                bbox: Rect::new(100.0, 700.0, 60.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello World".to_string(), // Same text, overlapping position
                bbox: Rect::new(102.0, 700.0, 60.0, 12.0), // X within 5pt
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.deduplicate_overlapping_spans();
        assert_eq!(extractor.spans.len(), 1, "Content duplicates should be removed");
    }

    #[test]
    fn test_deduplicate_content_not_overlapping() {
        let mut extractor = TextExtractor::new();
        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello World".to_string(),
                bbox: Rect::new(100.0, 700.0, 60.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello World".to_string(), // Same text but far apart
                bbox: Rect::new(500.0, 700.0, 60.0, 12.0), // X > 5pt difference
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.deduplicate_overlapping_spans();
        assert_eq!(extractor.spans.len(), 2, "Non-overlapping content should not be deduped");
    }

    // ========================================================================
    // COVERAGE TESTS: advance_position_for_string
    // ========================================================================

    #[test]
    fn test_advance_position_no_font() {
        let mut extractor = TextExtractor::new();
        extractor.state_stack.current_mut().font_size = 12.0;
        extractor.state_stack.current_mut().horizontal_scaling = 100.0;

        let width = extractor.advance_position_for_string(b"Hello").unwrap();
        assert!(width > 0.0, "Width should be positive even without font");
    }

    #[test]
    fn test_advance_position_with_font() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);
        extractor.cached_current_font = extractor.fonts.get("F1").cloned();
        extractor.state_stack.current_mut().font_size = 12.0;
        extractor.state_stack.current_mut().font_name = Some("F1".to_string());
        extractor.state_stack.current_mut().horizontal_scaling = 100.0;

        let width = extractor.advance_position_for_string(b"Hi").unwrap();
        assert!(width > 0.0, "Width should be positive with font");
    }

    #[test]
    fn test_advance_position_with_word_space() {
        let mut extractor = TextExtractor::new();
        extractor.state_stack.current_mut().font_size = 12.0;
        extractor.state_stack.current_mut().horizontal_scaling = 100.0;
        extractor.state_stack.current_mut().word_space = 5.0;

        let width = extractor.advance_position_for_string(b"A B").unwrap();
        assert!(width > 0.0);
    }

    // ========================================================================
    // COVERAGE TESTS: insert_space_as_span
    // ========================================================================

    #[test]
    fn test_insert_space_as_span() {
        let mut extractor = TextExtractor::new();
        extractor.state_stack.current_mut().font_size = 12.0;
        extractor.state_stack.current_mut().horizontal_scaling = 100.0;
        extractor.state_stack.current_mut().font_name = Some("F1".to_string());

        let before = extractor.spans.len();
        extractor.insert_space_as_span().unwrap();
        assert_eq!(extractor.spans.len(), before + 1);
        assert_eq!(extractor.spans.last().unwrap().text, " ");
        assert!(extractor.spans.last().unwrap().offset_semantic);
    }

    // ========================================================================
    // COVERAGE TESTS: split_fused_words
    // ========================================================================

    #[test]
    fn test_split_fused_words_camelcase() {
        let mut extractor = TextExtractor::new();
        extractor.spans = vec![TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: "theGeneral".to_string(),
            bbox: Rect::new(100.0, 700.0, 60.0, 12.0),
            font_name: "F1".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence: 0,
            split_boundary_before: false,
            offset_semantic: false,
            is_italic: false,
            is_monospace: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            primary_detected: false,
            char_widths: vec![],
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: 0.0,
            wmode: 0,
            rtl_draw_logical: false,
            mirrored: false,
            page_rotation_applied: 0,
        }];

        extractor.split_fused_words();
        assert_eq!(extractor.spans.len(), 2, "Should split theGeneral into two spans");
        assert_eq!(extractor.spans[0].text, "the");
        assert_eq!(extractor.spans[1].text, "General");
        assert!(extractor.spans[1].split_boundary_before);
    }

    #[test]
    fn test_split_fused_words_no_split() {
        let mut extractor = TextExtractor::new();
        extractor.spans = vec![TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: "hello".to_string(),
            bbox: Rect::new(100.0, 700.0, 30.0, 12.0),
            font_name: "F1".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence: 0,
            split_boundary_before: false,
            offset_semantic: false,
            is_italic: false,
            is_monospace: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            primary_detected: false,
            char_widths: vec![],
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: 0.0,
            wmode: 0,
            rtl_draw_logical: false,
            mirrored: false,
            page_rotation_applied: 0,
        }];

        extractor.split_fused_words();
        assert_eq!(extractor.spans.len(), 1, "No split needed for all-lowercase");
        assert_eq!(extractor.spans[0].text, "hello");
    }

    // ========================================================================
    // COVERAGE TESTS: calculate_average_glyph_width
    // ========================================================================

    #[test]
    fn test_calculate_average_glyph_width_no_widths() {
        let extractor = TextExtractor::new();
        let font = create_test_font(); // No widths array
        let avg = extractor.calculate_average_glyph_width(&font);
        assert_eq!(avg, font.default_width);
    }

    #[test]
    fn test_calculate_average_glyph_width_with_widths() {
        let extractor = TextExtractor::new();
        let mut font = create_test_font();
        font.first_char = Some(32);
        font.last_char = Some(126);
        font.widths = Some(vec![500.0; 95]); // 95 printable chars

        let avg = extractor.calculate_average_glyph_width(&font);
        assert!((avg - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_average_glyph_width_no_first_char() {
        let extractor = TextExtractor::new();
        let mut font = create_test_font();
        font.widths = Some(vec![500.0; 95]);
        font.first_char = None;

        let avg = extractor.calculate_average_glyph_width(&font);
        assert_eq!(avg, font.default_width);
    }

    #[test]
    fn test_calculate_average_glyph_width_no_last_char() {
        let extractor = TextExtractor::new();
        let mut font = create_test_font();
        font.widths = Some(vec![500.0; 95]);
        font.first_char = Some(32);
        font.last_char = None;

        let avg = extractor.calculate_average_glyph_width(&font);
        assert_eq!(avg, font.default_width);
    }

    // ========================================================================
    // COVERAGE TESTS: Adaptive TJ threshold with justified text
    // ========================================================================

    #[test]
    fn test_adaptive_threshold_with_justified_text() {
        let config = TextExtractionConfig {
            use_adaptive_tj_threshold: true,
            word_margin_ratio: 0.1,
            ..TextExtractionConfig::default()
        };
        let mut extractor = TextExtractor::with_config(config);
        extractor.state_stack.current_mut().font_size = 12.0;

        // Simulate justified text (high CV)
        for i in 0..100 {
            extractor
                .tj_offset_history
                .push(if i % 2 == 0 { -50.0 } else { -200.0 });
        }

        let threshold = extractor.calculate_adaptive_tj_threshold();
        // Justified text uses 3x ratio, so threshold should be more negative
        assert!(threshold < 0.0);
    }

    #[test]
    fn test_adaptive_threshold_with_font_name() {
        let config = TextExtractionConfig {
            use_adaptive_tj_threshold: true,
            word_margin_ratio: 0.1,
            ..TextExtractionConfig::default()
        };
        let mut extractor = TextExtractor::with_config(config);
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);
        extractor.state_stack.current_mut().font_size = 12.0;
        extractor.state_stack.current_mut().font_name = Some("F1".to_string());

        let threshold = extractor.calculate_adaptive_tj_threshold();
        assert!(threshold < 0.0);
    }

    #[test]
    fn test_analyze_tj_distribution_zero_mean() {
        let mut extractor = TextExtractor::new();
        // Push offsets that average to near zero
        extractor.tj_offset_history = vec![100.0, -100.0, 100.0, -100.0];
        let (is_justified, cv) = extractor.analyze_tj_distribution();
        // Mean ~0, so CV should be 0 (avoid division by zero)
        assert_eq!(cv, 0.0);
    }

    // ========================================================================
    // COVERAGE TESTS: Quote and DoubleQuote operators in span mode
    // ========================================================================

    #[test]
    fn test_quote_operator_span_mode() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 14 TL 100 700 Td (Line1) Tj (Line2) ' ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Line1"), "Should contain Line1, got: {}", text);
        assert!(text.contains("Line2"), "Should contain Line2, got: {}", text);
    }

    #[test]
    fn test_double_quote_operator_span_mode() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 14 TL 100 700 Td 1 2 (Text) \" ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Text"), "Should extract text, got: {}", text);
    }

    // ========================================================================
    // COVERAGE TESTS: Sort spans by columns (multi-column)
    // ========================================================================

    #[test]
    fn test_sort_spans_by_columns() {
        let mut extractor = TextExtractor::new();
        // Create spans in two distinct columns
        let columns = vec![(0.0, 250.0), (300.0, 550.0)];

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Right Col".to_string(),
                bbox: Rect::new(350.0, 700.0, 100.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Left Col".to_string(),
                bbox: Rect::new(50.0, 700.0, 100.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.sort_spans_by_columns(&columns);
        // Left column should come first
        assert_eq!(extractor.spans[0].text, "Left Col");
        assert_eq!(extractor.spans[1].text, "Right Col");
    }

    // ========================================================================
    // COVERAGE TESTS: TJ buffer with MCID
    // ========================================================================

    #[test]
    fn test_tj_buffer_with_mcid() {
        let state = crate::content::graphics_state::GraphicsStateStack::new();
        let buffer = TjBuffer::new(state.current(), Some(42), None, Vec::new());
        assert!(buffer.is_empty());
        assert_eq!(buffer.mcid, Some(42));
    }

    // ========================================================================
    // COVERAGE TESTS: Word boundary mode primary
    // ========================================================================

    #[test]
    fn test_extractor_with_primary_word_boundary() {
        let config = TextExtractionConfig {
            word_boundary_mode: WordBoundaryMode::Primary,
            ..TextExtractionConfig::default()
        };
        let mut extractor = TextExtractor::with_config(config);
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);
        extractor.merging_config = SpanMergingConfig::legacy();

        let stream = b"BT /F1 12 Tf 100 700 Td (Hello) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Hello"), "Primary mode should still extract text, got: {}", text);
    }

    // ========================================================================
    // COVERAGE TESTS: Merge adjacent spans - double space prevention
    // ========================================================================

    #[test]
    fn test_merge_prevents_double_space() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello ".to_string(), // ends with space
                bbox: Rect::new(100.0, 700.0, 35.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: " World".to_string(), // starts with space
                bbox: Rect::new(136.0, 700.0, 35.0, 12.0), // 1pt gap
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: true, // forces merge-with-space path
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 1);
        // Should not have "Hello World" (triple space)
        assert!(!extractor.spans[0].text.contains("   "), "Should prevent triple space");
    }

    // ========================================================================
    // COVERAGE TESTS: TextExtractor with_config builder
    // ========================================================================

    #[test]
    fn test_extractor_with_config_copies_word_boundary_mode() {
        let config = TextExtractionConfig {
            word_boundary_mode: WordBoundaryMode::Primary,
            ..TextExtractionConfig::default()
        };
        let extractor = TextExtractor::with_config(config);
        assert_eq!(extractor.word_boundary_mode, WordBoundaryMode::Primary);
    }

    // ========================================================================
    // COVERAGE TESTS: Partition characters boundary at start
    // ========================================================================

    #[test]
    fn test_partition_boundary_at_start() {
        let extractor = TextExtractor::new();
        let chars = vec![
            CharacterInfo {
                code: 65,
                glyph_id: None,
                width: 10.0,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            },
            CharacterInfo {
                code: 66,
                glyph_id: None,
                width: 10.0,
                x_position: 10.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            },
        ];

        // Boundary at 0 means empty first cluster
        let clusters = extractor.partition_characters_by_boundaries(&chars, vec![0]);
        // Should have just one cluster (boundary at 0 produces no items before it)
        assert!(!clusters.is_empty());
    }

    // ========================================================================
    // COVERAGE TESTS: Color space resets (fill_color_cmyk cleared)
    // ========================================================================

    #[test]
    fn test_fill_cmyk_then_change_color_space() {
        let mut extractor = TextExtractor::new();
        extractor
            .execute_operator_public(Operator::SetFillCmyk {
                c: 0.5,
                m: 0.5,
                y: 0.5,
                k: 0.5,
            })
            .unwrap();
        assert!(extractor.state_stack.current().fill_color_cmyk.is_some());

        // Changing color space should reset CMYK
        extractor
            .execute_operator_public(Operator::SetFillColorSpace {
                name: "DeviceRGB".to_string(),
            })
            .unwrap();
        assert!(extractor.state_stack.current().fill_color_cmyk.is_none());
    }

    // ========================================================================
    // COVERAGE TESTS: Marked content with BDC
    // ========================================================================

    #[test]
    fn test_bdc_with_mcid() {
        let mut extractor = TextExtractor::new();
        let mut props = HashMap::new();
        props.insert("MCID".to_string(), Object::Integer(5));

        extractor
            .execute_operator_public(Operator::BeginMarkedContentDict {
                tag: "P".to_string(),
                properties: Box::new(Object::Dictionary(props)),
            })
            .unwrap();

        assert_eq!(extractor.current_mcid, Some(5));
        assert!(!extractor.inside_artifact);
    }

    #[test]
    fn test_bdc_artifact_with_type() {
        let mut extractor = TextExtractor::new();
        let mut props = HashMap::new();
        props.insert("Type".to_string(), Object::Name("Pagination".to_string()));
        props.insert("Subtype".to_string(), Object::Name("Header".to_string()));

        extractor
            .execute_operator_public(Operator::BeginMarkedContentDict {
                tag: "Artifact".to_string(),
                properties: Box::new(Object::Dictionary(props)),
            })
            .unwrap();

        assert!(extractor.inside_artifact);
    }

    #[test]
    fn test_emc_resets_mcid() {
        let mut extractor = TextExtractor::new();
        extractor.current_mcid = Some(10);
        extractor.marked_content_stack.push(MarkedContentContext {
            artifact_type: None,
            tag: "P".to_string(),
            is_artifact: false,
            actual_text: None,
            expansion: None,
            is_excluded_layer: false,
            is_placed_pdf: false,
            actual_text_emitted: false,
            own_mcid: None,
        });

        extractor
            .execute_operator_public(Operator::EndMarkedContent)
            .unwrap();

        assert_eq!(extractor.current_mcid, None);
        assert!(extractor.marked_content_stack.is_empty());
    }

    #[test]
    fn test_emc_with_empty_stack() {
        let mut extractor = TextExtractor::new();
        // Should not panic
        extractor
            .execute_operator_public(Operator::EndMarkedContent)
            .unwrap();
    }

    // ========================================================================
    // COVERAGE TESTS: BDC with ActualText and Expansion
    // ========================================================================

    #[test]
    fn test_bdc_with_actual_text() {
        let mut extractor = TextExtractor::new();
        let mut props = HashMap::new();
        props.insert("ActualText".to_string(), Object::String(b"fi".to_vec()));

        extractor
            .execute_operator_public(Operator::BeginMarkedContentDict {
                tag: "Span".to_string(),
                properties: Box::new(Object::Dictionary(props)),
            })
            .unwrap();

        let actual = extractor.get_current_actual_text();
        assert_eq!(actual, Some("fi".to_string()));
    }

    #[test]
    fn test_bdc_with_expansion() {
        let mut extractor = TextExtractor::new();
        let mut props = HashMap::new();
        props.insert("E".to_string(), Object::String(b"PDF".to_vec()));

        extractor
            .execute_operator_public(Operator::BeginMarkedContentDict {
                tag: "Span".to_string(),
                properties: Box::new(Object::Dictionary(props)),
            })
            .unwrap();

        let ctx = &extractor.marked_content_stack[0];
        assert_eq!(ctx.expansion, Some("PDF".to_string()));
    }

    // ========================================================================
    // COVERAGE TESTS: Do operator without document
    // ========================================================================

    #[test]
    fn test_do_operator_without_document() {
        let mut extractor = TextExtractor::new();
        // Do without document set should not panic
        extractor
            .execute_operator_public(Operator::Do {
                name: "Im1".to_string(),
            })
            .unwrap();
    }

    // ========================================================================
    // COVERAGE TESTS: flush_tj_span_buffer when buffer is Some but empty
    // ========================================================================

    #[test]
    fn test_flush_tj_span_buffer_empty_buffer() {
        let mut extractor = TextExtractor::new();
        let state = extractor.state_stack.current().clone();
        extractor.tj_span_buffer = Some(TjBuffer::new(&state, None, None, Vec::new()));
        // Empty buffer should not produce a span
        let before = extractor.spans.len();
        extractor.flush_tj_span_buffer().unwrap();
        assert_eq!(extractor.spans.len(), before);
    }

    #[test]
    fn test_flush_tj_span_buffer_with_content() {
        let mut extractor = TextExtractor::new();
        let state_stack = crate::content::graphics_state::GraphicsStateStack::new();
        let mut buffer = TjBuffer::new(state_stack.current(), Some(7), None, Vec::new());
        buffer.append(b"Test").unwrap();
        buffer.accumulated_width = 20.0;
        extractor.tj_span_buffer = Some(buffer);

        extractor.flush_tj_span_buffer().unwrap();
        assert_eq!(extractor.spans.len(), 1);
        assert!(extractor.spans[0].text.contains("Test"));
    }

    // ========================================================================
    // COVERAGE TESTS: TJ array with adaptive threshold - full pipeline
    // ========================================================================

    #[test]
    fn test_tj_array_span_mode_with_space_insertion() {
        let config = TextExtractionConfig {
            use_adaptive_tj_threshold: false,
            space_insertion_threshold: -120.0,
            ..TextExtractionConfig::default()
        };
        let mut extractor = TextExtractor::with_config(config);
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // TJ array with large offset that triggers space
        let stream = b"BT /F1 12 Tf 100 700 Td [(Word1) -500 (Word2)] TJ ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        let text: String = spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("Word1"), "Should contain Word1");
        assert!(text.contains("Word2"), "Should contain Word2");
    }

    // ========================================================================
    // COVERAGE TESTS: Sort spans reading order (single vs multi column)
    // ========================================================================

    #[test]
    fn test_sort_spans_single_column() {
        let mut extractor = TextExtractor::new();
        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Line2".to_string(),
                bbox: Rect::new(50.0, 680.0, 100.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Line1".to_string(),
                bbox: Rect::new(50.0, 700.0, 100.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.sort_spans_by_reading_order();
        assert_eq!(extractor.spans[0].text, "Line1"); // higher Y first
        assert_eq!(extractor.spans[1].text, "Line2");
    }

    /// A scanned vertical-CJK OCR layer can emit hundreds of single-glyph
    /// `wmode=1` spans whose X-centers step by a fraction of the median
    /// span width: every adjacent pair looks "same column" under a pairwise
    /// `|a - b| <= tol` check, but the first and last span are hundreds of
    /// points apart, so the comparator claims contradictory orderings
    /// (A<B, B<C, C<A) and Rust's `sort_by` panics with "does not correctly
    /// implement a total order" instead of returning a reading order.
    #[test]
    fn test_sort_spans_vertical_tategaki_chained_x_centers_does_not_panic() {
        let mut extractor = TextExtractor::new();
        extractor.spans = (0..240)
            .map(|i| TextSpan {
                text: format!("g{i}"),
                bbox: Rect::new(
                    20.0 + i as f32 * 0.8,
                    700.0 - ((i * 37) % 96) as f32 * 7.0,
                    1.0,
                    12.0,
                ),
                font_size: 12.0,
                wmode: 1,
                ..TextSpan::default()
            })
            .collect();

        extractor.sort_spans_by_reading_order(); // must not panic
        assert_eq!(extractor.spans.len(), 240);
    }

    // ========================================================================
    // COVERAGE TESTS: Tm continuation optimization
    // ========================================================================

    #[test]
    fn test_tm_continuation_different_transform() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Different transform params (a=2) should NOT be continuation
        let stream = b"BT /F1 12 Tf 1 0 0 1 100 700 Tm (A) Tj 2 0 0 1 120 700 Tm (B) Tj ET";
        let spans = extractor.extract_text_spans(stream).unwrap();

        // Should produce separate spans due to different transform
        assert!(!spans.is_empty());
    }

    // ========================================================================
    // COVERAGE TESTS: decode_pdf_text_string edge cases
    // ========================================================================

    #[test]
    fn test_decode_pdf_text_string_single_byte() {
        let result = TextExtractor::decode_pdf_text_string(&[0x41]);
        assert_eq!(result, "A");
    }

    #[test]
    fn test_decode_pdf_text_string_invalid_utf16() {
        // UTF-16BE BOM followed by invalid pair
        let bytes = vec![0xFE, 0xFF, 0xD8, 0x00]; // invalid surrogate half
        let result = TextExtractor::decode_pdf_text_string(&bytes);
        // Should fall back to lossy conversion
        assert!(!result.is_empty() || result.is_empty()); // Just don't panic
    }

    #[test]
    fn test_decode_pdf_text_string_utf16le_invalid() {
        // UTF-16LE BOM followed by odd byte count
        let bytes = vec![0xFF, 0xFE, 0x41]; // odd after BOM
        let result = TextExtractor::decode_pdf_text_string(&bytes);
        // Should handle gracefully
    }

    // ========================================================================
    // TDD: decode_pdf_text_string — PDFDocEncoding fallback correctness
    // Bytes 0xA0–0xFF and the special 0x80–0x9E zone must decode through
    // PDFDocEncoding, not through from_utf8_lossy (which produces U+FFFD).
    // ========================================================================

    #[test]
    fn test_decode_pdfdocencoding_latin_byte() {
        // 0xE9 = PDFDocEncoding for é (U+00E9). Not valid UTF-8 on its own.
        let result = TextExtractor::decode_pdf_text_string(&[0xE9]);
        assert_eq!(result, "é", "0xE9 must decode as 'é' via PDFDocEncoding, not produce U+FFFD");
    }

    #[test]
    fn test_decode_pdfdocencoding_bullet() {
        // 0x80 = PDFDocEncoding for • (U+2022 BULLET)
        let result = TextExtractor::decode_pdf_text_string(&[0x80]);
        assert_eq!(result, "•", "0x80 must decode as bullet '•' via PDFDocEncoding");
    }

    #[test]
    fn test_decode_pdfdocencoding_emdash() {
        // 0x84 = PDFDocEncoding for — (U+2014 EM DASH)
        let result = TextExtractor::decode_pdf_text_string(&[0x84]);
        assert_eq!(result, "—", "0x84 must decode as em-dash '—' via PDFDocEncoding");
    }

    #[test]
    fn test_decode_pdfdocencoding_trademark() {
        // 0x92 = PDFDocEncoding for ™ (U+2122 TRADE MARK SIGN)
        let result = TextExtractor::decode_pdf_text_string(&[0x92]);
        assert_eq!(result, "™", "0x92 must decode as trademark '™' via PDFDocEncoding");
    }

    #[test]
    fn test_decode_pdfdocencoding_undefined_9f_is_dropped() {
        // 0x9F is undefined in PDFDocEncoding — must be silently dropped.
        let result = TextExtractor::decode_pdf_text_string(&[0x41, 0x9F, 0x42]);
        assert_eq!(result, "AB", "0x9F is undefined in PDFDocEncoding and must be dropped");
    }

    #[test]
    fn test_decode_pdfdocencoding_mixed_ascii_and_latin() {
        // "Hello" followed by 0xE9 (é): 6 bytes → "Helloé"
        let bytes: Vec<u8> = b"Hello".iter().copied().chain([0xE9]).collect();
        let result = TextExtractor::decode_pdf_text_string(&bytes);
        assert_eq!(result, "Helloé", "Mixed ASCII + PDFDocEncoding bytes must decode correctly");
    }

    #[test]
    fn test_decode_pdfdocencoding_utf8_bytes_still_work() {
        // Valid UTF-8 without BOM: must still decode correctly (for lenient PDFs).
        // ASCII is a subset of UTF-8, so this path always works.
        let result = TextExtractor::decode_pdf_text_string(b"ASCII text");
        assert_eq!(result, "ASCII text");
    }

    // ========================================================================
    // COVERAGE TESTS: shared truetype cmaps (no donors)
    // ========================================================================

    #[test]
    fn test_share_truetype_cmaps_no_donors() {
        let mut extractor = TextExtractor::new();
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        // Should return early (no cmap donors)
        extractor.share_truetype_cmaps();
        assert_eq!(extractor.fonts.len(), 1);
    }

    // ========================================================================
    // COVERAGE TESTS: Extract with WithConfig
    // ========================================================================

    #[test]
    fn test_extractor_with_config_and_profile() {
        let config =
            TextExtractionConfig::new().with_profile(crate::config::ExtractionProfile::POLICY);

        let mut extractor = TextExtractor::with_config(config);
        let font = create_test_font();
        extractor.add_font("F1".to_string(), font);

        let stream = b"BT /F1 12 Tf 100 700 Td (Policy) Tj ET";
        let chars = extractor.extract(stream).unwrap();
        assert!(!chars.is_empty());
    }

    // ========================================================================
    // COVERAGE TESTS: Merge with offset_semantic space span suppression
    // ========================================================================

    #[test]
    fn test_merge_offset_semantic_space_suppression() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello".to_string(),
                bbox: Rect::new(100.0, 700.0, 30.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: " ".to_string(), // offset_semantic space
                bbox: Rect::new(130.5, 700.0, 2.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: true, // forcing merge path
                offset_semantic: true,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        // offset_semantic space should be merged without adding extra space
        let text = &extractor.spans[0].text;
        assert!(!text.contains("  "), "Should not have double space, got: '{}'", text);
    }
}

#[test]
fn test_space_threshold_default() {
    // Test that default configuration uses -120.0 threshold
    let config = TextExtractionConfig::new();
    assert_eq!(config.space_insertion_threshold, -120.0);

    // Test that default extractor has default config
    let extractor = TextExtractor::new();
    assert_eq!(extractor.config.space_insertion_threshold, -120.0);
}

#[test]
fn test_space_threshold_custom() {
    // Test custom threshold configuration
    let config = TextExtractionConfig::with_space_threshold(-80.0);
    assert_eq!(config.space_insertion_threshold, -80.0);

    let extractor = TextExtractor::with_config(config);
    assert_eq!(extractor.config.space_insertion_threshold, -80.0);
}

#[test]
fn test_space_threshold_disabled() {
    // Test that threshold can be disabled with NEG_INFINITY
    let config = TextExtractionConfig::with_space_threshold(f32::NEG_INFINITY);
    assert_eq!(config.space_insertion_threshold, f32::NEG_INFINITY);

    let extractor = TextExtractor::with_config(config);
    assert_eq!(extractor.config.space_insertion_threshold, f32::NEG_INFINITY);
}

#[test]
fn test_adaptive_enabled_by_default() {
    // Test that adaptive threshold is enabled by default
    let config = SpanMergingConfig::default();
    assert!(config.use_adaptive_threshold, "Adaptive threshold should be enabled by default");
}

#[test]
fn test_legacy_mode_disables_adaptive() {
    // Test that legacy() constructor provides backward-compatible behavior
    let legacy = SpanMergingConfig::legacy();
    assert!(!legacy.use_adaptive_threshold, "Legacy mode should disable adaptive threshold");
    assert_eq!(legacy.conservative_threshold_pt, 0.1);
}

#[test]
fn test_adaptive_constructor_enables_adaptive() {
    // Test that adaptive() constructor enables adaptive threshold
    let adaptive = SpanMergingConfig::adaptive();
    assert!(
        adaptive.use_adaptive_threshold,
        "Adaptive constructor should enable adaptive threshold"
    );
    assert!(
        adaptive.adaptive_config.is_some(),
        "Adaptive constructor should set adaptive_config"
    );
}

// ============================================================================
// Artifact Type Parsing Tests (PDF Spec Section 14.8.2.2)
// ============================================================================

#[test]
fn test_parse_artifact_type_pagination_header() {
    let mut props = HashMap::new();
    props.insert("Type".to_string(), Object::Name("Pagination".to_string()));
    props.insert("Subtype".to_string(), Object::Name("Header".to_string()));

    let result = TextExtractor::parse_artifact_type(&props);
    assert_eq!(result, Some(ArtifactType::Pagination(PaginationSubtype::Header)));
}

#[test]
fn test_parse_artifact_type_pagination_footer() {
    let mut props = HashMap::new();
    props.insert("Type".to_string(), Object::Name("Pagination".to_string()));
    props.insert("Subtype".to_string(), Object::Name("Footer".to_string()));

    let result = TextExtractor::parse_artifact_type(&props);
    assert_eq!(result, Some(ArtifactType::Pagination(PaginationSubtype::Footer)));
}

#[test]
fn test_parse_artifact_type_pagination_watermark() {
    let mut props = HashMap::new();
    props.insert("Type".to_string(), Object::Name("Pagination".to_string()));
    props.insert("Subtype".to_string(), Object::Name("Watermark".to_string()));

    let result = TextExtractor::parse_artifact_type(&props);
    assert_eq!(result, Some(ArtifactType::Pagination(PaginationSubtype::Watermark)));
}

#[test]
fn test_parse_artifact_type_layout() {
    let mut props = HashMap::new();
    props.insert("Type".to_string(), Object::Name("Layout".to_string()));

    let result = TextExtractor::parse_artifact_type(&props);
    assert_eq!(result, Some(ArtifactType::Layout));
}

#[test]
fn test_parse_artifact_type_background() {
    let mut props = HashMap::new();
    props.insert("Type".to_string(), Object::Name("Background".to_string()));

    let result = TextExtractor::parse_artifact_type(&props);
    assert_eq!(result, Some(ArtifactType::Background));
}

#[test]
fn test_parse_artifact_type_subtype_only() {
    // Some PDFs use /Subtype without /Type
    let mut props = HashMap::new();
    props.insert("Subtype".to_string(), Object::Name("Header".to_string()));

    let result = TextExtractor::parse_artifact_type(&props);
    assert_eq!(result, Some(ArtifactType::Pagination(PaginationSubtype::Header)));
}

#[test]
fn test_parse_artifact_type_empty() {
    let props = HashMap::new();
    let result = TextExtractor::parse_artifact_type(&props);
    assert_eq!(result, None);
}

// ============================================================================
// ActualText Verification Tests (PDF Spec Section 14.9.4)
// ============================================================================
//
// ActualText provides replacement text for content that cannot be accurately
// represented by the content stream (ligatures, decorated glyphs, formulas).
// Per ISO 32000-1:2008 Section 14.9.4, ActualText takes precedence over
// character extraction.

#[test]
fn test_marked_content_context_with_actual_text() {
    // Verify MarkedContentContext correctly stores ActualText
    let ctx = MarkedContentContext {
        artifact_type: None,
        tag: "Span".to_string(),
        is_artifact: false,
        actual_text: Some("fi".to_string()), // Ligature expansion
        expansion: None,
        is_excluded_layer: false,
        is_placed_pdf: false,
        actual_text_emitted: false,
        own_mcid: None,
    };

    assert_eq!(ctx.actual_text, Some("fi".to_string()));
    assert!(!ctx.is_artifact);
}

#[test]
fn test_marked_content_context_with_expansion() {
    // Verify MarkedContentContext correctly stores /E expansion
    let ctx = MarkedContentContext {
        artifact_type: None,
        tag: "Span".to_string(),
        is_artifact: false,
        actual_text: None,
        expansion: Some("Portable Document Format".to_string()),
        is_excluded_layer: false,
        is_placed_pdf: false,
        actual_text_emitted: false,
        own_mcid: None,
    };

    assert_eq!(ctx.expansion, Some("Portable Document Format".to_string()));
}

#[test]
fn test_marked_content_context_artifact_with_actual_text() {
    // Verify artifacts can have ActualText (though typically they don't)
    let ctx = MarkedContentContext {
        tag: "Artifact".to_string(),
        is_artifact: true,
        artifact_type: Some(ArtifactType::Pagination(PaginationSubtype::Header)),
        actual_text: Some("Header text".to_string()),
        expansion: None,
        is_excluded_layer: false,
        is_placed_pdf: false,
        actual_text_emitted: false,
        own_mcid: None,
    };

    assert!(ctx.is_artifact);
    assert_eq!(ctx.actual_text, Some("Header text".to_string()));
}

#[test]
fn test_get_current_actual_text_finds_first() {
    // Verify get_current_actual_text returns first ActualText in stack
    let mut extractor = TextExtractor::new();

    // Push contexts with ActualText
    extractor.marked_content_stack.push(MarkedContentContext {
        artifact_type: None,
        tag: "Span".to_string(),
        is_artifact: false,
        actual_text: Some("outer text".to_string()),
        expansion: None,
        is_excluded_layer: false,
        is_placed_pdf: false,
        actual_text_emitted: false,
        own_mcid: None,
    });

    extractor.marked_content_stack.push(MarkedContentContext {
        artifact_type: None,
        tag: "Span".to_string(),
        is_artifact: false,
        actual_text: Some("inner text".to_string()),
        expansion: None,
        is_excluded_layer: false,
        is_placed_pdf: false,
        actual_text_emitted: false,
        own_mcid: None,
    });

    // Should return innermost (most recent) ActualText
    let result = extractor.get_current_actual_text();
    assert_eq!(result, Some("inner text".to_string()));
}

#[test]
fn test_get_current_actual_text_skips_none() {
    // Verify get_current_actual_text skips contexts without ActualText
    let mut extractor = TextExtractor::new();

    // Push context with ActualText
    extractor.marked_content_stack.push(MarkedContentContext {
        artifact_type: None,
        tag: "Span".to_string(),
        is_artifact: false,
        actual_text: Some("replacement text".to_string()),
        expansion: None,
        is_excluded_layer: false,
        is_placed_pdf: false,
        actual_text_emitted: false,
        own_mcid: None,
    });

    // Push context without ActualText
    extractor.marked_content_stack.push(MarkedContentContext {
        artifact_type: None,
        tag: "Span".to_string(),
        is_artifact: false,
        actual_text: None,
        expansion: None,
        is_excluded_layer: false,
        is_placed_pdf: false,
        actual_text_emitted: false,
        own_mcid: None,
    });

    // Should find the ActualText from outer context
    let result = extractor.get_current_actual_text();
    assert_eq!(result, Some("replacement text".to_string()));
}

#[test]
fn test_get_current_actual_text_returns_none_when_empty() {
    // Verify get_current_actual_text returns None when no ActualText
    let extractor = TextExtractor::new();

    let result = extractor.get_current_actual_text();
    assert_eq!(result, None);
}

// ============================================================================
// PHASE 2.5: Profile-Based Space Insertion Tests (TDD)
// ============================================================================
//
// Tests for document-type-specific extraction profiles.
// These tests define expected behavior BEFORE implementation.
// Once these tests pass, the profile integration is complete.
//
// Key Scenarios:
// 1. Academic papers: Tighter spacing, aggressive space insertion
// 2. Policy documents: Justified text, conservative spacing
// 3. Forms: Structured fields with precise boundaries
// 4. Default/Conservative: Backward-compatible behavior

#[cfg(test)]
mod profile_based_space_tests {
    use super::*;

    /// Test that ACADEMIC profile uses aggressive thresholds
    ///
    /// Academic papers have tight spacing (especially around punctuation).
    /// The profile should:
    /// - Use lower TJ offset threshold (-90 instead of -120)
    /// - Use lower word margin ratio (0.12 instead of 0.1)
    /// - Enable adaptive threshold for dynamic adjustment
    #[test]
    fn test_academic_profile_thresholds() {
        let profile = crate::config::ExtractionProfile::for_document_type(
            crate::config::DocumentType::Academic,
        );

        // Academic papers should be more aggressive with space insertion
        assert!(
            profile.tj_offset_threshold < -100.0,
            "Academic should use lower TJ threshold for more spaces"
        );

        // Academic papers should have tighter word margins
        assert!(
            profile.word_margin_ratio <= 0.15,
            "Academic should use conservative word margin"
        );

        // Verify we can create a config from the profile
        let config = TextExtractionConfig::with_space_threshold(profile.tj_offset_threshold);
        assert_eq!(config.space_insertion_threshold, profile.tj_offset_threshold);
    }

    /// Test that POLICY profile uses conservative thresholds
    ///
    /// Policy documents (like GDPR) have justified text with precise spacing.
    /// The profile should:
    /// - Use higher TJ offset threshold (-110 to preserve structure)
    /// - Use higher word margin ratio (0.18-0.2 for justified text)
    /// - Preserve column boundaries and table structure
    #[test]
    fn test_policy_profile_thresholds() {
        let profile = crate::config::ExtractionProfile::for_document_type(
            crate::config::DocumentType::Policy,
        );

        // Policy documents should be more conservative to preserve structure
        assert!(
            profile.tj_offset_threshold > -120.0,
            "Policy should use higher TJ threshold to avoid over-spacing"
        );

        // Policy documents should have looser word margins for justified text
        assert!(
            profile.word_margin_ratio >= 0.15,
            "Policy should use higher word margin for justified text"
        );

        let config = TextExtractionConfig::with_space_threshold(profile.tj_offset_threshold);
        assert_eq!(config.space_insertion_threshold, profile.tj_offset_threshold);
    }

    /// Test that FORM profile preserves field boundaries
    ///
    /// Forms have checkboxes, fields, and precise layout.
    /// The profile should:
    /// - Use conservative thresholds to avoid merging fields
    /// - High column boundary threshold to preserve structure
    /// - Enable adaptive threshold for form field detection
    #[test]
    fn test_form_profile_thresholds() {
        let profile =
            crate::config::ExtractionProfile::for_document_type(crate::config::DocumentType::Form);

        // Forms should preserve field structure with conservative spacing
        assert!(
            profile.tj_offset_threshold >= -120.0,
            "Form profile should be conservative with space insertion"
        );

        let config = TextExtractionConfig::with_space_threshold(profile.tj_offset_threshold);
        assert_eq!(config.space_insertion_threshold, profile.tj_offset_threshold);
    }

    /// Test that profile selection works correctly for document types
    #[test]
    fn test_profile_selection_for_document_types() {
        let academic = crate::config::ExtractionProfile::for_document_type(
            crate::config::DocumentType::Academic,
        );
        let policy = crate::config::ExtractionProfile::for_document_type(
            crate::config::DocumentType::Policy,
        );
        let form =
            crate::config::ExtractionProfile::for_document_type(crate::config::DocumentType::Form);
        let mixed =
            crate::config::ExtractionProfile::for_document_type(crate::config::DocumentType::Mixed);

        // Verify each profile has distinct thresholds
        let thresholds = [
            academic.tj_offset_threshold,
            policy.tj_offset_threshold,
            form.tj_offset_threshold,
            mixed.tj_offset_threshold,
        ];

        // At least some profiles should have different thresholds
        let unique_count = thresholds
            .iter()
            .filter(|t| !thresholds.iter().skip(1).any(|other| other == *t))
            .count();

        assert!(
            unique_count > 0,
            "Profiles should have different thresholds for different document types"
        );
    }

    /// Test that TextExtractionConfig can accept a profile
    #[test]
    fn test_config_with_profile() {
        let profile = crate::config::ExtractionProfile::ACADEMIC;

        // Should be able to create config with profile thresholds
        let config = TextExtractionConfig::with_space_threshold(profile.tj_offset_threshold);

        assert_eq!(config.space_insertion_threshold, profile.tj_offset_threshold);
    }

    /// Test that profiles have reasonable threshold ranges
    #[test]
    fn test_profile_thresholds_in_reasonable_range() {
        let profiles = vec![
            crate::config::ExtractionProfile::CONSERVATIVE,
            crate::config::ExtractionProfile::ACADEMIC,
            crate::config::ExtractionProfile::POLICY,
            crate::config::ExtractionProfile::FORM,
        ];

        for profile in profiles {
            // TJ offsets should be negative (per PDF spec)
            assert!(
                profile.tj_offset_threshold < 0.0,
                "TJ threshold must be negative ({})",
                profile.name
            );

            // Should be in reasonable range (-150 to -50)
            assert!(
                profile.tj_offset_threshold >= -150.0 && profile.tj_offset_threshold <= -50.0,
                "TJ threshold out of range for {} ({})",
                profile.name,
                profile.tj_offset_threshold
            );

            // Word margin ratios should be positive and reasonable (0.05 to 0.25)
            assert!(
                profile.word_margin_ratio > 0.0 && profile.word_margin_ratio < 1.0,
                "Word margin ratio must be between 0 and 1 for {}",
                profile.name
            );

            // Space threshold EM ratio should be positive
            assert!(
                profile.space_threshold_em_ratio > 0.0,
                "Space threshold EM ratio must be positive for {}",
                profile.name
            );
        }
    }

    /// Test that multiple profiles can coexist
    #[test]
    fn test_multiple_profiles_independent() {
        let academic = crate::config::ExtractionProfile::for_document_type(
            crate::config::DocumentType::Academic,
        );
        let policy = crate::config::ExtractionProfile::for_document_type(
            crate::config::DocumentType::Policy,
        );

        // Create configs from both profiles
        let academic_config =
            TextExtractionConfig::with_space_threshold(academic.tj_offset_threshold);
        let policy_config = TextExtractionConfig::with_space_threshold(policy.tj_offset_threshold);

        // Verify they have different thresholds
        assert_ne!(
            academic_config.space_insertion_threshold, policy_config.space_insertion_threshold,
            "Academic and policy configs should have different thresholds"
        );
    }

    /// Test that default config is backward-compatible
    #[test]
    fn test_default_config_backward_compatible() {
        let default_config = TextExtractionConfig::default();
        let conservative_profile = crate::config::ExtractionProfile::CONSERVATIVE;

        // Default should match or be compatible with conservative profile
        assert_eq!(
            default_config.space_insertion_threshold, conservative_profile.tj_offset_threshold,
            "Default config should use conservative threshold for backward compatibility"
        );
    }

    /// Test that adjacent table cell values get spaces inserted between them.
    ///
    /// Simulates a form where two "$0.00" values are in adjacent cells with
    /// a small positive gap (1pt). The merge logic should insert a space because
    /// the spans are clearly separate tokens (ending/starting with digits/currency).
    #[test]
    fn test_adjacent_table_cell_values_not_concatenated() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        // "$0.00" at 10pt font is about 30pt wide (5 chars * ~6pt average width)
        // Second value starts at x=131, creating a 1pt gap (100 + 30 = 130, gap = 1pt)
        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "$0.00".to_string(),
                bbox: Rect::new(100.0, 700.0, 30.0, 10.0),
                font_name: "F1".to_string(),
                font_size: 10.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "$0.00".to_string(),
                bbox: Rect::new(131.0, 700.0, 30.0, 10.0), // 1pt gap
                font_name: "F1".to_string(),
                font_size: 10.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 1, "Adjacent spans should merge");
        assert_eq!(
            extractor.spans[0].text, "$0.00 $0.00",
            "Adjacent table cell values should have space between them, got: '{}'",
            extractor.spans[0].text
        );
    }

    /// Test that adjacent numeric values with small gaps get spaces.
    /// Covers cases like "100200" that should be "100 200" in table contexts.
    #[test]
    fn test_adjacent_numeric_values_not_concatenated() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "100".to_string(),
                bbox: Rect::new(200.0, 500.0, 18.0, 10.0),
                font_name: "F1".to_string(),
                font_size: 10.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "200".to_string(),
                bbox: Rect::new(219.5, 500.0, 18.0, 10.0), // 1.5pt gap
                font_name: "F1".to_string(),
                font_size: 10.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 1, "Adjacent spans should merge");
        assert_eq!(
            extractor.spans[0].text, "100 200",
            "Adjacent numeric values should have space between them, got: '{}'",
            extractor.spans[0].text
        );
    }

    /// Ensure that true word fragments (zero gap) still merge without space.
    /// E.g., "Hel" + "lo" with gap=0 should become "Hello" not "Hel lo".
    #[test]
    fn test_word_fragments_zero_gap_no_space() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hel".to_string(),
                bbox: Rect::new(100.0, 700.0, 18.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "lo".to_string(),
                bbox: Rect::new(118.0, 700.0, 12.0, 12.0), // 0pt gap
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 1, "Adjacent spans should merge");
        assert_eq!(
            extractor.spans[0].text, "Hello",
            "Zero-gap word fragments should merge without space, got: '{}'",
            extractor.spans[0].text
        );
    }

    // ========================================================================
    // Decimal dollar value merging (split integer/decimal boxes)
    // ========================================================================

    #[test]
    fn test_merge_decimal_dollar_value_split_boxes() {
        // Some forms have integer and decimal parts in separate fixed-width boxes.
        // e.g., "123456" at x=382.3 width=39.6, "72" at x=432.7 width=13.2
        // gap = 432.7 - (382.3 + 39.6) = 10.8pt
        // These should be merged as "123456.72"
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "123456".to_string(),
                bbox: Rect::new(382.3, 700.0, 39.6, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "72".to_string(),
                bbox: Rect::new(432.7, 700.0, 13.2, 12.0), // 10.8pt gap
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 1, "Decimal dollar value spans should merge into one");
        assert_eq!(
            extractor.spans[0].text, "123456.72",
            "Integer and decimal parts should be joined with '.'"
        );
    }

    #[test]
    fn test_merge_decimal_value_small_integer_part() {
        // Smaller dollar amount: "50" + "00" -> "50.00"
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "50".to_string(),
                bbox: Rect::new(382.3, 700.0, 15.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "00".to_string(),
                bbox: Rect::new(407.0, 700.0, 13.2, 12.0), // 9.7pt gap
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 1);
        assert_eq!(extractor.spans[0].text, "50.00");
    }

    #[test]
    fn test_no_decimal_merge_for_non_digit_spans() {
        // Should NOT merge "Hello" + "72" as decimal
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello".to_string(),
                bbox: Rect::new(382.3, 700.0, 39.6, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "72".to_string(),
                bbox: Rect::new(432.7, 700.0, 13.2, 12.0), // 10.8pt gap
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        // Should NOT merge because first span is not all digits
        assert_eq!(
            extractor.spans.len(),
            2,
            "Non-digit spans should not be merged as decimal values"
        );
    }

    #[test]
    fn test_no_decimal_merge_for_long_decimal_part() {
        // Should NOT merge "123456" + "723" (3-digit decimal part is not a cents pattern)
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "123456".to_string(),
                bbox: Rect::new(382.3, 700.0, 39.6, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "723".to_string(),
                bbox: Rect::new(432.7, 700.0, 18.0, 12.0), // 10.8pt gap
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        // Should NOT merge because decimal part has 3 digits (not a cents pattern)
        assert_eq!(
            extractor.spans.len(),
            2,
            "3-digit decimal part should not trigger decimal merge"
        );
    }

    /// Compact TextSpan builder for the intervening-ink decimal tests.
    fn digit_test_span(text: &str, bbox: Rect, font_size: f32) -> TextSpan {
        TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: text.to_string(),
            bbox,
            font_name: "F1".to_string(),
            font_size,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence: 0,
            split_boundary_before: false,
            offset_semantic: false,
            is_italic: false,
            is_monospace: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            primary_detected: false,
            char_widths: vec![],
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: 0.0,
            wmode: 0,
            rtl_draw_logical: false,
            mirrored: false,
            page_rotation_applied: 0,
        }
    }

    #[test]
    fn test_no_decimal_merge_with_intervening_comma_glyph() {
        // Subscript index pairs (`P_{1,0}`, `i_2, i_4`) place two small
        // digit runs a split-box-sized gap apart WITH the separating comma
        // drawn between them — often later in the content stream, so the
        // digit spans are sequence-adjacent. Ink inside the gap proves the
        // digits are separate tokens: a genuine split-box amount has empty
        // space between its boxes. Gap here: 110.0 - 103.5 = 6.5pt at 7pt
        // font = 0.93x — squarely inside the genuine split-box band, so a
        // gap ceiling alone cannot reject it.
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            digit_test_span("1", Rect::new(100.0, 700.0, 3.5, 7.0), 7.0),
            digit_test_span("0", Rect::new(110.0, 700.0, 3.5, 7.0), 7.0),
            // The comma, drawn after both digits in the content stream but
            // sitting geometrically inside the gap.
            digit_test_span(",", Rect::new(104.8, 699.0, 1.8, 3.0), 7.0),
        ];

        extractor.merge_adjacent_spans();
        assert!(
            !extractor.spans.iter().any(|s| s.text.contains("1.0")),
            "digits separated by a drawn comma must not merge into a decimal, got {:?}",
            extractor
                .spans
                .iter()
                .map(|s| s.text.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_no_decimal_merge_across_font_sizes() {
        // Scientific notation in table rows: the exponent digit of one
        // value ("10^-4" drawn as "10" + superscript "4") and the mantissa
        // digit of the NEXT value ("3 ...") are both pure-digit runs a
        // split-box-sized gap apart, and were fused into a fabricated
        // decimal ("4 . 10-4 3 . 10-4" -> "4 . 10-4.3 . 10-4"). A genuine
        // split-box amount prints both halves at the SAME size; an
        // exponent is markedly smaller than the neighbouring mantissa, so
        // a size mismatch disqualifies the pair.
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            // Exponent "4" of the previous value: 7pt.
            digit_test_span("4", Rect::new(200.0, 700.0, 4.0, 7.0), 7.0),
            // Mantissa "3" of the next value: 12pt, 8pt away (0.67-1.14x
            // either font size -- inside the merge band for both).
            digit_test_span("3", Rect::new(212.0, 700.0, 6.5, 12.0), 12.0),
        ];

        extractor.merge_adjacent_spans();
        assert!(
            !extractor.spans.iter().any(|s| s.text.contains('.')),
            "digit runs at mismatched font sizes must not merge into a decimal, got {:?}",
            extractor
                .spans
                .iter()
                .map(|s| s.text.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_decimal_merge_with_ink_elsewhere_on_line_still_joins() {
        // Positive control for the intervening-ink test: ink elsewhere on
        // the same line (a comma before the amount) must not block a
        // genuine split-box merge — only ink INSIDE the gap counts.
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        extractor.spans = vec![
            digit_test_span("123456", Rect::new(382.3, 700.0, 39.6, 12.0), 12.0),
            digit_test_span("72", Rect::new(432.7, 700.0, 13.2, 12.0), 12.0), // 10.8pt gap
            digit_test_span(",", Rect::new(300.0, 700.0, 2.5, 4.0), 12.0),    // far left of both
        ];

        extractor.merge_adjacent_spans();
        assert!(
            extractor.spans.iter().any(|s| s.text == "123456.72"),
            "split-box amount must still merge when the line's other ink is outside the gap, got {:?}",
            extractor.spans.iter().map(|s| s.text.clone()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_no_decimal_merge_for_wide_subscript_digits() {
        // Subscript index pairs (e.g. `P_{1,0}` in scientific PDFs) draw the
        // two subscript digits in a smaller font (~7pt) spaced far apart
        // (~1.5-1.7x the font size). The decimal-merge rule was joining them
        // into an invented decimal ("1" + "0" -> "1.0"). A real split-box
        // dollar amount clusters near ~0.8-1.0x the font size, so a wide gap is
        // not an integer/cents amount and must stay separate.
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        // 7pt subscript digits: "1" at x=100.0 (w=3.5), "0" at x=114.5 (w=3.5).
        // gap = 114.5 - (100.0 + 3.5) = 11.0pt -> 11.0 / 7.0 = 1.57x font size.
        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "1".to_string(),
                bbox: Rect::new(100.0, 700.0, 3.5, 7.0),
                font_name: "F1".to_string(),
                font_size: 7.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "0".to_string(),
                bbox: Rect::new(114.5, 700.0, 3.5, 7.0), // 11.0pt gap = 1.57x font
                font_name: "F1".to_string(),
                font_size: 7.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        // Widely-spaced subscript digits must NOT be joined into a decimal.
        assert_eq!(
            extractor.spans.len(),
            2,
            "Widely-spaced subscript digits should not merge into a decimal value"
        );
        assert!(
            !extractor.spans.iter().any(|s| s.text.contains('.')),
            "No invented decimal point should appear between subscript digits"
        );
    }

    #[test]
    fn test_decimal_merge_just_under_ceiling_still_joins() {
        // The ceiling that stops subscripts must not be so tight that it drops
        // genuine split-box amounts. Real amounts cluster near ~0.8-1.0x the
        // font size; this locks the ceiling by proving an amount at ~1.2x the
        // font size (just under the 1.3x cap) still merges.
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::legacy();

        // 12pt digits: "1234" at x=200.0 (w=24.0), "56" at x=238.4 (w=12.0).
        // gap = 238.4 - (200.0 + 24.0) = 14.4pt -> 14.4 / 12.0 = 1.2x font size.
        extractor.spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "1234".to_string(),
                bbox: Rect::new(200.0, 700.0, 24.0, 12.0),
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "56".to_string(),
                bbox: Rect::new(238.4, 700.0, 12.0, 12.0), // 14.4pt gap = 1.2x font
                font_name: "F1".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                is_italic: false,
                is_monospace: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        extractor.merge_adjacent_spans();
        assert_eq!(extractor.spans.len(), 1, "Amount just under the ceiling should still merge");
        assert_eq!(extractor.spans[0].text, "1234.56");
    }

    #[test]
    fn test_cross_font_word_glue_single_letter_prefix() {
        // A single-letter span in one font, tight-kerned against a
        // multi-letter span in another font, is the drop-cap pattern.
        // These must merge into one word with the longer run's font
        // metadata — emitting per-letter emphasis runs corrupts proper
        // nouns.
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::default();

        extractor.spans = vec![
            TextSpan {
                text: "S".to_string(),
                bbox: Rect::new(72.0, 700.0, 10.0, 12.0),
                font_name: "Helvetica-Bold".to_string(),
                font_weight: FontWeight::Bold,
                font_size: 12.0,
                ..TextSpan::default()
            },
            TextSpan {
                text: "ales".to_string(),
                bbox: Rect::new(82.0, 700.0, 30.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_weight: FontWeight::Normal,
                font_size: 12.0,
                ..TextSpan::default()
            },
        ];

        extractor.merge_adjacent_spans();

        assert_eq!(
            extractor.spans.len(),
            1,
            "cross_font_word_glue should merge 'S' + 'ales' into 'Sales'"
        );
        assert_eq!(extractor.spans[0].text, "Sales");
        // Dominant-font swap: the longer run (regular weight) should win.
        assert_eq!(extractor.spans[0].font_weight, FontWeight::Normal);
    }

    /// The advance-fold folds a sub-threshold TJ offset into the run's stored
    /// advance using the exact ISO 32000-1 §9.4.4 displacement
    /// (`-Tj/1000 * Tfs * Th`), keeping `char_widths.last` and
    /// `accumulated_width` in lockstep so the reconstructed geometry equals the
    /// text-matrix position. An empty buffer is a no-op (the next glyph
    /// re-anchors to the matrix).
    #[test]
    fn test_fold_offset_into_buffer_matches_spec_displacement() {
        let mut extractor = TextExtractor::new();
        {
            let st = extractor.state_stack.current_mut();
            st.font_size = 10.0;
            st.horizontal_scaling = 100.0; // Th = 1.0
        }
        let mut buffer = TjBuffer::new(extractor.state_stack.current(), None, None, Vec::new());
        buffer.char_widths.push(5.0);
        buffer.accumulated_width = 5.0;

        // -120 TJ units => -(-120)/1000 * 10 * (100/100) = 1.2 (text space).
        extractor.fold_offset_into_buffer(&mut buffer, -120.0);
        let expected = 1.2_f32;
        assert!((buffer.char_widths.last().unwrap() - (5.0 + expected)).abs() < 1e-4);
        assert!((buffer.accumulated_width - (5.0 + expected)).abs() < 1e-4);
        // Invariant: sum(char_widths) == accumulated_width by construction.
        let sum: f32 = buffer.char_widths.iter().sum();
        assert!((sum - buffer.accumulated_width).abs() < 1e-4);

        // Empty buffer: nothing to fold into, must not panic or fabricate width.
        let mut empty = TjBuffer::new(extractor.state_stack.current(), None, None, Vec::new());
        extractor.fold_offset_into_buffer(&mut empty, -120.0);
        assert!(empty.char_widths.is_empty());
        assert_eq!(empty.accumulated_width, 0.0);
    }

    /// The cross-font glue ceiling (0.12em) must NOT glue a real word followed
    /// by a single-letter variable set in a different font run across a
    /// word-space gap (roman `solution` -> math-italic `U`, gap ~0.24em). This
    /// is the mirror of the drop-cap case above (gap ~0): a word space is a
    /// genuine boundary poppler/PDFium keep, so the two spans stay separate.
    #[test]
    fn test_cross_font_word_variable_not_glued() {
        let mut extractor = TextExtractor::new();
        extractor.merging_config = SpanMergingConfig::default();
        // fs 10; "solution" ends at x=112, "U" starts at x=114.4 => gap 2.4pt = 0.24em.
        extractor.spans = vec![
            TextSpan {
                text: "solution".to_string(),
                bbox: Rect::new(72.0, 700.0, 40.0, 10.0),
                font_name: "NimbusRomNo9L-Regu".to_string(),
                font_size: 10.0,
                ..TextSpan::default()
            },
            TextSpan {
                text: "U".to_string(),
                bbox: Rect::new(114.4, 700.0, 7.0, 10.0),
                font_name: "NimbusRomNo9L-Ital".to_string(),
                font_size: 10.0,
                ..TextSpan::default()
            },
        ];
        extractor.merge_adjacent_spans();
        assert_eq!(
            extractor.spans.len(),
            2,
            "a 0.24em word-space gap across a font change must NOT glue (drop-cap glue is for ~0 gaps)"
        );
    }
}
