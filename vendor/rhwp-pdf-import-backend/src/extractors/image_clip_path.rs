//! Exact image clip geometry. Bounds only restrict allocation; they never
//! replace the winding/curve mask (ISO 32000-1 8.5.2–8.5.4).
use super::image_clip::ImageClip;
use crate::{
    content::{Matrix, Operator},
    elements::PathOperation,
    error::{Error, Result},
    geometry::{Point, Rect},
    import_limits::ImportBudget,
};
use std::sync::Arc;
use tiny_skia::{FillRule, Mask, Path, PathBuilder, Transform};

// Match the editable import's per-path and graphics-state complexity ceilings.
const MAX_OPERATIONS: usize = 20_000;
const MAX_LAYERS: usize = 128;

#[derive(Debug)]
struct Layer {
    path: Path,
    even_odd: bool,
}

#[derive(Debug)]
pub struct ComplexClip {
    pub(super) bounds: Rect,
    layers: Vec<Arc<Layer>>,
    budget: Option<Arc<ImportBudget>>,
}

fn limit() -> Error {
    Error::ImportResourceLimit("image clip complexity")
}
fn charge(budget: Option<&ImportBudget>, bytes: usize) -> Result<()> {
    if let Some(b) = budget {
        b.check()?;
        b.charge_decoded(bytes)?;
    }
    Ok(())
}

impl ComplexClip {
    pub(super) fn intersect(a: ImageClip, b: ImageClip) -> ImageClip {
        match Self::combine(a, b) {
            Ok(c) => c,
            Err(_) => ImageClip::LimitExceeded,
        }
    }
    fn combine(a: ImageClip, b: ImageClip) -> Result<ImageClip> {
        let bounds = match a
            .bounds()?
            .zip(b.bounds()?)
            .and_then(|(a, b)| a.intersection(&b))
        {
            Some(b) => b,
            None => return Ok(ImageClip::Empty),
        };
        let budget = [&a, &b].into_iter().find_map(|c| match c {
            ImageClip::Complex(c) => c.budget.clone(),
            _ => None,
        });
        let count = |c: &ImageClip| match c {
            ImageClip::Complex(c) => c.layers.len(),
            _ => 1,
        };
        let count = count(&a) + count(&b);
        if count > MAX_LAYERS {
            return Err(budget
                .as_deref()
                .map_or_else(limit, |b| b.reject("image clip layers")));
        }
        charge(budget.as_deref(), count * std::mem::size_of::<Arc<Layer>>() + 1024)?;
        let mut layers = Vec::new();
        layers.try_reserve_exact(count).map_err(|_| limit())?;
        for c in [a, b] {
            if let ImageClip::Complex(c) = c {
                layers.extend(c.layers.iter().cloned());
            } else {
                let polygon = c.polygon();
                let mut pb = PathBuilder::new();
                let points = &polygon.points[..polygon.len];
                pb.move_to(points[0].x, points[0].y);
                for p in &points[1..] {
                    pb.line_to(p.x, p.y);
                }
                pb.close();
                layers.push(Arc::new(Layer {
                    path: pb.finish().ok_or_else(limit)?,
                    even_odd: false,
                }));
            }
        }
        Ok(ImageClip::Complex(Arc::new(Self {
            bounds,
            layers,
            budget,
        })))
    }
    pub(super) fn raster(
        &self,
        frame: Rect,
        width: usize,
        height: usize,
        budget: &ImportBudget,
    ) -> Result<Mask> {
        let bytes = width
            .checked_mul(height)
            .filter(|&n| n <= budget.limits.stream_bytes)
            .ok_or_else(limit)?;
        charge(Some(budget), bytes)?;
        let mut data = Vec::new();
        data.try_reserve_exact(bytes)
            .map_err(|_| budget.reject("image clip mask allocation"))?;
        data.resize(bytes, 255);
        let size = tiny_skia::IntSize::from_wh(width as u32, height as u32).ok_or_else(limit)?;
        let mut mask = Mask::from_vec(data, size).ok_or_else(limit)?;
        let sx = width as f32 / frame.width;
        let sy = height as f32 / frame.height;
        let transform =
            Transform::from_row(sx, 0., 0., -sy, -frame.x * sx, (frame.y + frame.height) * sy);
        for layer in &self.layers {
            // intersect_path allocates a temporary mask. Charge pixel work and
            // path/edge scratch before entering the synchronous rasterizer.
            charge(Some(budget), bytes + layer.path.points().len() * 128)?;
            budget.charge_codec_work(bytes)?;
            mask.intersect_path(
                &layer.path,
                if layer.even_odd {
                    FillRule::EvenOdd
                } else {
                    FillRule::Winding
                },
                true,
                transform,
            );
            budget.check()?;
        }
        Ok(mask)
    }
}

#[derive(Default)]
pub(super) struct ExactPath {
    operations: Vec<PathOperation>,
    current: Option<Point>,
    start: Option<Point>,
    pub(super) budget: Option<Arc<ImportBudget>>,
}
impl ExactPath {
    pub(super) fn clear(&mut self) {
        self.operations.clear();
        self.current = None;
        self.start = None;
    }
    fn push(&mut self, op: PathOperation) -> Result<()> {
        if self.operations.len() == MAX_OPERATIONS {
            return Err(self
                .budget
                .as_deref()
                .map_or_else(limit, |b| b.reject("image clip path operations")));
        }
        charge(self.budget.as_deref(), 128)?;
        self.operations.try_reserve(1).map_err(|_| limit())?;
        self.operations.push(op);
        Ok(())
    }
    fn point(m: Matrix, x: f32, y: f32) -> Result<Point> {
        let p = m.transform_point(x, y);
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(Error::Unsupported("non-finite image clipping path".into()));
        }
        Ok(p)
    }
    pub(super) fn apply(&mut self, op: &Operator, m: Matrix) -> Result<()> {
        use Operator::*;
        match *op {
            MoveTo { x, y } => {
                let p = Self::point(m, x, y)?;
                if matches!(self.operations.last(), Some(PathOperation::MoveTo(..))) {
                    self.operations.pop();
                }
                self.push(PathOperation::MoveTo(p.x, p.y))?;
                self.current = Some(p);
                self.start = Some(p);
            },
            LineTo { x, y } => {
                let p = Self::point(m, x, y)?;
                self.push(PathOperation::LineTo(p.x, p.y))?;
                self.current = Some(p);
            },
            CurveTo {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                let a = Self::point(m, x1, y1)?;
                let b = Self::point(m, x2, y2)?;
                let c = Self::point(m, x3, y3)?;
                self.curve(a, b, c)?;
            },
            CurveToV { x2, y2, x3, y3 } => {
                let a = self.current.ok_or_else(|| {
                    Error::Unsupported("image curve without current point".into())
                })?;
                self.curve(a, Self::point(m, x2, y2)?, Self::point(m, x3, y3)?)?;
            },
            CurveToY { x1, y1, x3, y3 } => {
                let c = Self::point(m, x3, y3)?;
                self.curve(Self::point(m, x1, y1)?, c, c)?;
            },
            Rectangle {
                x,
                y,
                width,
                height,
            } => {
                self.apply(&MoveTo { x, y }, m)?;
                for (x, y) in [(x + width, y), (x + width, y + height), (x, y + height)] {
                    self.apply(&LineTo { x, y }, m)?;
                }
                self.apply(&ClosePath, m)?;
            },
            ClosePath => {
                self.push(PathOperation::ClosePath)?;
                self.current = self.start;
            },
            _ => {},
        }
        Ok(())
    }
    fn curve(&mut self, a: Point, b: Point, c: Point) -> Result<()> {
        self.push(PathOperation::CurveTo(a.x, a.y, b.x, b.y, c.x, c.y))?;
        self.current = Some(c);
        Ok(())
    }
    pub(super) fn region(&self, even_odd: bool) -> Result<ImageClip> {
        charge(self.budget.as_deref(), self.operations.len() * 128)?;
        let mut pb =
            PathBuilder::with_capacity(self.operations.len() * 3, self.operations.len() * 3);
        for op in &self.operations {
            match *op {
                PathOperation::MoveTo(x, y) => pb.move_to(x, y),
                PathOperation::LineTo(x, y) => pb.line_to(x, y),
                PathOperation::CurveTo(a, b, c, d, e, f) => pb.cubic_to(a, b, c, d, e, f),
                PathOperation::ClosePath => pb.close(),
                PathOperation::Rectangle(..) => unreachable!("rectangles expanded in page space"),
            }
        }
        let Some(path) = pb.finish() else {
            return Ok(ImageClip::Empty);
        };
        let b = path.bounds();
        if b.width() <= 0. || b.height() <= 0. {
            return Ok(ImageClip::Empty);
        }
        Ok(ImageClip::Complex(Arc::new(ComplexClip {
            bounds: Rect::new(b.x(), b.y(), b.width(), b.height()),
            layers: vec![Arc::new(Layer { path, even_odd })],
            budget: self.budget.clone(),
        })))
    }
}
