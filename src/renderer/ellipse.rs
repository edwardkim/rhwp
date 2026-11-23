use super::{PathCommand, ShapeStyle};

/// 한컴 도형의 타원은 오른쪽 끝에서 위쪽으로 윤곽을 돈다.
/// 단위 타원에 좌표 변환을 적용해 묶음 도형에서도 같은 시작점과 진행 방향을 쓴다.
pub(crate) fn outline(mut project: impl FnMut(f64, f64) -> (f64, f64)) -> Vec<PathCommand> {
    // ¼원을 양 끝의 접선과 중간점에서 일치시키는 cubic 제어점 거리.
    const K: f64 = 0.552_284_749_830_793_6;
    let (x, y) = project(1.0, 0.0);
    let mut commands = Vec::with_capacity(6);
    commands.push(PathCommand::MoveTo(x, y));
    for quarter in [
        [(1.0, -K), (K, -1.0), (0.0, -1.0)],
        [(-K, -1.0), (-1.0, -K), (-1.0, 0.0)],
        [(-1.0, K), (-K, 1.0), (0.0, 1.0)],
        [(K, 1.0), (1.0, K), (1.0, 0.0)],
    ] {
        let [a, b, end] = quarter.map(|(x, y)| project(x, y));
        commands.push(PathCommand::CurveTo(a.0, a.1, b.0, b.1, end.0, end.1));
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
        .then(|| outline(|x, y| (cx + rx * x, cy + ry * y)))
}
