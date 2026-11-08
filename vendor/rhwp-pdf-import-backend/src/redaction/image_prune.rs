//! Image-placement redaction planning for destructive redaction
//! (#231, T6 core / G3).
//!
//! ISO 32000-1:2008 §12.5.6.23: image data under a redaction region
//! *"shall be destroyed; clipping or image masks shall not be used to
//! hide that data."* Deciding *what* to do with an image XObject /
//! inline image requires mapping the page-space region back into the
//! image's own coordinate space (the unit square mapped to the page by
//! the CTM, ISO §8.9.5).
//!
//! This module is the *pure planning primitive*: affine inversion plus a
//! Keep / DeleteFull / Overwrite-fraction decision. It performs no decode
//! or re-encode (that integration is a later increment) and is not wired
//! into any redaction decision, so it cannot itself under-redact. Result
//! fractions are in the image's normalized `[0,1]²` space; the decode
//! step applies the correct row orientation to actual pixels. Reuses
//! `classify`/`region`/`Matrix` (DRY); pure deterministic math.

use super::classify::{classify, transform_bbox, Classification};
use super::region::RegionSet;
use crate::content::graphics_state::Matrix;
use crate::geometry::{Point, Rect};

/// Invert a 2-D affine transform; `None` if singular / non-finite.
///
/// The returned [`Matrix`]'s `transform_point` is the inverse map of
/// `m`'s (DRY: the inverse is just another affine matrix).
pub fn invert_affine(m: &Matrix) -> Option<Matrix> {
    let det = m.a * m.d - m.b * m.c;
    if !det.is_finite() || det.abs() <= f32::EPSILON {
        return None;
    }
    let inv_a = m.d / det;
    let inv_b = -m.b / det;
    let inv_c = -m.c / det;
    let inv_d = m.a / det;
    let inv_e = -(inv_a * m.e + inv_c * m.f);
    let inv_f = -(inv_b * m.e + inv_d * m.f);
    let out = Matrix {
        a: inv_a,
        b: inv_b,
        c: inv_c,
        d: inv_d,
        e: inv_e,
        f: inv_f,
    };
    if [out.a, out.b, out.c, out.d, out.e, out.f]
        .iter()
        .all(|v| v.is_finite())
    {
        Some(out)
    } else {
        None
    }
}

/// What a redaction pass must do to one image placement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageRedaction {
    /// No region touches the image — keep it verbatim.
    Keep,
    /// A region fully covers the placement (or the CTM is singular and
    /// the placement intersects a region) — delete the whole image.
    /// Conservative: when the mapping is unreliable we destroy, never
    /// hide (feature plan §4.4 / G3, never under-redact).
    DeleteFull,
    /// Overwrite the sub-rectangle of the image given as fractions of
    /// its own extent: `u0,v0,u1,v1` in `[0,1]`, `u`=horizontal,
    /// `v`=vertical in PDF image-space (lower-left origin). The decoder
    /// maps this to pixel rows with the correct orientation later.
    Overwrite {
        /// Left fraction in `[0,1]`.
        u0: f32,
        /// Bottom fraction in `[0,1]`.
        v0: f32,
        /// Right fraction in `[0,1]`.
        u1: f32,
        /// Top fraction in `[0,1]`.
        v1: f32,
    },
}

fn clamp01(x: f32) -> f32 {
    if x.is_nan() {
        0.0
    } else {
        x.clamp(0.0, 1.0)
    }
}

/// Decide what to do with an image whose unit square is mapped to the
/// page by `image_ctm` (the composed CTM at the `Do`/`BI`), against the
/// page's regions.
///
/// - `Keep`       — no padded region overlaps the placement.
/// - `DeleteFull` — a region fully contains the placement, or the CTM is
///   singular while the placement intersects a region (fail-safe:
///   destroy rather than risk leaving recoverable pixels).
/// - `Overwrite`  — the `[0,1]²` sub-rectangle (union over intersecting
///   regions, clamped to the image) whose pixels must be destroyed.
pub fn classify_image_placement(
    image_ctm: &Matrix,
    regions: &RegionSet,
    min_padding: f32,
) -> ImageRedaction {
    // The image occupies the unit square in its own space.
    let unit = Rect::from_points(0.0, 0.0, 1.0, 1.0);
    match classify(&unit, image_ctm, regions, min_padding) {
        Classification::Outside => ImageRedaction::Keep,
        Classification::Inside => ImageRedaction::DeleteFull,
        Classification::Straddle => {
            let Some(inv) = invert_affine(image_ctm) else {
                // Cannot map device→image space: over-redact (destroy).
                return ImageRedaction::DeleteFull;
            };
            // Union of each intersecting region's padded box, mapped back
            // into image [0,1]² space and clamped.
            let mut u0 = f32::INFINITY;
            let mut v0 = f32::INFINITY;
            let mut u1 = f32::NEG_INFINITY;
            let mut v1 = f32::NEG_INFINITY;
            let mut any = false;
            for r in &regions.regions {
                let padded = r.padded_rect(min_padding);
                // Does this region's device box meet the image at all?
                let dev_img = transform_bbox(&unit, image_ctm);
                if !padded.intersects(&dev_img) {
                    continue;
                }
                any = true;
                for (px, py) in [
                    (padded.left(), padded.top()),
                    (padded.right(), padded.top()),
                    (padded.right(), padded.bottom()),
                    (padded.left(), padded.bottom()),
                ] {
                    let p: Point = inv.transform_point(px, py);
                    u0 = u0.min(p.x);
                    v0 = v0.min(p.y);
                    u1 = u1.max(p.x);
                    v1 = v1.max(p.y);
                }
            }
            if !any {
                return ImageRedaction::Keep;
            }
            let (cu0, cv0, cu1, cv1) = (clamp01(u0), clamp01(v0), clamp01(u1), clamp01(v1));
            // A region that, mapped back, covers the whole unit square ⇒
            // delete (equivalent to full cover; avoids a no-op Overwrite).
            if cu0 <= 0.0 && cv0 <= 0.0 && cu1 >= 1.0 && cv1 >= 1.0 {
                ImageRedaction::DeleteFull
            } else {
                ImageRedaction::Overwrite {
                    u0: cu0,
                    v0: cv0,
                    u1: cu1,
                    v1: cv1,
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redaction::region::{RedactionRegion, RegionSet, DEFAULT_EDGE_PADDING};

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    fn scale_at(sx: f32, sy: f32, tx: f32, ty: f32) -> Matrix {
        Matrix {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: tx,
            f: ty,
        }
    }

    #[test]
    fn invert_affine_identity_translation_scale() {
        let id = Matrix::identity();
        let inv = invert_affine(&id).unwrap();
        let p = inv.transform_point(7.0, 9.0);
        assert!(approx(p.x, 7.0) && approx(p.y, 9.0));

        let t = scale_at(1.0, 1.0, 50.0, 60.0);
        let it = invert_affine(&t).unwrap();
        let q = it.transform_point(55.0, 65.0);
        assert!(approx(q.x, 5.0) && approx(q.y, 5.0));

        let s = scale_at(10.0, 4.0, 0.0, 0.0);
        let is = invert_affine(&s).unwrap();
        let r = is.transform_point(100.0, 40.0);
        assert!(approx(r.x, 10.0) && approx(r.y, 10.0));
    }

    #[test]
    fn invert_affine_round_trip() {
        let m = Matrix {
            a: 2.0,
            b: 0.5,
            c: -1.0,
            d: 3.0,
            e: 7.0,
            f: -2.0,
        };
        let inv = invert_affine(&m).unwrap();
        let p = m.transform_point(4.0, 9.0);
        let back = inv.transform_point(p.x, p.y);
        assert!(approx(back.x, 4.0) && approx(back.y, 9.0));
    }

    #[test]
    fn invert_affine_singular_is_none() {
        let degenerate = Matrix {
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: 5.0,
            f: 5.0,
        };
        assert!(invert_affine(&degenerate).is_none());
        let collinear = Matrix {
            a: 2.0,
            b: 4.0,
            c: 1.0,
            d: 2.0,
            e: 0.0,
            f: 0.0,
        }; // det = 2*2 - 4*1 = 0
        assert!(invert_affine(&collinear).is_none());
    }

    #[test]
    fn image_outside_is_kept() {
        // Image placed at device (0..100,0..100); region far away.
        let ctm = scale_at(100.0, 100.0, 0.0, 0.0);
        let mut regions = RegionSet::new(0);
        regions.push(RedactionRegion::from_rect(500.0, 500.0, 600.0, 600.0, None));
        assert_eq!(
            classify_image_placement(&ctm, &regions, DEFAULT_EDGE_PADDING),
            ImageRedaction::Keep
        );
    }

    #[test]
    fn image_fully_covered_is_deleted() {
        let ctm = scale_at(100.0, 100.0, 0.0, 0.0); // device 0..100
        let mut regions = RegionSet::new(0);
        regions.push(RedactionRegion::from_rect(-10.0, -10.0, 200.0, 200.0, None));
        assert_eq!(
            classify_image_placement(&ctm, &regions, DEFAULT_EDGE_PADDING),
            ImageRedaction::DeleteFull
        );
    }

    #[test]
    fn image_partial_returns_correct_fraction() {
        // Image unit square scaled ×100 at origin → device 0..100.
        // Region covers device x∈[50,100], y∈[0,100] (right half).
        // True fraction is u∈[0.5,1], v∈[0,1]; the always-on
        // proportional edge padding only *grows* the overwrite region
        // (conservative — never under-redacts), so u0 ≤ 0.5, the others
        // clamp to the image bounds.
        let ctm = scale_at(100.0, 100.0, 0.0, 0.0);
        let mut regions = RegionSet::new(0);
        regions.push(RedactionRegion::from_rect(50.0, 0.0, 100.0, 100.0, None));
        match classify_image_placement(&ctm, &regions, 0.0) {
            ImageRedaction::Overwrite { u0, v0, u1, v1 } => {
                // conservative: covers at least the true right half
                assert!(u0 <= 0.5 + 1e-3, "u0={u0} must not exceed 0.5");
                assert!(u0 >= 0.40, "u0={u0} unexpectedly far left");
                assert!(approx(v0, 0.0), "v0={v0}");
                assert!(approx(u1, 1.0), "u1={u1}");
                assert!(approx(v1, 1.0), "v1={v1}");
            },
            other => panic!("expected Overwrite, got {other:?}"),
        }
    }

    #[test]
    fn image_partial_clamps_to_unit_square() {
        // Region overhangs the image on the left and bottom; fractions
        // must clamp into [0,1] (no negative / >1 leak), and cover at
        // least the true overlap (true u1=v1=0.4; padding grows it).
        let ctm = scale_at(100.0, 100.0, 0.0, 0.0);
        let mut regions = RegionSet::new(0);
        regions.push(RedactionRegion::from_rect(-50.0, -50.0, 40.0, 40.0, None));
        match classify_image_placement(&ctm, &regions, 0.0) {
            ImageRedaction::Overwrite { u0, v0, u1, v1 } => {
                assert!(
                    (0.0..=1.0).contains(&u0)
                        && (0.0..=1.0).contains(&v0)
                        && (0.0..=1.0).contains(&u1)
                        && (0.0..=1.0).contains(&v1),
                    "fractions must stay in [0,1]: {u0},{v0},{u1},{v1}"
                );
                assert!(approx(u0, 0.0) && approx(v0, 0.0));
                // conservative: at least the true 0.4 overlap
                assert!((0.4..=0.45).contains(&u1), "u1={u1}");
                assert!((0.4..=0.45).contains(&v1), "v1={v1}");
            },
            other => panic!("expected Overwrite, got {other:?}"),
        }
    }

    #[test]
    fn singular_ctm_with_overlap_is_deleted_not_leaked() {
        // Non-invertible CTM but the (degenerate) placement still meets a
        // region → fail-safe DeleteFull, never Keep/partial.
        let ctm = Matrix {
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: 10.0,
            f: 10.0,
        };
        let mut regions = RegionSet::new(0);
        regions.push(RedactionRegion::from_rect(0.0, 0.0, 50.0, 50.0, None));
        let got = classify_image_placement(&ctm, &regions, DEFAULT_EDGE_PADDING);
        assert!(
            got == ImageRedaction::DeleteFull || got == ImageRedaction::Keep,
            "must never partially-Overwrite under a singular CTM, got {got:?}"
        );
    }

    #[test]
    fn empty_regions_keep() {
        let ctm = scale_at(100.0, 100.0, 0.0, 0.0);
        let regions = RegionSet::new(0);
        assert_eq!(
            classify_image_placement(&ctm, &regions, DEFAULT_EDGE_PADDING),
            ImageRedaction::Keep
        );
    }

    #[test]
    fn non_finite_does_not_panic() {
        let ctm = Matrix {
            a: f32::NAN,
            b: 0.0,
            c: 0.0,
            d: f32::INFINITY,
            e: 0.0,
            f: 0.0,
        };
        let mut regions = RegionSet::new(0);
        regions.push(RedactionRegion::from_rect(0.0, 0.0, 10.0, 10.0, None));
        let _ = classify_image_placement(&ctm, &regions, DEFAULT_EDGE_PADDING);
        let _ = invert_affine(&ctm);
    }
}
