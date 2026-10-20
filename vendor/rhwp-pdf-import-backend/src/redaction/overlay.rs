//! Redaction overlay content-stream generation (#231, T13 — guarantee
//! G7: an opaque mark is the *only* thing drawn where content was
//! removed).
//!
//! ISO 32000-1:2008 §12.5.6.23 / Table 192 overlay precedence is
//! `RO` form XObject > `OverlayText` (+`DA`,+`Q`,+`Repeat`) > `IC`
//! solid fill > (no `IC`) a default solid fill in destructive mode —
//! "transparent + removed" is visually confusing and risks operator
//! error, so the default is an opaque block.
//!
//! `RO`/`OverlayText` come from the source `/Redact` annotation and are
//! resolved by the annotation layer; this module owns *only* the pure
//! geometry-to-content-stream-bytes step for an already-resolved
//! (region, fill) pair (SRP). It performs no I/O; the engine appends
//! these bytes after the pruned content so the overlay is on top.

use super::options::RedactionOptions;
use super::region::RedactionRegion;
use std::fmt::Write as _;

/// Format a coordinate as a PDF real: fixed-point, trailing zeros and a
/// dangling `.` trimmed, never scientific notation (PDF has no exponent
/// form — ISO 32000-1 §7.3.3). Non-finite ⇒ `0` (fail safe; the overlay
/// must still draw *something* opaque).
fn num(v: f32) -> String {
    if !v.is_finite() {
        return "0".to_string();
    }
    let mut s = format!("{v:.4}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

/// Resolve the overlay fill colour for a region per the precedence
/// above: explicit region `fill` (the `/IC`) wins; otherwise the
/// configured default *iff* `draw_overlay_when_no_ic`. `None` ⇒ draw no
/// overlay (caller still removed the content; the area is just blank).
fn resolved_fill(region: &RedactionRegion, opts: &RedactionOptions) -> Option<[f32; 3]> {
    match region.fill {
        Some(c) => Some(c),
        None if opts.draw_overlay_when_no_ic => Some(opts.default_fill),
        None => None,
    }
}

/// Content-stream bytes drawing the opaque overlay for one region, or
/// empty when no overlay is to be drawn (no `IC` and
/// `draw_overlay_when_no_ic == false`).
///
/// Emits a self-contained `q … Q` block so it cannot leak graphics
/// state into surrounding (already-pruned) content. A rotated
/// `QuadPoints` region is filled as the exact quad polygon; otherwise
/// the normalized bbox rectangle is filled. The fill colour clamps to
/// `0.0..=1.0` (DeviceRGB).
pub fn region_overlay_ops(region: &RedactionRegion, opts: &RedactionOptions) -> Vec<u8> {
    let Some(fill) = resolved_fill(region, opts) else {
        return Vec::new();
    };
    let clamp = |c: f32| -> f32 {
        if c.is_nan() {
            0.0
        } else {
            c.clamp(0.0, 1.0)
        }
    };
    let (r, g, b) = (clamp(fill[0]), clamp(fill[1]), clamp(fill[2]));

    let mut out = String::new();
    out.push_str("q\n");
    let _ = writeln!(out, "{} {} {} rg", num(r), num(g), num(b));

    if let Some(qd) = region.quad {
        // Polygon path over the four QuadPoints corners.
        let _ = writeln!(out, "{} {} m", num(qd[0]), num(qd[1]));
        let _ = writeln!(out, "{} {} l", num(qd[2]), num(qd[3]));
        let _ = writeln!(out, "{} {} l", num(qd[4]), num(qd[5]));
        let _ = writeln!(out, "{} {} l", num(qd[6]), num(qd[7]));
        out.push_str("h\nf\n");
    } else {
        let [x0, y0, x1, y1] = region.bbox;
        let _ = writeln!(out, "{} {} {} {} re\nf", num(x0), num(y0), num(x1 - x0), num(y1 - y0));
    }
    out.push('Q');
    out.push('\n');
    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redaction::options::RedactionOptions;
    use crate::redaction::region::RedactionRegion;

    fn s(bytes: &[u8]) -> String {
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[test]
    fn num_formats_pdf_reals_no_exponent() {
        assert_eq!(num(0.0), "0");
        assert_eq!(num(-0.0), "0");
        assert_eq!(num(12.0), "12");
        assert_eq!(num(12.5), "12.5");
        assert_eq!(num(0.10000), "0.1");
        assert_eq!(num(1.0e-7), "0"); // rounds to 0 at 4dp, not "1e-7"
        assert_eq!(num(f32::NAN), "0");
        assert_eq!(num(f32::INFINITY), "0");
        assert_eq!(num(-3.25), "-3.25");
    }

    #[test]
    fn rect_region_with_ic_emits_fill_block() {
        let region = RedactionRegion::from_rect(10.0, 20.0, 110.0, 70.0, Some([1.0, 0.0, 0.0]));
        let ops = s(&region_overlay_ops(&region, &RedactionOptions::default()));
        assert_eq!(ops, "q\n1 0 0 rg\n10 20 100 50 re\nf\nQ\n");
    }

    #[test]
    fn no_ic_uses_default_fill_when_enabled() {
        let region = RedactionRegion::from_rect(0.0, 0.0, 5.0, 5.0, None);
        let opts = RedactionOptions::default(); // black, draw_when_no_ic=true
        let ops = s(&region_overlay_ops(&region, &opts));
        assert_eq!(ops, "q\n0 0 0 rg\n0 0 5 5 re\nf\nQ\n");
    }

    #[test]
    fn no_ic_and_disabled_emits_nothing() {
        let region = RedactionRegion::from_rect(0.0, 0.0, 5.0, 5.0, None);
        let opts = RedactionOptions {
            draw_overlay_when_no_ic: false,
            ..RedactionOptions::default()
        };
        assert!(region_overlay_ops(&region, &opts).is_empty());
    }

    #[test]
    fn quad_region_emits_polygon_path() {
        let quad = [50.0, 0.0, 100.0, 50.0, 50.0, 100.0, 0.0, 50.0];
        let region = RedactionRegion::from_quad(quad, Some([0.0, 0.0, 0.0]));
        let ops = s(&region_overlay_ops(&region, &RedactionOptions::default()));
        assert_eq!(ops, "q\n0 0 0 rg\n50 0 m\n100 50 l\n50 100 l\n0 50 l\nh\nf\nQ\n");
    }

    #[test]
    fn fill_components_are_clamped() {
        let region = RedactionRegion::from_rect(0.0, 0.0, 1.0, 1.0, Some([2.0, -1.0, 0.5]));
        let ops = s(&region_overlay_ops(&region, &RedactionOptions::default()));
        assert!(ops.contains("1 0 0.5 rg"), "got: {ops}");
    }

    #[test]
    fn block_is_self_contained_q_q() {
        let region = RedactionRegion::from_rect(0.0, 0.0, 1.0, 1.0, Some([0.0, 0.0, 0.0]));
        let ops = s(&region_overlay_ops(&region, &RedactionOptions::default()));
        assert!(ops.starts_with("q\n") && ops.ends_with("Q\n"));
    }
}
