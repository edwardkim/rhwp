//! Right-to-Left (RTL) Script Support
//!
//! This module provides comprehensive support for Arabic and Hebrew scripts,
//! including:
//! - Script detection (Arabic, Hebrew, supplements, presentation forms)
//! - Diacritical mark handling (no boundaries before marks)
//! - Letter and punctuation detection
//! - Number handling (Western and Eastern Arabic digits)
//! - LAM-ALEF ligature support
//! - Contextual form normalization
//! - RTL-specific word boundary detection
//!
//! The implementation follows Unicode standards and common RTL text processing rules.

use crate::text::{BoundaryContext, CharacterInfo};

/// Detected RTL script types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTLScript {
    /// Arabic main block (U+0600-U+06FF)
    Arabic,
    /// Arabic Supplement (U+0750-U+077F)
    ArabicSupplement,
    /// Arabic Extended-A (U+08A0-U+08FF)
    ArabicExtendedA,
    /// Hebrew (U+0590-U+05FF)
    Hebrew,
    /// Arabic Presentation Forms-A (U+FB50-U+FDFF)
    PresentationFormsA,
    /// Arabic Presentation Forms-B (U+FE70-U+FEFF)
    PresentationFormsB,
}

// ============================================================================
// SCRIPT DETECTION
// ============================================================================

/// Detect RTL script for a character code (O(1) complexity)
///
/// Returns the specific RTL script type if the character belongs to an RTL script,
/// or None if it's not an RTL character.
///
/// # Fast Path
/// The implementation checks the Arabic main range first as it's most common.
pub fn detect_rtl_script(code: u32) -> Option<RTLScript> {
    // Fast path: Arabic main range (most common)
    if matches!(code, 0x0600..=0x06FF) {
        return Some(RTLScript::Arabic);
    }

    // Other ranges
    match code {
        0x0590..=0x05FF => Some(RTLScript::Hebrew),
        0x0750..=0x077F => Some(RTLScript::ArabicSupplement),
        0x08A0..=0x08FF => Some(RTLScript::ArabicExtendedA),
        0xFB50..=0xFDFF => Some(RTLScript::PresentationFormsA),
        0xFE70..=0xFEFF => Some(RTLScript::PresentationFormsB),
        _ => None,
    }
}

/// Check if a character code is any RTL text
///
/// This is a convenience function that returns true if the character
/// belongs to any RTL script (Arabic or Hebrew).
#[inline]
pub fn is_rtl_text(code: u32) -> bool {
    detect_rtl_script(code).is_some()
}

// ============================================================================
// ARABIC DIACRITICS
// ============================================================================

/// Check if a code is an Arabic diacritical mark
///
/// Arabic diacritics include:
/// - Basic marks (U+064B-U+0658): FATHATAN, DAMMATAN, KASRATAN, FATHA, DAMMA, KASRA, SHADDA, SUKUN, etc.
/// - Extended marks (U+06D6-U+06ED): Various small high and low marks
///
/// Diacritics should never create word boundaries.
pub fn is_arabic_diacritic(code: u32) -> bool {
    matches!(code,
        0x064B..=0x0658 |  // Basic Arabic diacritics
        0x06D6..=0x06DC |  // Small high marks
        0x06DF..=0x06E4 |  // Small high marks continued
        0x06E7..=0x06E8 |  // Small high marks continued
        0x06EA..=0x06ED    // Small low marks
    )
}

/// Check if a code is an Arabic letter (not diacritic or punctuation)
///
/// Includes letters from:
/// - Arabic main block (U+0621-U+063A, U+0641-U+064A)
/// - Arabic Supplement (U+0750-U+076D)
/// - Arabic Extended-A (U+08A0-U+08B4, U+08B6-U+08BD)
pub fn is_arabic_letter(code: u32) -> bool {
    matches!(code,
        0x0621..=0x063A |  // Arabic letters (excluding TATWEEL at 0x0640)
        0x0641..=0x064A |  // More Arabic letters
        0x0750..=0x076D |  // Arabic Supplement letters
        0x08A0..=0x08B4 |  // Arabic Extended-A letters
        0x08B6..=0x08BD    // More Extended-A letters
    )
}

/// Arabic letters with Unicode `Joining_Type = R` (right-joining only): they
/// join to a preceding letter but NEVER to the following one, so the cursive
/// connection breaks *after* them regardless of any following letter. The
/// canonical set is the alef family (ا أ إ آ ٱ), waw (و ؤ), dal/thal (د ذ),
/// reh/zain (ر ز), teh marbuta (ة), and their block variants — per Unicode
/// `ArabicShaping.txt`.
///
/// Relevance (ISO 32000-1 §14.8.2.3.3): because the join already breaks after
/// an R-letter, a SPACE following one renders the same whether it is a genuine
/// word break or a producer artefact — the two are visually indistinguishable.
/// The interior-space stripper uses this to avoid concatenating two real words
/// across such a space (`دار اب` must stay two words, not become `داراب`).
pub fn is_right_joining_arabic(code: u32) -> bool {
    matches!(code,
        0x0622..=0x0625 | // alef madda / hamza-above / waw-hamza / hamza-below
        0x0627 |          // alef
        0x0629 |          // teh marbuta
        0x062F | 0x0630 | // dal, thal
        0x0631 | 0x0632 | // reh, zain
        0x0648 |          // waw
        0x0671..=0x0673 | 0x0675 | // alef wasla and variants
        0x0688..=0x0699 | // dal / reh block variants (all Joining_Type R)
        0x06C0 | 0x06C3..=0x06CB | 0x06CD | 0x06CF |
        0x06D2 | 0x06D3 | // yeh barree
        0x06EE | 0x06EF   // dal / reh with inverted V
    )
}

// ============================================================================
// HEBREW DIACRITICS AND PUNCTUATION
// ============================================================================

/// Check if a code is a Hebrew diacritical mark
///
/// Hebrew diacritics include:
/// - Vowel points (U+05B0-U+05BB): SHEVA, HATAF SEGOL, HOLAM, etc.
/// - Other marks (U+05BC-U+05C7): DAGESH, METEG, RAFE, SHIN DOT, SIN DOT, etc.
///
/// Diacritics should never create word boundaries.
pub fn is_hebrew_diacritic(code: u32) -> bool {
    matches!(code,
        0x05B0..=0x05BB |  // Hebrew vowel points
        0x05BC |           // DAGESH
        0x05BD |           // METEG
        0x05BF |           // RAFE
        0x05C1..=0x05C2 |  // SHIN DOT, SIN DOT
        0x05C4..=0x05C5 |  // UPPER DOT, LOWER DOT
        0x05C7             // QAMATS QATAN
    )
}

/// Check if a code is a Hebrew letter
///
/// Hebrew alphabet: U+05D0-U+05EA (ALEF through TAV)
pub fn is_hebrew_letter(code: u32) -> bool {
    matches!(code, 0x05D0..=0x05EA)
}

/// Check if a code is Hebrew punctuation
///
/// Includes:
/// - GERESH (U+05F3): Used for abbreviations
/// - GERSHAYIM (U+05F4): Used for acronyms and abbreviations
pub fn is_hebrew_punctuation(code: u32) -> bool {
    matches!(code, 0x05F3 | 0x05F4)
}

// ============================================================================
// SHARED DIACRITIC DETECTION
// ============================================================================

/// Check if a code is any RTL diacritical mark (Arabic or Hebrew)
#[inline]
pub fn is_rtl_diacritic(code: u32) -> bool {
    is_arabic_diacritic(code) || is_hebrew_diacritic(code)
}

// ============================================================================
// ARABIC CONTEXTUAL FORMS AND LIGATURES
// ============================================================================

/// Normalize Arabic contextual form to base character
///
/// Arabic letters have multiple presentation forms (isolated, initial, medial, final).
/// This function maps presentation forms back to their base characters.
///
/// Handles:
/// - Presentation Forms-A (U+FB50-U+FDFF)
/// - Presentation Forms-B (U+FE70-U+FEFF)
///
/// Returns the base character if a presentation form, otherwise returns the original code.
pub fn normalize_arabic_contextual_form(code: u32) -> u32 {
    match code {
        // Presentation Forms-A mappings (partial list - common forms)
        0xFB50 => 0x0671, // ALEF WASLA
        0xFE82 => 0x0627, // ALEF FINAL
        0xFE8D => 0x0627, // ALEF ISOLATED
        0xFE8E => 0x0627, // ALEF FINAL

        // Presentation Forms-B mappings (BEH as example)
        0xFE8F => 0x0628, // BEH ISOLATED
        0xFE90 => 0x0628, // BEH FINAL
        0xFE91 => 0x0628, // BEH INITIAL
        0xFE92 => 0x0628, // BEH MEDIAL

        // For full implementation, would need all ~600 mappings
        // For now, if in presentation form range but not mapped, return as-is
        0xFB50..=0xFDFF | 0xFE70..=0xFEFF => {
            // Generic approximation: many presentation forms follow patterns
            // In production, use a complete lookup table
            code
        },

        // Not a presentation form - return unchanged
        _ => code,
    }
}

/// Check if a code is a LAM-ALEF ligature
///
/// LAM-ALEF is a mandatory ligature in Arabic consisting of LAM (ل) + ALEF (ا) or variants.
/// Unicode has dedicated code points for these ligatures:
/// - U+FEFB, U+FEFC: LAM with ALEF
/// - U+FEF5-U+FEFA: LAM with ALEF variants (with MADDA, HAMZA ABOVE, HAMZA BELOW)
pub fn is_lam_alef_ligature(code: u32) -> bool {
    matches!(code, 0xFEF5..=0xFEFC) // LAM-ALEF ligatures (all forms)
}

/// Decompose a LAM-ALEF ligature into its constituent characters
///
/// Returns (LAM, ALEF_VARIANT) if the code is a LAM-ALEF ligature, None otherwise.
pub fn decompose_lam_alef(code: u32) -> Option<(u32, u32)> {
    match code {
        0xFEFB | 0xFEFC => Some((0x0644, 0x0627)), // LAM + ALEF
        0xFEF5 | 0xFEF6 => Some((0x0644, 0x0622)), // LAM + ALEF WITH MADDA ABOVE
        0xFEF7 | 0xFEF8 => Some((0x0644, 0x0623)), // LAM + ALEF WITH HAMZA ABOVE
        0xFEF9 | 0xFEFA => Some((0x0644, 0x0625)), // LAM + ALEF WITH HAMZA BELOW
        _ => None,
    }
}

// ============================================================================
// NUMBER HANDLING
// ============================================================================

/// Check if a code is an Eastern Arabic-Indic digit (٠-٩)
///
/// Eastern Arabic digits: U+06F0-U+06F9
/// These are commonly used in Persian, Urdu, and some Arabic contexts.
pub fn is_eastern_arabic_digit(code: u32) -> bool {
    matches!(code, 0x06F0..=0x06F9)
}

/// Check if a code is a number in RTL context (Western or Eastern Arabic digit)
///
/// Includes:
/// - Western digits (0-9): U+0030-U+0039
/// - Eastern Arabic-Indic digits (٠-٩): U+06F0-U+06F9
///
/// In RTL text, both types of digits may appear and should be kept together.
pub fn is_arabic_number(code: u32) -> bool {
    matches!(code,
        0x0030..=0x0039 |  // Western digits 0-9
        0x06F0..=0x06F9    // Eastern Arabic-Indic digits ٠-٩
    )
}

// ============================================================================
// RTL BOUNDARY DETECTION
// ============================================================================

/// Determine if a word boundary should be created between two characters in RTL context
///
/// Returns:
/// - Some(true): Definitely create a boundary
/// - Some(false): Definitely do NOT create a boundary
/// - None: Not applicable (let other detectors handle)
///
/// # Boundary Rules (in priority order)
///
/// 1. **Space (U+0020)**: Always creates a boundary
/// 2. **TATWEEL (U+0640)**: Never creates a boundary (Arabic kashida for elongation)
/// 3. **Diacritics**: Never create boundaries (must stay with base character)
/// 4. **Multiple marks on same base**: No boundary between marks
/// 5. **TJ offset**: Large negative offset (< -50) in RTL context creates boundary
/// 6. **Script transitions**: RTL-to-LTR or LTR-to-RTL creates boundary
/// 7. **RTL punctuation**: Creates boundary
/// 8. **Number sequences**: No boundaries within digit sequences
/// 9. **Normal letters**: No boundary between consecutive letters of same script
///
/// # Arguments
///
/// * `prev_char` - The previous character
/// * `curr_char` - The current character
/// * `context` - Optional boundary context (unused for RTL currently)
pub fn should_split_at_rtl_boundary(
    prev_char: &CharacterInfo,
    curr_char: &CharacterInfo,
    _context: Option<&BoundaryContext>,
) -> Option<bool> {
    let prev_code = prev_char.code;
    let curr_code = curr_char.code;

    let prev_is_rtl = is_rtl_text(prev_code);
    let curr_is_rtl = is_rtl_text(curr_code);

    // Rule 1: Space always creates boundary
    if curr_code == 0x0020 || prev_code == 0x0020 {
        return Some(true);
    }

    // Rule 8 (early): Number sequences - no boundaries between digits
    // This must come before the RTL check because Western digits (0-9) are not RTL
    if is_arabic_number(prev_code) && is_arabic_number(curr_code) {
        return Some(false);
    }

    // If neither character is RTL (and not numbers), return None (not our concern)
    if !prev_is_rtl && !curr_is_rtl {
        return None;
    }

    // Rule 2: TATWEEL (Arabic kashida) never creates boundary
    if curr_code == 0x0640 || prev_code == 0x0640 {
        return Some(false);
    }

    // Rule 3: Diacritical marks never create boundaries
    if is_rtl_diacritic(curr_code) {
        return Some(false);
    }

    // Rule 4: Multiple marks on same base (prev is also a mark)
    if is_rtl_diacritic(prev_code) && is_rtl_diacritic(curr_code) {
        return Some(false);
    }

    // Rule 5: TJ offset - large negative offset in RTL creates boundary
    if let Some(tj_offset) = prev_char.tj_offset {
        if tj_offset < -50 {
            return Some(true);
        }
    }

    // Rule 6: RTL-to-LTR or LTR-to-RTL transitions create boundary
    // But skip if both are numbers (already handled above)
    if prev_is_rtl != curr_is_rtl && !(is_arabic_number(prev_code) && is_arabic_number(curr_code)) {
        return Some(true);
    }

    // Rule 7: RTL punctuation creates boundary
    if is_arabic_punctuation(curr_code) || is_hebrew_punctuation(curr_code) {
        return Some(true);
    }

    // Rule 9: Normal letter sequences - no boundary
    if (is_arabic_letter(prev_code)
        || is_arabic_letter(normalize_arabic_contextual_form(prev_code)))
        && (is_arabic_letter(curr_code)
            || is_arabic_letter(normalize_arabic_contextual_form(curr_code)))
    {
        return Some(false);
    }

    if is_hebrew_letter(prev_code) && is_hebrew_letter(curr_code) {
        return Some(false);
    }

    // Both are RTL but fall through all rules - assume no boundary for same-script RTL
    if prev_is_rtl && curr_is_rtl {
        return Some(false);
    }

    // Shouldn't reach here, but return None as fallback
    None
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Check if a code is Arabic punctuation
///
/// Common Arabic punctuation marks that should create word boundaries.
fn is_arabic_punctuation(code: u32) -> bool {
    matches!(
        code,
        0x060C |  // ARABIC COMMA
        0x061B |  // ARABIC SEMICOLON
        0x061F |  // ARABIC QUESTION MARK
        0x066A |  // ARABIC PERCENT SIGN
        0x066B |  // ARABIC DECIMAL SEPARATOR
        0x066C |  // ARABIC THOUSANDS SEPARATOR
        0x066D // ARABIC FIVE POINTED STAR (paragraph mark)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_script_detection() {
        assert_eq!(detect_rtl_script(0x0627), Some(RTLScript::Arabic)); // ALEF
        assert_eq!(detect_rtl_script(0x05D0), Some(RTLScript::Hebrew)); // ALEF
        assert_eq!(detect_rtl_script(0x0041), None); // Latin 'A'
    }

    #[test]
    fn test_right_joining_arabic() {
        // Joining_Type = R (right-joining only): alef, dal, thal, reh, zain,
        // waw, teh marbuta.
        for r in [
            0x0627, 0x0622, 0x0623, 0x0625, 0x062F, 0x0630, 0x0631, 0x0632, 0x0648, 0x0629,
        ] {
            assert!(is_right_joining_arabic(r), "{r:#06X} should be right-joining");
        }
        // Dual-joining letters (beh, teh, lam, qaf, …) and yeh-hamza are NOT R.
        for d in [0x0628, 0x062A, 0x0644, 0x0642, 0x0639, 0x0626] {
            assert!(!is_right_joining_arabic(d), "{d:#06X} should be dual-joining");
        }
        // Non-Arabic is never right-joining.
        assert!(!is_right_joining_arabic(0x05D0)); // Hebrew alef
        assert!(!is_right_joining_arabic(0x0041)); // Latin 'A'
    }

    #[test]
    fn test_basic_diacritic_detection() {
        assert!(is_arabic_diacritic(0x064E)); // FATHA
        assert!(is_hebrew_diacritic(0x05BC)); // DAGESH
        assert!(!is_arabic_diacritic(0x0627)); // ALEF (letter)
    }

    #[test]
    fn test_basic_letter_detection() {
        assert!(is_arabic_letter(0x0628)); // BEH
        assert!(is_hebrew_letter(0x05D1)); // BET
        assert!(!is_arabic_letter(0x064B)); // FATHATAN (diacritic)
    }

    #[test]
    fn test_lam_alef_basic() {
        assert!(is_lam_alef_ligature(0xFEFC));
        assert_eq!(decompose_lam_alef(0xFEFC), Some((0x0644, 0x0627)));
    }
}
