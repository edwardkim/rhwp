//! ISO 32000-1:2008 Section 9.4.4 Word Boundary Detection
//!
//! This module implements specification-compliant word boundary detection for PDF text extraction.
//! Word boundaries are identified through multiple mechanisms defined in the PDF specification:
//!
//! 1. **TJ Array Offsets** (Section 9.4.4): Character-level spacing information from text positioning
//! 2. **Geometric Positioning** (Section 9.4): Layout-based word breaking through character positions
//! 3. **Space Characters** (Section 5.3.2): Explicit word separators (U+0020 and variants)
//! 4. **Font Metrics** (Section 9.3): Character width, font size, and scaling adjustments
//! 5. **Script-Aware Detection**: CJK text, custom encodings, and special characters
//!
//! # Specification References
//!
//! - ISO 32000-1:2008 Section 9.4: Text Objects
//! - ISO 32000-1:2008 Section 9.4.3: Text Positioning Operators
//! - ISO 32000-1:2008 Section 9.4.4: Text Objects and Word Spacing
//! - ISO 32000-1:2008 Section 9.3: Text State Parameters (Tc, Tw, Tz, TL)
//! - ISO 32000-1:2008 Section 9.6-9.8: Font Metrics

#![forbid(unsafe_code)]

use crate::text::cjk_punctuation;
use crate::text::complex_script_detector::{
    detect_complex_script, handle_devanagari_boundary, handle_indic_boundary,
    handle_khmer_boundary, handle_thai_boundary, ComplexScript,
};
use crate::text::rtl_detector::should_split_at_rtl_boundary;
use crate::text::script_detector::{
    detect_cjk_script, handle_japanese_text, handle_korean_text, should_split_on_script_transition,
    DocumentLanguage,
};

/// Information about a character in the text stream for boundary detection.
///
/// This type captures all the information needed to determine word boundaries
/// per PDF specification Section 9.4.4.
#[derive(Clone, Debug)]
pub struct CharacterInfo {
    /// Unicode code point of the character
    pub code: u32,

    /// Glyph ID in the font (if available)
    pub glyph_id: Option<u16>,

    /// Character width in text space units (thousandths of em)
    pub width: f32,

    /// X position (horizontal) in text space
    pub x_position: f32,

    /// TJ array offset value (in thousandths of em) - negative = extra space
    /// Per spec: Negative values in TJ array increase spacing between characters
    pub tj_offset: Option<i32>,

    /// Current font size in points
    pub font_size: f32,

    /// Whether this character is a ligature (U+FB00-U+FB04)
    pub is_ligature: bool,

    /// Original ligature character if this was split from a ligature
    /// Used for debugging and tracking ligature expansion
    pub original_ligature: Option<char>,

    /// Whether this character is protected from word boundary splitting
    ///
    /// When true, word boundary detection will skip creating boundaries
    /// before or after this character. Used to preserve email addresses
    /// (`user@example.com`) and URLs (`http://example.com`) as single tokens.
    pub protected_from_split: bool,
}

/// Context information for word boundary detection.
///
/// Provides the font metrics and text state parameters that influence
/// how word boundaries are determined (per Section 9.3).
#[derive(Clone, Debug)]
pub struct BoundaryContext {
    /// Font size (Tf parameter in text state)
    pub font_size: f32,

    /// Horizontal scaling percentage (Tz parameter, default 100.0)
    pub horizontal_scaling: f32,

    /// Word spacing adjustment (Tw parameter, added after space character)
    pub word_spacing: f32,

    /// Character spacing adjustment (Tc parameter, added after every character)
    pub char_spacing: f32,
}

impl BoundaryContext {
    /// Create a new boundary context with default text state parameters.
    pub fn new(font_size: f32) -> Self {
        Self {
            font_size,
            horizontal_scaling: 100.0,
            word_spacing: 0.0,
            char_spacing: 0.0,
        }
    }

    /// Get the effective font size accounting for horizontal scaling
    fn effective_font_size(&self) -> f32 {
        self.font_size * (self.horizontal_scaling / 100.0)
    }
}

/// Document script profile for optimization.
///
/// OPTIMIZATION (Issue #1 fix): Detect document primary script once,
/// then skip unnecessary script detection functions for faster boundary detection.
///
/// When documents contain only Latin text, we skip RTL and CJK detection entirely.
/// When documents are CJK-dominant, we skip RTL detection.
/// This reduces function call overhead from millions per batch to thousands.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentScript {
    /// Latin-only document (ASCII + extended Latin)
    /// Fast path: only check space, TJ offset, geometric gap
    Latin,

    /// CJK-dominant document (Chinese, Japanese, Korean)
    /// Skip RTL detection, use optimized CJK path
    CJK,

    /// Right-to-left dominant (Arabic, Hebrew)
    /// Skip CJK detection, use optimized RTL path
    RTL,

    /// Complex scripts (Devanagari, Thai, Khmer, etc.)
    /// Use specialized complex script detection
    Complex,

    /// Mixed scripts or unknown
    /// Check all detection functions (slowest path)
    Mixed,
}

impl DocumentScript {
    /// Detect document script profile by sampling first 1000 characters.
    ///
    /// This optimization reduces boundary detection overhead by skipping
    /// unnecessary script detection for documents with known script profiles.
    ///
    /// PERFORMANCE: O(min(n, 1000)) sampling, executed once per extraction
    pub fn detect_from_characters(characters: &[CharacterInfo]) -> Self {
        if characters.is_empty() {
            return Self::Latin; // Default to Latin for empty documents
        }

        let mut has_rtl = false;
        let mut has_cjk = false;
        let mut has_complex = false;
        let sample_size = characters.len().min(1000);

        // Sample first 1000 characters to classify document
        for ch in &characters[..sample_size] {
            // Check for RTL (fast range check)
            if (0x0590..=0x08FF).contains(&ch.code) || (0xFB1D..=0xFDFF).contains(&ch.code) {
                has_rtl = true;
            }

            // Check for CJK (fast range checks - common ranges first)
            if (0x4E00..=0x9FFF).contains(&ch.code) // Han
                || (0x3040..=0x309F).contains(&ch.code) // Hiragana
                || (0x30A0..=0x30FF).contains(&ch.code) // Katakana
                || (0xAC00..=0xD7AF).contains(&ch.code)
            {
                // Hangul
                has_cjk = true;
            }

            // Check for complex scripts. The Brahmic South-Asian blocks
            // (Bengali, Tamil, Telugu, Kannada, Malayalam) were previously
            // absent, so those docs classified as Latin/Mixed and never reached
            // the complex-script boundary rules — leaking spurious spaces after
            // matras (#656-class Indic gap). They share the same matra/virama
            // boundary semantics as Devanagari.
            if (0x0900..=0x097F).contains(&ch.code) // Devanagari
                || (0x0980..=0x09FF).contains(&ch.code) // Bengali
                || (0x0B80..=0x0BFF).contains(&ch.code) // Tamil
                || (0x0C00..=0x0C7F).contains(&ch.code) // Telugu
                || (0x0C80..=0x0CFF).contains(&ch.code) // Kannada
                || (0x0D00..=0x0D7F).contains(&ch.code) // Malayalam
                || (0x0E00..=0x0E7F).contains(&ch.code) // Thai
                || (0x1780..=0x17FF).contains(&ch.code)
            {
                // Khmer
                has_complex = true;
            }
        }

        // Decision tree: classify based on what we found
        #[allow(clippy::let_and_return)]
        let script = match (has_rtl, has_cjk, has_complex) {
            (false, false, false) => Self::Latin, // Pure Latin (fast path)
            (false, true, _) => Self::CJK,        // CJK-dominant (skip RTL)
            (true, false, _) => Self::RTL,        // RTL-dominant (skip CJK)
            (_, _, true) => Self::Complex,        // Complex scripts present
            _ => Self::Mixed,                     // Mixed scripts
        };

        // Log detected script at TRACE level for debugging
        crate::extract_log_trace!(
            "Detected document script: {:?} (sampled {} characters)",
            script,
            sample_size
        );

        script
    }
}

/// Main word boundary detection engine.
///
/// Implements the specification-compliant word boundary detection algorithm
/// that considers TJ offsets, geometric spacing, and font metrics.
#[derive(Debug)]
pub struct WordBoundaryDetector {
    /// Threshold for TJ offset values that indicate word boundaries
    /// Default: -100 (representing 0.1em in thousand-units of em)
    tj_offset_threshold: i32,

    /// Ratio of font size to use as geometric gap threshold
    /// Default: 0.3 (30% of font size indicates a word boundary)
    geometric_gap_ratio: f32,

    /// Enable CJK-aware boundary detection
    cjk_enabled: bool,

    /// Enable script-aware transition detection
    detect_script_transitions: bool,

    /// Document language context (if known)
    document_language: Option<DocumentLanguage>,

    /// Detected document script profile (Issue #1 optimization)
    /// Cached at detector creation to skip unnecessary detection functions
    primary_script: DocumentScript,

    /// Enable adaptive TJ threshold calculation based on font metrics
    /// When true, uses calculate_tj_threshold() instead of static tj_offset_threshold
    /// Default: true (adaptive mode enabled)
    use_adaptive_threshold: bool,
}

impl Default for WordBoundaryDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl WordBoundaryDetector {
    /// Create a new word boundary detector with default settings.
    pub fn new() -> Self {
        Self {
            tj_offset_threshold: -100,
            // Geometric gap threshold: 80% of font size
            // This is conservative enough to avoid false positives from normal character spacing
            // but sensitive enough to detect actual word breaks
            geometric_gap_ratio: 0.8,
            cjk_enabled: true,
            detect_script_transitions: true,
            document_language: None,
            primary_script: DocumentScript::Mixed, // Default to Mixed, will be set by caller
            use_adaptive_threshold: true,          // Enable adaptive threshold by default
        }
    }

    /// Set the TJ offset threshold for boundary detection.
    ///
    /// Negative values in TJ arrays that are more negative than this threshold
    /// are considered word boundaries. Default: -100
    pub fn with_tj_threshold(mut self, threshold: i32) -> Self {
        self.tj_offset_threshold = threshold;
        self
    }

    /// Set the geometric gap ratio as a fraction of font size.
    ///
    /// Gaps between characters larger than (font_size * ratio) are considered
    /// word boundaries. Default: 0.3
    pub fn with_geometric_gap_ratio(mut self, ratio: f32) -> Self {
        self.geometric_gap_ratio = ratio;
        self
    }

    /// Enable or disable CJK-aware word boundary detection.
    pub fn with_cjk_enabled(mut self, enabled: bool) -> Self {
        self.cjk_enabled = enabled;
        self
    }

    /// Enable or disable script-aware transition detection.
    ///
    /// When enabled, the detector will analyze script transitions (e.g., Hiragana→Katakana)
    /// and apply language-specific rules for word boundaries.
    pub fn with_script_detection(mut self, enabled: bool) -> Self {
        self.detect_script_transitions = enabled;
        self
    }

    /// Set the document language context.
    ///
    /// This helps apply appropriate script transition rules:
    /// - Japanese: Allow Han↔Kana transitions
    /// - Korean: Allow Hangul↔Hanja transitions
    /// - Chinese: Use conservative Han character boundaries
    pub fn with_document_language(mut self, lang: DocumentLanguage) -> Self {
        self.document_language = Some(lang);
        self
    }

    /// Set the document script profile (Issue #1 optimization).
    ///
    /// When set, the detector will skip unnecessary script detection functions
    /// for documents with known script profiles, significantly improving performance.
    pub fn with_document_script(mut self, script: DocumentScript) -> Self {
        self.primary_script = script;
        self
    }

    /// Enable or disable adaptive TJ threshold calculation.
    ///
    /// When enabled (default), TJ offset thresholds are calculated dynamically
    /// based on font metrics (size, scaling, spacing). When disabled, uses the
    /// static threshold set via `with_tj_threshold()`.
    ///
    /// Adaptive mode provides better accuracy across documents with varying
    /// font sizes and text state parameters.
    pub fn with_adaptive_threshold(mut self, enabled: bool) -> Self {
        self.use_adaptive_threshold = enabled;
        self
    }

    /// Calculate adaptive TJ threshold based on font metrics and text state.
    ///
    /// Per PDF Spec Section 9.3, TJ array offsets depend on:
    /// - Font size (Tf): Larger fonts need larger thresholds
    /// - Character spacing (Tc): Manual spacing offsets base threshold
    /// - Word spacing (Tw): Applied after space characters
    /// - Horizontal scaling (Tz): Affects text width calculations
    ///
    /// Formula: base_threshold = -font_size * (h_scale / 100.0) * 0.025
    /// Then adjust by: -(char_spacing.abs() + word_spacing.abs()) * 0.5
    fn calculate_tj_threshold(&self, context: &BoundaryContext) -> f32 {
        let font_size = context.font_size.max(1.0);
        let h_scale = (context.horizontal_scaling / 100.0).max(0.01);

        // Base threshold as percentage of font size (2.5%)
        // 12pt font → -0.3, 24pt font → -0.6
        let base_threshold = -font_size * h_scale * 0.025;

        // Adjust for explicit spacing parameters
        let spacing_adjustment = (context.char_spacing.abs() + context.word_spacing.abs()) * 0.5;

        base_threshold - spacing_adjustment
    }

    /// Detect word boundaries in a character stream.
    ///
    /// Returns a vector of indices where word boundaries occur.
    /// A boundary at index `i` means there is a word break between
    /// characters at indices `i-1` and `i`.
    ///
    /// Per ISO 32000-1:2008 Section 9.4.4, word boundaries are determined by:
    /// 1. Space characters (U+0020, U+200B)
    /// 2. TJ array offset signals (negative values below threshold)
    /// 3. Geometric gaps exceeding font-size relative threshold
    /// 4. CJK character transitions
    ///
    /// # Arguments
    ///
    /// * `characters` - Sequence of characters with positioning information
    /// * `context` - Font metrics and text state parameters
    ///
    /// # Returns
    ///
    /// Vector of indices where word boundaries occur (between characters)
    pub fn detect_word_boundaries(
        &self,
        characters: &[CharacterInfo],
        context: &BoundaryContext,
    ) -> Vec<usize> {
        if characters.is_empty() {
            return Vec::new();
        }

        let mut boundaries = Vec::new();

        for i in 1..characters.len() {
            let prev_char = &characters[i - 1];
            let curr_char = &characters[i];

            if self.is_word_boundary(prev_char, curr_char, context) {
                boundaries.push(i);
            }
        }

        // Log boundary count at TRACE level for debugging word boundary detection
        crate::extract_log_trace!(
            "Word boundary detection: {} boundaries in {} characters",
            boundaries.len(),
            characters.len()
        );

        boundaries
    }

    /// Determine if a word boundary exists between two consecutive characters.
    ///
    /// Implements the specification rules per ISO 32000-1:2008 Section 9.4.4:
    ///
    /// 1. **Space characters** (U+0020, U+200B): Always create boundaries
    /// 2. **TJ array offsets**: Negative values below threshold indicate spacing
    /// 3. **Geometric gaps**: Gaps larger than font-size-relative threshold
    /// 4. **CJK script transitions**: Script-aware word boundaries
    /// 5. **CJK characters**: Each non-punctuation CJK character creates boundary (legacy)
    ///
    /// # Arguments
    ///
    /// * `prev_char` - Previous character in the stream
    /// * `curr_char` - Current character
    /// * `context` - Font metrics and text state
    ///
    /// # Returns
    ///
    /// `true` if a word boundary should be placed between these characters
    fn is_word_boundary(
        &self,
        prev_char: &CharacterInfo,
        curr_char: &CharacterInfo,
        context: &BoundaryContext,
    ) -> bool {
        // Skip boundaries in protected contexts (emails, URLs)
        if prev_char.protected_from_split || curr_char.protected_from_split {
            return false;
        }

        // Rule 1: ASCII space (U+0020) or zero-width space (U+200B)
        if prev_char.code == 0x20 || prev_char.code == 0x200B {
            return true;
        }

        // OPTIMIZATION (Issue #1): Use script-aware dispatch to avoid unnecessary function calls
        // This reduces millions of function calls per batch by skipping detection for known script types
        match self.primary_script {
            // Fast path: Latin-only documents - skip RTL and CJK detection entirely
            DocumentScript::Latin => self.is_word_boundary_basic(prev_char, curr_char, context),

            // CJK path: Skip RTL detection, use only CJK detection
            DocumentScript::CJK => {
                if self.detect_script_transitions {
                    if let Some(decision) = self.should_split_at_cjk_boundary(prev_char, curr_char)
                    {
                        return decision;
                    }
                }
                self.is_word_boundary_basic(prev_char, curr_char, context)
            },

            // RTL path: Skip CJK detection, use only RTL detection
            DocumentScript::RTL => {
                if let Some(decision) =
                    should_split_at_rtl_boundary(prev_char, curr_char, Some(context))
                {
                    return decision;
                }
                self.is_word_boundary_basic(prev_char, curr_char, context)
            },

            // Complex script path: Use complex script detection, skip RTL/CJK
            DocumentScript::Complex => {
                if let Some(decision) =
                    self.should_split_at_complex_script_boundary(prev_char, curr_char)
                {
                    return decision;
                }
                self.is_word_boundary_basic(prev_char, curr_char, context)
            },

            // Mixed path: Check all detection functions (original behavior)
            DocumentScript::Mixed => {
                // RTL (Arabic/Hebrew) boundary detection
                if let Some(decision) =
                    should_split_at_rtl_boundary(prev_char, curr_char, Some(context))
                {
                    return decision;
                }

                // CJK script-aware boundaries
                if self.detect_script_transitions {
                    if let Some(decision) = self.should_split_at_cjk_boundary(prev_char, curr_char)
                    {
                        return decision;
                    }
                }

                // Complex script boundary detection
                if let Some(decision) =
                    self.should_split_at_complex_script_boundary(prev_char, curr_char)
                {
                    return decision;
                }

                self.is_word_boundary_basic(prev_char, curr_char, context)
            },
        }
    }

    /// Basic boundary detection used by all script paths.
    ///
    /// This contains the core TJ offset and geometric gap checks
    /// that apply to all scripts.
    fn is_word_boundary_basic(
        &self,
        prev_char: &CharacterInfo,
        curr_char: &CharacterInfo,
        context: &BoundaryContext,
    ) -> bool {
        // Rule 2: TJ array offset signals explicit spacing (adaptive threshold)
        if let Some(tj_offset) = prev_char.tj_offset {
            let threshold = if self.use_adaptive_threshold {
                self.calculate_tj_threshold(context)
            } else {
                self.tj_offset_threshold as f32
            };
            if (tj_offset as f32) < threshold {
                return true;
            }
        }

        // Rule 3: Geometric spacing detection
        if self.has_significant_geometric_gap(prev_char, curr_char, context) {
            return true;
        }

        // Rule 4: CJK character boundaries (legacy, if enabled but script detection disabled)
        if self.cjk_enabled
            && !self.detect_script_transitions
            && self.is_cjk_character(prev_char.code)
            && !self.is_cjk_punctuation(prev_char.code)
        {
            return true;
        }

        false
    }

    /// Determine if a complex script boundary should be created.
    ///
    /// This implements Complex Script support:
    /// - Devanagari virama and matras
    /// - Thai tone marks and vowel modifiers
    /// - Khmer COENG and vowels
    /// - Indic scripts (Tamil, Telugu, Kannada, Malayalam) diacritics
    ///
    /// # Arguments
    ///
    /// * `prev_char` - Previous character information
    /// * `curr_char` - Current character information
    ///
    /// # Returns
    ///
    /// - `Some(true)` - Must create boundary
    /// - `Some(false)` - Must not create boundary
    /// - `None` - Use other signals (TJ offset, geometry)
    fn should_split_at_complex_script_boundary(
        &self,
        prev_char: &CharacterInfo,
        curr_char: &CharacterInfo,
    ) -> Option<bool> {
        let prev_script = detect_complex_script(prev_char.code);
        let curr_script = detect_complex_script(curr_char.code);

        // If neither is complex script, not our concern
        if prev_script.is_none() && curr_script.is_none() {
            return None;
        }

        // Apply script-specific rules based on which scripts are involved
        match (prev_script, curr_script) {
            // Devanagari boundaries
            (Some(ComplexScript::Devanagari), _) | (_, Some(ComplexScript::Devanagari)) => {
                handle_devanagari_boundary(prev_char, curr_char)
            },
            // Thai boundaries
            (Some(ComplexScript::Thai), _) | (_, Some(ComplexScript::Thai)) => {
                handle_thai_boundary(prev_char, curr_char)
            },
            // Khmer boundaries
            (Some(ComplexScript::Khmer), _) | (_, Some(ComplexScript::Khmer)) => {
                handle_khmer_boundary(prev_char, curr_char)
            },
            // South Asian Indic scripts (Tamil, Telugu, Kannada, Malayalam)
            (Some(ComplexScript::Tamil), _)
            | (_, Some(ComplexScript::Tamil))
            | (Some(ComplexScript::Telugu), _)
            | (_, Some(ComplexScript::Telugu))
            | (Some(ComplexScript::Kannada), _)
            | (_, Some(ComplexScript::Kannada))
            | (Some(ComplexScript::Malayalam), _)
            | (_, Some(ComplexScript::Malayalam))
            | (Some(ComplexScript::Bengali), _)
            | (_, Some(ComplexScript::Bengali)) => handle_indic_boundary(prev_char, curr_char),
            // Other complex scripts - use conservative default (let other signals decide)
            _ => None,
        }
    }

    /// Determine if a CJK boundary should be created based on script analysis.
    ///
    /// This implements CJK script support:
    /// - CJK punctuation detection
    /// - Script type detection
    /// - Language-specific transition rules
    /// - Japanese modifier handling
    ///
    /// # Arguments
    ///
    /// * `prev_char` - Previous character information
    /// * `curr_char` - Current character information
    ///
    /// # Returns
    ///
    /// - `Some(true)` - Must create boundary
    /// - `Some(false)` - Must not create boundary
    /// - `None` - Use other signals (TJ offset, geometry)
    fn should_split_at_cjk_boundary(
        &self,
        prev_char: &CharacterInfo,
        curr_char: &CharacterInfo,
    ) -> Option<bool> {
        // Check CJK punctuation (always creates boundary with high confidence)
        // Note: Using None for density to maintain current behavior
        // Future: Could integrate document-wide density measurement here
        let prev_punctuation_score =
            cjk_punctuation::get_cjk_punctuation_boundary_score(prev_char.code, None);
        if prev_punctuation_score >= 0.9 {
            // Sentence-ending and enumeration punctuation create boundaries
            return Some(true);
        }

        // Detect scripts for both characters
        let prev_script = detect_cjk_script(prev_char.code);
        let curr_script = detect_cjk_script(curr_char.code);

        // If neither character is CJK, not our concern
        if prev_script.is_none() && curr_script.is_none() {
            return None;
        }

        // Apply language-specific rules
        match self.document_language {
            Some(DocumentLanguage::Japanese) => {
                handle_japanese_text(prev_char, curr_char, prev_script, curr_script)
            },
            Some(DocumentLanguage::Korean) => {
                handle_korean_text(prev_char, curr_char, prev_script, curr_script)
            },
            Some(DocumentLanguage::Chinese) | None => {
                // Chinese or unknown: use script transition analysis
                should_split_on_script_transition(prev_script, curr_script, self.document_language)
            },
        }
    }

    /// Check if a gap is internal to a ligature expansion.
    ///
    /// When a ligature like 'fi' (U+FB01) is expanded into 'f' + 'i',
    /// the geometric gap between expanded components should not create a word boundary.
    fn is_ligature_internal_gap(
        &self,
        prev_char: &CharacterInfo,
        curr_char: &CharacterInfo,
    ) -> bool {
        // Ligature Unicode range: U+FB00-U+FB06
        const LIGATURES: [u32; 7] = [0xFB00, 0xFB01, 0xFB02, 0xFB03, 0xFB04, 0xFB05, 0xFB06];

        // Check if either character is a ligature or was expanded from one
        LIGATURES.contains(&prev_char.code)
            || prev_char.is_ligature
            || LIGATURES.contains(&curr_char.code)
            || curr_char.is_ligature
    }

    /// Check if a character code represents punctuation that attaches to words.
    ///
    /// Punctuation like periods, commas, colons should use reduced threshold
    /// to avoid creating unwanted boundaries when appearing after words.
    pub fn is_punctuation(code: u32) -> bool {
        matches!(
            code,
            // ASCII punctuation
            0x21     // ! EXCLAMATION MARK
            | 0x22   // " QUOTATION MARK
            | 0x27   // ' APOSTROPHE
            | 0x2C   // , COMMA
            | 0x2E   // . FULL STOP
            | 0x3A   // : COLON
            | 0x3B   // ; SEMICOLON
            | 0x3F   // ? QUESTION MARK
            // Unicode quotation marks
            | 0x2018..=0x201F  // General Punctuation: quotes, dashes, etc.
            // Unicode dashes and hyphens
            | 0x2010..=0x2015  // Hyphen, dash variants
        )
    }

    /// Check if there is a significant geometric gap between two characters.
    ///
    /// Per Section 9.4, character positions and widths determine visual spacing.
    /// A gap larger than the threshold (font_size * ratio) indicates a word boundary.
    ///
    /// Special cases:
    /// 1. **Ligature internal gaps**: Gaps inside expanded ligatures never create boundaries
    /// 2. **Punctuation attachment**: Punctuation uses 50% threshold to attach to preceding words
    /// 3. **Character spacing**: Tc parameter adjusts baseline gap calculation
    fn has_significant_geometric_gap(
        &self,
        prev_char: &CharacterInfo,
        curr_char: &CharacterInfo,
        context: &BoundaryContext,
    ) -> bool {
        // Special case 1: Ligatures - gaps inside ligature expansions are NOT boundaries
        if self.is_ligature_internal_gap(prev_char, curr_char) {
            return false;
        }

        // Calculate the expected end position of previous character
        let prev_end_x = prev_char.x_position + prev_char.width;

        // Calculate raw gap between characters
        let raw_gap = curr_char.x_position - prev_end_x;

        // Adjust for character spacing (Tc parameter)
        // Tc is added after every character, so subtract it from the gap
        let adjusted_gap = raw_gap - context.char_spacing;

        // Base threshold is relative to font size (accounting for horizontal scaling)
        let base_threshold = context.effective_font_size() * self.geometric_gap_ratio;

        // Special case 2: Punctuation - use reduced threshold (50% of normal)
        // This keeps punctuation attached to the preceding word
        if Self::is_punctuation(curr_char.code) {
            return adjusted_gap > (base_threshold * 0.5);
        }

        // Normal case: full threshold
        adjusted_gap > base_threshold
    }

    /// Check if a character code represents a CJK (Chinese/Japanese/Korean) character.
    ///
    /// CJK Unicode ranges per Unicode Standard:
    /// - CJK Unified Ideographs: U+4E00-U+9FFF
    /// - CJK Unified Ideographs Extension A: U+3400-U+4DBF
    /// - CJK Unified Ideographs Extension B and beyond: higher ranges
    /// - Hiragana: U+3040-U+309F
    /// - Katakana: U+30A0-U+30FF
    fn is_cjk_character(&self, code: u32) -> bool {
        matches!(
            code,
            0x3040..=0x309F   // Hiragana
            | 0x30A0..=0x30FF // Katakana
            | 0x3400..=0x4DBF // CJK Unified Ideographs Extension A
            | 0x4E00..=0x9FFF // CJK Unified Ideographs
            | 0x20000..=0x2A6DF // CJK Unified Ideographs Extension B
            | 0x2A700..=0x2B73F // CJK Unified Ideographs Extension C
            | 0x2B740..=0x2B81F // CJK Unified Ideographs Extension D
            | 0x2B820..=0x2CEAF // CJK Unified Ideographs Extension E
            | 0x2CEB0..=0x2EBEF // CJK Unified Ideographs Extension F
        )
    }

    /// Check if a character is CJK punctuation that attaches to words.
    ///
    /// CJK punctuation like ideographic commas and periods attach to the preceding
    /// word and should not create boundaries.
    fn is_cjk_punctuation(&self, code: u32) -> bool {
        matches!(
            code,
            0x3001 // IDEOGRAPHIC COMMA
            | 0x3002 // IDEOGRAPHIC FULL STOP
            | 0x3008 // LEFT ANGLE BRACKET
            | 0x3009 // RIGHT ANGLE BRACKET
            | 0x300A // LEFT DOUBLE ANGLE BRACKET
            | 0x300B // RIGHT DOUBLE ANGLE BRACKET
            | 0x300C // LEFT CORNER BRACKET
            | 0x300D // RIGHT CORNER BRACKET
            | 0x300E // LEFT WHITE CORNER BRACKET
            | 0x300F // RIGHT WHITE CORNER BRACKET
            | 0x3010 // LEFT BLACK LENTICULAR BRACKET
            | 0x3011 // RIGHT BLACK LENTICULAR BRACKET
            | 0x3014 // LEFT TORTOISE SHELL BRACKET
            | 0x3015 // RIGHT TORTOISE SHELL BRACKET
        )
    }
}

/// Detect word boundaries in a character stream.
///
/// This is a convenience function that creates a detector with default settings
/// and performs boundary detection in one call.
///
/// # Arguments
///
/// * `characters` - Sequence of characters with positioning information
/// * `context` - Font metrics and text state parameters
///
/// # Returns
///
/// Vector of indices where word boundaries occur
pub fn detect_word_boundaries(
    characters: &[CharacterInfo],
    context: &BoundaryContext,
) -> Vec<usize> {
    let detector = WordBoundaryDetector::new();
    detector.detect_word_boundaries(characters, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_space_detection() {
        let characters = vec![
            CharacterInfo {
                code: 0x48,
                glyph_id: Some(1),
                width: 0.5,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'H'
            CharacterInfo {
                code: 0x65,
                glyph_id: Some(2),
                width: 0.4,
                x_position: 6.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'e'
            CharacterInfo {
                code: 0x20,
                glyph_id: Some(5),
                width: 0.25,
                x_position: 10.8,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // SPACE
            CharacterInfo {
                code: 0x57,
                glyph_id: Some(6),
                width: 0.7,
                x_position: 16.2,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'W'
        ];

        let context = BoundaryContext::new(12.0);
        let boundaries = WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // Space character at index 2 creates boundary at index 3
        assert!(boundaries.contains(&3));
    }

    #[test]
    fn test_tj_offset_threshold() {
        let characters = vec![
            CharacterInfo {
                code: 0x54,
                glyph_id: Some(1),
                width: 0.5,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'T'
            CharacterInfo {
                code: 0x2D,
                glyph_id: Some(5),
                width: 0.25,
                x_position: 6.0,
                tj_offset: Some(-200),
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // '-' with large negative offset
            CharacterInfo {
                code: 0x6F,
                glyph_id: Some(6),
                width: 0.4,
                x_position: 18.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'o'
        ];

        let context = BoundaryContext::new(12.0);
        let boundaries = WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // TJ offset at index 1 creates boundary at index 2
        assert!(boundaries.contains(&2));
    }

    #[test]
    fn test_geometric_gap_detection() {
        let characters = vec![
            CharacterInfo {
                code: 0x54,
                glyph_id: Some(1),
                width: 0.5,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'T'
            CharacterInfo {
                code: 0x65,
                glyph_id: Some(2),
                width: 0.4,
                x_position: 6.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'e'
            CharacterInfo {
                code: 0x78,
                glyph_id: Some(3),
                width: 0.4,
                x_position: 10.8,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'x'
            CharacterInfo {
                code: 0x74,
                glyph_id: Some(4),
                width: 0.3,
                x_position: 15.6,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 't'
            // Gap of ~11.1 units (much larger than threshold ~3.6)
            CharacterInfo {
                code: 0x42,
                glyph_id: Some(5),
                width: 0.5,
                x_position: 27.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'B'
        ];

        let context = BoundaryContext::new(12.0);
        let boundaries = WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // Gap between 't' (ends at 15.9) and 'B' (at 27.0) is 11.1 units > threshold (3.6)
        // This creates a boundary at index 4 (the 'B' character)
        assert!(boundaries.contains(&4), "Expected boundary at index 4, got: {:?}", boundaries);
    }

    #[test]
    fn test_cjk_character_boundaries() {
        let characters = vec![
            CharacterInfo {
                code: 0x4E2D,
                glyph_id: Some(1),
                width: 1.0,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // CJK UNIFIED IDEOGRAPH
            CharacterInfo {
                code: 0x6587,
                glyph_id: Some(2),
                width: 1.0,
                x_position: 12.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // CJK UNIFIED IDEOGRAPH
            CharacterInfo {
                code: 0x5B57,
                glyph_id: Some(3),
                width: 1.0,
                x_position: 24.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // CJK UNIFIED IDEOGRAPH
        ];

        let context = BoundaryContext::new(12.0);
        let boundaries = WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // Each CJK character creates a boundary after it
        // Character 0 -> boundary at 1, Character 1 -> boundary at 2
        assert!(boundaries.contains(&1), "Expected boundary at index 1");
        assert!(boundaries.contains(&2), "Expected boundary at index 2");
    }

    #[test]
    fn test_zero_width_space() {
        let characters = vec![
            CharacterInfo {
                code: 0x6E,
                glyph_id: Some(1),
                width: 0.4,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'n'
            CharacterInfo {
                code: 0x200B,
                glyph_id: Some(2),
                width: 0.0,
                x_position: 4.8,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // ZERO WIDTH SPACE
            CharacterInfo {
                code: 0x72,
                glyph_id: Some(3),
                width: 0.3,
                x_position: 4.8,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'r'
        ];

        let context = BoundaryContext::new(12.0);
        let boundaries = WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // Zero-width space creates boundary
        assert!(boundaries.contains(&2));
    }

    #[test]
    fn test_horizontal_scaling_affects_gap_threshold() {
        // Create a gap that's on the threshold boundary
        // Gap = 7.5 units
        // At 100% scaling (font size 12): threshold = 12 * 0.8 = 9.6, gap < threshold = no boundary
        // At 75% scaling (font size 9): threshold = 9 * 0.8 = 7.2, gap > threshold = boundary!
        let characters = vec![
            CharacterInfo {
                code: 0x41,
                glyph_id: Some(1),
                width: 0.5,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'A' ends at 0.5
            CharacterInfo {
                code: 0x42,
                glyph_id: Some(2),
                width: 0.5,
                x_position: 8.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'B' starts at 8.0
        ];

        // With 100% scaling, gap (7.5) doesn't exceed threshold (9.6)
        let mut context = BoundaryContext::new(12.0);
        context.horizontal_scaling = 100.0;
        let boundaries_normal =
            WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // With 75% scaling, gap (7.5) exceeds threshold (7.2)
        context.horizontal_scaling = 75.0;
        let boundaries_scaled =
            WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // Scaling affects the effective threshold, so results should differ
        // Normal: no boundary, Scaled: boundary at index 1
        assert!(boundaries_normal.is_empty(), "Should have no boundaries at 100% scaling");
        assert!(boundaries_scaled.contains(&1), "Should have boundary at 75% scaling");
    }

    #[test]
    fn test_detect_word_boundaries_ascii_space() {
        // Test that ASCII space creates word boundary
        let characters = vec![
            CharacterInfo {
                code: 0x48,
                glyph_id: Some(1),
                width: 0.5,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'H'
            CharacterInfo {
                code: 0x65,
                glyph_id: Some(2),
                width: 0.4,
                x_position: 6.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'e'
            CharacterInfo {
                code: 0x20,
                glyph_id: Some(5),
                width: 0.25,
                x_position: 10.8,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // SPACE
            CharacterInfo {
                code: 0x57,
                glyph_id: Some(6),
                width: 0.7,
                x_position: 16.2,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'W'
        ];

        let context = BoundaryContext::new(12.0);
        let boundaries = WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // Space character at index 2 creates boundary at index 3
        assert!(boundaries.contains(&3), "Should have boundary after space");
    }

    #[test]
    fn test_detect_word_boundaries_tj_offset() {
        // Test that large negative TJ offset creates boundary
        let characters = vec![
            CharacterInfo {
                code: 0x54,
                glyph_id: Some(1),
                width: 0.5,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'T'
            CharacterInfo {
                code: 0x2D,
                glyph_id: Some(5),
                width: 0.25,
                x_position: 6.0,
                tj_offset: Some(-200),
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // '-' with large negative offset
            CharacterInfo {
                code: 0x6F,
                glyph_id: Some(6),
                width: 0.4,
                x_position: 18.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // 'o'
        ];

        let context = BoundaryContext::new(12.0);
        let boundaries = WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // TJ offset at index 1 creates boundary at index 2
        assert!(boundaries.contains(&2), "Should have boundary after large TJ offset");
    }

    #[test]
    fn test_detect_word_boundaries_cjk() {
        // Test that CJK characters create boundaries
        let characters = vec![
            CharacterInfo {
                code: 0x4E2D,
                glyph_id: Some(1),
                width: 1.0,
                x_position: 0.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // CJK character
            CharacterInfo {
                code: 0x6587,
                glyph_id: Some(2),
                width: 1.0,
                x_position: 12.0,
                tj_offset: None,
                font_size: 12.0,
                is_ligature: false,
                original_ligature: None,
                protected_from_split: false,
            }, // CJK character
        ];

        let context = BoundaryContext::new(12.0);
        let boundaries = WordBoundaryDetector::new().detect_word_boundaries(&characters, &context);

        // Each CJK character should create a boundary
        assert!(boundaries.contains(&1), "Should have boundary after first CJK character");
    }

    #[test]
    fn test_calculate_tj_threshold_default_font() {
        let detector = WordBoundaryDetector::new();
        let context = BoundaryContext::new(12.0); // 12pt
        let threshold = detector.calculate_tj_threshold(&context);
        // Expected: -12.0 * 1.0 * 0.025 = -0.3
        assert!(
            (threshold - (-0.3)).abs() < 0.01,
            "12pt font should give -0.3, got {}",
            threshold
        );
    }

    #[test]
    fn test_calculate_tj_threshold_large_font() {
        let detector = WordBoundaryDetector::new();
        let context = BoundaryContext::new(24.0); // 24pt
        let threshold = detector.calculate_tj_threshold(&context);
        // Expected: -24.0 * 1.0 * 0.025 = -0.6
        assert!(
            (threshold - (-0.6)).abs() < 0.01,
            "24pt font should give -0.6, got {}",
            threshold
        );
    }

    #[test]
    fn test_calculate_tj_threshold_with_char_spacing() {
        let detector = WordBoundaryDetector::new();
        let mut context = BoundaryContext::new(12.0);
        context.char_spacing = 2.0;
        let threshold = detector.calculate_tj_threshold(&context);
        // Expected: -0.3 - (2.0 * 0.5) = -1.3
        assert!(
            (threshold - (-1.3)).abs() < 0.01,
            "With char_spacing=2.0, expected -1.3, got {}",
            threshold
        );
    }

    #[test]
    fn test_calculate_tj_threshold_with_word_spacing() {
        let detector = WordBoundaryDetector::new();
        let mut context = BoundaryContext::new(12.0);
        context.word_spacing = 3.0;
        let threshold = detector.calculate_tj_threshold(&context);
        // Expected: -0.3 - (3.0 * 0.5) = -1.8
        assert!(
            (threshold - (-1.8)).abs() < 0.01,
            "With word_spacing=3.0, expected -1.8, got {}",
            threshold
        );
    }

    #[test]
    fn test_calculate_tj_threshold_with_horizontal_scaling() {
        let detector = WordBoundaryDetector::new();
        let mut context = BoundaryContext::new(12.0);
        context.horizontal_scaling = 80.0; // 80%
        let threshold = detector.calculate_tj_threshold(&context);
        // Expected: -12.0 * 0.8 * 0.025 = -0.24
        assert!(
            (threshold - (-0.24)).abs() < 0.01,
            "With 80% scaling, expected -0.24, got {}",
            threshold
        );
    }

    #[test]
    fn test_adaptive_threshold_affects_boundary_detection() {
        let detector = WordBoundaryDetector::new().with_adaptive_threshold(true);
        let context = BoundaryContext::new(12.0);

        let prev = CharacterInfo {
            code: 't' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 100.0,
            tj_offset: Some(-200), // Significant negative offset
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        let curr = CharacterInfo {
            code: 'h' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 110.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        let boundary = detector.is_word_boundary(&prev, &curr, &context);
        assert!(boundary, "TJ offset -200 should trigger boundary with 12pt font");
    }

    #[test]
    fn test_disable_adaptive_threshold_uses_static() {
        let detector = WordBoundaryDetector::new()
            .with_adaptive_threshold(false)
            .with_tj_threshold(-100);
        let context = BoundaryContext::new(12.0);

        let prev = CharacterInfo {
            code: 'a' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 100.0,
            tj_offset: Some(-50), // Below -100 threshold
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        let curr = CharacterInfo {
            code: 'b' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 110.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        let boundary = detector.is_word_boundary(&prev, &curr, &context);
        assert!(!boundary, "TJ offset -50 should NOT trigger boundary when static -100 is used");
    }

    #[test]
    fn test_geometric_gap_basic() {
        let detector = WordBoundaryDetector::new();
        let context = BoundaryContext::new(12.0);

        let prev = CharacterInfo {
            code: 't' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 100.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        // Large gap (10 units > 9.6 = 12*0.8 threshold)
        let curr = CharacterInfo {
            code: 'h' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 115.0, // 115 - 105 = 10 unit gap
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        assert!(
            detector.has_significant_geometric_gap(&prev, &curr, &context),
            "Gap of 10 units should exceed threshold of 9.6 (12pt * 0.8)"
        );
    }

    #[test]
    fn test_geometric_gap_with_char_spacing() {
        let detector = WordBoundaryDetector::new();
        let mut context = BoundaryContext::new(12.0);
        context.char_spacing = 2.0; // Tc = 2.0

        let prev = CharacterInfo {
            code: 'a' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 100.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        // Raw gap = 10, but Tc = 2.0 reduces it to 8.0
        // 8.0 < 9.6 (threshold), so NO boundary
        let curr = CharacterInfo {
            code: 'b' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 115.0, // 115 - 105 = 10 unit gap
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        assert!(
            !detector.has_significant_geometric_gap(&prev, &curr, &context),
            "Gap of 10 - 2.0 (Tc) = 8.0 should NOT exceed threshold of 9.6"
        );
    }

    #[test]
    fn test_ligature_internal_gap_fi() {
        let detector = WordBoundaryDetector::new();
        let context = BoundaryContext::new(12.0);

        // 'f' component from expanded 'fi' ligature
        let prev = CharacterInfo {
            code: 'f' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 100.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: true, // This is from ligature expansion
            original_ligature: Some('ﬁ'),
            protected_from_split: false,
        };

        // Large gap but prev is from ligature
        let curr = CharacterInfo {
            code: 'i' as u32,
            glyph_id: None,
            width: 3.0,
            x_position: 120.0, // 120 - 105 = 15 unit gap (would normally be boundary)
            tj_offset: None,
            font_size: 12.0,
            is_ligature: true,
            original_ligature: Some('ﬁ'),
            protected_from_split: false,
        };

        assert!(
            !detector.has_significant_geometric_gap(&prev, &curr, &context),
            "Ligature internal gap should NOT create boundary even with large gap"
        );
    }

    #[test]
    fn test_punctuation_reduced_threshold() {
        let detector = WordBoundaryDetector::new();
        let context = BoundaryContext::new(12.0);
        // Base threshold: 12.0 * 0.8 = 9.6
        // Punctuation threshold: 9.6 * 0.5 = 4.8

        let prev = CharacterInfo {
            code: 'd' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 100.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        // Gap of 6.0 units
        // Normal threshold would be 9.6 (no boundary)
        // Punctuation threshold is 4.8 (YES boundary)
        let curr_period = CharacterInfo {
            code: '.' as u32, // Period is punctuation
            glyph_id: None,
            width: 2.0,
            x_position: 111.0, // 111 - 105 = 6 unit gap
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        assert!(
            detector.has_significant_geometric_gap(&prev, &curr_period, &context),
            "Gap of 6 units should exceed punctuation threshold of 4.8 (50% of 9.6)"
        );
    }

    #[test]
    fn test_punctuation_does_not_trigger_on_normal_text() {
        let detector = WordBoundaryDetector::new();
        let context = BoundaryContext::new(12.0);

        let prev = CharacterInfo {
            code: 'd' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 100.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        // Same gap (6.0) but current character is 'e', not punctuation
        let curr = CharacterInfo {
            code: 'e' as u32, // 'e' is NOT punctuation
            glyph_id: None,
            width: 5.0,
            x_position: 111.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        assert!(
            !detector.has_significant_geometric_gap(&prev, &curr, &context),
            "Gap of 6 units should NOT exceed normal threshold of 9.6"
        );
    }

    #[test]
    fn test_is_punctuation_ascii() {
        assert!(WordBoundaryDetector::is_punctuation('.' as u32));
        assert!(WordBoundaryDetector::is_punctuation(',' as u32));
        assert!(WordBoundaryDetector::is_punctuation('!' as u32));
        assert!(WordBoundaryDetector::is_punctuation('?' as u32));
        assert!(WordBoundaryDetector::is_punctuation(':' as u32));
        assert!(WordBoundaryDetector::is_punctuation(';' as u32));
    }

    #[test]
    fn test_is_punctuation_non_punctuation() {
        assert!(!WordBoundaryDetector::is_punctuation('a' as u32));
        assert!(!WordBoundaryDetector::is_punctuation('1' as u32));
        assert!(!WordBoundaryDetector::is_punctuation(' ' as u32));
    }

    #[test]
    fn test_is_ligature_internal_gap_ffi() {
        let detector = WordBoundaryDetector::new();

        // 'f' from 'ffi' ligature
        let prev = CharacterInfo {
            code: 'f' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 100.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: true,
            original_ligature: Some('ﬄ'), // ffi ligature U+FB04
            protected_from_split: false,
        };

        let curr = CharacterInfo {
            code: 'f' as u32,
            glyph_id: None,
            width: 5.0,
            x_position: 110.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: true,
            original_ligature: Some('ﬄ'),
            protected_from_split: false,
        };

        assert!(
            detector.is_ligature_internal_gap(&prev, &curr),
            "Should detect ligature internal gap when both have is_ligature=true"
        );
    }

    #[test]
    fn test_is_ligature_internal_gap_actual_ligature_code() {
        let detector = WordBoundaryDetector::new();

        // Previous character IS the ligature U+FB00 ('ff')
        let prev = CharacterInfo {
            code: 0xFB00, // 'ff' ligature
            glyph_id: None,
            width: 10.0,
            x_position: 100.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false, // Not expanded, still the ligature
            original_ligature: None,
            protected_from_split: false,
        };

        let curr = CharacterInfo {
            code: 'i' as u32,
            glyph_id: None,
            width: 3.0,
            x_position: 115.0,
            tj_offset: None,
            font_size: 12.0,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        };

        assert!(
            detector.is_ligature_internal_gap(&prev, &curr),
            "Should detect ligature internal gap when prev code is U+FB00"
        );
    }
}
