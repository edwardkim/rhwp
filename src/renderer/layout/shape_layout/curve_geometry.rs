//! HWP/HWPX의 점·구간 체인을 공통 경로로 변환한다.
//!
//! 저장 점은 곡선이 통과하는 점이며 베지어 제어점 세 개씩의 배열이 아니다.
//! 원본 단위에서 연결을 계산한 뒤 호출 경로의 배율·아핀 변환을 한 번 적용한다.

use crate::model::shape::CurveShape;
use crate::renderer::render_tree::BoundingBox;
use crate::renderer::PathCommand;

type Knot = (f64, f64);

fn add(a: Knot, b: Knot) -> Knot {
    (a.0 + b.0, a.1 + b.1)
}

fn sub(a: Knot, b: Knot) -> Knot {
    (a.0 - b.0, a.1 - b.1)
}

fn scale(a: Knot, factor: f64) -> Knot {
    (a.0 * factor, a.1 * factor)
}

fn midpoint(a: Knot, b: Knot) -> Knot {
    scale(add(a, b), 0.5)
}

pub(super) fn commands(
    curve: &CurveShape,
    mut map: impl FnMut(f64, f64) -> Knot,
) -> Vec<PathCommand> {
    let points: Vec<Knot> = curve
        .points
        .iter()
        .map(|p| (p.x as f64, p.y as f64))
        .collect();
    let Some(&first) = points.first() else {
        return Vec::new();
    };
    let count = points.len();
    let is_curve = |i: usize| curve.segment_types.get(i).copied() == Some(1);
    let cyclic = count > 2 && first == points[count - 1] && is_curve(0) && is_curve(count - 2);
    let (x, y) = map(first.0, first.1);
    let mut result = vec![PathCommand::MoveTo(x, y)];
    let mut previous_end = None;
    let mut first_start = first;

    for i in 0..count - 1 {
        let start = points[i];
        let end = points[i + 1];
        if count == 2 || !is_curve(i) {
            let (x, y) = map(end.0, end.1);
            result.push(PathCommand::LineTo(x, y));
            previous_end = None;
            continue;
        }

        let (a, b) = if cyclic && i == 0 {
            // 닫힌 첫 구간은 마지막 구간과 접선을 공유한다.
            let a = add(first, scale(sub(points[1], points[count - 2]), 1.0 / 6.0));
            (a, midpoint(a, end))
        } else if i + 2 < count {
            let b = sub(end, scale(sub(points[i + 2], start), 1.0 / 6.0));
            let a = previous_end
                .map_or_else(|| midpoint(start, b), |prior| sub(scale(start, 2.0), prior));
            (a, b)
        } else {
            let a = previous_end.map_or_else(
                || add(start, scale(sub(end, points[i - 1]), 1.0 / 6.0)),
                |prior| sub(scale(start, 2.0), prior),
            );
            let b = if cyclic {
                sub(scale(end, 2.0), first_start)
            } else {
                midpoint(a, end)
            };
            (a, b)
        };

        if i == 0 {
            first_start = a;
        }
        previous_end = Some(b);
        let (ax, ay) = map(a.0, a.1);
        let (bx, by) = map(b.0, b.1);
        let (x, y) = map(end.0, end.1);
        result.push(PathCommand::CurveTo(ax, ay, bx, by, x, y));
    }
    result
}

/// 그룹 경로가 실제 소비하는 제어점의 보수적인 외곽이다. 빈 경로는 외곽이 없다.
pub(super) fn control_bounds(commands: &[PathCommand]) -> Option<BoundingBox> {
    let mut bounds: Option<(Knot, Knot)> = None;
    let mut include = |point: Knot| {
        bounds = Some(match bounds {
            None => (point, point),
            Some((min, max)) => (
                (min.0.min(point.0), min.1.min(point.1)),
                (max.0.max(point.0), max.1.max(point.1)),
            ),
        });
    };
    for command in commands {
        match *command {
            PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => include((x, y)),
            PathCommand::CurveTo(ax, ay, bx, by, x, y) => {
                include((ax, ay));
                include((bx, by));
                include((x, y));
            }
            _ => {}
        }
    }
    bounds.map(|(min, max)| BoundingBox::new(min.0, min.1, max.0 - min.0, max.1 - min.1))
}
