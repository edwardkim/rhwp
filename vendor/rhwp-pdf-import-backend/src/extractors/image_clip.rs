//! Rectangular image clipping for the editable import path (ISO 32000-1 §8.5.4).
//! Unsupported paths are explicit: their bounding box is not an equivalent clip.

use crate::content::{Matrix, Operator};
use crate::geometry::{Point, Rect};

/// Active page-space clip understood by the editable image importer.
#[derive(Clone, Copy, Debug)]
pub enum ImageClip {
    /// No restriction has been applied.
    Unbounded,
    /// An axis-aligned rectangular intersection.
    Rectangle(Rect),
    /// The clip has no visible area.
    Empty,
    /// A path that cannot be represented by a rectangular picture crop.
    Unsupported,
}

impl ImageClip {
    pub(crate) fn intersect(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, _) | (_, Self::Empty) => Self::Empty,
            (Self::Unsupported, _) | (_, Self::Unsupported) => Self::Unsupported,
            (Self::Unbounded, clip) | (clip, Self::Unbounded) => clip,
            (Self::Rectangle(a), Self::Rectangle(b)) => {
                a.intersection(&b).map_or(Self::Empty, Self::Rectangle)
            },
        }
    }

    /// Intersect a full image frame with the clip, rejecting unsupported paths.
    pub fn visible_bounds(self, image: Rect) -> crate::error::Result<Option<Rect>> {
        match self.intersect(Self::Rectangle(image)) {
            Self::Rectangle(rect) => Ok(Some(rect)),
            Self::Empty => Ok(None),
            _ => Err(crate::error::Error::Unsupported("non-rectangular image clipping".into())),
        }
    }

    pub(crate) fn transformed_rectangle(ctm: Matrix, bounds: [f32; 4]) -> Self {
        let [x0, y0, x1, y1] = bounds;
        let mut path = ClipPath::default();
        for (x, y) in [(x0, y0), (x1, y0), (x1, y1), (x0, y1)] {
            path.push(ctm.transform_point(x, y));
        }
        path.region()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ImageGraphicsState {
    pub ctm: Matrix,
    pub clip: ImageClip,
    pub alpha: f32,
}

#[derive(Default)]
struct ClipPath {
    points: [Point; 5],
    len: usize,
    unsupported: bool,
}

impl ClipPath {
    fn push(&mut self, point: Point) {
        if self.len == self.points.len() || !point.x.is_finite() || !point.y.is_finite() {
            self.unsupported = true;
        } else {
            self.points[self.len] = point;
            self.len += 1;
        }
    }

    fn region(&self) -> ImageClip {
        if self.unsupported {
            return ImageClip::Unsupported;
        }
        if self.len == 0 {
            return ImageClip::Empty;
        }
        let len = if self.len == 5 && self.points[4] == self.points[0] {
            4
        } else {
            self.len
        };
        if len != 4 {
            return ImageClip::Unsupported;
        }
        let [a, b, c, d, _] = self.points;
        let axis_rectangle = (a.y == b.y && b.x == c.x && c.y == d.y && d.x == a.x)
            || (a.x == b.x && b.y == c.y && c.x == d.x && d.y == a.y);
        if !axis_rectangle {
            return ImageClip::Unsupported;
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
}

impl ImageClipTracker {
    pub fn new(ctm: Matrix, clip: ImageClip) -> Self {
        Self {
            current: ImageGraphicsState {
                ctm,
                clip,
                alpha: 1.0,
            },
            saved: Vec::new(),
            path: ClipPath::default(),
            pending: false,
        }
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.current.alpha = alpha;
    }

    pub fn current(&self) -> ImageGraphicsState {
        self.current
    }

    pub fn apply(&mut self, op: &Operator) {
        use Operator::*;
        match *op {
            SaveState => self.saved.push(self.current),
            RestoreState => {
                if let Some(state) = self.saved.pop() {
                    self.current = state;
                }
            },
            Cm { a, b, c, d, e, f } => {
                self.current.ctm = Matrix { a, b, c, d, e, f }.multiply(&self.current.ctm)
            },
            MoveTo { x, y } => {
                if self.path.len != 0 {
                    self.path.unsupported = true;
                }
                self.path.push(self.current.ctm.transform_point(x, y));
            },
            LineTo { x, y } => self.path.push(self.current.ctm.transform_point(x, y)),
            Rectangle {
                x,
                y,
                width,
                height,
            } => {
                if self.path.len != 0 {
                    self.path.unsupported = true;
                }
                for (px, py) in [
                    (x, y),
                    (x + width, y),
                    (x + width, y + height),
                    (x, y + height),
                ] {
                    self.path.push(self.current.ctm.transform_point(px, py));
                }
            },
            ClosePath => {
                if self.path.len > 0 {
                    self.path.push(self.path.points[0]);
                }
            },
            CurveTo { .. } | CurveToV { .. } | CurveToY { .. } => self.path.unsupported = true,
            ClipNonZero | ClipEvenOdd => self.pending = true,
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
                    self.current.clip = self.current.clip.intersect(self.path.region());
                }
                self.path = ClipPath::default();
                self.pending = false;
            },
            _ => {},
        }
    }
}
