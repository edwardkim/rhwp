//! Bounded image clipping for the editable import path (ISO 32000-1 §8.5.4).
//! Unsupported paths are explicit: their bounding box is not an equivalent clip.

use super::image_clip_path::{ComplexClip, ExactPath};
use crate::content::{Matrix, Operator};
use crate::error::Result;
use crate::geometry::{Point, Rect};

/// Active page-space clip understood by the editable image importer.
#[derive(Clone, Debug)]
pub enum ImageClip {
    /// No restriction has been applied.
    Unbounded,
    /// An axis-aligned rectangular intersection.
    Rectangle(Rect),
    /// Straight, convex page-space boundary, including an affine Form BBox.
    Polygon(std::sync::Arc<ClipPolygon>),
    /// Exact curves/compound paths, intersected as alpha masks at decode time.
    Complex(std::sync::Arc<ComplexClip>),
    /// A finite clip allocation or complexity budget was exhausted.
    LimitExceeded,
    /// The clip has no visible area.
    Empty,
    /// A curved, compound or non-convex boundary outside this bounded path.
    Unsupported,
}

const MAX_VERTICES: usize = 32;

#[derive(Clone, Copy, Debug)]
/// A finite convex boundary; vertices are validated before construction.
pub struct ClipPolygon {
    pub(super) points: [Point; MAX_VERTICES],
    pub(super) len: usize,
}

fn cross(a: Point, b: Point, p: Point) -> f64 {
    (f64::from(b.x) - f64::from(a.x)) * (f64::from(p.y) - f64::from(a.y))
        - (f64::from(b.y) - f64::from(a.y)) * (f64::from(p.x) - f64::from(a.x))
}

impl ClipPolygon {
    fn winding(&self) -> f64 {
        let area = (0..self.len)
            .map(|i| {
                let a = self.points[i];
                let b = self.points[(i + 1) % self.len];
                cross(self.points[0], a, b)
            })
            .sum::<f64>();
        if area == 0.0 {
            0.0
        } else {
            area.signum()
        }
    }

    fn push(&mut self, point: Point) -> bool {
        if self.len > 0 && self.points[self.len - 1] == point {
            return true;
        }
        if self.len == MAX_VERTICES || !point.x.is_finite() || !point.y.is_finite() {
            return false;
        }
        self.points[self.len] = point;
        self.len += 1;
        true
    }

    fn bounds(&self) -> Rect {
        let mut min = self.points[0];
        let mut max = min;
        for p in &self.points[..self.len] {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Rect::new(min.x, min.y, max.x - min.x, max.y - min.y)
    }

    fn intersect(mut self, clip: Self) -> ImageClip {
        let direction = clip.winding();
        for i in 0..clip.len {
            let a = clip.points[i];
            let b = clip.points[(i + 1) % clip.len];
            let mut result = Self {
                points: [Point::default(); MAX_VERTICES],
                len: 0,
            };
            let mut previous = self.points[self.len - 1];
            let mut before = cross(a, b, previous) * direction;
            for point in &self.points[..self.len] {
                let after = cross(a, b, *point) * direction;
                if (before >= 0.0) != (after >= 0.0) {
                    let t = before / (before - after);
                    let hit = Point::new(
                        (f64::from(previous.x) + t * (f64::from(point.x) - f64::from(previous.x)))
                            as f32,
                        (f64::from(previous.y) + t * (f64::from(point.y) - f64::from(previous.y)))
                            as f32,
                    );
                    if !result.push(hit) {
                        return ImageClip::Unsupported;
                    }
                }
                if after >= 0.0 && !result.push(*point) {
                    return ImageClip::Unsupported;
                }
                previous = *point;
                before = after;
            }
            if result.len > 1 && result.points[0] == result.points[result.len - 1] {
                result.len -= 1;
            }
            if result.len < 3 || result.winding() == 0.0 {
                return ImageClip::Empty;
            }
            self = result;
        }
        ImageClip::Polygon(std::sync::Arc::new(self))
    }
}

impl ImageClip {
    /// Conservative finite bounds for limiting pattern tile enumeration.
    pub(crate) fn bounds(&self) -> crate::error::Result<Option<Rect>> {
        match self {
            Self::Rectangle(r) => Ok(Some(*r)),
            Self::Polygon(p) => Ok(Some(p.bounds())),
            Self::Complex(p) => Ok(Some(p.bounds)),
            Self::LimitExceeded => {
                Err(crate::error::Error::ImportResourceLimit("image clip complexity"))
            },
            Self::Empty => Ok(None),
            _ => Err(crate::error::Error::Unsupported("pattern requires a finite clip".into())),
        }
    }
    pub(crate) fn intersect(self, other: Self) -> Self {
        match (self, other) {
            (Self::LimitExceeded, _) | (_, Self::LimitExceeded) => Self::LimitExceeded,
            (Self::Empty, _) | (_, Self::Empty) => Self::Empty,
            (Self::Unsupported, _) | (_, Self::Unsupported) => Self::Unsupported,
            (Self::Unbounded, clip) | (clip, Self::Unbounded) => clip,
            (Self::Rectangle(a), Self::Rectangle(b)) => {
                a.intersection(&b).map_or(Self::Empty, Self::Rectangle)
            },
            (a @ Self::Complex(_), b) | (a, b @ Self::Complex(_)) => ComplexClip::intersect(a, b),
            (a, b) => a.polygon().intersect(b.polygon()),
        }
    }

    pub(super) fn polygon(self) -> ClipPolygon {
        if let Self::Polygon(p) = self {
            return *p;
        }
        let Self::Rectangle(r) = self else {
            unreachable!("only finite regions intersect")
        };
        let mut p = ClipPolygon {
            points: [Point::default(); MAX_VERTICES],
            len: 0,
        };
        for (x, y) in [
            (r.x, r.y),
            (r.x + r.width, r.y),
            (r.x + r.width, r.y + r.height),
            (r.x, r.y + r.height),
        ] {
            p.push(Point::new(x, y));
        }
        p
    }

    pub(crate) fn contains(&self, point: Point) -> bool {
        match self {
            Self::Unbounded => true,
            // Complex clips are sampled from their antialiased raster mask.
            Self::Complex(_) => true,
            Self::Empty | Self::Unsupported | Self::LimitExceeded => false,
            Self::Rectangle(r) => {
                point.x >= r.x
                    && point.x <= r.x + r.width
                    && point.y >= r.y
                    && point.y <= r.y + r.height
            },
            Self::Polygon(p) => {
                let direction = p.winding();
                (0..p.len).all(|i| {
                    cross(p.points[i], p.points[(i + 1) % p.len], point) * direction >= 0.0
                })
            },
        }
    }

    /// Intersect a full image frame with the clip, rejecting unsupported paths.
    pub fn visible_bounds(&self, image: Rect) -> crate::error::Result<Option<Rect>> {
        match self.clone().intersect(Self::Rectangle(image)) {
            Self::Rectangle(rect) => Ok(Some(rect)),
            Self::Polygon(p) => Ok(Some(p.bounds())),
            Self::Complex(p) => Ok(Some(p.bounds)),
            Self::LimitExceeded => {
                Err(crate::error::Error::ImportResourceLimit("image clip complexity"))
            },
            Self::Empty => Ok(None),
            _ => Err(crate::error::Error::Unsupported("non-rectangular image clipping".into())),
        }
    }

    pub(crate) fn transformed_rectangle(ctm: Matrix, bounds: [f32; 4]) -> Self {
        Self::rectangle_region(ctm, bounds, false)
    }

    pub(crate) fn transformed_image_rectangle(ctm: Matrix, bounds: [f32; 4]) -> Self {
        Self::rectangle_region(ctm, bounds, true)
    }

    fn rectangle_region(ctm: Matrix, bounds: [f32; 4], convex: bool) -> Self {
        let [x0, y0, x1, y1] = bounds;
        let mut path = ClipPath::default();
        for (x, y) in [(x0, y0), (x1, y0), (x1, y1), (x0, y1)] {
            path.push(ctm.transform_point(x, y));
        }
        path.region(convex)
    }
}

#[derive(Clone)]
pub(crate) struct ImageGraphicsState {
    pub ctm: Matrix,
    pub clip: ImageClip,
    pub alpha: f32,
    pub fill_space: crate::path_color::PathColorSpace,
    pub fill_colour: Option<(f32, f32, f32)>,
}

#[derive(Default)]
struct ClipPath {
    points: [Point; MAX_VERTICES],
    len: usize,
    unsupported: bool,
}

impl ClipPath {
    fn move_to(&mut self, point: Point) {
        // ISO 32000-1 Table 59: a consecutive m replaces the previous m.
        // A lone move has no segment and must not create a compound boundary.
        if self.len == 1 {
            self.len = 0;
        } else if self.len != 0 {
            self.unsupported = true;
        }
        self.push(point);
    }

    fn push(&mut self, point: Point) {
        if self.len == self.points.len() || !point.x.is_finite() || !point.y.is_finite() {
            self.unsupported = true;
        } else {
            self.points[self.len] = point;
            self.len += 1;
        }
    }

    fn region(&self, convex: bool) -> ImageClip {
        if self.unsupported {
            return ImageClip::Unsupported;
        }
        if self.len == 0 {
            return ImageClip::Empty;
        }
        let len = if self.len > 1 && self.points[self.len - 1] == self.points[0] {
            self.len - 1
        } else {
            self.len
        };
        if len < 3 {
            return ImageClip::Unsupported;
        }
        let [a, b, c, d, ..] = self.points;
        let axis_rectangle = len == 4
            && ((a.y == b.y && b.x == c.x && c.y == d.y && d.x == a.x)
                || (a.x == b.x && b.y == c.y && c.x == d.x && d.y == a.y));
        if !axis_rectangle {
            if !convex {
                return ImageClip::Unsupported;
            }
            let p = ClipPolygon {
                points: self.points,
                len,
            };
            let direction = p.winding();
            if direction == 0.0 {
                let second = p.points[..len]
                    .iter()
                    .find(|v| **v != a)
                    .copied()
                    .unwrap_or(a);
                return if p.points[..len].iter().all(|v| cross(a, second, *v) == 0.0) {
                    ImageClip::Empty
                } else {
                    ImageClip::Unsupported
                };
            }
            if (0..len).any(|i| {
                p.points[..len]
                    .iter()
                    .any(|v| cross(p.points[i], p.points[(i + 1) % len], *v) * direction < 0.0)
            }) {
                return ImageClip::Unsupported;
            }
            return ImageClip::Polygon(std::sync::Arc::new(p));
        }
        let rect = Rect::new(a.x.min(c.x), a.y.min(c.y), (c.x - a.x).abs(), (c.y - a.y).abs());
        if rect.width == 0.0 || rect.height == 0.0 {
            ImageClip::Empty
        } else {
            ImageClip::Rectangle(rect)
        }
    }
}

/// The current path is deliberately outside the q/Q graphics-state stack.
pub(crate) struct ImageClipTracker {
    current: ImageGraphicsState,
    saved: Vec<ImageGraphicsState>,
    path: ClipPath,
    pending: bool,
    convex: bool,
    exact: ExactPath,
    even_odd: bool,
}

impl ImageClipTracker {
    pub(crate) fn from_image_state(state: ImageGraphicsState) -> Self {
        Self {
            current: state,
            saved: Vec::new(),
            path: ClipPath::default(),
            pending: false,
            convex: true,
            exact: ExactPath::default(),
            even_odd: false,
        }
    }
    pub fn new(ctm: Matrix, clip: ImageClip) -> Self {
        Self {
            current: ImageGraphicsState {
                ctm,
                clip,
                alpha: 1.0,
                fill_space: crate::path_color::PathColorSpace::GRAY,
                fill_colour: Some((0., 0., 0.)),
            },
            saved: Vec::new(),
            path: ClipPath::default(),
            pending: false,
            convex: false,
            exact: ExactPath::default(),
            even_odd: false,
        }
    }

    /// Image import has a bounded alpha compositor. Vector path paint keeps
    /// its existing explicit unsupported-path handling and original clip path.
    pub fn for_images(ctm: Matrix, clip: ImageClip) -> Self {
        Self {
            convex: true,
            ..Self::new(ctm, clip)
        }
    }

    pub(crate) fn set_budget(
        &mut self,
        budget: Option<std::sync::Arc<crate::import_limits::ImportBudget>>,
    ) {
        self.exact.budget = budget;
    }

    fn region(&self, even_odd: bool) -> Result<ImageClip> {
        let fast = self.path.region(self.convex);
        if self.convex && matches!(fast, ImageClip::Unsupported) {
            self.exact.region(even_odd)
        } else {
            Ok(fast)
        }
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.current.alpha = alpha;
    }

    pub fn set_fill_colour(
        &mut self,
        space: crate::path_color::PathColorSpace,
        colour: Option<(f32, f32, f32)>,
    ) {
        self.current.fill_space = space;
        self.current.fill_colour = colour;
    }

    pub fn current(&self) -> ImageGraphicsState {
        self.current.clone()
    }

    /// A fill uses the current path before a painting operator consumes it.
    /// The pending W/W* clip takes effect only after painting (ISO 8.5.4).
    pub(crate) fn fill_clip(&self, even_odd: bool) -> Result<ImageClip> {
        Ok(self.current.clip.clone().intersect(self.region(even_odd)?))
    }

    pub fn apply(&mut self, op: &Operator) -> Result<()> {
        if self.convex {
            self.exact.apply(op, self.current.ctm)?;
        }
        use Operator::*;
        match *op {
            SaveState => self.saved.push(self.current.clone()),
            RestoreState => {
                if let Some(state) = self.saved.pop() {
                    self.current = state;
                }
            },
            Cm { a, b, c, d, e, f } => {
                self.current.ctm = Matrix { a, b, c, d, e, f }.multiply(&self.current.ctm)
            },
            MoveTo { x, y } => {
                self.path.move_to(self.current.ctm.transform_point(x, y));
            },
            LineTo { x, y } => self.path.push(self.current.ctm.transform_point(x, y)),
            Rectangle {
                x,
                y,
                width,
                height,
            } => {
                // re begins with an implicit m, then appends three segments.
                self.path.move_to(self.current.ctm.transform_point(x, y));
                for (px, py) in [(x + width, y), (x + width, y + height), (x, y + height)] {
                    self.path.push(self.current.ctm.transform_point(px, py));
                }
            },
            ClosePath => {
                if self.path.len > 0 {
                    self.path.push(self.path.points[0]);
                }
            },
            CurveTo { .. } | CurveToV { .. } | CurveToY { .. } => self.path.unsupported = true,
            ClipNonZero | ClipEvenOdd => {
                self.pending = true;
                self.even_odd = matches!(op, ClipEvenOdd);
            },
            EndPath
            | Stroke
            | Fill
            | FillEvenOdd
            | CloseAndStroke
            | CloseFillStroke
            | FillStroke
            | FillStrokeEvenOdd
            | CloseFillStrokeEvenOdd => {
                if self.pending {
                    self.current.clip = self
                        .current
                        .clip
                        .clone()
                        .intersect(self.region(self.even_odd)?);
                }
                self.path = ClipPath::default();
                self.exact.clear();
                self.pending = false;
            },
            _ => {},
        }
        Ok(())
    }
}
