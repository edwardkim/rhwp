//! Path/vector graphics content element types.
//!
//! This module provides the `PathContent` type for representing
//! vector graphics in PDFs.

use crate::extractors::text::ArtifactType;
use crate::geometry::Rect;
use crate::layout::Color;

/// One PDF clipping path, intersected with its parent in page user space.
#[derive(Debug, Clone)]
pub struct PathClip {
    /// Transformed path geometry.
    pub operations: Vec<PathOperation>,
    /// W* uses even-odd winding.
    pub even_odd: bool,
    /// Previously active clipping path.
    pub parent: Option<std::sync::Arc<PathClip>>,
}

/// An observed clipping operation, including clips ended by `n` with no paint.
/// Geometry is in page user space; this is not evidence of a painted border.
#[derive(Debug, Clone)]
pub struct PathClipObservation {
    /// Path-extractor invocation path of the terminating operator.
    pub source_order: Vec<usize>,
    /// CTM in effect when the path ended; geometry has already been transformed.
    pub transform: [f32; 6],
    /// The observed clip and its prior active clip, without inventing paint.
    pub clip: std::sync::Arc<PathClip>,
}

/// Bounded observation ledger, separate from the live graphics state.
#[derive(Debug, Default)]
pub struct PathClipObservations {
    /// Recorded clipping operations, in deterministic extraction order.
    pub clips: Vec<PathClipObservation>,
    /// Unrecorded clips remain in the live graphics state; inference is incomplete.
    pub omitted: usize,
}

/// Paint metadata retained by the bounded positional importer.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PathPaint {
    /// Content-stream invocation path, compared lexicographically.
    pub source_order: Vec<usize>,
    /// Nonstroking alpha constant (ca).
    pub fill_alpha: f32,
    /// Stroking alpha constant (CA).
    pub stroke_alpha: f32,
    /// f*/B* select even-odd rather than nonzero winding.
    pub even_odd: bool,
    /// Supported page-space clipping region.
    #[serde(skip)]
    pub clip: crate::extractors::image_clip::ImageClip,
    /// Full clipping geometry when a rectangular crop is insufficient.
    #[serde(skip)]
    pub clip_path: Option<std::sync::Arc<PathClip>>,
}
impl Default for PathPaint {
    fn default() -> Self {
        Self {
            source_order: Vec::new(),
            fill_alpha: 1.0,
            stroke_alpha: 1.0,
            even_odd: false,
            clip: crate::extractors::image_clip::ImageClip::Unbounded,
            clip_path: None,
        }
    }
}

/// Vector path content that can be extracted from or written to a PDF.
///
/// This represents vector graphics such as lines, curves, and shapes.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PathContent {
    /// Source paint state; independent of reading order.
    pub paint: PathPaint,
    /// Bounding box of the path
    pub bbox: Rect,
    /// Path operations
    pub operations: Vec<PathOperation>,
    /// Stroke color (None for no stroke)
    pub stroke_color: Option<Color>,
    /// Fill color (None for no fill)
    pub fill_color: Option<Color>,
    /// Stroke width in points
    pub stroke_width: f32,
    /// Line cap style
    pub line_cap: LineCap,
    /// Line join style
    pub line_join: LineJoin,
    /// Optional dash pattern: `Some((dashes, phase))` emits a
    /// `[dashes...] phase d` operator before stroking. `dashes` is on/
    /// off lengths in points (e.g. `[3.0, 2.0]` = dash 3 pt, gap 2 pt,
    /// repeating); `phase` is the starting offset. `None` leaves the
    /// line solid.
    #[serde(default)]
    pub dash_pattern: Option<(Vec<f32>, f32)>,
    /// Optional 2D affine transform in PDF row order `[a b c d e f]`.
    /// When set, the path is wrapped in `q ... cm ... Q` on emission
    /// so graphics-state stays scoped. Populated by
    /// `FluentPageBuilder::{rotated, scaled, translated, with_transform}`
    /// closures — v0.3.39 (text-only) extended here to cover paths.
    /// #393 Bundle A-2 follow-up.
    #[serde(default)]
    pub matrix: Option<[f32; 6]>,
    /// Reading order index
    pub reading_order: Option<usize>,
    /// When set, this path is wrapped in `/Artifact <</Type /T>>  BDC … EMC`
    /// markers so accessibility tools can skip it. Useful for decorative
    /// separator lines (footnote rules, header/footer rules).
    #[serde(skip)]
    pub artifact_type: Option<ArtifactType>,
    /// Optional Content Group (PDF "layer") name resolved from the
    /// surrounding `BDC /OC … EMC` markers in the content stream. Set
    /// during path extraction by `PathExtractor` when an OCG is active;
    /// `None` for paths emitted outside any `/OC`-tagged marked-content
    /// region or in PDFs that do not declare optional content.
    ///
    /// The string is the human-readable `/Name` entry of the referenced
    /// `OptionalContentGroup`, e.g. `"A-GRID"`, `"S-COLS"`, `"A-WALL-DIM"`
    /// for PDFs exported from Revit/AutoCAD with layer metadata intact.
    /// Reference: ISO 32000-1:2008 §8.11 (Optional Content) + §14.6
    /// (Marked Content).
    #[serde(default)]
    pub layer: Option<String>,
}

impl PathContent {
    /// Create a new empty path content element.
    pub fn new(bbox: Rect) -> Self {
        Self {
            paint: PathPaint::default(),
            bbox,
            operations: Vec::new(),
            stroke_color: Some(Color::black()),
            fill_color: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            dash_pattern: None,
            matrix: None,
            reading_order: None,
            artifact_type: None,
            layer: None,
        }
    }

    /// Create a path from operations.
    pub fn from_operations(operations: Vec<PathOperation>) -> Self {
        let bbox = Self::compute_bbox(&operations);
        Self {
            paint: PathPaint::default(),
            bbox,
            operations,
            stroke_color: Some(Color::black()),
            fill_color: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            dash_pattern: None,
            matrix: None,
            reading_order: None,
            artifact_type: None,
            layer: None,
        }
    }

    /// Set stroke color.
    pub fn with_stroke(mut self, color: Color) -> Self {
        self.stroke_color = Some(color);
        self
    }

    /// Set fill color.
    pub fn with_fill(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    /// Set stroke width.
    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Set reading order.
    pub fn with_reading_order(mut self, order: usize) -> Self {
        self.reading_order = Some(order);
        self
    }

    /// Set the Optional Content Group (PDF "layer") name. Used by
    /// `PathExtractor` while walking the content stream to attach the
    /// active OCG name to each extracted path.
    pub fn with_layer(mut self, layer: impl Into<String>) -> Self {
        self.layer = Some(layer.into());
        self
    }

    /// Add a path operation.
    pub fn push(&mut self, op: PathOperation) {
        self.operations.push(op);
    }

    /// Check if this path has a stroke.
    pub fn has_stroke(&self) -> bool {
        self.stroke_color.is_some() && self.stroke_width > 0.0
    }

    /// Check if this path has a fill.
    pub fn has_fill(&self) -> bool {
        self.fill_color.is_some()
    }

    /// Check if this path represents a single straight line (v0.3.14).
    ///
    /// A path is a straight line if it has exactly 2 operations:
    /// MoveTo followed by LineTo.
    pub fn is_straight_line(&self) -> bool {
        (self.operations.len() == 2
            && matches!(self.operations[0], PathOperation::MoveTo(_, _))
            && matches!(self.operations[1], PathOperation::LineTo(_, _)))
            || (self.operations.len() == 3
                && matches!(self.operations[0], PathOperation::MoveTo(_, _))
                && matches!(self.operations[1], PathOperation::LineTo(_, _))
                && matches!(self.operations[2], PathOperation::ClosePath))
    }

    /// Rendered extents of this path: the geometric [`bbox`](Self::bbox)
    /// inflated by the stroke (ISO 32000-1:2008 §8.4.3.2 — half the line
    /// width straddles each side of the path).
    ///
    /// Print-era producers draw a table's vertical rule as a ~1 pt
    /// horizontal segment stroked as wide as the table is tall, so the
    /// geometric bbox (1×0 pt) bears no resemblance to the bar the reader
    /// sees. Classifiers that reason about what is *rendered*
    /// (line orientation, table primitives, region hit-testing) should use
    /// this; consumers that need the raw path geometry (corner tests, SVG
    /// path data, editing) keep `bbox`.
    ///
    /// For a single straight segment the inflation is exact: half the
    /// stroke width perpendicular to the segment, plus the cap extension
    /// along it (`Butt` caps add nothing axially; `Round`/`Square` add half
    /// the width per end, §8.4.3.3). For any other shape the bbox is
    /// inflated by half the width on all four sides — a conservative
    /// superset that is exact for axis-aligned stroked outlines.
    ///
    /// Unstroked paths return `bbox` unchanged.
    pub fn rendered_bbox(&self) -> Rect {
        if !self.has_stroke() {
            return self.bbox;
        }
        let half = self.stroke_width * 0.5;

        if let [PathOperation::MoveTo(x0, y0), PathOperation::LineTo(x1, y1), ..] =
            self.operations.as_slice()
        {
            if self.is_straight_line() {
                let (dx, dy) = (x1 - x0, y1 - y0);
                let len = dx.hypot(dy);
                // A zero-length butt-capped segment paints nothing
                // (§8.4.3.3), so its rendered extent is its degenerate
                // geometry; Round/Square caps paint a dot and fall through
                // to the all-sides outset below.
                if len <= f32::EPSILON && self.line_cap == LineCap::Butt {
                    return self.bbox;
                }
                if len > f32::EPSILON {
                    // Perpendicular unit vector scaled to the half-width,
                    // plus the axial cap extension.
                    let (px, py) = ((dy / len * half).abs(), (dx / len * half).abs());
                    let (cx, cy) = if self.line_cap == LineCap::Butt {
                        (0.0, 0.0)
                    } else {
                        ((dx / len * half).abs(), (dy / len * half).abs())
                    };
                    return Rect::new(
                        self.bbox.x - px - cx,
                        self.bbox.y - py - cy,
                        self.bbox.width + 2.0 * (px + cx),
                        self.bbox.height + 2.0 * (py + cy),
                    );
                }
            }
        }

        Rect::new(
            self.bbox.x - half,
            self.bbox.y - half,
            self.bbox.width + 2.0 * half,
            self.bbox.height + 2.0 * half,
        )
    }

    /// Check if this path is a horizontal line within a tolerance (v0.3.16).
    ///
    /// `tolerance` bounds the *geometric* minor extent as before (a nearly
    /// axis-aligned segment or sliver rectangle). Orientation is then
    /// confirmed on the [`rendered_bbox`](Self::rendered_bbox): a 1 pt
    /// segment stroked 430 pt wide renders as a vertical bar and must not
    /// classify as a horizontal line just because its raw geometry is a
    /// horizontal speck.
    pub fn is_horizontal_line(&self, tolerance: f32) -> bool {
        if !self.is_straight_line() && !self.is_rectangle() {
            return false;
        }
        let rendered = self.rendered_bbox();
        self.bbox.height.abs() < tolerance && rendered.width.abs() >= rendered.height.abs()
    }

    /// Check if this path is a vertical line within a tolerance (v0.3.16).
    ///
    /// Same contract as [`is_horizontal_line`](Self::is_horizontal_line):
    /// geometric thinness within `tolerance`, orientation confirmed on the
    /// rendered extents.
    pub fn is_vertical_line(&self, tolerance: f32) -> bool {
        if !self.is_straight_line() && !self.is_rectangle() {
            return false;
        }
        let rendered = self.rendered_bbox();
        self.bbox.width.abs() < tolerance && rendered.height.abs() >= rendered.width.abs()
    }

    /// Check if this path's bounding box is nearly touching another (v0.3.16).
    pub fn is_nearly_touching(&self, other: &Rect, tolerance: f32) -> bool {
        let expanded = Rect::new(
            self.bbox.x - tolerance,
            self.bbox.y - tolerance,
            self.bbox.width + 2.0 * tolerance,
            self.bbox.height + 2.0 * tolerance,
        );
        expanded.intersects(other)
    }

    /// Check if this path represents a single rectangle (v0.3.14).
    ///
    /// A path is a rectangle if it has exactly 1 operation: Rectangle,
    /// or if it has 5 operations: MoveTo, 3x LineTo, ClosePath that form a rectangle.
    pub fn is_rectangle(&self) -> bool {
        // Case 1: Simple Rectangle operator (re)
        if self.operations.len() == 1
            && matches!(self.operations[0], PathOperation::Rectangle(_, _, _, _))
        {
            return true;
        }

        // Case 2: MoveTo + 3x LineTo + (Optional ClosePath)
        // Must be axis-aligned. We check that consecutive points share X or Y.
        if (self.operations.len() == 5 && matches!(self.operations[4], PathOperation::ClosePath))
            || (self.operations.len() == 4)
        {
            if let (
                PathOperation::MoveTo(x0, y0),
                PathOperation::LineTo(x1, y1),
                PathOperation::LineTo(x2, y2),
                PathOperation::LineTo(x3, y3),
            ) = (
                &self.operations[0],
                &self.operations[1],
                &self.operations[2],
                &self.operations[3],
            ) {
                let tol = 0.1;
                // Check if p0..p3 form 3 sides of an axis-aligned rect
                let side1 = ((x0 - x1).abs() < tol) || ((y0 - y1).abs() < tol);
                let side2 = ((x1 - x2).abs() < tol) || ((y1 - y2).abs() < tol);
                let side3 = ((x2 - x3).abs() < tol) || ((y2 - y3).abs() < tol);

                return side1 && side2 && side3;
            }
        }

        false
    }

    /// Check if this path is "box-like" or "line-like" based on its dimensions (v0.3.16).
    /// This is a fuzzy heuristic for table detection.
    ///
    /// Line length is judged on the [`rendered_bbox`](Self::rendered_bbox)
    /// while thinness stays geometric, so a table rule encoded as a short
    /// segment with a table-height stroke width — a 1×0 pt geometric speck
    /// rendering as a 1×430 pt bar — is recognized as the ruling it draws
    ///, and an ordinary rule with a thick stroke keeps
    /// classifying as a line exactly as before.
    ///
    /// Thresholds (inherited from the v0.3.16 heuristic): a ruling must
    /// run more than 5 pt — half an em at 10 pt body text — so tick marks
    /// and list bullets don't qualify; it must be under 2 pt across, twice
    /// the heaviest common ruling weight (typical PDF rules are 0.5–1 pt)
    /// while safely below any filled cell; and a box must stay under
    /// 1000 pt per side, which exceeds a US-Letter page (612×792), so only
    /// full-page frames and margin decorations are excluded.
    pub fn is_table_primitive(&self) -> bool {
        let rendered = self.rendered_bbox();

        // Very thin horizontal or vertical line: extends > 5.0 along its
        // rendered axis, geometrically thinner than 2.0 across it, and
        // rendered at least 2:1 elongated — a zero-length segment with a
        // fat round/square cap (dot leaders, bullets) renders as a square
        // blob, not a ruling, and must not seed line clusters.
        if (rendered.width.abs() > 5.0
            && self.bbox.height.abs() < 2.0
            && rendered.width.abs() >= 2.0 * rendered.height.abs())
            || (rendered.height.abs() > 5.0
                && self.bbox.width.abs() < 2.0
                && rendered.height.abs() >= 2.0 * rendered.width.abs())
        {
            return true;
        }

        // Rectangular-ish box (not too small, not too large) — geometric,
        // as before.
        let w = self.bbox.width.abs();
        let h = self.bbox.height.abs();
        if w > 5.0 && h > 5.0 && w < 1000.0 && h < 1000.0 {
            return true;
        }

        false
    }

    // === Convenience Constructors ===

    /// Create a line path from (x1, y1) to (x2, y2).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let line = PathContent::line(10.0, 10.0, 100.0, 100.0);
    /// ```
    pub fn line(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let ops = vec![PathOperation::MoveTo(x1, y1), PathOperation::LineTo(x2, y2)];
        Self::from_operations(ops)
    }

    /// Create a rectangle path.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let rect = PathContent::rect(10.0, 10.0, 100.0, 50.0);
    /// ```
    pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Self {
        let ops = vec![PathOperation::Rectangle(x, y, width, height)];
        Self::from_operations(ops)
    }

    /// Create an approximate circle path using Bezier curves.
    ///
    /// Uses 4 cubic Bezier curves to approximate a circle.
    /// The approximation uses the constant k = 0.5522847498 for control points.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let circle = PathContent::circle(100.0, 100.0, 50.0);
    /// ```
    pub fn circle(cx: f32, cy: f32, radius: f32) -> Self {
        // Magic constant for approximating a quarter circle with a cubic Bezier
        // k = 4 * (sqrt(2) - 1) / 3 ≈ 0.5522847498
        const K: f32 = 0.552_284_8;
        let k = radius * K;

        let ops = vec![
            // Start at top
            PathOperation::MoveTo(cx, cy + radius),
            // Top-right quadrant
            PathOperation::CurveTo(cx + k, cy + radius, cx + radius, cy + k, cx + radius, cy),
            // Bottom-right quadrant
            PathOperation::CurveTo(cx + radius, cy - k, cx + k, cy - radius, cx, cy - radius),
            // Bottom-left quadrant
            PathOperation::CurveTo(cx - k, cy - radius, cx - radius, cy - k, cx - radius, cy),
            // Top-left quadrant
            PathOperation::CurveTo(cx - radius, cy + k, cx - k, cy + radius, cx, cy + radius),
            PathOperation::ClosePath,
        ];
        Self::from_operations(ops)
    }

    /// Create a rounded rectangle path.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the bottom-left corner
    /// * `y` - Y coordinate of the bottom-left corner
    /// * `width` - Width of the rectangle
    /// * `height` - Height of the rectangle
    /// * `radius` - Corner radius (clamped to min(width, height) / 2)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let rounded = PathContent::rounded_rect(10.0, 10.0, 100.0, 50.0, 5.0);
    /// ```
    pub fn rounded_rect(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self {
        // Clamp radius to maximum valid value
        let max_radius = width.min(height) / 2.0;
        let r = radius.min(max_radius).max(0.0);

        if r <= 0.0 {
            return Self::rect(x, y, width, height);
        }

        // Magic constant for approximating a quarter circle with a cubic Bezier
        const K: f32 = 0.552_284_8;
        let k = r * K;

        let x_right = x + width;
        let y_top = y + height;

        let ops = vec![
            // Start at bottom-left corner, right of curve
            PathOperation::MoveTo(x + r, y),
            // Bottom edge
            PathOperation::LineTo(x_right - r, y),
            // Bottom-right corner curve
            PathOperation::CurveTo(x_right - r + k, y, x_right, y + r - k, x_right, y + r),
            // Right edge
            PathOperation::LineTo(x_right, y_top - r),
            // Top-right corner curve
            PathOperation::CurveTo(
                x_right,
                y_top - r + k,
                x_right - r + k,
                y_top,
                x_right - r,
                y_top,
            ),
            // Top edge
            PathOperation::LineTo(x + r, y_top),
            // Top-left corner curve
            PathOperation::CurveTo(x + r - k, y_top, x, y_top - r + k, x, y_top - r),
            // Left edge
            PathOperation::LineTo(x, y + r),
            // Bottom-left corner curve
            PathOperation::CurveTo(x, y + r - k, x + r - k, y, x + r, y),
            PathOperation::ClosePath,
        ];
        Self::from_operations(ops)
    }

    /// Flatten this path into polylines — one `Vec<(x, y)>` per subpath.
    ///
    /// Straight segments (`MoveTo`/`LineTo`) pass through unchanged; cubic
    /// Béziers (`CurveTo`) are adaptively subdivided so the resulting polyline
    /// stays within `tolerance` of the true curve. A `Rectangle` becomes its
    /// own closed 5-point subpath, and `ClosePath` appends the subpath's start
    /// point. A new subpath begins at every `MoveTo` (and every `Rectangle`).
    ///
    /// `tolerance` is expressed in the same coordinate space as the path's own
    /// points, and the returned points are in that space verbatim. For paths
    /// from [`crate::document::PdfDocument::extract_paths`] that is the page's
    /// default user space (PDF points, y-up), with any content-stream `cm` and
    /// Form-XObject `/Matrix` transforms already folded in — so a path drawn
    /// under a scaling transform is returned (and flattened) at its final
    /// on-page size, and no tolerance rescaling is needed. It is *not* device or
    /// pixel space: page `/Rotate` and rendering DPI are not applied. Smaller
    /// values yield more, finer points. A non-positive or non-finite tolerance
    /// is floored to a small epsilon so subdivision always terminates.
    ///
    /// This is intended for consumers that need sampled coordinates rather than
    /// drawing operators — e.g. digitising chart/ECG/CAD traces from vector PDFs.
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::elements::{PathContent, PathOperation};
    ///
    /// let path = PathContent::from_operations(vec![
    ///     PathOperation::MoveTo(0.0, 0.0),
    ///     PathOperation::CurveTo(0.0, 10.0, 10.0, 10.0, 10.0, 0.0),
    /// ]);
    /// let subpaths = path.to_points(0.25);
    /// assert_eq!(subpaths.len(), 1);
    /// assert_eq!(subpaths[0][0], (0.0, 0.0)); // starts at P0
    /// ```
    pub fn to_points(&self, tolerance: f32) -> Vec<Vec<(f32, f32)>> {
        // A non-finite or non-positive tolerance would make the flatness test
        // never pass, recursing to the depth cap on every curve. Floor it.
        let tol = if tolerance.is_finite() && tolerance > 0.0 {
            tolerance
        } else {
            FLATTEN_TOLERANCE_FLOOR
        };

        let mut subpaths: Vec<Vec<(f32, f32)>> = Vec::new();
        let mut current: Vec<(f32, f32)> = Vec::new();
        let mut pos = (0.0_f32, 0.0_f32);
        let mut start = (0.0_f32, 0.0_f32);

        for op in &self.operations {
            match *op {
                PathOperation::MoveTo(x, y) => {
                    // `m` begins a new subpath; a consecutive `m` overrides the
                    // previous one with "no vestige" (§8.5.2, Table 59), which
                    // flush_subpath honours by dropping the lone start point.
                    flush_subpath(&mut current, &mut subpaths);
                    pos = (x, y);
                    start = pos;
                    current.push(pos);
                },
                PathOperation::LineTo(x, y) => {
                    // A segment with no open subpath (e.g. right after `re`/`h`,
                    // or a malformed leading `l`) starts at the current point.
                    if current.is_empty() {
                        start = pos;
                        current.push(pos);
                    }
                    pos = (x, y);
                    current.push(pos);
                },
                PathOperation::CurveTo(c1x, c1y, c2x, c2y, ex, ey) => {
                    if current.is_empty() {
                        start = pos;
                        current.push(pos);
                    }
                    flatten_cubic(pos, (c1x, c1y), (c2x, c2y), (ex, ey), tol, 0, &mut current);
                    pos = (ex, ey);
                },
                PathOperation::Rectangle(x, y, w, h) => {
                    flush_subpath(&mut current, &mut subpaths);
                    // `re` is a complete closed subpath equivalent to
                    // `x y m / (x+w) y l / (x+w) (y+h) l / x (y+h) l / h`
                    // (§8.5.2, Table 59). The current point afterwards is (x, y).
                    subpaths.push(vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)]);
                    pos = (x, y);
                    start = pos;
                },
                PathOperation::ClosePath => {
                    // `h` appends a segment back to the subpath start and
                    // terminates the subpath; a following segment begins a new
                    // one (§8.5.2, Table 59). On an already-closed/empty subpath
                    // it does nothing.
                    if !current.is_empty() {
                        current.push(start);
                        flush_subpath(&mut current, &mut subpaths);
                        pos = start;
                    }
                },
            }
        }
        flush_subpath(&mut current, &mut subpaths);
        subpaths
    }

    /// Compute bounding box from path operations.
    fn compute_bbox(operations: &[PathOperation]) -> Rect {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for op in operations {
            match op {
                PathOperation::MoveTo(x, y) | PathOperation::LineTo(x, y) => {
                    min_x = min_x.min(*x);
                    min_y = min_y.min(*y);
                    max_x = max_x.max(*x);
                    max_y = max_y.max(*y);
                },
                PathOperation::CurveTo(x1, y1, x2, y2, x3, y3) => {
                    for (x, y) in [(*x1, *y1), (*x2, *y2), (*x3, *y3)] {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                },
                PathOperation::Rectangle(x, y, w, h) => {
                    min_x = min_x.min(*x);
                    min_y = min_y.min(*y);
                    max_x = max_x.max(*x + *w);
                    max_y = max_y.max(*y + *h);
                },
                PathOperation::ClosePath => {},
            }
        }

        if min_x == f32::MAX {
            Rect::new(0.0, 0.0, 0.0, 0.0)
        } else {
            Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
        }
    }
}

/// Minimum flattening tolerance (path units) used by [`PathContent::to_points`]
/// when given a non-positive or non-finite value, so subdivision terminates.
const FLATTEN_TOLERANCE_FLOOR: f32 = 1e-3;

/// Hard recursion-depth backstop for adaptive cubic subdivision. The flatness
/// test halts long before this in practice; it only guards pathological input.
const FLATTEN_MAX_DEPTH: u8 = 24;

/// Move a finished subpath from `current` into `out`, discarding it if it has
/// fewer than two points. A lone point is a subpath that adds no segment and
/// paints nothing (§8.5.2: construction operators place no marks), e.g. a
/// trailing `m` or the first of two consecutive `m` operators ("no vestige").
fn flush_subpath(current: &mut Vec<(f32, f32)>, out: &mut Vec<Vec<(f32, f32)>>) {
    if current.len() >= 2 {
        out.push(std::mem::take(current));
    } else {
        current.clear();
    }
}

/// Adaptively subdivide a cubic Bézier (`p0`→`p3`, controls `p1`,`p2`),
/// appending flattened vertices to `out`. The start point `p0` is assumed to
/// already be the last element of `out`, so only intermediate vertices and the
/// endpoint `p3` are pushed.
fn flatten_cubic(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    tol: f32,
    depth: u8,
    out: &mut Vec<(f32, f32)>,
) {
    // The curve lies inside the convex hull of its control points; since p0 and
    // p3 are on the chord, the curve's deviation from the chord is bounded by
    // the larger control-point offset. When that is within `tol`, a single
    // straight segment is a good-enough approximation.
    if depth >= FLATTEN_MAX_DEPTH || cubic_is_flat(p0, p1, p2, p3, tol) {
        out.push(p3);
        return;
    }
    // de Casteljau split at t = 0.5; the split point lies on the true curve.
    let p01 = midpoint(p0, p1);
    let p12 = midpoint(p1, p2);
    let p23 = midpoint(p2, p3);
    let p012 = midpoint(p01, p12);
    let p123 = midpoint(p12, p23);
    let mid = midpoint(p012, p123);
    flatten_cubic(p0, p01, p012, mid, tol, depth + 1, out);
    flatten_cubic(mid, p123, p23, p3, tol, depth + 1, out);
}

fn midpoint(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5)
}

/// True when both control points lie within `tol` of the chord `p0`→`p3`.
fn cubic_is_flat(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), tol: f32) -> bool {
    dist_point_to_segment(p1, p0, p3).max(dist_point_to_segment(p2, p0, p3)) <= tol
}

/// Shortest distance from point `p` to the line segment `a`→`b`.
fn dist_point_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        // Degenerate chord (a == b): distance to the single point.
        return ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
    }
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0);
    let (cx, cy) = (a.0 + t * dx, a.1 + t * dy);
    ((p.0 - cx).powi(2) + (p.1 - cy).powi(2)).sqrt()
}

impl Default for PathContent {
    fn default() -> Self {
        Self {
            paint: PathPaint::default(),
            bbox: Rect::new(0.0, 0.0, 0.0, 0.0),
            operations: Vec::new(),
            stroke_color: Some(Color::black()),
            fill_color: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            dash_pattern: None,
            matrix: None,
            reading_order: None,
            artifact_type: None,
            layer: None,
        }
    }
}

/// A single path operation.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum PathOperation {
    /// Move to a point (m operator)
    MoveTo(f32, f32),
    /// Line to a point (l operator)
    LineTo(f32, f32),
    /// Bezier curve to a point (c operator)
    /// (control1_x, control1_y, control2_x, control2_y, end_x, end_y)
    CurveTo(f32, f32, f32, f32, f32, f32),
    /// Rectangle (re operator)
    /// (x, y, width, height)
    Rectangle(f32, f32, f32, f32),
    /// Close the current path (h operator)
    ClosePath,
}

/// Line cap style for strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub enum LineCap {
    /// Butt cap - line ends exactly at endpoint
    #[default]
    Butt,
    /// Round cap - semicircle at endpoint
    Round,
    /// Square cap - half square at endpoint
    Square,
}

/// Line join style for strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub enum LineJoin {
    /// Miter join - sharp corner
    #[default]
    Miter,
    /// Round join - circular arc
    Round,
    /// Bevel join - diagonal corner
    Bevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_content_creation() {
        let path = PathContent::new(Rect::new(0.0, 0.0, 100.0, 100.0))
            .with_stroke(Color::black())
            .with_stroke_width(2.0);

        assert!(path.has_stroke());
        assert!(!path.has_fill());
        assert_eq!(path.stroke_width, 2.0);
    }

    #[test]
    fn test_path_from_operations() {
        let ops = vec![
            PathOperation::MoveTo(10.0, 10.0),
            PathOperation::LineTo(50.0, 10.0),
            PathOperation::LineTo(50.0, 50.0),
            PathOperation::LineTo(10.0, 50.0),
            PathOperation::ClosePath,
        ];

        let path = PathContent::from_operations(ops);

        assert_eq!(path.bbox.x, 10.0);
        assert_eq!(path.bbox.y, 10.0);
        assert_eq!(path.bbox.width, 40.0);
        assert_eq!(path.bbox.height, 40.0);
    }

    #[test]
    fn test_path_with_fill() {
        let path = PathContent::new(Rect::new(0.0, 0.0, 100.0, 100.0))
            .with_fill(Color::new(1.0, 0.0, 0.0));

        assert!(path.has_fill());
        assert!(path.has_stroke()); // Default has stroke
    }

    #[test]
    fn test_compute_bbox_from_rectangle() {
        let ops = vec![PathOperation::Rectangle(20.0, 30.0, 100.0, 50.0)];
        let path = PathContent::from_operations(ops);

        assert_eq!(path.bbox.x, 20.0);
        assert_eq!(path.bbox.y, 30.0);
        assert_eq!(path.bbox.width, 100.0);
        assert_eq!(path.bbox.height, 50.0);
    }

    // === to_points (issue #147) ===

    /// Ground-truth cubic Bézier evaluation, used to validate flattening.
    fn cubic_at(
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
        t: f32,
    ) -> (f32, f32) {
        let mt = 1.0 - t;
        let x = mt * mt * mt * p0.0
            + 3.0 * mt * mt * t * p1.0
            + 3.0 * mt * t * t * p2.0
            + t * t * t * p3.0;
        let y = mt * mt * mt * p0.1
            + 3.0 * mt * mt * t * p1.1
            + 3.0 * mt * t * t * p2.1
            + t * t * t * p3.1;
        (x, y)
    }

    /// Shortest distance from a point to a polyline (min over its segments),
    /// reusing the production segment-distance helper.
    fn dist_to_polyline(p: (f32, f32), poly: &[(f32, f32)]) -> f32 {
        poly.windows(2)
            .map(|s| dist_point_to_segment(p, s[0], s[1]))
            .fold(f32::MAX, f32::min)
    }

    #[test]
    fn test_to_points_straight_line_passthrough() {
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(10.0, 10.0),
            PathOperation::LineTo(100.0, 40.0),
        ]);
        let pts = path.to_points(0.5);
        assert_eq!(pts, vec![vec![(10.0, 10.0), (100.0, 40.0)]]);
    }

    #[test]
    fn test_to_points_curve_endpoints_preserved() {
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(0.0, 0.0),
            PathOperation::CurveTo(0.0, 100.0, 100.0, 100.0, 100.0, 0.0),
        ]);
        let sub = &path.to_points(0.1)[0];
        assert_eq!(sub.first().copied(), Some((0.0, 0.0)), "must start at P0");
        let last = sub.last().copied().unwrap();
        assert!(
            (last.0 - 100.0).abs() < 1e-3 && (last.1 - 0.0).abs() < 1e-3,
            "must end at P3, got {last:?}"
        );
    }

    #[test]
    fn test_to_points_curve_within_tolerance() {
        let p0 = (0.0, 0.0);
        let p1 = (0.0, 100.0);
        let p2 = (100.0, 100.0);
        let p3 = (100.0, 0.0);
        let tol = 0.5;
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(p0.0, p0.1),
            PathOperation::CurveTo(p1.0, p1.1, p2.0, p2.1, p3.0, p3.1),
        ]);
        let poly = &path.to_points(tol)[0];
        // Every point on the true curve must lie within `tol` of the polyline.
        for i in 0..=200 {
            let t = i as f32 / 200.0;
            let truth = cubic_at(p0, p1, p2, p3, t);
            let d = dist_to_polyline(truth, poly);
            assert!(d <= tol + 1e-3, "curve point at t={t} is {d} from polyline (tol={tol})");
        }
    }

    #[test]
    fn test_to_points_tolerance_monotonic() {
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(0.0, 0.0),
            PathOperation::CurveTo(0.0, 100.0, 100.0, 100.0, 100.0, 0.0),
        ]);
        let coarse = path.to_points(10.0)[0].len();
        let fine = path.to_points(0.05)[0].len();
        assert!(
            fine >= coarse,
            "finer tolerance must not reduce point count ({fine} < {coarse})"
        );
        assert!(fine > 2, "fine flattening must densify the curve, got {fine}");
    }

    #[test]
    fn test_to_points_rectangle_is_closed_subpath() {
        let path =
            PathContent::from_operations(vec![PathOperation::Rectangle(10.0, 20.0, 30.0, 40.0)]);
        let pts = path.to_points(1.0);
        assert_eq!(
            pts,
            vec![vec![
                (10.0, 20.0),
                (40.0, 20.0),
                (40.0, 60.0),
                (10.0, 60.0),
                (10.0, 20.0)
            ]]
        );
    }

    #[test]
    fn test_to_points_segment_after_rectangle_continues_from_current_point() {
        // Per PDF §8.5.2 the current point after `re` is the rectangle's
        // lower-left; a following segment continues from there as a new subpath.
        let path = PathContent::from_operations(vec![
            PathOperation::Rectangle(0.0, 0.0, 10.0, 10.0),
            PathOperation::LineTo(20.0, 20.0),
        ]);
        let pts = path.to_points(1.0);
        assert_eq!(pts.len(), 2);
        assert_eq!(
            pts[0],
            vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (0.0, 10.0),
                (0.0, 0.0)
            ]
        );
        assert_eq!(pts[1], vec![(0.0, 0.0), (20.0, 20.0)]);
    }

    #[test]
    fn test_to_points_closepath_appends_start() {
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(0.0, 0.0),
            PathOperation::LineTo(10.0, 0.0),
            PathOperation::LineTo(10.0, 10.0),
            PathOperation::ClosePath,
        ]);
        assert_eq!(
            path.to_points(1.0),
            vec![vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 0.0)]]
        );
    }

    #[test]
    fn test_to_points_closepath_terminates_subpath() {
        // Per §8.5.2 Table 59, `h` terminates the subpath; a following segment
        // begins a NEW subpath (seeded from the subpath start = current point),
        // rather than extending the closed loop.
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(0.0, 0.0),
            PathOperation::LineTo(10.0, 0.0),
            PathOperation::ClosePath,
            PathOperation::LineTo(5.0, 5.0),
        ]);
        let pts = path.to_points(1.0);
        assert_eq!(pts.len(), 2, "h must terminate the subpath");
        assert_eq!(pts[0], vec![(0.0, 0.0), (10.0, 0.0), (0.0, 0.0)]);
        assert_eq!(pts[1], vec![(0.0, 0.0), (5.0, 5.0)]);
    }

    #[test]
    fn test_to_points_consecutive_moveto_leaves_no_vestige() {
        // Per §8.5.2 Table 59, a consecutive `m` overrides the previous one with
        // no vestige; the orphaned start point must not appear as a subpath.
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(1.0, 1.0),
            PathOperation::MoveTo(2.0, 2.0),
            PathOperation::LineTo(3.0, 3.0),
        ]);
        assert_eq!(path.to_points(1.0), vec![vec![(2.0, 2.0), (3.0, 3.0)]]);
    }

    #[test]
    fn test_to_points_lone_moveto_is_dropped() {
        // A subpath that adds no segment paints nothing and yields no polyline.
        let path = PathContent::from_operations(vec![PathOperation::MoveTo(5.0, 5.0)]);
        assert!(path.to_points(1.0).is_empty());
    }

    #[test]
    fn test_to_points_multiple_subpaths() {
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(0.0, 0.0),
            PathOperation::LineTo(1.0, 0.0),
            PathOperation::MoveTo(5.0, 5.0),
            PathOperation::LineTo(6.0, 5.0),
        ]);
        assert_eq!(
            path.to_points(1.0),
            vec![vec![(0.0, 0.0), (1.0, 0.0)], vec![(5.0, 5.0), (6.0, 5.0)]]
        );
    }

    #[test]
    fn test_to_points_empty() {
        let path = PathContent::from_operations(vec![]);
        assert!(path.to_points(1.0).is_empty());
    }

    #[test]
    fn test_to_points_nonpositive_tolerance_terminates() {
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(0.0, 0.0),
            PathOperation::CurveTo(0.0, 100.0, 100.0, 100.0, 100.0, 0.0),
        ]);
        for tol in [0.0, -1.0, f32::NAN] {
            let pts = path.to_points(tol);
            assert_eq!(pts.len(), 1);
            let n = pts[0].len();
            assert!((2..100_000).contains(&n), "tol={tol} produced {n} points (expected bounded)");
            assert!(pts[0].iter().all(|(x, y)| x.is_finite() && y.is_finite()));
        }
    }

    #[test]
    fn test_to_points_curve_starts_at_current_point() {
        // The curve's implicit P0 is the current point (here the LineTo endpoint),
        // not the path origin. Guards against using the wrong start point.
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(0.0, 0.0),
            PathOperation::LineTo(50.0, 0.0),
            PathOperation::CurveTo(60.0, 20.0, 70.0, 20.0, 80.0, 0.0),
        ]);
        let sub = &path.to_points(0.5)[0];
        assert_eq!(sub[0], (0.0, 0.0));
        assert_eq!(sub[1], (50.0, 0.0));
        // First flattened curve vertex departs from the LineTo endpoint (50,0),
        // so it must sit to its right (x > 50), never back near the origin.
        assert!(sub[2].0 > 50.0, "curve did not start at current point; sub[2]={:?}", sub[2]);
        assert!((sub.last().unwrap().0 - 80.0).abs() < 1e-3);
    }

    #[test]
    fn test_to_points_degenerate_curve() {
        // All control points coincide: a zero-length curve must not subdivide forever.
        let path = PathContent::from_operations(vec![
            PathOperation::MoveTo(5.0, 5.0),
            PathOperation::CurveTo(5.0, 5.0, 5.0, 5.0, 5.0, 5.0),
        ]);
        let sub = &path.to_points(0.1)[0];
        assert!(sub.len() < 10, "degenerate curve over-subdivided: {} points", sub.len());
        assert!(sub.iter().all(|&p| p == (5.0, 5.0)));
    }
}
