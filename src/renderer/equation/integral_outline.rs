//! 공개 Latin Modern Math의 적분 윤곽을 공통 ink 영역에 배치한다.
//! Copyright 2012--2014 B. Jackowski, P. Strzelczyk, P. Pianowski.
//! 윤곽 데이터: GUST Font License (동봉 LatinModernMath-GUST-FONT-LICENSE.txt).
//! 원본: assets/fonts/LatinModernMath-Regular.woff2의 integral.
//! 사설 HYhwpEQ의 윤곽을 사용하지 않는다. 원 glyph의 bbox는 (56,-306,609,805).

use super::layout::integral_geom;

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum Command {
    Move((f64, f64)),
    Line((f64, f64)),
    Cubic([(f64, f64); 3]),
    Close,
}

const OUTLINE: &[Command] = &[
    Command::Move((609.0, 722.0)),
    Command::Cubic([(609.0, 770.0), (559.0, 805.0), (499.0, 805.0)]),
    Command::Cubic([(404.0, 805.0), (365.0, 708.0), (348.0, 632.0)]),
    Command::Cubic([(336.0, 577.0), (300.0, 295.0), (291.0, 174.0)]),
    Command::Line((272.0, -75.0)),
    Command::Cubic([(259.0, -239.0), (223.0, -284.0), (166.0, -284.0)]),
    Command::Cubic([(153.0, -284.0), (123.0, -281.0), (102.0, -263.0)]),
    Command::Cubic([(131.0, -259.0), (139.0, -236.0), (139.0, -222.0)]),
    Command::Cubic([(139.0, -193.0), (117.0, -180.0), (98.0, -180.0)]),
    Command::Cubic([(78.0, -180.0), (56.0, -193.0), (56.0, -223.0)]),
    Command::Cubic([(56.0, -271.0), (106.0, -306.0), (166.0, -306.0)]),
    Command::Cubic([(261.0, -306.0), (309.0, -219.0), (331.0, -129.0)]),
    Command::Cubic([(344.0, -77.0), (380.0, 214.0), (388.0, 325.0)]),
    Command::Line((407.0, 574.0)),
    Command::Cubic([(421.0, 758.0), (455.0, 783.0), (500.0, 783.0)]),
    Command::Cubic([(510.0, 783.0), (541.0, 781.0), (563.0, 762.0)]),
    Command::Cubic([(534.0, 758.0), (526.0, 735.0), (526.0, 721.0)]),
    Command::Cubic([(526.0, 692.0), (548.0, 679.0), (567.0, 679.0)]),
    Command::Cubic([(587.0, 679.0), (609.0, 692.0), (609.0, 722.0)]),
    Command::Close,
];

pub(crate) fn commands(x: f64, y: f64, fs: f64) -> Vec<Command> {
    let geom = integral_geom(fs);
    let sx = (geom.top_hook_x - geom.bottom_hook_x) / 553.0;
    let sy = (geom.bottom_y - geom.top_y) / 1111.0;
    let map = |(gx, gy): (f64, f64)| {
        (
            x + geom.bottom_hook_x + (gx - 56.0) * sx,
            y + geom.top_y + (805.0 - gy) * sy,
        )
    };
    OUTLINE
        .iter()
        .map(|command| match *command {
            Command::Move(p) => Command::Move(map(p)),
            Command::Line(p) => Command::Line(map(p)),
            Command::Cubic(p) => Command::Cubic(p.map(map)),
            Command::Close => Command::Close,
        })
        .collect()
}
