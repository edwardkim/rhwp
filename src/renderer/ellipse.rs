use super::stroke_pattern::PRINT_PEN_PIXEL;
use super::{PathCommand, ShapeStyle};

type Point = (f64, f64);
type Cubic = [Point; 4];

fn split(curve: Cubic, t: f64) -> (Cubic, Cubic) {
    let lerp = |a: Point, b: Point| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
    let [p, a, b, q] = curve;
    let ab = lerp(a, b);
    let pa = lerp(p, a);
    let bq = lerp(b, q);
    let left = lerp(pa, ab);
    let right = lerp(ab, bq);
    let middle = lerp(left, right);
    ([p, pa, left, middle], [middle, right, bq, q])
}

/// 한컴 Print에서 닫힌 타원의 첫 파선은 오른쪽 끝 직전의 마지막 평탄화 꼭짓점에서
/// 시작한다. GDI+의 기본 flatness는 출력 pen 좌표에서 ¼픽셀이다. 마지막 호만
/// 이분해 그 꼭짓점의 매개변수를 구하면 실제 윤곽과 채움에는 원래의 cubic을 쓸 수 있다.
fn last_segment_start(mut curve: Cubic) -> f64 {
    let mut interval = 1.0;
    loop {
        let [p, a, b, q] = curve;
        let dx = q.0 - p.0;
        let dy = q.1 - p.1;
        let chord = dx.hypot(dy);
        let deviation = |v: Point| ((v.0 - p.0) * dy - (v.1 - p.1) * dx).abs();
        if chord == 0.0
            || !chord.is_finite()
            || deviation(a).max(deviation(b)) <= chord * (0.25 * PRINT_PEN_PIXEL)
        {
            return 1.0 - interval;
        }
        curve = split(curve, 0.5).1;
        interval *= 0.5;
    }
}

fn curve_to([_, a, b, end]: Cubic) -> PathCommand {
    PathCommand::CurveTo(a.0, a.1, b.0, b.1, end.0, end.1)
}

pub(crate) fn outline(
    patterned: bool,
    mut project: impl FnMut(f64, f64) -> Point,
) -> Vec<PathCommand> {
    // ¼원의 양 끝 접선과 중간점에서 일치하는 베지어 제어점. 화면 좌표의 시계 방향.
    const K: f64 = 0.552_284_749_830_793_6;
    let quarters = [
        [(1.0, 0.0), (1.0, K), (K, 1.0), (0.0, 1.0)],
        [(0.0, 1.0), (-K, 1.0), (-1.0, K), (-1.0, 0.0)],
        [(-1.0, 0.0), (-1.0, -K), (-K, -1.0), (0.0, -1.0)],
        [(0.0, -1.0), (K, -1.0), (1.0, -K), (1.0, 0.0)],
    ]
    .map(|quarter| quarter.map(|(x, y)| project(x, y)));
    let mut commands = Vec::with_capacity(7);
    if patterned {
        let last = quarters[3];
        let t = last_segment_start(last);
        let (head, tail) = split(last, t);
        commands.push(PathCommand::MoveTo(tail[0].0, tail[0].1));
        commands.push(curve_to(tail));
        commands.extend(quarters[..3].iter().copied().map(curve_to));
        if t > 0.0 {
            commands.push(curve_to(head));
        }
    } else {
        let (x, y) = quarters[0][0];
        commands.push(PathCommand::MoveTo(x, y));
        commands.extend(quarters.map(curve_to));
    }
    commands.push(PathCommand::ClosePath);
    commands
}

pub(crate) fn patterned_outline(
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    style: &ShapeStyle,
) -> Option<Vec<PathCommand>> {
    (style.stroke_color.is_some() && style.stroke_dash.is_hwp_pattern() && rx > 0.0 && ry > 0.0)
        .then(|| outline(true, |x, y| (cx + rx * x, cy + ry * y)))
}
