//! CJK Script Detection and Transition Analysis
//!
//! This module provides comprehensive script detection for CJK (Chinese, Japanese, Korean)
//! languages and analyzes script transitions to determine word boundary behavior.
//!
//! # Script Detection
//!
//! Detects the following CJK scripts with O(1) performance:
//! - Han (Chinese/Japanese Kanji)
//! - Hiragana (Japanese)
//! - Katakana (Japanese)
//! - Hangul (Korean)
//!
//! # Language Inference
//!
//! Infers document language from script distribution:
//! - Japanese: Contains Hiragana or Katakana
//! - Korean: Contains Hangul
//! - Chinese: Only Han characters (default)
//!
//! # Word Boundary Rules
//!
//! - **Japanese**: Allow Hiragana↔Katakana and Han↔Kana transitions
//! - **Korean**: Allow Hangul↔Hanja (Han) transitions
//! - **Chinese**: Create boundaries between Han characters
//! - **Script Changes**: Create boundaries for non-CJK transitions

#![forbid(unsafe_code)]

use crate::text::CharacterInfo;

/// Detected CJK script types.
///
/// Each script represents a distinct Unicode range used in CJK writing systems.
/// Script detection is O(1) using range matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CJKScript {
    /// Han ideographs (U+4E00-U+9FFF)
    /// Used in Chinese, Japanese (Kanji), and Korean (Hanja)
    Han,

    /// Han Extension A (U+3400-U+4DBF)
    /// Additional rare Han characters
    HanExtensionA,

    /// Han Extension B-F (U+20000-U+2EBEF)
    /// Very rare Han characters requiring 4-byte UTF-8 encoding
    HanExtensionBF,

    /// Hiragana (U+3040-U+309F)
    /// Japanese phonetic script for native words
    Hiragana,

    /// Katakana (U+30A0-U+30FF)
    /// Japanese phonetic script for foreign words
    Katakana,

    /// Halfwidth Katakana (U+FF61-U+FF9F)
    /// Narrow form of Katakana for legacy encoding compatibility
    HalfwidthKatakana,

    /// Hangul (U+AC00-U+D7AF)
    /// Korean alphabet syllables
    Hangul,

    /// CJK Symbols and Punctuation (U+3190-U+319F subset)
    /// Ideographic annotations and symbols
    CJKSymbol,
}

/// Document language inferred from script distribution.
///
/// Language detection helps determine appropriate word boundary rules
/// for script transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentLanguage {
    /// Japanese text (contains Hiragana or Katakana)
    Japanese,

    /// Korean text (contains Hangul)
    Korean,

    /// Chinese text (Han only, no phonetic scripts)
    Chinese,
}

/// Detect the CJK script type for a character code.
///
/// Uses O(1) range matching with fast-path optimization for the most common
/// Han range (U+4E00-U+9FFF), which covers ~90% of CJK text.
///
/// # Arguments
///
/// * `code` - Unicode code point to analyze
///
/// # Returns
///
/// - `Some(CJKScript)` if the character is CJK
/// - `None` if the character is not CJK
pub fn detect_cjk_script(code: u32) -> Option<CJKScript> {
    // Fast path: Check most common Han range first (90% of CJK text)
    if matches!(code, 0x4E00..=0x9FFF) {
        return Some(CJKScript::Han);
    }

    // Check other ranges
    match code {
        0x3400..=0x4DBF => Some(CJKScript::HanExtensionA),
        0x20000..=0x2EBEF => Some(CJKScript::HanExtensionBF),
        0x3040..=0x309F => Some(CJKScript::Hiragana),
        0x30A0..=0x30FF => Some(CJKScript::Katakana),
        0xFF61..=0xFF9F => Some(CJKScript::HalfwidthKatakana),
        0xAC00..=0xD7AF => Some(CJKScript::Hangul),
        0x3190..=0x319F => Some(CJKScript::CJKSymbol),
        _ => None,
    }
}

/// Check if a transition between scripts should create a word boundary.
///
/// This function implements language-specific rules for script transitions:
///
/// # Japanese Rules
/// - Han ↔ Hiragana: No boundary (kanji-okurigana pattern)
/// - Han ↔ Katakana: No boundary (kanji-katakana compounds)
/// - Hiragana ↔ Katakana: No boundary (mixed phonetic text)
///
/// # Korean Rules
/// - Hangul ↔ Han: No boundary (Hanja-Hangul mixing)
///
/// # Chinese Rules
/// - All Han-to-Han: Consult other signals (TJ offset, geometry)
///
/// # Cross-Script Rules
/// - CJK → Latin/ASCII: Boundary (language change)
/// - Latin/ASCII → CJK: Boundary (language change)
///
/// # Arguments
///
/// * `prev_script` - Script of previous character (None if not CJK)
/// * `curr_script` - Script of current character (None if not CJK)
/// * `language` - Document language context (if known)
///
/// # Returns
///
/// - `Some(true)` - Must create boundary
/// - `Some(false)` - Must not create boundary
/// - `None` - Use other signals (TJ offset, geometry)
pub fn should_split_on_script_transition(
    prev_script: Option<CJKScript>,
    curr_script: Option<CJKScript>,
    language: Option<DocumentLanguage>,
) -> Option<bool> {
    match (prev_script, curr_script) {
        // Both are CJK scripts
        (Some(prev), Some(curr)) => should_split_cjk_transition(prev, curr, language),

        // Transition from CJK to non-CJK (or vice versa)
        (Some(_), None) | (None, Some(_)) => Some(true),

        // Both non-CJK, not our concern
        (None, None) => None,
    }
}

/// Check if a transition between two CJK scripts should create a boundary.
fn should_split_cjk_transition(
    prev: CJKScript,
    curr: CJKScript,
    language: Option<DocumentLanguage>,
) -> Option<bool> {
    // Same script: no boundary (unless other signals indicate otherwise)
    if prev == curr {
        return None;
    }

    // Language-specific rules
    match language {
        Some(DocumentLanguage::Japanese) => handle_japanese_transition(prev, curr),
        Some(DocumentLanguage::Korean) => handle_korean_transition(prev, curr),
        Some(DocumentLanguage::Chinese) | None => {
            // Chinese or unknown: use conservative defaults
            handle_chinese_transition(prev, curr)
        },
    }
}

/// Handle script transitions for Japanese text.
///
/// Japanese freely mixes Han (Kanji), Hiragana, and Katakana without word boundaries.
fn handle_japanese_transition(prev: CJKScript, curr: CJKScript) -> Option<bool> {
    use CJKScript::*;

    // Allow all Japanese script transitions
    match (prev, curr) {
        // Han ↔ Hiragana (kanji + okurigana)
        (Han | HanExtensionA | HanExtensionBF, Hiragana) => Some(false),
        (Hiragana, Han | HanExtensionA | HanExtensionBF) => Some(false),

        // Han ↔ Katakana (kanji + katakana)
        (Han | HanExtensionA | HanExtensionBF, Katakana | HalfwidthKatakana) => Some(false),
        (Katakana | HalfwidthKatakana, Han | HanExtensionA | HanExtensionBF) => Some(false),

        // Hiragana ↔ Katakana
        (Hiragana, Katakana | HalfwidthKatakana) => Some(false),
        (Katakana | HalfwidthKatakana, Hiragana) => Some(false),

        // Katakana variants
        (Katakana, HalfwidthKatakana) | (HalfwidthKatakana, Katakana) => Some(false),

        // Other transitions: consult other signals
        _ => None,
    }
}

/// Handle script transitions for Korean text.
///
/// Korean allows Hangul-Hanja (Han) mixing without word boundaries.
fn handle_korean_transition(prev: CJKScript, curr: CJKScript) -> Option<bool> {
    use CJKScript::*;

    match (prev, curr) {
        // Hangul ↔ Han (Hanja)
        (Hangul, Han | HanExtensionA | HanExtensionBF) => Some(false),
        (Han | HanExtensionA | HanExtensionBF, Hangul) => Some(false),

        // Other transitions: consult other signals
        _ => None,
    }
}

/// Handle script transitions for Chinese text.
///
/// Chinese primarily uses Han characters with minimal script mixing.
fn handle_chinese_transition(_prev: CJKScript, _curr: CJKScript) -> Option<bool> {
    // For Chinese, rely on other signals (TJ offset, geometry, punctuation)
    None
}

/// Infer document language from script distribution.
///
/// Analyzes the frequency of different scripts to determine the document's
/// primary language. This helps apply appropriate word boundary rules.
///
/// # Algorithm
///
/// 1. If Hiragana or Katakana present: Japanese
/// 2. If Hangul present: Korean
/// 3. If only Han: Chinese (default)
///
/// # Arguments
///
/// * `scripts` - List of (script, count) pairs from text analysis
///
/// # Returns
///
/// - `Some(DocumentLanguage)` if language can be inferred
/// - `None` if insufficient data or no CJK content
pub fn infer_document_language(scripts: &[(CJKScript, usize)]) -> Option<DocumentLanguage> {
    if scripts.is_empty() {
        return None;
    }

    let mut has_hiragana = false;
    let mut has_katakana = false;
    let mut has_hangul = false;
    let mut has_han = false;

    for (script, _count) in scripts {
        match script {
            CJKScript::Hiragana => has_hiragana = true,
            CJKScript::Katakana | CJKScript::HalfwidthKatakana => has_katakana = true,
            CJKScript::Hangul => has_hangul = true,
            CJKScript::Han | CJKScript::HanExtensionA | CJKScript::HanExtensionBF => has_han = true,
            _ => {},
        }
    }

    // Japanese: presence of phonetic scripts
    if has_hiragana || has_katakana {
        return Some(DocumentLanguage::Japanese);
    }

    // Korean: presence of Hangul
    if has_hangul {
        return Some(DocumentLanguage::Korean);
    }

    // Chinese: only Han characters
    if has_han {
        return Some(DocumentLanguage::Chinese);
    }

    None
}

/// Check if a character is a small Hiragana that attaches to the previous character.
///
/// Small Kana (sokuon, yōon) never create word boundaries.
///
/// # Small Hiragana
///
/// - ぁぃぅぇぉ (small vowels)
/// - ゃゅょ (small y-vowels)
/// - ゎ (small wa)
/// - っ (small tsu / sokuon)
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is small Hiragana
pub fn is_small_hiragana(code: u32) -> bool {
    matches!(
        code,
        0x3041  // ぁ SMALL A
        | 0x3043 // ぃ SMALL I
        | 0x3045 // ぅ SMALL U
        | 0x3047 // ぇ SMALL E
        | 0x3049 // ぉ SMALL O
        | 0x3063 // っ SMALL TSU
        | 0x3083 // ゃ SMALL YA
        | 0x3085 // ゅ SMALL YU
        | 0x3087 // ょ SMALL YO
        | 0x308E // ゎ SMALL WA
    )
}

/// Check if a character is a small Katakana that attaches to the previous character.
///
/// # Small Katakana
///
/// - ァィゥェォ (small vowels)
/// - ャュョ (small y-vowels)
/// - ヮ (small wa)
/// - ッ (small tsu / sokuon)
/// - ヵヶ (small ka/ke)
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is small Katakana
pub fn is_small_katakana(code: u32) -> bool {
    matches!(
        code,
        0x30A1  // ァ SMALL A
        | 0x30A3 // ィ SMALL I
        | 0x30A5 // ゥ SMALL U
        | 0x30A7 // ェ SMALL E
        | 0x30A9 // ォ SMALL O
        | 0x30C3 // ッ SMALL TSU
        | 0x30E3 // ャ SMALL YA
        | 0x30E5 // ュ SMALL YU
        | 0x30E7 // ョ SMALL YO
        | 0x30EE // ヮ SMALL WA
        | 0x30F5 // ヵ SMALL KA
        | 0x30F6 // ヶ SMALL KE
    )
}

/// Check if a character is a combining mark (dakuten, handakuten).
///
/// Combining marks modify the preceding character and never create boundaries.
///
/// # Combining Marks
///
/// - U+3099: COMBINING KATAKANA-HIRAGANA VOICED SOUND MARK (dakuten)
/// - U+309A: COMBINING KATAKANA-HIRAGANA SEMI-VOICED SOUND MARK (handakuten)
/// - U+FF9E: HALFWIDTH KATAKANA VOICED SOUND MARK
/// - U+FF9F: HALFWIDTH KATAKANA SEMI-VOICED SOUND MARK
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is a combining mark
pub fn is_combining_mark(code: u32) -> bool {
    matches!(
        code,
        0x3099  // Combining dakuten
        | 0x309A // Combining handakuten
        | 0xFF9E // Halfwidth dakuten
        | 0xFF9F // Halfwidth handakuten
    )
}

/// Check if a character is a Japanese modifier (small Kana or combining mark).
///
/// Modifiers always attach to the previous character and never create boundaries.
///
/// # Arguments
///
/// * `code` - Unicode code point to check
///
/// # Returns
///
/// `true` if the character is a Japanese modifier
pub fn is_japanese_modifier(code: u32) -> bool {
    is_small_hiragana(code) || is_small_katakana(code) || is_combining_mark(code)
}

/// Handle Japanese-specific word boundary decisions.
///
/// Implements Japanese text segmentation rules:
/// - No boundary before small Kana or combining marks
/// - Allow seamless Hiragana↔Katakana transitions
/// - Allow Han↔Kana transitions
///
/// # Arguments
///
/// * `prev_char` - Previous character information
/// * `curr_char` - Current character information
/// * `prev_script` - Detected script of previous character
/// * `curr_script` - Detected script of current character
///
/// # Returns
///
/// - `Some(true)` - Create boundary
/// - `Some(false)` - Do not create boundary
/// - `None` - Use other signals
pub fn handle_japanese_text(
    _prev_char: &CharacterInfo,
    curr_char: &CharacterInfo,
    prev_script: Option<CJKScript>,
    curr_script: Option<CJKScript>,
) -> Option<bool> {
    // Never create boundary before Japanese modifiers
    if is_japanese_modifier(curr_char.code) {
        return Some(false);
    }

    // Use script transition rules
    should_split_on_script_transition(prev_script, curr_script, Some(DocumentLanguage::Japanese))
}

/// Handle Korean-specific word boundary decisions.
///
/// Implements Korean text segmentation rules:
/// - Allow seamless Hangul↔Hanja transitions
///
/// # Arguments
///
/// * `_prev_char` - Previous character information (unused currently)
/// * `_curr_char` - Current character information (unused currently)
/// * `prev_script` - Detected script of previous character
/// * `curr_script` - Detected script of current character
///
/// # Returns
///
/// - `Some(true)` - Create boundary
/// - `Some(false)` - Do not create boundary
/// - `None` - Use other signals
pub fn handle_korean_text(
    _prev_char: &CharacterInfo,
    _curr_char: &CharacterInfo,
    prev_script: Option<CJKScript>,
    curr_script: Option<CJKScript>,
) -> Option<bool> {
    // Use script transition rules
    should_split_on_script_transition(prev_script, curr_script, Some(DocumentLanguage::Korean))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Script detection tests
    #[test]
    fn test_detect_han_main_range() {
        assert_eq!(detect_cjk_script(0x4E00), Some(CJKScript::Han));
        assert_eq!(detect_cjk_script(0x6587), Some(CJKScript::Han));
        assert_eq!(detect_cjk_script(0x9FFF), Some(CJKScript::Han));
    }

    #[test]
    fn test_detect_han_extension_a() {
        assert_eq!(detect_cjk_script(0x3400), Some(CJKScript::HanExtensionA));
        assert_eq!(detect_cjk_script(0x4DBF), Some(CJKScript::HanExtensionA));
    }

    #[test]
    fn test_detect_hiragana() {
        assert_eq!(detect_cjk_script(0x3042), Some(CJKScript::Hiragana)); // あ
        assert_eq!(detect_cjk_script(0x3093), Some(CJKScript::Hiragana)); // ん
    }

    #[test]
    fn test_detect_katakana() {
        assert_eq!(detect_cjk_script(0x30A2), Some(CJKScript::Katakana)); // ア
        assert_eq!(detect_cjk_script(0x30F3), Some(CJKScript::Katakana)); // ン
    }

    #[test]
    fn test_detect_hangul() {
        assert_eq!(detect_cjk_script(0xAC00), Some(CJKScript::Hangul)); // 가
        assert_eq!(detect_cjk_script(0xD7AF), Some(CJKScript::Hangul)); // 힣
    }

    #[test]
    fn test_detect_non_cjk() {
        assert_eq!(detect_cjk_script(0x0041), None); // A
        assert_eq!(detect_cjk_script(0x0020), None); // Space
    }

    // Language inference tests
    #[test]
    fn test_infer_japanese_with_hiragana() {
        let scripts = vec![(CJKScript::Han, 100), (CJKScript::Hiragana, 50)];
        assert_eq!(infer_document_language(&scripts), Some(DocumentLanguage::Japanese));
    }

    #[test]
    fn test_infer_japanese_with_katakana() {
        let scripts = vec![(CJKScript::Han, 100), (CJKScript::Katakana, 30)];
        assert_eq!(infer_document_language(&scripts), Some(DocumentLanguage::Japanese));
    }

    #[test]
    fn test_infer_korean() {
        let scripts = vec![(CJKScript::Hangul, 100), (CJKScript::Han, 20)];
        assert_eq!(infer_document_language(&scripts), Some(DocumentLanguage::Korean));
    }

    #[test]
    fn test_infer_chinese() {
        let scripts = vec![(CJKScript::Han, 100)];
        assert_eq!(infer_document_language(&scripts), Some(DocumentLanguage::Chinese));
    }

    // Script transition tests
    #[test]
    fn test_japanese_han_hiragana_no_split() {
        let result = should_split_on_script_transition(
            Some(CJKScript::Han),
            Some(CJKScript::Hiragana),
            Some(DocumentLanguage::Japanese),
        );
        assert_eq!(result, Some(false));
    }

    #[test]
    fn test_japanese_hiragana_katakana_no_split() {
        let result = should_split_on_script_transition(
            Some(CJKScript::Hiragana),
            Some(CJKScript::Katakana),
            Some(DocumentLanguage::Japanese),
        );
        assert_eq!(result, Some(false));
    }

    #[test]
    fn test_korean_hangul_han_no_split() {
        let result = should_split_on_script_transition(
            Some(CJKScript::Hangul),
            Some(CJKScript::Han),
            Some(DocumentLanguage::Korean),
        );
        assert_eq!(result, Some(false));
    }

    #[test]
    fn test_cjk_to_latin_split() {
        let result = should_split_on_script_transition(
            Some(CJKScript::Han),
            None,
            Some(DocumentLanguage::Chinese),
        );
        assert_eq!(result, Some(true));
    }

    // Japanese modifier tests
    #[test]
    fn test_small_hiragana_detection() {
        assert!(is_small_hiragana(0x3041)); // ぁ
        assert!(is_small_hiragana(0x3063)); // っ
        assert!(is_small_hiragana(0x3083)); // ゃ
        assert!(!is_small_hiragana(0x3042)); // あ (normal)
    }

    #[test]
    fn test_small_katakana_detection() {
        assert!(is_small_katakana(0x30A1)); // ァ
        assert!(is_small_katakana(0x30C3)); // ッ
        assert!(is_small_katakana(0x30E3)); // ャ
        assert!(!is_small_katakana(0x30A2)); // ア (normal)
    }

    #[test]
    fn test_combining_marks() {
        assert!(is_combining_mark(0x3099)); // Dakuten
        assert!(is_combining_mark(0x309A)); // Handakuten
        assert!(is_combining_mark(0xFF9E)); // Halfwidth dakuten
    }

    #[test]
    fn test_japanese_modifier() {
        assert!(is_japanese_modifier(0x3063)); // Small tsu
        assert!(is_japanese_modifier(0x30C3)); // Small katakana tsu
        assert!(is_japanese_modifier(0x3099)); // Dakuten
    }
}
