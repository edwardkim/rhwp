//! 독립 Print의 부정 등호 메트릭을 따르는 수학 선분 표시.
//! 두 가로획과 빗금을 해석적으로 구성하며 사설 글꼴의 윤곽을 복제하지 않는다.
//! 전진폭과 잉크 영역은 em 단위이며 문서의 화면 좌표와 무관하다.

use super::GlyphOutlineCommand as Command;

pub(super) fn commands(fs: f64, baseline: f64) -> Vec<Command> {
    let mut result = Vec::new();
    let map = |(x, y): (f64, f64)| (x * fs, baseline - y * fs);
    let mut polygon = |points: [(f64, f64); 4]| {
        result.push(Command::Move(map(points[0])));
        result.extend(points[1..].iter().map(|&p| Command::Line(map(p))));
        result.push(Command::Close);
    };
    for center in [0.19, 0.39] {
        polygon([
            (0.18, center - 0.025),
            (0.82, center - 0.025),
            (0.82, center + 0.025),
            (0.18, center + 0.025),
        ]);
    }
    polygon([(0.32, 0.014), (0.37, 0.014), (0.67, 0.568), (0.62, 0.568)]);
    result
}
