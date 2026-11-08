//! Classify content marks against redaction regions in page space (#231, T3).
//!
//! A content stream is a sequence of operators drawn in *local* coordinate
//! systems nested by `q … cm … Q`. Deciding whether a mark (glyph run,
//! image, path) is redacted requires mapping its local-space bounding box
//! through the composed CTM into page (default user) space and comparing it
//! to the page's [`RegionSet`].
//!
//! This module owns *only* that mapping + classification (SRP). The region
//! geometry lives in [`super::region`]; the CTM stack machinery is reused
//! from [`crate::content::graphics_state`] (DRY) rather than reimplemented.
//!
//! Coordinate-space errors are the worst failure for redaction
//! (feature plan §9 risk 1: a mis-composed CTM silently *under*-redacts a
//! glyph that visually sits in the region). The envelope transform here is
//! exact for translation/scale and a conservative superset under
//! rotation/shear — i.e. it errs toward over-redaction, never under.

use super::region::RegionSet;
use crate::content::graphics_state::{GraphicsStateStack, Matrix};
use crate::content::operators::Operator;
use crate::geometry::Rect;

/// How a mark relates to the redaction regions on its page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    /// No region overlaps the mark — keep it verbatim.
    Outside,
    /// The mark lies wholly within a (padded) region — remove entirely.
    Inside,
    /// The mark crosses a region boundary — split (text) or geometry-clip
    /// (image/path). Conservatively treated as "affected".
    Straddle,
}

impl Classification {
    /// Whether the mark is affected by redaction (`Inside` or `Straddle`).
    ///
    /// Conservative rule (feature plan §4.3): *any* overlap ⇒ affected.
    pub fn is_affected(self) -> bool {
        !matches!(self, Classification::Outside)
    }
}

/// Map a local-space axis-aligned bbox through `ctm` into a page-space
/// [`Rect`] (the envelope of the four transformed corners).
///
/// Exact for translation/scale; a conservative *superset* under
/// rotation/shear — the safe direction (never under-redact).
pub fn transform_bbox(local: &Rect, ctm: &Matrix) -> Rect {
    let corners = [
        ctm.transform_point(local.left(), local.top()),
        ctm.transform_point(local.right(), local.top()),
        ctm.transform_point(local.right(), local.bottom()),
        ctm.transform_point(local.left(), local.bottom()),
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

/// Classify a mark given its local-space bbox and the active CTM, against
/// the page's regions (each compared by its conservative padded box).
///
/// - `Outside`  — no region's padded box overlaps the mapped mark.
/// - `Inside`   — every overlapping region's padded box fully contains it.
/// - `Straddle` — it overlaps but is not contained (crosses a boundary).
pub fn classify(
    local_bbox: &Rect,
    ctm: &Matrix,
    regions: &RegionSet,
    min_padding: f32,
) -> Classification {
    let page = transform_bbox(local_bbox, ctm);
    let mut any = false;
    let mut all_contained = true;
    for r in &regions.regions {
        let padded = r.padded_rect(min_padding);
        if padded.intersects(&page) {
            any = true;
            if !padded.contains_rect(&page) {
                all_contained = false;
            }
        }
    }
    match (any, all_contained) {
        (false, _) => Classification::Outside,
        (true, true) => Classification::Inside,
        (true, false) => Classification::Straddle,
    }
}

/// Update a [`GraphicsStateStack`]'s CTM for one operator.
///
/// Handles only the transformation-affecting operators (`q` → save,
/// `Q` → restore, `cm` → concat); every other operator leaves the CTM
/// unchanged. Pruners call this while iterating so the CTM is correct at
/// each mark. `Q` at the base stack is a safe no-op (malformed input must
/// not panic — feature plan §4.2 no-panic property).
pub fn apply_ctm(stack: &mut GraphicsStateStack, op: &Operator) {
    match op {
        Operator::SaveState => stack.save(),
        Operator::RestoreState => stack.restore(),
        Operator::Cm { a, b, c, d, e, f } => {
            let m = Matrix {
                a: *a,
                b: *b,
                c: *c,
                d: *d,
                e: *e,
                f: *f,
            };
            // PDF `cm` pre-concatenates: CTM' = M × CTM_old. This
            // codebase's `Matrix::multiply(self, other)` applies `self`
            // first then `other` (row-vector p·self·other); the
            // established convention is `cm_matrix.multiply(&old_ctm)`
            // (mirrors src/content/parser.rs:687).
            let old = stack.current().ctm;
            stack.current_mut().ctm = m.multiply(&old);
        },
        _ => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redaction::region::{RedactionRegion, DEFAULT_EDGE_PADDING};

    fn approx_rect(r: &Rect, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
        let e = 1e-3;
        (r.left() - x0).abs() < e
            && (r.top() - y0).abs() < e
            && (r.right() - x1).abs() < e
            && (r.bottom() - y1).abs() < e
    }

    #[test]
    fn transform_bbox_identity_is_unchanged() {
        let local = Rect::from_points(10.0, 20.0, 30.0, 40.0);
        let out = transform_bbox(&local, &Matrix::identity());
        assert!(approx_rect(&out, 10.0, 20.0, 30.0, 40.0));
    }

    #[test]
    fn transform_bbox_translation_and_scale() {
        let local = Rect::from_points(0.0, 0.0, 10.0, 10.0);
        let t = Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 50.0,
            f: 60.0,
        };
        assert!(approx_rect(&transform_bbox(&local, &t), 50.0, 60.0, 60.0, 70.0));
        let s = Matrix {
            a: 3.0,
            b: 0.0,
            c: 0.0,
            d: 2.0,
            e: 0.0,
            f: 0.0,
        };
        assert!(approx_rect(&transform_bbox(&local, &s), 0.0, 0.0, 30.0, 20.0));
    }

    #[test]
    fn transform_bbox_rotation_envelope() {
        // 90° rotation: (a,b,c,d) = (0,1,-1,0). Unit square → envelope.
        let local = Rect::from_points(0.0, 0.0, 2.0, 4.0);
        let rot = Matrix {
            a: 0.0,
            b: 1.0,
            c: -1.0,
            d: 0.0,
            e: 0.0,
            f: 0.0,
        };
        // corners: (0,0)->(0,0) (2,0)->(0,2) (2,4)->(-4,2) (0,4)->(-4,0)
        assert!(approx_rect(&transform_bbox(&local, &rot), -4.0, 0.0, 0.0, 2.0));
    }

    #[test]
    fn classify_outside_when_no_region() {
        let regions = RegionSet::new(0);
        let m = Rect::from_points(0.0, 0.0, 5.0, 5.0);
        assert_eq!(
            classify(&m, &Matrix::identity(), &regions, DEFAULT_EDGE_PADDING),
            Classification::Outside
        );
    }

    #[test]
    fn classify_inside_and_straddle() {
        let mut regions = RegionSet::new(0);
        regions.push(RedactionRegion::from_rect(100.0, 100.0, 200.0, 200.0, None));

        // Fully inside.
        let inside = Rect::from_points(120.0, 120.0, 180.0, 180.0);
        assert_eq!(
            classify(&inside, &Matrix::identity(), &regions, DEFAULT_EDGE_PADDING),
            Classification::Inside
        );

        // Crosses the right boundary.
        let straddle = Rect::from_points(180.0, 120.0, 260.0, 180.0);
        assert_eq!(
            classify(&straddle, &Matrix::identity(), &regions, DEFAULT_EDGE_PADDING),
            Classification::Straddle
        );

        // Far away.
        let outside = Rect::from_points(500.0, 500.0, 510.0, 510.0);
        assert_eq!(
            classify(&outside, &Matrix::identity(), &regions, DEFAULT_EDGE_PADDING),
            Classification::Outside
        );
    }

    #[test]
    fn classify_maps_through_scaled_ctm_no_under_redaction() {
        // Regression intent for feature plan §9 risk 1: a mark drawn in a
        // scaled `q cm … Q` block must still be caught by a page-space
        // region. Region at page (100..200, 100..200); content unit
        // square at local (10..20, 10..20) under scale ×10 → page
        // (100..200, 100..200) ⇒ Inside.
        let mut regions = RegionSet::new(0);
        regions.push(RedactionRegion::from_rect(100.0, 100.0, 200.0, 200.0, None));

        let mut stack = GraphicsStateStack::new();
        apply_ctm(&mut stack, &Operator::SaveState);
        apply_ctm(
            &mut stack,
            &Operator::Cm {
                a: 10.0,
                b: 0.0,
                c: 0.0,
                d: 10.0,
                e: 0.0,
                f: 0.0,
            },
        );
        let local = Rect::from_points(10.0, 10.0, 20.0, 20.0);
        assert_eq!(
            classify(&local, &stack.current().ctm, &regions, DEFAULT_EDGE_PADDING),
            Classification::Inside
        );
        apply_ctm(&mut stack, &Operator::RestoreState);
        // CTM restored to identity after the balanced q/Q.
        let id = Matrix::identity();
        let c = stack.current().ctm;
        assert!(
            (c.a - id.a).abs() < 1e-6
                && (c.d - id.d).abs() < 1e-6
                && (c.e - id.e).abs() < 1e-6
                && (c.f - id.f).abs() < 1e-6
        );
    }

    #[test]
    fn apply_ctm_nested_concatenation_order() {
        // `cm translate(100,0)` then `cm scale(2,2)`: a local point (5,5)
        // must map scale-first then translate → (110,10).
        let mut stack = GraphicsStateStack::new();
        apply_ctm(
            &mut stack,
            &Operator::Cm {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 100.0,
                f: 0.0,
            },
        );
        apply_ctm(
            &mut stack,
            &Operator::Cm {
                a: 2.0,
                b: 0.0,
                c: 0.0,
                d: 2.0,
                e: 0.0,
                f: 0.0,
            },
        );
        let p = stack.current().ctm.transform_point(5.0, 5.0);
        assert!((p.x - 110.0).abs() < 1e-4 && (p.y - 10.0).abs() < 1e-4);
    }

    #[test]
    fn apply_ctm_restore_at_base_is_safe_noop() {
        // Malformed: unbalanced Q must not panic and must not corrupt CTM.
        let mut stack = GraphicsStateStack::new();
        let d0 = stack.depth();
        apply_ctm(&mut stack, &Operator::RestoreState);
        apply_ctm(&mut stack, &Operator::RestoreState);
        assert_eq!(stack.depth(), d0);
        let c = stack.current().ctm;
        assert!((c.a - 1.0).abs() < 1e-6 && (c.d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn is_affected_semantics() {
        assert!(!Classification::Outside.is_affected());
        assert!(Classification::Inside.is_affected());
        assert!(Classification::Straddle.is_affected());
    }
}
