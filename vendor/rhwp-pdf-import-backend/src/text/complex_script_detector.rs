//! Complex Script Support
//!
//! This module provides comprehensive support for Devanagari, Thai, Khmer, and South Asian scripts,
//! including:
//! - Script detection for 15 complex script types
//! - Devanagari matras, virama, and conjunct consonant handling
//! - Thai tone marks and vowel modifiers
//! - Khmer COENG and subscript consonants
//! - Indic scripts (Tamil, Telugu, Kannada, Malayalam) diacritic preservation
//! - Complex script word boundary detection
//!
//! The implementation follows Unicode standards and common complex script processing rules.

use crate::text::CharacterInfo;

/// Detected complex script types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexScript {
    /// Devanagari (U+0900-U+097F) - Hindi, Sanskrit, Nepali, Marathi
    Devanagari,
    /// Bengali (U+0980-U+09FF)
    Bengali,
    /// Gurmukhi (U+0A00-U+0A7F) - Punjabi
    Gurmukhi,
    /// Gujarati (U+0A80-U+0AFF)
    Gujarati,
    /// Oriya (U+0B00-U+0B7F)
    Oriya,
    /// Tamil (U+0B80-U+0BFF)
    Tamil,
    /// Telugu (U+0C00-U+0C7F)
    Telugu,
    /// Kannada (U+0C80-U+0CFF)
    Kannada,
    /// Malayalam (U+0D00-U+0D7F)
    Malayalam,
    /// Sinhala (U+0D80-U+0DFF)
    Sinhala,
    /// Thai (U+0E00-U+0E7F)
    Thai,
    /// Lao (U+0E80-U+0EFF)
    Lao,
    /// Khmer (U+1780-U+17FF)
    Khmer,
    /// Burmese (U+1000-U+109F)
    Burmese,
    /// Mongolian (U+1800-U+18AF)
    Mongolian,
}

// ============================================================================
// SCRIPT DETECTION
// ============================================================================

/// Detect complex script for a character code (O(1) complexity)
///
/// Returns the specific complex script type if the character belongs to a complex script,
/// or None if it's not a complex script character.
///
/// # Fast Path
/// The implementation checks the Devanagari range first as it's most common.
pub fn detect_complex_script(code: u32) -> Option<ComplexScript> {
    // Fast path: Devanagari (most common South Asian script)
    if matches!(code, 0x0900..=0x097F) {
        return Some(ComplexScript::Devanagari);
    }

    // Other ranges
    match code {
        0x0980..=0x09FF => Some(ComplexScript::Bengali),
        0x0A00..=0x0A7F => Some(ComplexScript::Gurmukhi),
        0x0A80..=0x0AFF => Some(ComplexScript::Gujarati),
        0x0B00..=0x0B7F => Some(ComplexScript::Oriya),
        0x0B80..=0x0BFF => Some(ComplexScript::Tamil),
        0x0C00..=0x0C7F => Some(ComplexScript::Telugu),
        0x0C80..=0x0CFF => Some(ComplexScript::Kannada),
        0x0D00..=0x0D7F => Some(ComplexScript::Malayalam),
        0x0D80..=0x0DFF => Some(ComplexScript::Sinhala),
        0x0E00..=0x0E7F => Some(ComplexScript::Thai),
        0x0E80..=0x0EFF => Some(ComplexScript::Lao),
        0x1780..=0x17FF => Some(ComplexScript::Khmer),
        0x1000..=0x109F => Some(ComplexScript::Burmese),
        0x1800..=0x18AF => Some(ComplexScript::Mongolian),
        _ => None,
    }
}

/// Check if a character code is any complex script
///
/// This is a convenience function that returns true if the character
/// belongs to any complex script.
#[inline]
pub fn is_complex_script(code: u32) -> bool {
    detect_complex_script(code).is_some()
}

// ============================================================================
// DEVANAGARI SUPPORT
// ============================================================================

/// Check if a code is a Devanagari diacritic (general category)
///
/// Includes matras, nukta, anusvara, visarga, and other combining marks.
pub fn is_devanagari_diacritic(code: u32) -> bool {
    matches!(code,
        0x0901..=0x0903 |  // SIGN CANDRABINDU, ANUSVARA, VISARGA
        0x093A..=0x093C |  // Vowel signs
        0x093E..=0x094C |  // Dependent vowel signs (matras)
        0x094D |           // VIRAMA
        0x094E..=0x0950 |  // Various marks
        0x0951..=0x0957 |  // Tone marks
        0x0962..=0x0963    // Vocalic L/R marks
    )
}

/// Check if a code is the Devanagari virama (halant)
///
/// Virama (्, U+094D) marks a dead consonant in conjunct consonants.
/// No word boundary should occur after virama.
pub fn is_devanagari_virama(code: u32) -> bool {
    code == 0x094D
}

/// Check if a code is a Devanagari consonant
///
/// Consonants: U+0915-U+0939 (KA through HA)
pub fn is_devanagari_consonant(code: u32) -> bool {
    matches!(code, 0x0915..=0x0939)
}

/// Check if a code is a Devanagari matra (dependent vowel sign)
///
/// Matras (vowel modifiers): U+093E-U+094C
/// These attach to consonants and should not create word boundaries.
pub fn is_devanagari_matra(code: u32) -> bool {
    matches!(code, 0x093E..=0x094C)
}

/// Check if a code is Devanagari anusvara or visarga
///
/// Anusvara (ं, U+0902): Nasalization mark
/// Visarga (ः, U+0903): Aspiration mark
pub fn is_devanagari_anusvar_visarga(code: u32) -> bool {
    matches!(code, 0x0902 | 0x0903)
}

/// Check if a code is Devanagari nukta
///
/// Nukta (़, U+093C): Modifies letter sounds
pub fn is_devanagari_nukta(code: u32) -> bool {
    code == 0x093C
}

/// Handle Devanagari-specific word boundary decisions
///
/// Returns:
/// - Some(true): Definitely create a boundary
/// - Some(false): Definitely do NOT create a boundary
/// - None: Not applicable (let other detectors handle)
///
/// # Boundary Rules (in priority order)
///
/// 1. **No boundary after virama**: Virama + consonant = conjunct
/// 2. **No boundary before matras**: Matras attach to base consonant
/// 3. **No boundary before nukta**: Nukta modifies preceding character
/// 4. **No boundary before anusvara/visarga**: These attach to syllables
/// 5. **No boundary between multiple diacritics**: Keep all marks together
pub fn handle_devanagari_boundary(
    prev_char: &CharacterInfo,
    curr_char: &CharacterInfo,
) -> Option<bool> {
    let prev_code = prev_char.code;
    let curr_code = curr_char.code;

    // Rule 1: No boundary after virama (conjunct consonant formation)
    if is_devanagari_virama(prev_code) {
        return Some(false);
    }

    // Rule 2: No boundary before matras (dependent vowel signs)
    if is_devanagari_matra(curr_code) {
        return Some(false);
    }

    // Rule 3: No boundary before nukta (sound modifier)
    if is_devanagari_nukta(curr_code) {
        return Some(false);
    }

    // Rule 4: No boundary before anusvara/visarga
    if is_devanagari_anusvar_visarga(curr_code) {
        return Some(false);
    }

    // Rule 5: No boundary between multiple diacritics
    if is_devanagari_diacritic(prev_code) && is_devanagari_diacritic(curr_code) {
        return Some(false);
    }

    // Rule 6: No boundary AFTER a matra / dependent sign when the next glyph is
    // another Devanagari character. matra→consonant is intra-word continuation;
    // real word breaks carry an explicit space glyph. Mirrors `handle_indic_
    // boundary` Rule 3 (the dominant spurious-space direction).
    if is_devanagari_diacritic(prev_code) && matches!(curr_code, 0x0900..=0x097F) {
        return Some(false);
    }

    // Not a Devanagari-specific case - let other signals decide
    None
}

// ============================================================================
// THAI SUPPORT
// ============================================================================

/// Check if a code is a Thai tone mark
///
/// Thai has 4 tone marks: U+0E48-U+0E4B
/// - MAI EK (่)
/// - MAI THO (้)
/// - MAI TRI (๊)
/// - MAI CHATTAWA (๋)
pub fn is_thai_tone_mark(code: u32) -> bool {
    matches!(code, 0x0E48..=0x0E4B)
}

/// Check if a code is a Thai vowel modifier
///
/// Thai vowel modifiers include:
/// - U+0E31: SARA AM (ั)
/// - U+0E34-U+0E37: Above vowels
/// - U+0E39-U+0E3A: Below vowels
pub fn is_thai_vowel_modifier(code: u32) -> bool {
    matches!(code,
        0x0E31 |           // MAI HAN-AKAT
        0x0E34..=0x0E37 |  // Above vowels
        0x0E39..=0x0E3A    // Below vowels
    )
}

/// Check if a code is a Thai digit (Thai or Western)
///
/// Thai digits: U+0E50-U+0E59 (๐-๙)
/// Western digits: U+0030-U+0039 (0-9)
pub fn is_thai_digit(code: u32) -> bool {
    matches!(code,
        0x0030..=0x0039 |  // Western digits 0-9
        0x0E50..=0x0E59    // Thai digits ๐-๙
    )
}

/// Check if a code is Thai major punctuation
///
/// Major punctuation creates word boundaries:
/// - U+0E2F: PAIYANNOI (ฯ)
/// - U+0E46: MAIYAMOK (ๆ)
/// - U+0E4F: FONGMAN (๏)
pub fn is_thai_major_punctuation(code: u32) -> bool {
    matches!(code, 0x0E2F | 0x0E46 | 0x0E4F)
}

/// Handle Thai-specific word boundary decisions
///
/// Returns:
/// - Some(true): Definitely create a boundary
/// - Some(false): Definitely do NOT create a boundary
/// - None: Not applicable (let other detectors handle)
///
/// # Boundary Rules (in priority order)
///
/// 1. **No boundary before tone marks**: Tone marks attach to base character
/// 2. **No boundary before vowel modifiers**: Vowels attach to consonants
/// 3. **No boundary within digit sequences**: Keep numbers together
/// 4. **Boundary at major punctuation**: Sentence/phrase markers
pub fn handle_thai_boundary(prev_char: &CharacterInfo, curr_char: &CharacterInfo) -> Option<bool> {
    let prev_code = prev_char.code;
    let curr_code = curr_char.code;

    // Rule 1: No boundary before tone marks
    if is_thai_tone_mark(curr_code) {
        return Some(false);
    }

    // Rule 2: No boundary before vowel modifiers
    if is_thai_vowel_modifier(curr_code) {
        return Some(false);
    }

    // Rule 3: No boundary within digit sequences
    if is_thai_digit(prev_code) && is_thai_digit(curr_code) {
        return Some(false);
    }

    // Rule 4: Boundary at major punctuation
    if is_thai_major_punctuation(curr_code) {
        return Some(true);
    }

    // Not a Thai-specific case - let other signals decide
    None
}

// ============================================================================
// KHMER SUPPORT
// ============================================================================

/// Check if a code is Khmer COENG (virama-equivalent)
///
/// COENG (◌្, U+17D2) marks a subscript consonant in Khmer.
/// No word boundary should occur after COENG.
pub fn is_khmer_coeng(code: u32) -> bool {
    code == 0x17D2
}

/// Check if a code is a Khmer vowel (inherent or dependent)
///
/// Khmer vowels include:
/// - U+17B4-U+17B5: Inherent vowels
/// - U+17B7-U+17BD: Vowel signs above
/// - U+17BE-U+17C5: Vowel signs below/around
/// - U+17C6: NIKAHIT (ំ)
pub fn is_khmer_vowel_inherent(code: u32) -> bool {
    matches!(code,
        0x17B4..=0x17B5 |  // Inherent vowels
        0x17B7..=0x17BD |  // Above vowels
        0x17BE..=0x17C5 |  // Below/around vowels
        0x17C6             // NIKAHIT
    )
}

/// Check if a code is a Khmer tone mark
///
/// Khmer tone marks: U+17C9-U+17CC
/// - MUUSIKATOAN (◌៉)
/// - TRIISAP (◌៊)
/// - BANTOC (◌់)
pub fn is_khmer_tone_mark(code: u32) -> bool {
    matches!(code, 0x17C9..=0x17CC)
}

/// Handle Khmer-specific word boundary decisions
///
/// Returns:
/// - Some(true): Definitely create a boundary
/// - Some(false): Definitely do NOT create a boundary
/// - None: Not applicable (let other detectors handle)
///
/// # Boundary Rules (in priority order)
///
/// 1. **No boundary after COENG**: COENG + consonant = subscript
/// 2. **No boundary before vowels**: Vowels attach to consonants
/// 3. **No boundary before tone marks**: Tone marks attach to syllables
pub fn handle_khmer_boundary(prev_char: &CharacterInfo, curr_char: &CharacterInfo) -> Option<bool> {
    let prev_code = prev_char.code;
    let curr_code = curr_char.code;

    // Rule 1: No boundary after COENG (subscript consonant marker)
    if is_khmer_coeng(prev_code) {
        return Some(false);
    }

    // Rule 2: No boundary before vowels
    if is_khmer_vowel_inherent(curr_code) {
        return Some(false);
    }

    // Rule 3: No boundary before tone marks
    if is_khmer_tone_mark(curr_code) {
        return Some(false);
    }

    // Not a Khmer-specific case - let other signals decide
    None
}

// ============================================================================
// INDIC SCRIPTS (SHARED PATTERN)
// ============================================================================

/// Check if a code is an Indic diacritic (Bengali, Tamil, Telugu, Kannada, Malayalam)
///
/// This function uses the general pattern that diacritics in Indic scripts
/// typically occupy the upper portion of each script block.
pub fn is_indic_diacritic(code: u32) -> bool {
    match code {
        // Bengali diacritics
        0x0981..=0x0983 |  // Candrabindu, anusvara, visarga
        0x09BC |           // Nukta
        0x09BE..=0x09CC |  // Matras
        0x09CD |           // Virama
        0x09D7 |           // AU length mark
        0x09E2..=0x09E3 => true, // Vocalic marks

        // Tamil diacritics
        0x0B82..=0x0B83 |  // Anusvara
        0x0BBE..=0x0BCC |  // Matras
        0x0BCD |           // Virama
        0x0BD7 => true,    // AU length mark

        // Telugu diacritics
        0x0C01..=0x0C03 |  // Candrabindu, anusvara, visarga
        0x0C3E..=0x0C4C |  // Matras
        0x0C4D |           // Virama
        0x0C55..=0x0C56 |  // Length marks
        0x0C62..=0x0C63 => true, // Vocalic marks

        // Kannada diacritics
        0x0C81..=0x0C83 |  // Candrabindu, anusvara, visarga
        0x0CBC |           // Nukta
        0x0CBE..=0x0CCC |  // Matras
        0x0CCD |           // Virama
        0x0CD5..=0x0CD6 |  // Length marks
        0x0CE2..=0x0CE3 => true, // Vocalic marks

        // Malayalam diacritics
        0x0D01..=0x0D03 |  // Candrabindu, anusvara, visarga
        0x0D3E..=0x0D4C |  // Matras
        0x0D4D |           // Virama
        0x0D57 |           // AU length mark
        0x0D62..=0x0D63 => true, // Vocalic marks

        _ => false,
    }
}

/// Handle Indic script (Tamil, Telugu, Kannada, Malayalam) boundary decisions
///
/// Returns:
/// - Some(true): Definitely create a boundary
/// - Some(false): Definitely do NOT create a boundary
/// - None: Not applicable (let other detectors handle)
///
/// # Boundary Rules
///
/// 1. **No boundary before diacritics**: All matras, viramas, and marks attach to base
/// 2. **No boundary between multiple marks**: Keep all diacritics together
pub fn handle_indic_boundary(prev_char: &CharacterInfo, curr_char: &CharacterInfo) -> Option<bool> {
    let prev_code = prev_char.code;
    let curr_code = curr_char.code;

    // Rule 1: No boundary before any Indic diacritic
    if is_indic_diacritic(curr_code) {
        return Some(false);
    }

    // Rule 2: No boundary between multiple diacritics
    if is_indic_diacritic(prev_code) && is_indic_diacritic(curr_code) {
        return Some(false);
    }

    // Rule 3: No boundary AFTER a matra / dependent vowel sign / virama when the
    // next glyph is another character of the same Brahmic script. A dependent
    // vowel sign followed by a base consonant is always intra-word — real word
    // breaks carry an explicit space glyph (U+0020), handled upstream. This is
    // the dominant spurious-space direction (matra→consonant) that a purely
    // geometric gap test inserts because the matra carries its own advance.
    if is_indic_diacritic(prev_code) && detect_complex_script(curr_code).is_some() {
        return Some(false);
    }

    // Not an Indic-specific case - let other signals decide
    None
}

// ============================================================================
// SHARED UTILITIES
// ============================================================================

/// Check if a code is any complex script mark (combining character)
///
/// This is a convenience function that checks if a character is a diacritic
/// or mark in any complex script.
pub fn is_complex_script_mark(code: u32) -> bool {
    is_devanagari_diacritic(code)
        || is_indic_diacritic(code)
        || is_thai_tone_mark(code)
        || is_thai_vowel_modifier(code)
        || is_khmer_vowel_inherent(code)
        || is_khmer_tone_mark(code)
        || is_khmer_coeng(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Script detection tests
    #[test]
    fn test_detect_devanagari_range() {
        assert_eq!(detect_complex_script(0x0915), Some(ComplexScript::Devanagari)); // क (KA)
        assert_eq!(detect_complex_script(0x0928), Some(ComplexScript::Devanagari)); // न (NA)
        assert_eq!(detect_complex_script(0x0971), Some(ComplexScript::Devanagari));
    }

    #[test]
    fn test_detect_thai_range() {
        assert_eq!(detect_complex_script(0x0E01), Some(ComplexScript::Thai)); // ก
        assert_eq!(detect_complex_script(0x0E3F), Some(ComplexScript::Thai));
    }

    #[test]
    fn test_detect_khmer_range() {
        assert_eq!(detect_complex_script(0x1780), Some(ComplexScript::Khmer)); // ក
        assert_eq!(detect_complex_script(0x17FF), Some(ComplexScript::Khmer));
    }

    #[test]
    fn test_detect_tamil_range() {
        assert_eq!(detect_complex_script(0x0B85), Some(ComplexScript::Tamil)); // அ
        assert_eq!(detect_complex_script(0x0BBF), Some(ComplexScript::Tamil));
    }

    #[test]
    fn test_detect_non_complex_script() {
        assert_eq!(detect_complex_script(0x0041), None); // Latin 'A'
        assert_eq!(detect_complex_script(0x0020), None); // Space
    }

    // Devanagari tests
    #[test]
    fn test_devanagari_virama_detection() {
        assert!(is_devanagari_virama(0x094D)); // ्
    }

    #[test]
    fn test_devanagari_matra_detection() {
        assert!(is_devanagari_matra(0x093F)); // ि
        assert!(is_devanagari_matra(0x0940)); // ी
        assert!(is_devanagari_matra(0x0947)); // े
        assert!(!is_devanagari_matra(0x0915)); // क (consonant)
    }

    #[test]
    fn test_devanagari_consonant_detection() {
        assert!(is_devanagari_consonant(0x0915)); // क (KA)
        assert!(is_devanagari_consonant(0x0928)); // न (NA)
        assert!(!is_devanagari_consonant(0x0905)); // अ (vowel)
    }

    // Thai tests
    #[test]
    fn test_thai_tone_mark_detection() {
        assert!(is_thai_tone_mark(0x0E48)); // ่
        assert!(is_thai_tone_mark(0x0E49)); // ้
        assert!(!is_thai_tone_mark(0x0E01)); // ก (consonant)
    }

    #[test]
    fn test_thai_vowel_modifier_detection() {
        assert!(is_thai_vowel_modifier(0x0E31)); // ั
        assert!(is_thai_vowel_modifier(0x0E34)); // ิ
        assert!(!is_thai_vowel_modifier(0x0E01)); // ก (consonant)
    }

    #[test]
    fn test_thai_digit_detection() {
        assert!(is_thai_digit(0x0E50)); // ๐
        assert!(is_thai_digit(0x0031)); // 1 (Western)
        assert!(!is_thai_digit(0x0E01)); // ก (consonant)
    }

    // Khmer tests
    #[test]
    fn test_khmer_coeng_detection() {
        assert!(is_khmer_coeng(0x17D2)); // ្
        assert!(!is_khmer_coeng(0x1780)); // ក (consonant)
    }

    #[test]
    fn test_khmer_vowel_detection() {
        assert!(is_khmer_vowel_inherent(0x17BE)); // ើ
        assert!(is_khmer_vowel_inherent(0x17C6)); // ំ
        assert!(!is_khmer_vowel_inherent(0x1780)); // ក (consonant)
    }

    // Indic scripts tests
    #[test]
    fn test_indic_diacritic_detection() {
        // Bengali
        assert!(is_indic_diacritic(0x09CD)); // Virama
                                             // Tamil
        assert!(is_indic_diacritic(0x0BCD)); // Virama
                                             // Telugu
        assert!(is_indic_diacritic(0x0C4D)); // Virama
                                             // Kannada
        assert!(is_indic_diacritic(0x0CCD)); // Virama
                                             // Malayalam
        assert!(is_indic_diacritic(0x0D4D)); // Virama
    }
}
