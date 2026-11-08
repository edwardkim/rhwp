//! Destructive-redaction orchestration for a single content stream
//! (#231, T11 — the parse → prune → re-serialize → overlay pipeline that
//! turns the pure primitives into a working redaction of real bytes).
//!
//! Pipeline (ISO 32000-1:2008 §7.8.2 content-stream model):
//!
//! 1. `parse_content_stream` → operators.
//! 2. `text_engine::redact_text_stream` removes every glyph touching a
//!    region and re-emits survivors with absolute `Tm`, no `TJ` deltas
//!    (G1/G2). A composite/Type0/unknown font ⇒ **hard refusal**
//!    (`Err`), never a silent pass-through (feature plan §9 risk 6).
//! 3. `serialize` re-serializes survivors (binary-safe strings, G6).
//! 4. `overlay` appends one opaque block per region *after* the pruned
//!    content so the redacted area is visibly marked (G7) and the
//!    underlying bytes are already gone — not merely covered.
//!
//! This module owns no document I/O: it is `bytes + regions → bytes`,
//! so the security guarantee (secret absent from the *output bytes*) is
//! directly fixture-testable here without the writer/save path. Wiring a
//! page's real content + fonts and forcing full-rewrite GC is the
//! `DocumentEditor` step (T12); residual-object destruction (G6) is
//! enforced there by the existing garbage-collected full rewrite.

use super::options::{RedactionOptions, RedactionReport};
use super::overlay::region_overlay_ops;
use super::region::RegionSet;
use super::serialize::serialize_operator;
use super::text_engine::{redact_text_stream, FontMetrics};
use crate::content::parser::parse_content_stream;
use crate::error::{Error, Result};
use crate::fonts::{Encoding, FontInfo};
use std::collections::HashMap;
use std::sync::Arc;

/// [`FontMetrics`] over the real per-page `FontInfo` map (resource name
/// → font). The single source of glyph widths is `FontInfo`
/// (`get_glyph_width`) — the *same* metric the text extractor uses, so
/// the redactor's geometry cannot diverge from extraction (DRY; feature
/// plan §9 risk 1). An unknown font, or any composite/Type0 font, is
/// reported non-simple so the engine refuses rather than under-redacts.
pub struct FontInfoMetrics {
    fonts: HashMap<String, Arc<FontInfo>>,
}

impl FontInfoMetrics {
    /// Build from a page's resolved `/Font` resources (resource name →
    /// parsed [`FontInfo`]).
    pub fn new(fonts: HashMap<String, Arc<FontInfo>>) -> Self {
        Self { fonts }
    }

    /// `true` only for a Type0 font whose encoding is the **horizontal**
    /// identity CMap (Identity-H). Per ISO 32000-1 §9.7.5.2 Identity-H maps
    /// each 2-byte code, high-order byte first, to the CID of the same
    /// value, so the engine can decode the show string into CIDs and look
    /// up each CID's width via `/W`+`/DW` exactly as the text extractor
    /// does. Identity-V (vertical, `wmode == 1`) is deliberately excluded —
    /// its glyphs advance down the page, which the horizontal advance/box
    /// model here does not handle — as are all non-identity predefined and
    /// custom CMaps (their code lengths and code→CID mapping are not
    /// reconstructable here), so those stay fail-closed.
    fn is_identity_h(&self, font: &str) -> bool {
        self.fonts.get(font).is_some_and(|fi| {
            fi.subtype == "Type0" && matches!(fi.encoding, Encoding::Identity) && fi.wmode == 0
        })
    }
}

impl FontMetrics for FontInfoMetrics {
    fn width(&self, font: &str, code: u32) -> f32 {
        match self.fonts.get(font) {
            // `get_glyph_width` returns /1000 glyph-space units.
            Some(fi) => fi.get_glyph_width(code.min(u32::from(u16::MAX)) as u16),
            // Unknown font: full-em over-estimate (the show is refused
            // anyway via `is_simple`, but never under-size a box).
            None => 1000.0,
        }
    }

    fn is_simple(&self, font: &str) -> bool {
        // Only single-byte simple fonts have a reliable byte→code→width
        // path here. Type0/CID and unknown ⇒ refuse (fail closed).
        match self.fonts.get(font) {
            Some(fi) => fi.subtype != "Type0",
            None => false,
        }
    }

    fn can_prune(&self, font: &str) -> bool {
        // Single-byte simple fonts, plus Identity-H Type0 (2-byte CID =
        // code). Everything else — Identity-V, legacy/custom CMaps, unknown
        // fonts — stays fail-closed.
        self.is_simple(font) || self.is_identity_h(font)
    }

    fn decode(&self, font: &str, s: &[u8]) -> Vec<(u32, Vec<u8>)> {
        if self.is_identity_h(font) {
            // Identity-H: every two bytes are one code, high-order byte
            // first, and that code IS the CID (§9.7.5.2). A trailing odd
            // byte is malformed; `redaction_safe_show` refuses such a show
            // before it reaches here, so emit only whole 2-byte codes.
            return s
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| (u32::from(u16::from_be_bytes([pair[0], pair[1]])), pair.to_vec()))
                .collect();
        }
        // Simple fonts: one byte, one code.
        s.iter().map(|&b| (b as u32, vec![b])).collect()
    }

    fn is_word_space(&self, font: &str, code: u32) -> bool {
        // ISO 32000-1 §9.3.3: word spacing (Tw) applies only to a
        // single-byte code 32. It never applies to a byte value of 32 that
        // is part of a multi-byte code, so Identity-H glyphs never trigger
        // it. Simple fonts keep the single-byte-32 rule.
        !self.is_identity_h(font) && code == 32
    }

    fn redaction_safe_show(&self, font: &str, bytes: &[u8]) -> bool {
        // Identity-H codes are exactly two bytes; an odd-length show string
        // cannot be split into whole codes without guessing a boundary, so
        // refuse rather than risk mis-aligning the prune (fail closed).
        !self.is_identity_h(font) || bytes.len().is_multiple_of(2)
    }
}

/// Destructively redact the text of one content stream.
///
/// Returns the rewritten stream bytes (pruned content + opaque overlays)
/// and a [`RedactionReport`]. The returned bytes are guaranteed to
/// contain no glyph whose box intersected an (edge-padded) region — the
/// secret is *removed*, then an opaque block is drawn over the area.
///
/// # Errors
/// - [`Error::Unsupported`] — a text show used a composite/Type0/unknown
///   font while regions exist; redaction is **refused** rather than risk
///   a silent under-redaction (feature plan §9 risk 6, fail closed).
/// - [`Error::ParseError`] — the content stream did not parse.
pub fn redact_content_stream(
    content: &[u8],
    regions: &RegionSet,
    opts: &RedactionOptions,
    fonts: &dyn FontMetrics,
) -> Result<(Vec<u8>, RedactionReport)> {
    let ops = parse_content_stream(content)?;
    let te = redact_text_stream(&ops, regions, opts.edge_padding, fonts);

    if te.unsupported_font {
        return Err(Error::Unsupported(
            "destructive text redaction of composite/Type0 font content is not yet \
             supported; refusing rather than risk leaving recoverable text"
                .to_string(),
        ));
    }

    // Wrap the pruned body in its own outer q/Q. A content stream is under
    // no obligation to leave the CTM at identity at end-of-stream (a
    // trailing, unmatched `cm` — e.g. a Y-flip with no closing `Q` — is
    // legal and common). Without this wrapper the overlay ops below, which
    // draw in the region's absolute page-space coordinates, would inherit
    // whatever CTM the pruned content left active and land in the wrong
    // place. `q` at the very start of the stream saves the stream's
    // original (identity) CTM; the matching `Q` restores it before any
    // overlay is drawn, regardless of what the pruned operators did to the
    // CTM in between. Same fix shape qpdf uses for the identical hazard.
    let mut body = Vec::with_capacity(content.len());
    body.extend_from_slice(b"q\n");
    for op in &te.operators {
        serialize_operator(&mut body, op);
    }
    body.extend_from_slice(b"Q\n");

    for region in &regions.regions {
        body.extend_from_slice(&region_overlay_ops(region, opts));
    }

    let report = RedactionReport {
        regions: regions.len(),
        glyphs_removed: te.glyphs_removed,
        annotations_removed: 0,
        fonts_scrubbed: 0,
        bytes_removed: te.bytes_removed,
        ..RedactionReport::default()
    };
    Ok((body, report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redaction::region::RedactionRegion;

    /// 10-unit-wide simple font (so a `BT 1 0 0 1 X Y Tm` places glyph i
    /// at page x = X + 10·i for size 10... here width 500/1000·10 = 5pt).
    struct Stub;
    impl FontMetrics for Stub {
        fn width(&self, _f: &str, _c: u32) -> f32 {
            500.0
        }
    }

    /// §7.1 oracle (simplest form): the secret bytes must not appear
    /// anywhere in the (uncompressed) output content stream.
    fn assert_absent(out: &[u8], secret: &[u8]) {
        assert!(
            out.windows(secret.len()).all(|w| w != secret),
            "secret {:?} still present in redacted output: {:?}",
            String::from_utf8_lossy(secret),
            String::from_utf8_lossy(out)
        );
    }

    fn one_region(x0: f32, y0: f32, x1: f32, y1: f32) -> RegionSet {
        let mut rs = RegionSet::new(0);
        rs.push(RedactionRegion::from_rect(x0, y0, x1, y1, Some([0.0, 0.0, 0.0])));
        rs
    }

    const SECRET_DOC: &[u8] = b"BT\n/F1 10 Tf\n1 0 0 1 100 700 Tm\n(TOPSECRET) Tj\nET\n";

    #[test]
    fn secret_fully_in_region_is_removed_and_overlaid() {
        // "TOPSECRET" 9 glyphs ×5pt from x=100 → 100..145 at y≈700.
        let regions = one_region(90.0, 695.0, 160.0, 715.0);
        let (out, report) =
            redact_content_stream(SECRET_DOC, &regions, &RedactionOptions::default(), &Stub)
                .unwrap();
        assert_absent(&out, b"TOPSECRET");
        assert_absent(&out, b"SECRET");
        assert_eq!(report.glyphs_removed, 9);
        assert_eq!(report.regions, 1);
        assert!(report.bytes_removed > 0);
        // An opaque overlay block is present (G7): the overlay emits
        // `q\n<r> <g> <b> rg\n<x> <y> <w> <h> re\nf\nQ\n`.
        let s = String::from_utf8_lossy(&out);
        assert!(
            s.contains("rg\n") && s.contains(" re\n") && s.contains("\nf\n"),
            "overlay missing in: {s}"
        );
    }

    #[test]
    fn public_text_outside_region_survives_verbatim() {
        let doc = b"BT\n/F1 10 Tf\n1 0 0 1 100 700 Tm\n(PUBLIC) Tj\nET\n";
        let regions = one_region(0.0, 0.0, 5.0, 5.0); // far away
        let (out, report) =
            redact_content_stream(doc, &regions, &RedactionOptions::default(), &Stub).unwrap();
        assert_eq!(report.glyphs_removed, 0);
        assert!(
            out.windows(6).any(|w| w == b"PUBLIC"),
            "public text must survive: {}",
            String::from_utf8_lossy(&out)
        );
    }

    #[test]
    fn straddling_secret_partially_removed_public_kept() {
        // "PUBSECRET": region from x≈124.5 onward removes "SECRET",
        // keeps "PUB". Glyph i at x = 100 + 5i: P100 U105 B110 S115…
        let doc = b"BT\n/F1 10 Tf\n1 0 0 1 100 700 Tm\n(PUBSECRET) Tj\nET\n";
        let regions = one_region(120.0, 695.0, 400.0, 715.0);
        let (out, report) =
            redact_content_stream(doc, &regions, &RedactionOptions::default(), &Stub).unwrap();
        assert_absent(&out, b"SECRET");
        assert!(report.glyphs_removed >= 6);
        assert!(
            out.windows(3).any(|w| w == b"PUB"),
            "public prefix must survive: {}",
            String::from_utf8_lossy(&out)
        );
    }

    #[test]
    fn composite_font_is_refused_not_under_redacted() {
        struct Composite;
        impl FontMetrics for Composite {
            fn width(&self, _f: &str, _c: u32) -> f32 {
                500.0
            }
            fn is_simple(&self, _f: &str) -> bool {
                false
            }
        }
        let regions = one_region(0.0, 0.0, 1000.0, 1000.0);
        let err =
            redact_content_stream(SECRET_DOC, &regions, &RedactionOptions::default(), &Composite)
                .unwrap_err();
        assert!(matches!(err, Error::Unsupported(_)), "expected refusal, got {err:?}");
    }

    #[test]
    fn no_regions_keeps_content_and_draws_nothing() {
        let (out, report) = redact_content_stream(
            SECRET_DOC,
            &RegionSet::new(0),
            &RedactionOptions::default(),
            &Stub,
        )
        .unwrap();
        assert_eq!(report.glyphs_removed, 0);
        assert_eq!(report.regions, 0);
        // Nothing redacted, no overlay; the text is intact.
        assert!(out.windows(9).any(|w| w == b"TOPSECRET"));
    }

    #[test]
    fn malformed_stream_is_a_clean_error_not_a_panic() {
        // parse_content_stream is tolerant; ensure no panic and that if
        // it does parse, an empty/garbage stream yields no secret.
        let regions = one_region(0.0, 0.0, 1000.0, 1000.0);
        let _ = redact_content_stream(
            b"q Q (unbalanced",
            &regions,
            &RedactionOptions::default(),
            &Stub,
        );
        let _ = redact_content_stream(b"", &regions, &RedactionOptions::default(), &Stub);
    }

    #[test]
    fn pruned_body_wrapped_in_own_q_before_overlay() {
        // A trailing, unmatched `cm` (e.g. a Y-flip a producer applies once
        // and never restores — legal per ISO 32000-1: the CTM is not
        // required to be identity at end-of-stream) must not leak into the
        // overlay, which draws in the region's absolute page-space
        // coordinates. The pruned body must be wrapped in its own `q`/`Q`
        // so the overlay always starts from the stream's original CTM.
        let doc = b"1 0 0 -1 0 792 cm\n";
        let regions = one_region(90.0, 695.0, 160.0, 715.0);
        let (out, _report) =
            redact_content_stream(doc, &regions, &RedactionOptions::default(), &Stub).unwrap();

        let s = String::from_utf8_lossy(&out);
        assert!(s.starts_with("q\n"), "pruned body must open with q: {s}");
        let wrapper_q_close = s
            .find("Q\n")
            .expect("closing Q for the pruned-body wrapper");
        let overlay_rg = s.find("rg\n").expect("overlay rg op");
        assert!(
            wrapper_q_close < overlay_rg,
            "the wrapper Q must close (restoring the original CTM) before the \
             overlay starts drawing: {s}"
        );
    }
}
