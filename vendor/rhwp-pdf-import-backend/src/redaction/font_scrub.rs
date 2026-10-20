//! Font scrubbing for destructive redaction (#231, T9 — completes
//! guarantee G2: no width/shift side channel).
//!
//! Removing the glyph *shows* (text_prune) is not enough. Bland et al.
//! (PETS 2023) reconstruct redacted words from a font's `/Widths`
//! array, `/MissingWidth` and `/ToUnicode` CMap even after the visible
//! glyphs are gone: a per-code advance or a code→Unicode mapping that
//! exists *only* for a redacted glyph still leaks it.
//!
//! So for every glyph code that was removed and is **not** used by any
//! surviving run, this module (feature plan §4 / G2):
//!
//! - resets its `/Widths` entry to `/MissingWidth` (erasing the
//!   per-glyph advance fingerprint), and
//! - drops its `/ToUnicode` mapping.
//!
//! Codes still used by surviving text keep their width and mapping (we
//! must not break visible glyphs). This is the pure transform over an
//! abstract per-font model so the security-critical logic is
//! exhaustively unit-testable in isolation; wiring it to real font
//! dictionaries / font cloning for shared fonts is the integration step
//! (plan T8/T11). It performs no I/O and cannot itself under-redact.

use std::collections::{BTreeMap, BTreeSet};

/// One font's redaction-relevant state. `widths[i]` is the advance for
/// glyph code `first_char + i` (PDF `/Widths` semantics); codes outside
/// `first_char .. first_char + widths.len()` use `missing_width`.
#[derive(Debug, Clone, PartialEq)]
pub struct FontScrubInput {
    /// `/FirstChar` — code of `widths[0]`.
    pub first_char: u32,
    /// `/Widths` array (advances, glyph-space /1000 units as stored).
    pub widths: Vec<f32>,
    /// `/MissingWidth` (default `0.0`) — the neutral advance a scrubbed
    /// code is reset to so it no longer fingerprints the removed glyph.
    pub missing_width: f32,
    /// `/ToUnicode` mappings: glyph code → UTF-8 text.
    pub to_unicode: BTreeMap<u32, String>,
    /// Glyph codes removed by text pruning (this resource).
    pub removed_codes: BTreeSet<u32>,
    /// Glyph codes still used by at least one *surviving* run.
    pub surviving_codes: BTreeSet<u32>,
}

/// The scrubbed font metrics.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FontScrubResult {
    /// `/Widths` with redaction-only codes reset to `missing_width`.
    pub widths: Vec<f32>,
    /// `/ToUnicode` with redaction-only mappings dropped.
    pub to_unicode: BTreeMap<u32, String>,
    /// Distinct codes actually scrubbed (removed and not surviving).
    pub codes_scrubbed: usize,
}

/// A code is *redaction-only* — safe to scrub — iff it was removed and
/// no surviving run still uses it.
fn is_redaction_only(code: u32, input: &FontScrubInput) -> bool {
    input.removed_codes.contains(&code) && !input.surviving_codes.contains(&code)
}

/// Scrub one font: reset redaction-only `/Widths` entries to
/// `/MissingWidth` and drop their `/ToUnicode` mappings. Codes still
/// used by surviving text are left untouched (must not break visible
/// glyphs — G2 conservative on the *surviving* side).
pub fn scrub_font(input: &FontScrubInput) -> FontScrubResult {
    let mut scrubbed: BTreeSet<u32> = BTreeSet::new();

    let mut widths = input.widths.clone();
    for (i, w) in widths.iter_mut().enumerate() {
        let code = input.first_char + i as u32;
        if is_redaction_only(code, input) {
            *w = input.missing_width;
            scrubbed.insert(code);
        }
    }

    let mut to_unicode = BTreeMap::new();
    for (&code, text) in &input.to_unicode {
        if is_redaction_only(code, input) {
            scrubbed.insert(code);
        } else {
            to_unicode.insert(code, text.clone());
        }
    }

    FontScrubResult {
        widths,
        to_unicode,
        codes_scrubbed: scrubbed.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tu(pairs: &[(u32, &str)]) -> BTreeMap<u32, String> {
        pairs.iter().map(|(c, s)| (*c, (*s).to_string())).collect()
    }

    fn codes(cs: &[u32]) -> BTreeSet<u32> {
        cs.iter().copied().collect()
    }

    #[test]
    fn redaction_only_code_is_scrubbed() {
        // FirstChar 65 ('A'). Code 66 ('B') redacted and not surviving.
        let input = FontScrubInput {
            first_char: 65,
            widths: vec![500.0, 600.0, 700.0], // A=500 B=600 C=700
            missing_width: 0.0,
            to_unicode: tu(&[(65, "A"), (66, "B"), (67, "C")]),
            removed_codes: codes(&[66]),
            surviving_codes: codes(&[65, 67]),
        };
        let out = scrub_font(&input);
        assert_eq!(out.codes_scrubbed, 1);
        // B's width reset to MissingWidth; A,C intact
        assert_eq!(out.widths, vec![500.0, 0.0, 700.0]);
        // B's ToUnicode dropped; A,C kept
        assert_eq!(out.to_unicode, tu(&[(65, "A"), (67, "C")]));
    }

    #[test]
    fn code_removed_but_also_surviving_is_kept() {
        // Same code appears in a redacted run AND a surviving run — must
        // not break the visible occurrence.
        let input = FontScrubInput {
            first_char: 0,
            widths: vec![400.0, 450.0],
            missing_width: 0.0,
            to_unicode: tu(&[(0, "x"), (1, "y")]),
            removed_codes: codes(&[0, 1]),
            surviving_codes: codes(&[1]), // code 1 still used somewhere
        };
        let out = scrub_font(&input);
        assert_eq!(out.codes_scrubbed, 1); // only code 0
        assert_eq!(out.widths, vec![0.0, 450.0]);
        assert_eq!(out.to_unicode, tu(&[(1, "y")]));
    }

    #[test]
    fn untouched_when_nothing_removed() {
        let input = FontScrubInput {
            first_char: 32,
            widths: vec![250.0, 333.0],
            missing_width: 0.0,
            to_unicode: tu(&[(32, " "), (33, "!")]),
            removed_codes: codes(&[]),
            surviving_codes: codes(&[32, 33]),
        };
        let out = scrub_font(&input);
        assert_eq!(out.codes_scrubbed, 0);
        assert_eq!(out.widths, input.widths);
        assert_eq!(out.to_unicode, input.to_unicode);
    }

    #[test]
    fn missing_width_value_is_used_for_reset() {
        let input = FontScrubInput {
            first_char: 10,
            widths: vec![900.0],
            missing_width: 42.0, // non-zero MissingWidth
            to_unicode: tu(&[(10, "Z")]),
            removed_codes: codes(&[10]),
            surviving_codes: codes(&[]),
        };
        let out = scrub_font(&input);
        assert_eq!(out.codes_scrubbed, 1);
        assert_eq!(out.widths, vec![42.0]);
        assert!(out.to_unicode.is_empty());
    }

    #[test]
    fn tounicode_only_code_outside_widths_range_still_scrubbed() {
        // A redaction-only code present only in /ToUnicode (e.g. CID
        // font, no Widths entry for it) must still lose its mapping.
        let input = FontScrubInput {
            first_char: 0,
            widths: vec![300.0],
            missing_width: 0.0,
            to_unicode: tu(&[(0, "a"), (999, "secret")]),
            removed_codes: codes(&[999]),
            surviving_codes: codes(&[0]),
        };
        let out = scrub_font(&input);
        assert_eq!(out.codes_scrubbed, 1);
        assert_eq!(out.widths, vec![300.0]); // code 0 untouched
        assert_eq!(out.to_unicode, tu(&[(0, "a")]));
    }

    #[test]
    fn empty_input_is_empty_result() {
        let input = FontScrubInput {
            first_char: 0,
            widths: vec![],
            missing_width: 0.0,
            to_unicode: BTreeMap::new(),
            removed_codes: BTreeSet::new(),
            surviving_codes: BTreeSet::new(),
        };
        assert_eq!(scrub_font(&input), FontScrubResult::default());
    }

    #[test]
    fn scrubbed_count_dedups_widths_and_tounicode() {
        // Code 5 is redaction-only and appears in BOTH widths range and
        // ToUnicode — counted once.
        let input = FontScrubInput {
            first_char: 5,
            widths: vec![700.0],
            missing_width: 0.0,
            to_unicode: tu(&[(5, "s")]),
            removed_codes: codes(&[5]),
            surviving_codes: codes(&[]),
        };
        let out = scrub_font(&input);
        assert_eq!(out.codes_scrubbed, 1);
    }
}
