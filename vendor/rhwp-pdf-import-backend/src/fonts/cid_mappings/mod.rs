//! CID to Unicode mappings for predefined Adobe character collections.
//!
//! This module provides CID (Character Identifier) to Unicode mappings for the
//! standard Adobe CJK character collections used in PDF documents.
//!
//! Per PDF Spec ISO 32000-1:2008 Section 9.7.5.2, these predefined CMaps map
//! CIDs from specific character collections to Unicode code points.
//!
//! # Supported Character Collections
//!
//! - **Adobe-GB1**: Simplified Chinese (GB 2312 + extensions)
//! - **Adobe-Japan1**: Japanese (JIS X 0208, JIS X 0212)
//! - **Adobe-CNS1**: Traditional Chinese (CNS 11643)
//! - **Adobe-Korea1**: Korean (KS X 1001)
//!
//! # References
//!
//! - Adobe Technical Note #5078: Adobe-Japan1-7
//! - Adobe Technical Note #5079: Adobe-GB1-5
//! - Adobe Technical Note #5080: Adobe-CNS1-7
//! - Adobe Technical Note #5093: Adobe-Korea1-2
//!
//! # Implementation Notes
//!
//! This module uses `phf_map!` for O(1) CID-to-Unicode lookup, following the
//! same pattern as `adobe_glyph_list.rs`.

mod adobe_arabic;
mod adobe_cns1;
mod adobe_gb1;
mod adobe_japan1;
mod adobe_korea1;

/// Look up Unicode code point for a CID in Adobe-GB1 (Simplified Chinese).
///
/// This mapping corresponds to the UniGB-UCS2-H CMap.
///
/// # Arguments
///
/// * `cid` - Character Identifier (0-29063 for Adobe-GB1-5)
///
/// # Returns
///
/// The corresponding Unicode code point, or None if not mapped.
#[inline]
pub fn lookup_adobe_gb1(cid: u16) -> Option<u32> {
    adobe_gb1::lookup(cid)
}

/// Look up Unicode code point for a CID in Adobe-Japan1 (Japanese).
///
/// This mapping corresponds to the UniJIS-UCS2-H CMap.
///
/// # Arguments
///
/// * `cid` - Character Identifier (0-23057 for Adobe-Japan1-7)
///
/// # Returns
///
/// The corresponding Unicode code point, or None if not mapped.
#[inline]
pub fn lookup_adobe_japan1(cid: u16) -> Option<u32> {
    adobe_japan1::lookup(cid)
}

/// Look up Unicode code point for a CID in Adobe-CNS1 (Traditional Chinese).
///
/// This mapping corresponds to the UniCNS-UCS2-H CMap.
///
/// # Arguments
///
/// * `cid` - Character Identifier (0-19155 for Adobe-CNS1-7)
///
/// # Returns
///
/// The corresponding Unicode code point, or None if not mapped.
#[inline]
pub fn lookup_adobe_cns1(cid: u16) -> Option<u32> {
    adobe_cns1::lookup(cid)
}

/// Look up Unicode code point for a CID in Adobe-Korea1 (Korean).
///
/// This mapping corresponds to the UniKS-UCS2-H CMap.
///
/// # Arguments
///
/// * `cid` - Character Identifier (0-18351 for Adobe-Korea1-2)
///
/// # Returns
///
/// The corresponding Unicode code point, or None if not mapped.
#[inline]
pub fn lookup_adobe_korea1(cid: u16) -> Option<u32> {
    adobe_korea1::lookup(cid)
}

/// look up Unicode code point for a CID in
/// Adobe-Arabic-1 / Adobe-Persian-1 (used by Persian / Farsi /
/// Pashto / Urdu fonts that ship without ToUnicode CMaps).
///
/// Stub implementation: identity mapping for the Arabic block
/// (U+0600–U+06FF) and Arabic Presentation Forms (U+FB50–U+FDFF +
/// U+FE70–U+FEFF). The official Adobe-Arabic-1-UCS2 CMap is
/// follow-up work.
///
/// Returns the Unicode code point, or `None` if the CID isn't in
/// the supported range (caller falls back to the existing chain).
///
/// # Arguments
///
/// * `cid` - Character Identifier
#[inline]
pub fn lookup_adobe_arabic(cid: u16) -> Option<u32> {
    adobe_arabic::lookup(cid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adobe_gb1_ascii_from_cid() {
        // CID 34 maps to 'A' (U+0041), CID 91 maps to 'z' (U+007A)
        // Note: CIDs are indices in the character collection, not Unicode values
        assert_eq!(lookup_adobe_gb1(34), Some(0x41)); // CID 34 -> A
        assert_eq!(lookup_adobe_gb1(91), Some(0x7A)); // CID 91 -> z
    }

    #[test]
    fn test_adobe_japan1_ascii_from_cid() {
        // CID 34 maps to 'A' (U+0041)
        assert_eq!(lookup_adobe_japan1(34), Some(0x41)); // CID 34 -> A
        assert_eq!(lookup_adobe_japan1(91), Some(0x7A)); // CID 91 -> z
    }

    #[test]
    fn test_adobe_japan1_hiragana() {
        // Test Hiragana from CID
        assert_eq!(lookup_adobe_japan1(843), Some(0x3042)); // CID 843 -> あ
    }

    #[test]
    fn test_adobe_cns1_ascii_from_cid() {
        // CID 34 maps to 'A' (U+0041)
        assert_eq!(lookup_adobe_cns1(34), Some(0x41)); // CID 34 -> A
    }

    #[test]
    fn test_adobe_korea1_ascii_from_cid() {
        // CID 34 maps to 'A' (U+0041)
        assert_eq!(lookup_adobe_korea1(34), Some(0x41)); // CID 34 -> A
    }

    #[test]
    fn test_adobe_korea1_hangul() {
        // Test Hangul syllable from CID (from Adobe cid2code.txt)
        assert_eq!(lookup_adobe_korea1(1086), Some(0xAC00)); // CID 1086 -> 가
    }
}
