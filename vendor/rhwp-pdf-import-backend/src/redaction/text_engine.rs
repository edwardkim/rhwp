//! Content-stream text-redaction engine (#231, T4 + the text path of
//! T11 — guarantees G1 "no recoverable text" and G2 "no width/shift
//! side channel").
//!
//! [`super::text_prune`] holds the *pure segmentation* logic over an
//! abstract [`super::text_prune::Glyph`]. This module is the piece that
//! reconstructs those glyphs from a real operator stream: it walks the
//! operators with the shared [`GraphicsStateStack`] (DRY — same CTM
//! machinery as the extractor), maintains the text state, computes each
//! glyph's page-space box via the ISO 32000-1:2008 §9.4.4 text-rendering
//! matrix, prunes the run, and re-emits survivors with a fresh **absolute**
//! `Tm` and no inter-glyph offsets (kills the Bland-2023 side channel).
//!
//! Coordinate-space correctness is the worst failure mode for redaction
//! (feature plan §9 risk 1: a mis-composed matrix silently *under*-redacts
//! a glyph that visually sits in the region). Mitigations encoded here:
//! the §9.4.4 matrix `T_fs·T_h / T_fs / T_rise → T_m → CTM` is built
//! exactly per spec; the glyph box uses a conservative ascent/descent
//! envelope; any font whose encoding this module cannot resolve to
//! single-byte codes is treated as **one opaque mark spanning its whole
//! advance** — over-redaction, never under. Pure: a [`FontMetrics`]
//! trait abstracts width/encoding so the security-critical logic is
//! exhaustively unit-testable without a real document.

use super::region::RegionSet;
use super::text_prune::{prune_run, Glyph, TextPruneResult};
use crate::content::graphics_state::{GraphicsStateStack, Matrix};
use crate::content::operators::{Operator, TextElement};
use crate::geometry::Rect;

/// Glyph metrics + encoding for the fonts referenced by a content stream.
///
/// Implemented by the engine over the real `FontInfo` (T11/T12); a stub
/// implementation drives the unit tests. All widths are in **glyph-space
/// units** (thousandths of a text-space unit), the PDF `/Widths`
/// convention (ISO 32000-1 §9.2.4).
pub trait FontMetrics {
    /// Advance width of `code` in the current font, in /1000 units.
    fn width(&self, font: &str, code: u32) -> f32;

    /// Split a show string into `(code, source_bytes)` pairs in order.
    ///
    /// Simple fonts map one byte to one code; the default does exactly
    /// that. Composite (Type0/CID) fonts override this; a font whose
    /// encoding is unknown should return a **single** pair spanning the
    /// whole string so the caller treats it as one opaque mark
    /// (conservative — never under-redact).
    fn decode(&self, _font: &str, s: &[u8]) -> Vec<(u32, Vec<u8>)> {
        s.iter().map(|&b| (b as u32, vec![b])).collect()
    }

    /// `true` if `code` is the single-byte space (only then does `Tw`
    /// apply, ISO 32000-1 §9.3.3). Default: ASCII space.
    fn is_word_space(&self, _font: &str, code: u32) -> bool {
        code == 32
    }

    /// Conservative glyph ascent/descent as fractions of the em
    /// (text-space units). Default envelopes the standard Latin range
    /// generously so a tall/low glyph never escapes the box.
    fn ascent_descent(&self, _font: &str) -> (f32, f32) {
        (1.0, -0.30)
    }

    /// Whether per-glyph boxes for `font` can be reconstructed reliably
    /// (single-byte simple fonts). Composite/Type0/CID or unknown fonts
    /// return `false`: the engine then **refuses** the redaction rather
    /// than risk a silent under-redaction from a mis-decoded multi-byte
    /// string (feature plan §9 risk 6 — fail closed, never under-redact).
    /// Default `true` (the stub/simple case).
    fn is_simple(&self, _font: &str) -> bool {
        true
    }

    /// Whether the engine can prune individual glyphs for `font` reliably
    /// enough to redact it. A superset of [`Self::is_simple`]: a real
    /// implementation may also support a multi-byte encoding whose
    /// code→glyph→box mapping it can reconstruct deterministically (e.g.
    /// Identity-H, whose 2-byte codes equal their CIDs, ISO 32000-1
    /// §9.7.5.2). Anything this returns `false` for is **refused** (fail
    /// closed). Default delegates to `is_simple` so existing single-byte
    /// behaviour is unchanged.
    fn can_prune(&self, font: &str) -> bool {
        self.is_simple(font)
    }

    /// Whether one show string is well-formed for `font` under the
    /// encoding the engine claims to support. For a fixed-width multi-byte
    /// encoding (Identity-H: 2 bytes per code) a string whose length is not
    /// a whole number of codes is malformed; pruning it could misalign code
    /// boundaries and silently under-redact, so the engine **refuses**.
    /// Default `true` (single-byte fonts have no alignment constraint).
    fn redaction_safe_show(&self, _font: &str, _bytes: &[u8]) -> bool {
        true
    }
}

/// One font's removed glyph codes, for `font_scrub` (G2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemovedGlyphs {
    /// `(font_resource_name_hash, code)` pairs removed, first-seen order.
    pub codes: Vec<(u32, u32)>,
}

/// Result of redacting a content stream's text.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextEngineResult {
    /// The rewritten operator sequence (text in regions removed).
    pub operators: Vec<Operator>,
    /// Total glyphs physically removed.
    pub glyphs_removed: usize,
    /// Show-string payload bytes physically removed (sum over redacted
    /// glyphs of their encoded source-byte length) — the meaningful
    /// "data removed" metric (re-serialization float-formatting makes a
    /// raw stream-length diff useless).
    pub bytes_removed: u64,
    /// Distinct `(font_hash, code)` removed, for font scrubbing.
    pub removed_codes: Vec<(u32, u32)>,
    /// Set when a text show used a font whose per-glyph boxes cannot be
    /// reconstructed reliably (composite/Type0/unknown) *and* regions
    /// exist. The caller must treat this as a hard refusal — emitting the
    /// original stream would risk a silent under-redaction
    /// (feature plan §9 risk 6, fail closed).
    pub unsupported_font: bool,
}

/// Stable non-cryptographic hash of a font resource name → the `u32`
/// "resource id" the pruner records (font names are short; FNV-1a is
/// ample and deterministic across runs/platforms).
fn font_id(name: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in name.as_bytes() {
        h ^= *b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Build the ISO 32000-1 §9.4.4 text-rendering matrix
/// `[[Tfs·Th,0],[0,Tfs],[0,Trise]] × Tm × CTM` (row-vector convention,
/// matching `Matrix::multiply`).
fn text_rendering_matrix(tfs: f32, th: f32, trise: f32, tm: &Matrix, ctm: &Matrix) -> Matrix {
    let params = Matrix {
        a: tfs * th,
        b: 0.0,
        c: 0.0,
        d: tfs,
        e: 0.0,
        f: trise,
    };
    params.multiply(tm).multiply(ctm)
}

/// Page-space box of a glyph occupying text-space `x ∈ [0, w0]`,
/// `y ∈ [descent, ascent]` under `trm` (envelope of the 4 transformed
/// corners — a conservative superset under rotation/shear).
fn glyph_box(w0: f32, ascent: f32, descent: f32, trm: &Matrix) -> Rect {
    let corners = [
        trm.transform_point(0.0, descent),
        trm.transform_point(w0, descent),
        trm.transform_point(w0, ascent),
        trm.transform_point(0.0, ascent),
    ];
    let mut x0 = f32::INFINITY;
    let mut y0 = f32::INFINITY;
    let mut x1 = f32::NEG_INFINITY;
    let mut y1 = f32::NEG_INFINITY;
    for p in corners {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    Rect::from_points(x0, y0, x1, y1)
}

/// Minimal text state tracked between text-showing operators.
#[derive(Clone)]
struct TextState {
    tm: Matrix,
    tlm: Matrix,
    tc: f32,
    tw: f32,
    th: f32,
    tfs: f32,
    trise: f32,
    leading: f32,
    font: String,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            tm: Matrix::identity(),
            tlm: Matrix::identity(),
            tc: 0.0,
            tw: 0.0,
            th: 1.0,
            tfs: 0.0,
            trise: 0.0,
            leading: 0.0,
            font: String::new(),
        }
    }
}

/// Reconstruct glyph boxes for one show string and prune it against the
/// page regions, advancing `ts.tm` per glyph (ISO 32000-1 §9.4.4). The
/// `tj_adjust` is the pending `TJ` numeric (already /1000-scaled) applied
/// before the *next* glyph; here per-string elements pass `0.0`.
fn show_string(
    bytes: &[u8],
    ts: &mut TextState,
    ctm: &Matrix,
    fonts: &dyn FontMetrics,
    regions: &RegionSet,
    min_padding: f32,
) -> TextPruneResult {
    let fid = font_id(&ts.font);
    let (ascent, descent) = fonts.ascent_descent(&ts.font);
    let decoded = fonts.decode(&ts.font, bytes);
    let mut glyphs: Vec<Glyph> = Vec::with_capacity(decoded.len());

    for (code, src) in decoded {
        let trm = text_rendering_matrix(ts.tfs, ts.th, ts.trise, &ts.tm, ctm);
        let w0 = fonts.width(&ts.font, code) / 1000.0;
        let bbox = glyph_box(w0, ascent, descent, &trm);
        glyphs.push(Glyph {
            bytes: src,
            bbox,
            render_matrix: [ts.tm.a, ts.tm.b, ts.tm.c, ts.tm.d, ts.tm.e, ts.tm.f],
            code: (fid, code),
        });
        // Advance the text matrix: tx = ((w0 - 0) Tfs + Tc + Tw?) Th.
        let tw = if fonts.is_word_space(&ts.font, code) {
            ts.tw
        } else {
            0.0
        };
        let tx = (w0 * ts.tfs + ts.tc + tw) * ts.th;
        ts.tm = Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: 0.0,
        }
        .multiply(&ts.tm);
    }
    prune_run(&glyphs, regions, min_padding)
}

/// Emit the pruned survivors of one show as `BT`-context operators: each
/// run gets an absolute `Tm` then a single `Tj` of its surviving bytes
/// (no `TJ` deltas — kills the width/shift side channel, G2).
fn emit_runs(out: &mut Vec<Operator>, res: &TextPruneResult) {
    for run in &res.runs {
        let [a, b, c, d, e, f] = run.anchor;
        out.push(Operator::Tm { a, b, c, d, e, f });
        out.push(Operator::Tj {
            text: run.bytes.clone(),
        });
    }
}

/// Redact text in `ops` that intersects `regions` (page space).
///
/// Non-text operators pass through unchanged; the graphics-state CTM and
/// the text state are tracked so every glyph's box is computed in page
/// space exactly per ISO 32000-1 §9.4.4. Each text show is replaced by
/// the absolute-re-anchored survivors (or nothing when fully redacted —
/// no compensating offset). `q`/`Q`/`cm` are never dropped.
pub fn redact_text_stream(
    ops: &[Operator],
    regions: &RegionSet,
    min_padding: f32,
    fonts: &dyn FontMetrics,
) -> TextEngineResult {
    let mut stack = GraphicsStateStack::new();
    let mut ts = TextState::default();
    let mut out: Vec<Operator> = Vec::with_capacity(ops.len());
    let mut result = TextEngineResult::default();

    for op in ops {
        match op {
            Operator::SaveState | Operator::RestoreState | Operator::Cm { .. } => {
                super::classify::apply_ctm(&mut stack, op);
                out.push(op.clone());
            },
            Operator::BeginText => {
                ts.tm = Matrix::identity();
                ts.tlm = Matrix::identity();
                out.push(op.clone());
            },
            Operator::EndText => out.push(op.clone()),
            Operator::Tf { font, size } => {
                ts.font = font.clone();
                ts.tfs = *size;
                out.push(op.clone());
            },
            Operator::Tc { char_space } => {
                ts.tc = *char_space;
                out.push(op.clone());
            },
            Operator::Tw { word_space } => {
                ts.tw = *word_space;
                out.push(op.clone());
            },
            Operator::Tz { scale } => {
                ts.th = *scale / 100.0;
                out.push(op.clone());
            },
            Operator::TL { leading } => {
                ts.leading = *leading;
                out.push(op.clone());
            },
            Operator::Ts { rise } => {
                ts.trise = *rise;
                out.push(op.clone());
            },
            Operator::Td { tx, ty } => {
                ts.tlm = Matrix {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 1.0,
                    e: *tx,
                    f: *ty,
                }
                .multiply(&ts.tlm);
                ts.tm = ts.tlm;
                out.push(op.clone());
            },
            Operator::TD { tx, ty } => {
                ts.leading = -*ty;
                ts.tlm = Matrix {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 1.0,
                    e: *tx,
                    f: *ty,
                }
                .multiply(&ts.tlm);
                ts.tm = ts.tlm;
                out.push(op.clone());
            },
            Operator::Tm { a, b, c, d, e, f } => {
                let m = Matrix {
                    a: *a,
                    b: *b,
                    c: *c,
                    d: *d,
                    e: *e,
                    f: *f,
                };
                ts.tm = m;
                ts.tlm = m;
                out.push(op.clone());
            },
            Operator::TStar => {
                ts.tlm = Matrix {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 1.0,
                    e: 0.0,
                    f: -ts.leading,
                }
                .multiply(&ts.tlm);
                ts.tm = ts.tlm;
                out.push(op.clone());
            },
            Operator::Tj { text } => {
                if refuse_unsupported(fonts, &ts.font, regions, &mut result, &mut out, op) {
                    continue;
                }
                let ctm = stack.current().ctm;
                let res = show_string(text, &mut ts, &ctm, fonts, regions, min_padding);
                account(&mut result, text.len(), &res);
                emit_runs(&mut out, &res);
            },
            Operator::Quote { text } => {
                if refuse_unsupported(fonts, &ts.font, regions, &mut result, &mut out, op) {
                    continue;
                }
                // `'` = T* then show.
                ts.tlm = Matrix {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 1.0,
                    e: 0.0,
                    f: -ts.leading,
                }
                .multiply(&ts.tlm);
                ts.tm = ts.tlm;
                let ctm = stack.current().ctm;
                let res = show_string(text, &mut ts, &ctm, fonts, regions, min_padding);
                account(&mut result, text.len(), &res);
                emit_runs(&mut out, &res);
            },
            Operator::DoubleQuote {
                word_space,
                char_space,
                text,
            } => {
                if refuse_unsupported(fonts, &ts.font, regions, &mut result, &mut out, op) {
                    continue;
                }
                ts.tw = *word_space;
                ts.tc = *char_space;
                ts.tlm = Matrix {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 1.0,
                    e: 0.0,
                    f: -ts.leading,
                }
                .multiply(&ts.tlm);
                ts.tm = ts.tlm;
                let ctm = stack.current().ctm;
                let res = show_string(text, &mut ts, &ctm, fonts, regions, min_padding);
                account(&mut result, text.len(), &res);
                emit_runs(&mut out, &res);
            },
            Operator::TJ { array } => {
                if refuse_unsupported(fonts, &ts.font, regions, &mut result, &mut out, op) {
                    continue;
                }
                // Concatenate the string elements (offsets are positional
                // hints we deliberately discard on rewrite — G2). Per
                // §9.4.4 a positive TJ number moves left by n/1000·Tfs·Th.
                let ctm = stack.current().ctm;
                let mut any_removed = false;
                let mut tj_orig = 0usize;
                let mut survived_runs = TextPruneResult::default();
                for el in array {
                    match el {
                        TextElement::String(s) => {
                            tj_orig += s.len();
                            let r = show_string(s, &mut ts, &ctm, fonts, regions, min_padding);
                            if r.glyphs_removed > 0 {
                                any_removed = true;
                            }
                            for c in &r.removed_codes {
                                if !survived_runs.removed_codes.contains(c) {
                                    survived_runs.removed_codes.push(*c);
                                }
                            }
                            survived_runs.glyphs_removed += r.glyphs_removed;
                            survived_runs.runs.extend(r.runs);
                        },
                        TextElement::Offset(off) => {
                            let dx = (-*off / 1000.0) * ts.tfs * ts.th;
                            ts.tm = Matrix {
                                a: 1.0,
                                b: 0.0,
                                c: 0.0,
                                d: 1.0,
                                e: dx,
                                f: 0.0,
                            }
                            .multiply(&ts.tm);
                        },
                    }
                }
                account(&mut result, tj_orig, &survived_runs);
                if any_removed {
                    emit_runs(&mut out, &survived_runs);
                } else {
                    // Nothing redacted in this TJ — emit it byte-identical
                    // (perf + minimal diff; no side channel since nothing
                    // was removed).
                    out.push(op.clone());
                }
            },
            other => out.push(other.clone()),
        }
    }

    result.operators = out;
    result
}

/// Fail-closed guard: a show with a non-simple (composite/Type0/unknown)
/// font while regions exist cannot be pruned reliably. Flag the result
/// as unsupported and emit the show **unchanged** (the caller treats
/// `unsupported_font` as a hard refusal and discards the output, so the
/// unredacted bytes are never persisted — feature plan §9 risk 6).
/// Returns `true` when refused (caller should skip normal handling).
fn refuse_unsupported(
    fonts: &dyn FontMetrics,
    font: &str,
    regions: &RegionSet,
    result: &mut TextEngineResult,
    out: &mut Vec<Operator>,
    op: &Operator,
) -> bool {
    if regions.is_empty() {
        return false;
    }
    // Refuse a font whose glyphs cannot be pruned reliably, OR a show whose
    // bytes do not align to the font's code length (a malformed multi-byte
    // string we must not risk mis-decoding). Either way emit the show
    // unchanged and flag the hard refusal.
    let aligned = show_strings(op)
        .iter()
        .all(|s| fonts.redaction_safe_show(font, s));
    if !fonts.can_prune(font) || !aligned {
        result.unsupported_font = true;
        out.push(op.clone());
        return true;
    }
    false
}

/// The raw show-string payload(s) carried by a text-showing operator, in
/// order. Non-show operators yield an empty list.
fn show_strings(op: &Operator) -> Vec<&[u8]> {
    match op {
        Operator::Tj { text } | Operator::Quote { text } => vec![text.as_slice()],
        Operator::DoubleQuote { text, .. } => vec![text.as_slice()],
        Operator::TJ { array } => array
            .iter()
            .filter_map(|el| match el {
                TextElement::String(s) => Some(s.as_slice()),
                TextElement::Offset(_) => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Accumulate one show's prune result. `orig_len` is the original
/// show-string payload byte count; removed bytes = `orig_len` minus the
/// surviving runs' bytes (re-serialization float bloat makes a raw
/// stream-length diff meaningless, so the byte metric is computed here).
fn account(result: &mut TextEngineResult, orig_len: usize, res: &TextPruneResult) {
    result.glyphs_removed += res.glyphs_removed;
    let kept: usize = res.runs.iter().map(|r| r.bytes.len()).sum();
    result.bytes_removed += orig_len.saturating_sub(kept) as u64;
    for c in &res.removed_codes {
        if !result.removed_codes.contains(c) {
            result.removed_codes.push(*c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redaction::region::{RedactionRegion, RegionSet, DEFAULT_EDGE_PADDING};

    /// Fixed-pitch stub: every code is 500/1000 em wide, simple 1-byte.
    struct Stub;
    impl FontMetrics for Stub {
        fn width(&self, _f: &str, _c: u32) -> f32 {
            500.0
        }
    }

    /// Composite-font stub: reports every font as non-simple.
    struct CompositeStub;
    impl FontMetrics for CompositeStub {
        fn width(&self, _f: &str, _c: u32) -> f32 {
            500.0
        }
        fn is_simple(&self, _f: &str) -> bool {
            false
        }
    }

    /// Identity-H stub: full-em (1000) glyphs, 2-byte codes (CID = code),
    /// prunable, never a word space, odd-length shows refused — mirroring
    /// `FontInfoMetrics`'s real Identity-H behaviour so the engine's
    /// multi-byte pruning is testable in isolation.
    struct IdentityHStub;
    impl FontMetrics for IdentityHStub {
        fn width(&self, _f: &str, _c: u32) -> f32 {
            1000.0
        }
        fn is_simple(&self, _f: &str) -> bool {
            false
        }
        fn can_prune(&self, _f: &str) -> bool {
            true
        }
        fn decode(&self, _f: &str, s: &[u8]) -> Vec<(u32, Vec<u8>)> {
            s.as_chunks::<2>()
                .0
                .iter()
                .map(|p| (u32::from(u16::from_be_bytes([p[0], p[1]])), p.to_vec()))
                .collect()
        }
        fn is_word_space(&self, _f: &str, _c: u32) -> bool {
            false
        }
        fn redaction_safe_show(&self, _f: &str, bytes: &[u8]) -> bool {
            bytes.len().is_multiple_of(2)
        }
    }

    fn regions(x0: f32, y0: f32, x1: f32, y1: f32) -> RegionSet {
        let mut rs = RegionSet::new(0);
        rs.push(RedactionRegion::from_rect(x0, y0, x1, y1, None));
        rs
    }

    /// Build `BT /F1 10 Tf <tm> Tm (text) Tj ET`.
    fn doc(tm: [f32; 6], text: &[u8]) -> Vec<Operator> {
        vec![
            Operator::BeginText,
            Operator::Tf {
                font: "F1".into(),
                size: 10.0,
            },
            Operator::Tm {
                a: tm[0],
                b: tm[1],
                c: tm[2],
                d: tm[3],
                e: tm[4],
                f: tm[5],
            },
            Operator::Tj {
                text: text.to_vec(),
            },
            Operator::EndText,
        ]
    }

    fn tj_text(ops: &[Operator]) -> Vec<Vec<u8>> {
        ops.iter()
            .filter_map(|o| match o {
                Operator::Tj { text } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn text_fully_in_region_is_removed_no_offset() {
        // 10pt font at (100,100); "SECRET" each glyph 5pt wide → x 100..130.
        let ops = doc([1.0, 0.0, 0.0, 1.0, 100.0, 100.0], b"SECRET");
        let r = regions(90.0, 95.0, 140.0, 115.0);
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &Stub);
        assert_eq!(out.glyphs_removed, 6);
        // No Tj survives, no compensating offset/TJ emitted.
        assert!(tj_text(&out.operators).is_empty());
        assert!(!out
            .operators
            .iter()
            .any(|o| matches!(o, Operator::TJ { .. })));
        // BT/ET/Tf/Tm structure preserved.
        assert!(matches!(out.operators.first(), Some(Operator::BeginText)));
        assert!(matches!(out.operators.last(), Some(Operator::EndText)));
    }

    #[test]
    fn text_fully_outside_is_untouched() {
        let ops = doc([1.0, 0.0, 0.0, 1.0, 100.0, 100.0], b"PUBLIC");
        let r = regions(0.0, 0.0, 10.0, 10.0);
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &Stub);
        assert_eq!(out.glyphs_removed, 0);
        assert_eq!(tj_text(&out.operators), vec![b"PUBLIC".to_vec()]);
    }

    #[test]
    fn straddle_splits_and_reanchors_absolutely() {
        // "PUBxSECRET" at 10pt, each glyph 5pt: P[100,105] U[105,110]
        // B[110,115] x[115,120] … Region starts at x=120 (padded left
        // 119.5) so B clearly survives and x onward is removed.
        let ops = doc([1.0, 0.0, 0.0, 1.0, 100.0, 100.0], b"PUBxSECRET");
        let r = regions(120.0, 95.0, 400.0, 115.0);
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &Stub);
        assert!(out.glyphs_removed >= 7, "removed={}", out.glyphs_removed);
        let surv = tj_text(&out.operators);
        assert_eq!(surv, vec![b"PUB".to_vec()]);
        // The surviving run is preceded by an absolute Tm anchored at the
        // first glyph's position (x=100), not a relative offset.
        let tm = out
            .operators
            .iter()
            .find_map(|o| match o {
                Operator::Tm { e, f, .. } => Some((*e, *f)),
                _ => None,
            })
            .unwrap();
        assert!((tm.0 - 100.0).abs() < 1e-3 && (tm.1 - 100.0).abs() < 1e-3);
    }

    #[test]
    fn identity_h_middle_cid_removed_others_survive() {
        // Identity-H: three 2-byte CIDs (1, 2, 3) → bytes 00 01 00 02 00 03.
        // Full-em glyphs at 10pt from (100,100): CID1 [100,110], CID2
        // [110,120], CID3 [120,130]. A tight region inside CID2 must remove
        // exactly that CID's two bytes and keep CID1 and CID3 intact.
        let ops = doc([1.0, 0.0, 0.0, 1.0, 100.0, 100.0], &[0, 1, 0, 2, 0, 3]);
        let r = regions(113.0, 95.0, 117.0, 115.0);
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &IdentityHStub);
        assert!(!out.unsupported_font, "Identity-H must be prunable, not refused");
        assert_eq!(out.glyphs_removed, 1, "only the middle CID is in the region");
        // CID1 and CID3 survive as their original 2-byte codes, in order.
        assert_eq!(tj_text(&out.operators), vec![vec![0u8, 1], vec![0u8, 3]]);
    }

    #[test]
    fn identity_h_fully_covered_run_removed() {
        let ops = doc([1.0, 0.0, 0.0, 1.0, 100.0, 100.0], &[0, 1, 0, 2, 0, 3]);
        let r = regions(90.0, 95.0, 140.0, 115.0); // covers all three CIDs
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &IdentityHStub);
        assert!(!out.unsupported_font);
        assert_eq!(out.glyphs_removed, 3);
        assert!(tj_text(&out.operators).is_empty(), "no target bytes may survive");
    }

    #[test]
    fn identity_h_odd_length_show_is_refused_fail_closed() {
        // A malformed 5-byte Identity-H show (not a whole number of 2-byte
        // codes) must be refused rather than mis-decoded — fail closed.
        let ops = doc([1.0, 0.0, 0.0, 1.0, 100.0, 100.0], &[0, 1, 0, 2, 9]);
        let r = regions(90.0, 95.0, 140.0, 115.0);
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &IdentityHStub);
        assert!(out.unsupported_font, "odd-length Identity-H show must refuse");
        // The original (unredacted) show is emitted unchanged; the caller
        // discards the whole result on refusal, so nothing is persisted.
        assert_eq!(tj_text(&out.operators), vec![vec![0u8, 1, 0, 2, 9]]);
    }

    #[test]
    fn ctm_scaled_block_still_caught_no_under_redaction() {
        // Regression for §9 risk 1: text drawn under `q 10 0 0 10 0 0 cm`
        // at text (10,10) → page (100,100). A page-space region at
        // 90..200 must still remove it.
        let mut ops = vec![
            Operator::SaveState,
            Operator::Cm {
                a: 10.0,
                b: 0.0,
                c: 0.0,
                d: 10.0,
                e: 0.0,
                f: 0.0,
            },
        ];
        ops.extend(doc([1.0, 0.0, 0.0, 1.0, 10.0, 10.0], b"HIDE"));
        ops.push(Operator::RestoreState);
        let r = regions(90.0, 90.0, 300.0, 300.0);
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &Stub);
        assert_eq!(out.glyphs_removed, 4);
        assert!(tj_text(&out.operators).is_empty());
        // q/Q preserved (stack balance).
        assert!(matches!(out.operators.first(), Some(Operator::SaveState)));
        assert!(matches!(out.operators.last(), Some(Operator::RestoreState)));
    }

    #[test]
    fn tj_array_with_offsets_redacted_drops_deltas() {
        // [(AB) -200 (CD)] TJ at (100,100). Region covers everything.
        let ops = vec![
            Operator::BeginText,
            Operator::Tf {
                font: "F1".into(),
                size: 10.0,
            },
            Operator::Tm {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 100.0,
                f: 100.0,
            },
            Operator::TJ {
                array: vec![
                    TextElement::String(b"AB".to_vec()),
                    TextElement::Offset(-200.0),
                    TextElement::String(b"CD".to_vec()),
                ],
            },
            Operator::EndText,
        ];
        let r = regions(90.0, 95.0, 300.0, 115.0);
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &Stub);
        assert_eq!(out.glyphs_removed, 4);
        // No TJ survives → no residual offset array that could encode
        // removed-glyph advances (G2).
        assert!(!out
            .operators
            .iter()
            .any(|o| matches!(o, Operator::TJ { .. })));
        assert!(tj_text(&out.operators).is_empty());
    }

    #[test]
    fn untouched_tj_array_emitted_byte_identical() {
        let ops = vec![
            Operator::BeginText,
            Operator::Tf {
                font: "F1".into(),
                size: 10.0,
            },
            Operator::Tm {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 100.0,
                f: 100.0,
            },
            Operator::TJ {
                array: vec![
                    TextElement::String(b"AB".to_vec()),
                    TextElement::Offset(-200.0),
                    TextElement::String(b"CD".to_vec()),
                ],
            },
            Operator::EndText,
        ];
        let r = regions(0.0, 0.0, 5.0, 5.0); // far away
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &Stub);
        assert_eq!(out.glyphs_removed, 0);
        // Original TJ preserved verbatim.
        let tj = out
            .operators
            .iter()
            .filter(|o| matches!(o, Operator::TJ { .. }))
            .count();
        assert_eq!(tj, 1);
    }

    #[test]
    fn composite_font_with_regions_refuses_and_keeps_original() {
        // Fail-closed (§9 risk 6): a Type0/composite font we cannot
        // reliably decode must NOT be silently passed through as redacted.
        let ops = doc([1.0, 0.0, 0.0, 1.0, 100.0, 100.0], b"SECRET");
        let r = regions(0.0, 0.0, 1000.0, 1000.0);
        let out = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &CompositeStub);
        assert!(out.unsupported_font, "must flag refusal");
        assert_eq!(out.glyphs_removed, 0);
        // Original show emitted unchanged (caller discards on refusal).
        assert_eq!(tj_text(&out.operators), vec![b"SECRET".to_vec()]);
    }

    #[test]
    fn composite_font_without_regions_is_not_a_refusal() {
        // No regions ⇒ nothing to redact ⇒ no refusal even for Type0.
        let ops = doc([1.0, 0.0, 0.0, 1.0, 100.0, 100.0], b"hello");
        let out =
            redact_text_stream(&ops, &RegionSet::new(0), DEFAULT_EDGE_PADDING, &CompositeStub);
        assert!(!out.unsupported_font);
        assert_eq!(tj_text(&out.operators), vec![b"hello".to_vec()]);
    }

    #[test]
    fn empty_and_no_region_are_safe() {
        let out = redact_text_stream(&[], &RegionSet::new(0), DEFAULT_EDGE_PADDING, &Stub);
        assert_eq!(out, TextEngineResult::default());
        let ops = doc([1.0, 0.0, 0.0, 1.0, 10.0, 10.0], b"abc");
        let out = redact_text_stream(&ops, &RegionSet::new(0), DEFAULT_EDGE_PADDING, &Stub);
        assert_eq!(out.glyphs_removed, 0);
        assert_eq!(tj_text(&out.operators), vec![b"abc".to_vec()]);
    }

    #[test]
    fn text_rendering_matrix_matches_spec_9_4_4() {
        // Tfs=12 Th=1 Trise=0, Tm=translate(50,60), CTM=identity.
        // A glyph-space point (0,0) → page (50,60); (1,0) → (62,60)
        // because x scales by Tfs·Th=12.
        let trm = text_rendering_matrix(
            12.0,
            1.0,
            0.0,
            &Matrix {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 50.0,
                f: 60.0,
            },
            &Matrix::identity(),
        );
        let o = trm.transform_point(0.0, 0.0);
        let x1 = trm.transform_point(1.0, 0.0);
        assert!((o.x - 50.0).abs() < 1e-3 && (o.y - 60.0).abs() < 1e-3);
        assert!((x1.x - 62.0).abs() < 1e-3 && (x1.y - 60.0).abs() < 1e-3);
    }

    #[test]
    fn unbalanced_q_and_malformed_do_not_panic() {
        // Adversarial: stray Q, Tj before BT, no font set.
        let ops = vec![
            Operator::RestoreState,
            Operator::Tj {
                text: b"x".to_vec(),
            },
            Operator::RestoreState,
        ];
        let r = regions(0.0, 0.0, 1000.0, 1000.0);
        let _ = redact_text_stream(&ops, &r, DEFAULT_EDGE_PADDING, &Stub);
    }
}
