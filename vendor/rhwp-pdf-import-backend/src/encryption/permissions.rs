//! PDF §7.6.3.2 `/P` permission flag set.
//!
//! `PdfDocument::permissions()` returns this struct when the document
//! is encrypted. Per PDF spec §7.6.3.2 Table 22:
//!
//! | Bit | Meaning |
//! |---|---|
//! | 3 | print (low resolution) |
//! | 4 | modify (other than annotation / form fill) |
//! | 5 | copy text and graphics |
//! | 6 | annotate and form-fill |
//! | 9 | fill forms (rev ≥ 3) |
//! | 10 | accessibility extract (rev ≥ 3) |
//! | 11 | assemble (rev ≥ 3) |
//! | 12 | print high resolution (rev ≥ 3) |
//!
//! Per PDF spec language, `/P` permissions are **advisory** — readers
//! shall not enforce them. pdf_oxide surfaces them so callers who want
//! to enforce can do so themselves.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Decoded `/P` permission flags from a PDF's standard encryption
/// dictionary (§7.6.3.2 Table 22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfPermissions {
    /// Bit 3: print (low resolution).
    pub print_low_res: bool,
    /// Bit 4: modify (other than annotation / form fill).
    pub modify: bool,
    /// Bit 5: copy text and graphics.
    pub copy: bool,
    /// Bit 6: annotate and form-fill.
    pub annotate: bool,
    /// Bit 9 (rev ≥ 3): fill interactive form fields.
    pub fill_forms: bool,
    /// Bit 10 (rev ≥ 3): extract for accessibility.
    pub accessibility: bool,
    /// Bit 11 (rev ≥ 3): assemble (insert/rotate/delete pages).
    pub assemble: bool,
    /// Bit 12 (rev ≥ 3): print high resolution.
    pub print_high_res: bool,
    /// Raw `/P` integer for callers that need the pre-decoded value.
    /// PDF uses two's-complement int32 with bits 13–32 reserved.
    pub raw_p: i32,
}

impl PdfPermissions {
    /// Decode a `/P` flag integer per spec §7.6.3.2 Table 22.
    ///
    /// Note: bit positions in the spec are 1-indexed; this code uses
    /// 0-indexed shifts. So "bit 3" in spec = shift `1 << 2` here.
    pub fn from_p_flag(p: i32) -> Self {
        Self {
            print_low_res: (p & (1 << 2)) != 0,
            modify: (p & (1 << 3)) != 0,
            copy: (p & (1 << 4)) != 0,
            annotate: (p & (1 << 5)) != 0,
            fill_forms: (p & (1 << 8)) != 0,
            accessibility: (p & (1 << 9)) != 0,
            assemble: (p & (1 << 10)) != 0,
            print_high_res: (p & (1 << 11)) != 0,
            raw_p: p,
        }
    }

    /// "All allowed" — convenience for the no-restrictions case where
    /// `/P = -1` (every bit set; common for unencrypted-but-tagged
    /// documents).
    pub fn all_allowed() -> Self {
        Self::from_p_flag(-1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_p_flag_all_bits_set() {
        // /P = -1 means every bit set
        let p = PdfPermissions::from_p_flag(-1);
        assert!(p.print_low_res);
        assert!(p.modify);
        assert!(p.copy);
        assert!(p.annotate);
        assert!(p.fill_forms);
        assert!(p.accessibility);
        assert!(p.assemble);
        assert!(p.print_high_res);
        assert_eq!(p.raw_p, -1);
    }

    #[test]
    fn from_p_flag_typical_restrictive() {
        // /P = -3904 (-1 minus bits 3, 5, 6, 9, 10, 11 set to 0) — common
        // "no print, no copy, no annotate, no forms, no accessibility,
        // no assemble" pattern.
        // Compute the expected /P: start from -1 (all set), clear the
        // forbidden bits.
        let mut p: i32 = -1;
        p &= !(1 << 2); // clear print_low_res (bit 3)
        p &= !(1 << 4); // clear copy (bit 5)
        p &= !(1 << 5); // clear annotate (bit 6)

        let perms = PdfPermissions::from_p_flag(p);
        assert!(!perms.print_low_res);
        assert!(!perms.copy);
        assert!(!perms.annotate);
        assert!(perms.modify); // still set
        assert_eq!(perms.raw_p, p);
    }

    #[test]
    fn from_p_flag_kreuzberg_password_protected_shape() {
        // kreuzberg's password_protected.pdf reports
        // `print:no copy:no change:no addNotes:no` per pdfinfo
        // — that's bits 3, 4, 5, 6 all cleared.
        let mut p: i32 = -1;
        p &= !(1 << 2); // clear print
        p &= !(1 << 3); // clear modify (change)
        p &= !(1 << 4); // clear copy
        p &= !(1 << 5); // clear annotate (addNotes)

        let perms = PdfPermissions::from_p_flag(p);
        assert!(!perms.print_low_res);
        assert!(!perms.modify);
        assert!(!perms.copy);
        assert!(!perms.annotate);
        // rev-3+ bits left untouched
        assert!(perms.fill_forms);
        assert!(perms.accessibility);
        assert!(perms.assemble);
        assert!(perms.print_high_res);
    }

    #[test]
    fn from_p_flag_zero_is_all_denied() {
        let perms = PdfPermissions::from_p_flag(0);
        assert!(!perms.print_low_res);
        assert!(!perms.modify);
        assert!(!perms.copy);
        assert!(!perms.annotate);
        assert!(!perms.fill_forms);
        assert!(!perms.accessibility);
        assert!(!perms.assemble);
        assert!(!perms.print_high_res);
        assert_eq!(perms.raw_p, 0);
    }

    #[test]
    fn all_allowed_matches_minus_one() {
        let allowed = PdfPermissions::all_allowed();
        let from_minus_one = PdfPermissions::from_p_flag(-1);
        assert_eq!(allowed, from_minus_one);
    }

    #[test]
    fn serializes_to_json() {
        let p = PdfPermissions::all_allowed();
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"print_low_res\":true"));
        assert!(json.contains("\"raw_p\":-1"));
    }
}
