//! 실제 채움 경계를 배치 bbox와 분리해 clip 계산에만 전달한다.

use super::render_tree::{BoundingBox, RenderNode, RenderNodeType, ShapeTransform};
use super::{svg_arc_to_beziers, PathCommand};

type Point = (f64, f64);

#[derive(Default)]
struct Bounds(Option<(Point, Point)>);

impl Bounds {
    fn include(&mut self, (x, y): Point) -> Option<()> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        match &mut self.0 {
            Some(((left, top), (right, bottom))) => {
                *left = left.min(x);
                *top = top.min(y);
                *right = right.max(x);
                *bottom = bottom.max(y);
            }
            value => *value = Some(((x, y), (x, y))),
        }
        Some(())
    }

    fn cubic(&mut self, points: [Point; 4]) -> Option<()> {
        self.include(points[0])?;
        self.include(points[3])?;
        for coordinates in [points.map(|p| p.0), points.map(|p| p.1)] {
            let [p0, p1, p2, p3] = coordinates;
            let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
            let b = 2.0 * (p0 - 2.0 * p1 + p2);
            let c = p1 - p0;
            if ![a, b, c].into_iter().all(f64::is_finite) {
                return None;
            }
            let roots = if a == 0.0 {
                [if b == 0.0 { f64::NAN } else { -c / b }, f64::NAN]
            } else {
                let discriminant = b * b - 4.0 * a * c;
                if !discriminant.is_finite() {
                    return None;
                }
                if discriminant < 0.0 {
                    continue;
                }
                // 서로 가까운 두 항의 뺄셈으로 유효한 극값을 잃지 않는다.
                let q = -0.5 * (b + discriminant.sqrt().copysign(b));
                if q == 0.0 {
                    [-b / (2.0 * a), f64::NAN]
                } else {
                    [q / a, c / q]
                }
            };
            for t in roots.into_iter().filter(|t| *t > 0.0 && *t < 1.0) {
                let weights = [
                    (1.0 - t).powi(3),
                    3.0 * (1.0 - t).powi(2) * t,
                    3.0 * (1.0 - t) * t * t,
                    t * t * t,
                ];
                self.include(
                    points
                        .into_iter()
                        .zip(weights)
                        .fold((0.0, 0.0), |(x, y), ((px, py), w)| (x + px * w, y + py * w)),
                )?;
            }
        }
        Some(())
    }
}

fn transformed(point: Point, bbox: BoundingBox, transform: ShapeTransform) -> Point {
    let cx = bbox.x + bbox.width / 2.0;
    let cy = bbox.y + bbox.height / 2.0;
    let (sin, cos) = transform.rotation.to_radians().sin_cos();
    let (dx, dy) = (point.0 - cx, point.1 - cy);
    let (x, y) = (dx * cos - dy * sin, dx * sin + dy * cos);
    // Native의 F·R, SVG/Canvas의 부호를 반영한 R·F와 같은 결과다.
    (
        cx + if transform.horz_flip { -x } else { x },
        cy + if transform.vert_flip { -y } else { y },
    )
}

/// 원래 배치 범위는 유지하면서 실제 채움이 넘친 쪽만 clip에 반영한다.
pub(crate) fn including_filled_path(node: &RenderNode) -> BoundingBox {
    filled_path(node).map_or(node.bbox, |ink| {
        let left = node.bbox.x.min(ink.x);
        let top = node.bbox.y.min(ink.y);
        let right = (node.bbox.x + node.bbox.width).max(ink.x + ink.width);
        let bottom = (node.bbox.y + node.bbox.height).max(ink.y + ink.height);
        BoundingBox::new(left, top, right - left, bottom - top)
    })
}

fn filled_path(node: &RenderNode) -> Option<BoundingBox> {
    let RenderNodeType::Path(path) = &node.node_type else {
        return None;
    };
    if !node.visible
        || !path.style.opacity.is_finite()
        || path.style.opacity <= 0.0
        || (path.style.fill_color.is_none() && path.gradient.is_none())
        || !path.transform.rotation.is_finite()
    {
        return None;
    }
    let map = |point| transformed(point, node.bbox, path.transform);
    let mut bounds = Bounds::default();
    let (mut current, mut start) = (None, None);
    for command in &path.commands {
        match *command {
            PathCommand::MoveTo(x, y) => {
                current = Some((x, y));
                start = current;
            }
            PathCommand::LineTo(x, y) => {
                bounds.include(map(current?))?;
                bounds.include(map((x, y)))?;
                current = Some((x, y));
            }
            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                bounds.cubic([current?, (x1, y1), (x2, y2), (x, y)].map(map))?;
                current = Some((x, y));
            }
            PathCommand::ArcTo(rx, ry, angle, large, sweep, x, y) => {
                let (x0, y0) = current?;
                for segment in svg_arc_to_beziers(x0, y0, rx, ry, angle, large, sweep, x, y) {
                    match segment {
                        PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                            bounds.cubic([current?, (x1, y1), (x2, y2), (x, y)].map(map))?;
                            current = Some((x, y));
                        }
                        PathCommand::LineTo(x, y) => {
                            bounds.include(map(current?))?;
                            bounds.include(map((x, y)))?;
                            current = Some((x, y));
                        }
                        _ => return None,
                    }
                }
                current = Some((x, y));
            }
            PathCommand::ClosePath => {
                bounds.include(map(current?))?;
                bounds.include(map(start?))?;
                current = start;
            }
        }
    }
    let ((left, top), (right, bottom)) = bounds.0?;
    (right > left && bottom > top).then(|| BoundingBox::new(left, top, right - left, bottom - top))
}
