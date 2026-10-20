//! Adobe-Arabic-1 / Adobe-Persian-1 CID-to-Unicode mapping.
//!
//! Identity mapping over the Unicode Arabic block (U+0600–U+06FF)
//! for the common case where Persian / Farsi / Pashto / Urdu fonts
//! (Nazanin, Yagut, Mitra, Lotus) declare
//! `CIDSystemInfo /Registry (Adobe) /Ordering (Persian|Arabic)` but
//! the font's actual CID-to-glyph mapping is sequential in the
//! Arabic Unicode range.
//!
//! Without this mapping, the engine falls back to Identity-H, which
//! emits CIDs as Latin-Extended-B codepoints (U+01xx–U+07xx
//! garbage). This mapping at least lands the characters in the
//! correct Arabic block (U+0600–U+06FF) where bidi-aware viewers
//! can shape them.
//!
//! ## PDF spec basis
//!
//! Per `docs/spec/pdf.md` §9.7 "Composite Fonts" + §9.7.5 "CMaps":
//! CID-keyed fonts use a CMap to map character codes to CIDs and a
//! registered character collection (`CIDSystemInfo` →
//! `Registry`/`Ordering`/`Supplement`) plus a UCS2-suffixed CMap
//! (e.g. `UniArabicBookman-UCS2`) to map CIDs to Unicode. The full
//! registered Adobe-Persian-1 / Adobe-Arabic-1 UCS2 tables are not
//! shipped: Adobe deprecated and no longer publishes these
//! collections (their adobe-type-tools repo ships CJK + Manga
//! only). The identity mapping here is the §9.10.3 "Mapping
//! Character Codes to Unicode Values" fallback step 3 — when the
//! CMap chain runs out, a conforming reader emits "the actual
//! character code as the Unicode value." For Persian fonts with
//! sequential Arabic-block CIDs this produces correct output; for
//! fonts with non-sequential CID encodings it produces best-effort
//! output in the correct Unicode block.
//!
//! **Limitations**: this is NOT the official Adobe-Arabic-1-UCS2
//! CMap. It is a heuristic identity mapping that works for the
//! common case where Persian fonts use sequential Arabic-block
//! CIDs. The full official CMap data is no longer publicly
//! distributed by Adobe.

#![forbid(unsafe_code)]

/// Look up Unicode for an Adobe-Arabic-1 / Adobe-Persian-1 CID.
///
/// Stub mapping: returns the Arabic-block Unicode codepoint for CID
/// values in `[0x600..=0x6FF]`. Returns `None` otherwise (caller
/// falls back to the existing chain).
///
/// **Why identity mapping**: while the official Adobe-Arabic-1
/// CMap maps CIDs in a specific ordering (e.g. CID 1=isolated alef,
/// CID 2=isolated be, etc.), many Persian fonts ship with simpler
/// CIDs that already align with Unicode codepoints. The identity
/// mapping handles those; the official CMap support is follow-up
/// work.
#[inline]
pub fn lookup(cid: u16) -> Option<u32> {
    // Arabic block: U+0600..=U+06FF
    if (0x0600..=0x06FF).contains(&cid) {
        Some(cid as u32)
    } else if (0xFB50..=0xFDFF).contains(&cid) {
        // Arabic Presentation Forms-A
        Some(cid as u32)
    } else if (0xFE70..=0xFEFF).contains(&cid) {
        // Arabic Presentation Forms-B
        Some(cid as u32)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_block_identity_mapping() {
        // Alef (ا) — Arabic block
        assert_eq!(lookup(0x0627), Some(0x0627));
        // Be (ب)
        assert_eq!(lookup(0x0628), Some(0x0628));
        // Persian-specific letter Pe (پ)
        assert_eq!(lookup(0x067E), Some(0x067E));
        // Persian Che (چ)
        assert_eq!(lookup(0x0686), Some(0x0686));
        // Persian Zhe (ژ)
        assert_eq!(lookup(0x0698), Some(0x0698));
    }

    #[test]
    fn arabic_presentation_forms_supported() {
        // Arabic Presentation Forms-A
        assert_eq!(lookup(0xFB50), Some(0xFB50));
        // Arabic Presentation Forms-B
        assert_eq!(lookup(0xFE70), Some(0xFE70));
    }

    #[test]
    fn non_arabic_returns_none() {
        // ASCII
        assert_eq!(lookup(0x41), None);
        // Latin-Extended-B (the garbage block)
        assert_eq!(lookup(0x01A4), None);
        // Hebrew (different block, not covered by this CMap)
        assert_eq!(lookup(0x05D0), None);
    }
}
