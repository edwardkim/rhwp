//! Vector-path geometry primitives for destructive redaction (#231, T7).
//!
//! Path redaction (feature plan §4.2 step d / G4) drops subpaths fully
//! inside a region and rewrites straddling subpaths to the clipped
//! polygon — never a `W` clip operator, which would leave the original
//! path data recoverable in the stream (ISO 32000-1:2008 §8.5.4).
//!
//! This module provides the *pure* clipping primitive — Sutherland–
//! Hodgman of a polygon against an axis-aligned rectangle — plus a
//! bounding-box helper. It owns no operator-stream logic (SRP); the path
//! walker that accumulates `m l c v y re h` and reacts to the paint
//! operators composes these. Pure deterministic geometry with a known
//! reference algorithm, independently testable, not yet wired into any
//! redaction decision (so it cannot itself under-redact).

use crate::geometry::{Point, Rect};

/// Axis-aligned bounding box of a point set, or `None` if `< 1` point.
pub fn polygon_bbox(poly: &[Point]) -> Option<Rect> {
    let first = poly.first()?;
    let mut x0 = first.x;
    let mut y0 = first.y;
    let mut x1 = first.x;
    let mut y1 = first.y;
    for p in &poly[1..] {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    Some(Rect::from_points(x0, y0, x1, y1))
}

/// The four axis-aligned clip half-planes of a [`Rect`], each as a
/// predicate "is `p` on the inside of this edge" plus the segment-edge
/// intersection. `Rect::from_points` normalizes so `left<=right` and
/// `top<=bottom`; inside means `left<=x<=right && top<=y<=bottom`.
#[derive(Clone, Copy)]
enum Edge {
    Left(f32),
    Right(f32),
    Top(f32),
    Bottom(f32),
}

impl Edge {
    fn inside(self, p: Point) -> bool {
        match self {
            Edge::Left(l) => p.x >= l,
            Edge::Right(r) => p.x <= r,
            Edge::Top(t) => p.y >= t,
            Edge::Bottom(b) => p.y <= b,
        }
    }

    /// Intersection of segment `a→b` with this (infinite) edge line.
    /// Only called when `a` and `b` are on opposite sides, so the
    /// denominator is non-zero for the relevant axis.
    fn intersect(self, a: Point, b: Point) -> Point {
        match self {
            Edge::Left(x) | Edge::Right(x) => {
                let t = (x - a.x) / (b.x - a.x);
                Point::new(x, a.y + t * (b.y - a.y))
            },
            Edge::Top(y) | Edge::Bottom(y) => {
                let t = (y - a.y) / (b.y - a.y);
                Point::new(a.x + t * (b.x - a.x), y)
            },
        }
    }
}

fn clip_against_edge(subject: &[Point], edge: Edge) -> Vec<Point> {
    if subject.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(subject.len() + 4);
    let n = subject.len();
    for i in 0..n {
        let cur = subject[i];
        let prev = subject[(i + n - 1) % n];
        let cur_in = edge.inside(cur);
        let prev_in = edge.inside(prev);
        match (prev_in, cur_in) {
            (true, true) => out.push(cur),
            (true, false) => out.push(edge.intersect(prev, cur)),
            (false, true) => {
                out.push(edge.intersect(prev, cur));
                out.push(cur);
            },
            (false, false) => {},
        }
    }
    out
}

/// Clip convex-or-simple polygon `subject` to the interior of axis-aligned
/// rectangle `clip` (Sutherland–Hodgman).
///
/// Returns the clipped polygon's vertices in order. An empty result means
/// the subject lies entirely outside `clip` (or had `< 3` vertices, or
/// `clip` has zero area). Exact for translation/scale geometry; the
/// classic algorithm — used here only as a building block, never alone to
/// decide what survives redaction.
pub fn clip_polygon_to_rect(subject: &[Point], clip: &Rect) -> Vec<Point> {
    if subject.len() < 3 || clip.width <= 0.0 || clip.height <= 0.0 {
        return Vec::new();
    }
    let edges = [
        Edge::Left(clip.left()),
        Edge::Right(clip.right()),
        Edge::Top(clip.top()),
        Edge::Bottom(clip.bottom()),
    ];
    let mut poly = subject.to_vec();
    for e in edges {
        poly = clip_against_edge(&poly, e);
        if poly.is_empty() {
            return Vec::new();
        }
    }
    poly
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shoelace area (absolute) — order-independent invariant for the
    /// clip tests (comparing raw vertex lists is representation-fragile).
    fn area(poly: &[Point]) -> f32 {
        if poly.len() < 3 {
            return 0.0;
        }
        let mut s = 0.0;
        let n = poly.len();
        for i in 0..n {
            let a = poly[i];
            let b = poly[(i + 1) % n];
            s += a.x * b.y - b.x * a.y;
        }
        (s / 2.0).abs()
    }

    fn rect_poly(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<Point> {
        vec![
            Point::new(x0, y0),
            Point::new(x1, y0),
            Point::new(x1, y1),
            Point::new(x0, y1),
        ]
    }

    #[test]
    fn polygon_bbox_basic_and_empty() {
        assert!(polygon_bbox(&[]).is_none());
        let b = polygon_bbox(&[
            Point::new(3.0, 9.0),
            Point::new(-1.0, 2.0),
            Point::new(5.0, 4.0),
        ])
        .unwrap();
        assert!((b.left() - -1.0).abs() < 1e-6);
        assert!((b.top() - 2.0).abs() < 1e-6);
        assert!((b.right() - 5.0).abs() < 1e-6);
        assert!((b.bottom() - 9.0).abs() < 1e-6);
    }

    #[test]
    fn fully_inside_is_area_preserving() {
        let subj = rect_poly(2.0, 2.0, 8.0, 8.0); // area 36
        let clip = Rect::from_points(0.0, 0.0, 20.0, 20.0);
        let out = clip_polygon_to_rect(&subj, &clip);
        assert!(!out.is_empty());
        assert!((area(&out) - 36.0).abs() < 1e-3);
    }

    #[test]
    fn fully_outside_is_empty() {
        let subj = rect_poly(100.0, 100.0, 110.0, 110.0);
        let clip = Rect::from_points(0.0, 0.0, 10.0, 10.0);
        assert!(clip_polygon_to_rect(&subj, &clip).is_empty());
    }

    #[test]
    fn straddling_one_corner_clips_to_overlap() {
        // subject (0..10,0..10) area 100; clip (5..20,5..20).
        // Overlap is (5..10,5..10) → area 25.
        let subj = rect_poly(0.0, 0.0, 10.0, 10.0);
        let clip = Rect::from_points(5.0, 5.0, 20.0, 20.0);
        let out = clip_polygon_to_rect(&subj, &clip);
        assert!(!out.is_empty());
        assert!((area(&out) - 25.0).abs() < 1e-3);
        let bb = polygon_bbox(&out).unwrap();
        assert!((bb.left() - 5.0).abs() < 1e-3);
        assert!((bb.top() - 5.0).abs() < 1e-3);
        assert!((bb.right() - 10.0).abs() < 1e-3);
        assert!((bb.bottom() - 10.0).abs() < 1e-3);
    }

    #[test]
    fn clip_smaller_than_subject_yields_clip_area() {
        // clip fully inside subject → result is the clip rectangle.
        let subj = rect_poly(0.0, 0.0, 100.0, 100.0);
        let clip = Rect::from_points(40.0, 40.0, 60.0, 70.0); // 20 x 30 = 600
        let out = clip_polygon_to_rect(&subj, &clip);
        assert!((area(&out) - 600.0).abs() < 1e-2);
    }

    #[test]
    fn triangle_clipped_by_rect() {
        // Right triangle (0,0)-(10,0)-(0,10), area 50. Clip to
        // (0,0)-(5,5): overlap is the pentagon under the hypotenuse
        // within the 5x5 box. Hypotenuse y = 10 - x; inside the box the
        // cut runs (5,5)-(0,5)... area = 25 - small corner triangle
        // above line x+y=10 within box (none, since max x+y in box =10
        // only at (5,5)). So full 25.
        let tri = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
        ];
        let clip = Rect::from_points(0.0, 0.0, 5.0, 5.0);
        let out = clip_polygon_to_rect(&tri, &clip);
        assert!((area(&out) - 25.0).abs() < 1e-3);
    }

    #[test]
    fn degenerate_inputs_are_empty() {
        let clip = Rect::from_points(0.0, 0.0, 10.0, 10.0);
        assert!(clip_polygon_to_rect(&[], &clip).is_empty());
        assert!(clip_polygon_to_rect(&[Point::new(1.0, 1.0)], &clip).is_empty());
        assert!(
            clip_polygon_to_rect(&[Point::new(1.0, 1.0), Point::new(2.0, 2.0)], &clip).is_empty()
        );
        // zero-area clip
        let zclip = Rect::from_points(5.0, 5.0, 5.0, 5.0);
        assert!(clip_polygon_to_rect(&rect_poly(0.0, 0.0, 10.0, 10.0), &zclip).is_empty());
    }

    #[test]
    fn non_finite_does_not_panic() {
        let clip = Rect::from_points(0.0, 0.0, 10.0, 10.0);
        let subj = vec![
            Point::new(f32::NAN, 0.0),
            Point::new(10.0, f32::INFINITY),
            Point::new(0.0, 10.0),
        ];
        let _ = clip_polygon_to_rect(&subj, &clip);
        let _ = polygon_bbox(&subj);
    }
}
