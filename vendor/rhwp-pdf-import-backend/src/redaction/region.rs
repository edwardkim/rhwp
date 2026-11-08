//! Geometric model for redaction regions (ISO 32000-1:2008 §12.5.6.23).
//!
//! A [`RedactionRegion`] is an axis-aligned (optionally rotated-quad) target
//! in PDF default user space. This module is *pure geometry*: it normalizes
//! coordinates, computes the conservative edge padding the spec's "remove
//! all traces" diligence requires, and answers intersection / containment
//! queries. Mapping marks (glyphs / images / paths) through the CTM and the
//! precise polygon classification live in `classify.rs`; keeping the region
//! model free of graphics-state concerns is the SRP boundary for #231.
//!
//! Safety rule (feature plan §4.3 / §9 risk 6): over-redaction is acceptable,
//! **under-redaction is not**. Every primitive here errs toward "inside".

use crate::geometry::{Point, Rect};

/// Default minimum edge padding, in points — the `0.5pt` floor that keeps an
/// anti-aliased glyph sliver from surviving at a region boundary
/// (feature plan §4.1).
pub const DEFAULT_EDGE_PADDING: f32 = 0.5;

/// Padding applied proportionally to region height (feature plan §4.1):
/// `epsilon = max(min_padding, PROPORTIONAL_PADDING * height)`.
const PROPORTIONAL_PADDING: f32 = 0.02;

/// A redaction target in PDF default user space (page coordinates, points).
///
/// `bbox` is always stored normalized so `x0 <= x1` and `y0 <= y1`. When the
/// source supplied `QuadPoints` (a possibly-rotated quadrilateral) the `quad`
/// is retained for precise classification and `bbox` becomes the
/// axis-aligned envelope of that quad.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct RedactionRegion {
    /// Normalized axis-aligned bounding box `[x0, y0, x1, y1]`.
    pub bbox: [f32; 4],
    /// Optional rotated quad: four corners `[x0,y0, x1,y1, x2,y2, x3,y3]`.
    pub quad: Option<[f32; 8]>,
    /// Overlay fill colour (DeviceRGB, each component in `0.0..=1.0`).
    /// `None` mirrors an absent `/IC` entry (feature plan §4.2 / Table 192).
    pub fill: Option<[f32; 3]>,
}

impl RedactionRegion {
    /// Build a region from an axis-aligned rectangle.
    ///
    /// Corners may be given in any order; the stored `bbox` is normalized so
    /// `x0 <= x1` and `y0 <= y1`. Non-finite components are tolerated without
    /// panicking (`f32::min`/`f32::max` propagate the finite operand).
    pub fn from_rect(x0: f32, y0: f32, x1: f32, y1: f32, fill: Option<[f32; 3]>) -> Self {
        Self {
            bbox: [x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1)],
            quad: None,
            fill,
        }
    }

    /// Build a region from a `QuadPoints`-style quadrilateral.
    ///
    /// `quad` is `[x0,y0, x1,y1, x2,y2, x3,y3]`. The `bbox` is the
    /// axis-aligned envelope of the four corners.
    pub fn from_quad(quad: [f32; 8], fill: Option<[f32; 3]>) -> Self {
        let xs = [quad[0], quad[2], quad[4], quad[6]];
        let ys = [quad[1], quad[3], quad[5], quad[7]];
        let x0 = xs.iter().copied().fold(f32::INFINITY, f32::min);
        let x1 = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let y0 = ys.iter().copied().fold(f32::INFINITY, f32::min);
        let y1 = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        Self {
            bbox: [x0, y0, x1, y1],
            quad: Some(quad),
            fill,
        }
    }

    /// Width of the normalized bounding box.
    pub fn width(&self) -> f32 {
        self.bbox[2] - self.bbox[0]
    }

    /// Height of the normalized bounding box.
    pub fn height(&self) -> f32 {
        self.bbox[3] - self.bbox[1]
    }

    /// The bounding box as the canonical [`Rect`] (DRY: reuse geometry).
    pub fn rect(&self) -> Rect {
        Rect::from_points(self.bbox[0], self.bbox[1], self.bbox[2], self.bbox[3])
    }

    /// Effective edge padding, `epsilon = max(min_padding, 0.02 * height)`.
    ///
    /// `min_padding` is the caller's `RedactionOptions::edge_padding`; a
    /// negative or non-finite value falls back to [`DEFAULT_EDGE_PADDING`].
    pub fn effective_padding(&self, min_padding: f32) -> f32 {
        let floor = if min_padding.is_finite() && min_padding >= 0.0 {
            min_padding
        } else {
            DEFAULT_EDGE_PADDING
        };
        floor.max(PROPORTIONAL_PADDING * self.height().abs())
    }

    /// The bounding box expanded by [`effective_padding`], as a [`Rect`].
    ///
    /// [`effective_padding`]: RedactionRegion::effective_padding
    pub fn padded_rect(&self, min_padding: f32) -> Rect {
        let e = self.effective_padding(min_padding);
        Rect::from_points(self.bbox[0] - e, self.bbox[1] - e, self.bbox[2] + e, self.bbox[3] + e)
    }

    /// Whether a mark's bounding box (same user space) intersects this region
    /// under the conservative overlap rule: *any* overlap of the
    /// edge-padded region counts as inside (feature plan §4.3 / G1). The
    /// padded bbox is a superset of any rotated quad, so this never
    /// under-redacts; the tighter polygon test is applied by the classifier.
    pub fn intersects_rect(&self, mark: &Rect, min_padding: f32) -> bool {
        self.padded_rect(min_padding).intersects(mark)
    }

    /// Whether `p` lies in this region under the conservative (padded-bbox)
    /// rule. Safe superset for rotated quads; see [`quad_contains_point`] for
    /// the precise polygon test the classifier composes with padding.
    ///
    /// [`quad_contains_point`]: RedactionRegion::quad_contains_point
    pub fn contains_point(&self, p: &Point, min_padding: f32) -> bool {
        self.padded_rect(min_padding).contains_point(p)
    }

    /// Precise point-in-quad test, or `None` when the region has no quad.
    ///
    /// Exact for convex quadrilaterals (all `QuadPoints` quads are convex),
    /// winding-order independent, edge-inclusive. The classifier (T3) pads
    /// this; on its own it is the *unpadded* polygon and must not be used
    /// directly for under-redaction-sensitive decisions.
    pub fn quad_contains_point(&self, p: &Point) -> Option<bool> {
        self.quad.map(|q| point_in_convex_quad(p, &q))
    }
}

/// Edge-inclusive point-in-convex-quad test, independent of winding order.
///
/// For each directed edge the sign of the cross product `edge × (p - v)`
/// must be consistent (all `>= 0` or all `<= 0`). A point on any edge
/// (cross product `0`) is treated as inside (conservative).
fn point_in_convex_quad(p: &Point, q: &[f32; 8]) -> bool {
    let v = [(q[0], q[1]), (q[2], q[3]), (q[4], q[5]), (q[6], q[7])];
    let mut saw_pos = false;
    let mut saw_neg = false;
    for i in 0..4 {
        let (ax, ay) = v[i];
        let (bx, by) = v[(i + 1) % 4];
        let cross = (bx - ax) * (p.y - ay) - (by - ay) * (p.x - ax);
        if cross > 0.0 {
            saw_pos = true;
        } else if cross < 0.0 {
            saw_neg = true;
        }
        if saw_pos && saw_neg {
            return false;
        }
    }
    true
}

/// All redaction regions targeting a single page.
///
/// `page_index` is zero-based. Regions are merged from `/Redact`
/// annotations, programmatic rectangles, and text-search matches
/// (feature plan §4.1); this type only stores and queries them.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RegionSet {
    /// Zero-based page index these regions apply to.
    pub page_index: usize,
    /// The regions, in insertion order.
    pub regions: Vec<RedactionRegion>,
}

impl RegionSet {
    /// An empty region set for `page_index` (zero-based).
    pub fn new(page_index: usize) -> Self {
        Self {
            page_index,
            regions: Vec::new(),
        }
    }

    /// Append a region.
    pub fn push(&mut self, region: RedactionRegion) {
        self.regions.push(region);
    }

    /// Whether there are no regions.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Number of regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Whether any region intersects `mark` (conservative padded test).
    pub fn any_intersects(&self, mark: &Rect, min_padding: f32) -> bool {
        self.regions
            .iter()
            .any(|r| r.intersects_rect(mark, min_padding))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn from_rect_normalizes_reversed_corners() {
        let r = RedactionRegion::from_rect(100.0, 80.0, 10.0, 20.0, None);
        assert_eq!(r.bbox, [10.0, 20.0, 100.0, 80.0]);
        assert!(approx(r.width(), 90.0));
        assert!(approx(r.height(), 60.0));
        assert!(r.quad.is_none());
    }

    #[test]
    fn from_rect_keeps_already_normalized() {
        let r = RedactionRegion::from_rect(10.0, 20.0, 100.0, 80.0, Some([0.0, 0.0, 0.0]));
        assert_eq!(r.bbox, [10.0, 20.0, 100.0, 80.0]);
        assert_eq!(r.fill, Some([0.0, 0.0, 0.0]));
    }

    #[test]
    fn from_quad_derives_axis_aligned_envelope() {
        // A 45°-ish rotated quad; envelope is the min/max of corners.
        let quad = [50.0, 0.0, 100.0, 50.0, 50.0, 100.0, 0.0, 50.0];
        let r = RedactionRegion::from_quad(quad, None);
        assert_eq!(r.bbox, [0.0, 0.0, 100.0, 100.0]);
        assert_eq!(r.quad, Some(quad));
    }

    #[test]
    fn effective_padding_floor_dominates_for_short_region() {
        // height 10 → proportional = 0.2 < 0.5 floor.
        let r = RedactionRegion::from_rect(0.0, 0.0, 100.0, 10.0, None);
        assert!(approx(r.effective_padding(DEFAULT_EDGE_PADDING), 0.5));
    }

    #[test]
    fn effective_padding_proportional_dominates_for_tall_region() {
        // height 100 → proportional = 2.0 > 0.5 floor.
        let r = RedactionRegion::from_rect(0.0, 0.0, 20.0, 100.0, None);
        assert!(approx(r.effective_padding(DEFAULT_EDGE_PADDING), 2.0));
    }

    #[test]
    fn effective_padding_custom_floor_is_honored() {
        let r = RedactionRegion::from_rect(0.0, 0.0, 10.0, 5.0, None);
        // proportional = 0.1; custom floor 3.0 dominates.
        assert!(approx(r.effective_padding(3.0), 3.0));
    }

    #[test]
    fn effective_padding_rejects_non_finite_floor() {
        let r = RedactionRegion::from_rect(0.0, 0.0, 10.0, 5.0, None);
        assert!(approx(r.effective_padding(f32::NAN), DEFAULT_EDGE_PADDING));
        assert!(approx(r.effective_padding(-1.0), DEFAULT_EDGE_PADDING));
    }

    #[test]
    fn padded_rect_expands_on_all_sides() {
        let r = RedactionRegion::from_rect(10.0, 10.0, 30.0, 20.0, None);
        // height 10 → epsilon 0.5.
        let p = r.padded_rect(DEFAULT_EDGE_PADDING);
        assert!(approx(p.left(), 9.5));
        assert!(approx(p.top(), 9.5));
        assert!(approx(p.right(), 30.5));
        assert!(approx(p.bottom(), 20.5));
    }

    #[test]
    fn intersects_rect_conservative_overlap() {
        let region = RedactionRegion::from_rect(0.0, 0.0, 100.0, 20.0, None);

        // Mark fully inside.
        let inside = Rect::from_points(10.0, 5.0, 20.0, 15.0);
        assert!(region.intersects_rect(&inside, DEFAULT_EDGE_PADDING));

        // Mark far away.
        let far = Rect::from_points(500.0, 500.0, 520.0, 520.0);
        assert!(!region.intersects_rect(&far, DEFAULT_EDGE_PADDING));

        // Mark just outside the bbox but within the 0.5pt padding ⇒ inside
        // (conservative — defeats the anti-aliased-sliver recovery).
        let sliver = Rect::from_points(100.3, 5.0, 100.4, 15.0);
        assert!(region.intersects_rect(&sliver, DEFAULT_EDGE_PADDING));

        // Beyond the padding margin ⇒ outside.
        let beyond = Rect::from_points(101.0, 5.0, 102.0, 15.0);
        assert!(!region.intersects_rect(&beyond, DEFAULT_EDGE_PADDING));
    }

    #[test]
    fn contains_point_uses_padded_bbox() {
        let region = RedactionRegion::from_rect(0.0, 0.0, 10.0, 10.0, None);
        assert!(region.contains_point(&Point::new(5.0, 5.0), DEFAULT_EDGE_PADDING));
        // within padding margin
        assert!(region.contains_point(&Point::new(10.3, 5.0), DEFAULT_EDGE_PADDING));
        // outside
        assert!(!region.contains_point(&Point::new(50.0, 50.0), DEFAULT_EDGE_PADDING));
    }

    #[test]
    fn quad_contains_point_is_precise_and_winding_independent() {
        // Diamond (rotated square) centered at (50,50).
        let quad = [50.0, 0.0, 100.0, 50.0, 50.0, 100.0, 0.0, 50.0];
        let r = RedactionRegion::from_quad(quad, None);

        // Center is inside the diamond.
        assert_eq!(r.quad_contains_point(&Point::new(50.0, 50.0)), Some(true));
        // A bbox corner (0,0) is inside the envelope but OUTSIDE the diamond.
        assert_eq!(r.quad_contains_point(&Point::new(2.0, 2.0)), Some(false));
        // Edge midpoint is inclusive.
        assert_eq!(r.quad_contains_point(&Point::new(25.0, 25.0)), Some(true));

        // Reverse winding: same geometry, reversed corner order.
        let rev = [0.0, 50.0, 50.0, 100.0, 100.0, 50.0, 50.0, 0.0];
        let rr = RedactionRegion::from_quad(rev, None);
        assert_eq!(rr.quad_contains_point(&Point::new(50.0, 50.0)), Some(true));
        assert_eq!(rr.quad_contains_point(&Point::new(2.0, 2.0)), Some(false));

        // No quad ⇒ None.
        let ax = RedactionRegion::from_rect(0.0, 0.0, 10.0, 10.0, None);
        assert_eq!(ax.quad_contains_point(&Point::new(5.0, 5.0)), None);
    }

    #[test]
    fn region_set_basic_and_any_intersects() {
        let mut set = RegionSet::new(3);
        assert_eq!(set.page_index, 3);
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);

        set.push(RedactionRegion::from_rect(0.0, 0.0, 10.0, 10.0, None));
        set.push(RedactionRegion::from_rect(100.0, 100.0, 110.0, 110.0, None));
        assert!(!set.is_empty());
        assert_eq!(set.len(), 2);

        let hits_second = Rect::from_points(105.0, 105.0, 108.0, 108.0);
        assert!(set.any_intersects(&hits_second, DEFAULT_EDGE_PADDING));

        let hits_none = Rect::from_points(50.0, 50.0, 60.0, 60.0);
        assert!(!set.any_intersects(&hits_none, DEFAULT_EDGE_PADDING));
    }

    #[test]
    fn full_page_region_intersects_any_mark() {
        // G8: a region == MediaBox intersects every mark on the page.
        let page = RedactionRegion::from_rect(0.0, 0.0, 612.0, 792.0, None);
        for mark in [
            Rect::from_points(0.0, 0.0, 1.0, 1.0),
            Rect::from_points(300.0, 400.0, 320.0, 412.0),
            Rect::from_points(611.0, 791.0, 612.0, 792.0),
        ] {
            assert!(page.intersects_rect(&mark, DEFAULT_EDGE_PADDING));
        }
    }

    #[test]
    fn non_finite_inputs_do_not_panic() {
        // Adversarial: NaN/Inf coords must not panic (no-panic property, §4.2).
        let r = RedactionRegion::from_rect(f32::NAN, 0.0, 10.0, f32::INFINITY, None);
        let _ = r.width();
        let _ = r.height();
        let _ = r.effective_padding(DEFAULT_EDGE_PADDING);
        let _ = r.padded_rect(DEFAULT_EDGE_PADDING);
        let q = RedactionRegion::from_quad([f32::NAN; 8], None);
        let _ = q.quad_contains_point(&Point::new(0.0, 0.0));
    }
}
