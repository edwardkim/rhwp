//! 공개 Latin Modern Math의 중괄호를 수학 글리프의 잉크 프레임에 배치한다.
//! Copyright 2012--2014 B. Jackowski, P. Strzelczyk, P. Pianowski.
//! GUST Font License: assets/fonts/LatinModernMath-GUST-FONT-LICENSE.txt.
//! 원본 LatinModernMath-Regular.woff2 SHA-256:
//! 1177d9025d1e797c54b3c7676976365abd90f903423091de7f0f4e7a4b372162.
//! braceleft, UPM 1000, bbox (75,-250,425,750). 사설 윤곽은 포함하지 않는다.
//! 원 face의 수학 중괄호는 bbox (113,-203,446,821), UPM 1024다.

use super::GlyphOutlineCommand as Command;

const OUTLINE: &[Command] = &[
    Command::Move((425.0, -238.0)),
    Command::Cubic([(425.0, -231.0), (420.0, -226.0), (413.0, -226.0)]),
    Command::Cubic([(338.0, -226.0), (283.0, -177.0), (283.0, -125.0)]),
    Command::Line((283.0, 125.0)),
    Command::Cubic([(283.0, 183.0), (235.0, 229.0), (168.0, 250.0)]),
    Command::Cubic([(235.0, 271.0), (283.0, 317.0), (283.0, 375.0)]),
    Command::Line((283.0, 625.0)),
    Command::Cubic([(283.0, 677.0), (338.0, 726.0), (413.0, 726.0)]),
    Command::Cubic([(420.0, 726.0), (425.0, 731.0), (425.0, 738.0)]),
    Command::Cubic([(425.0, 745.0), (420.0, 750.0), (413.0, 750.0)]),
    Command::Cubic([(308.0, 750.0), (217.0, 698.0), (217.0, 625.0)]),
    Command::Line((217.0, 375.0)),
    Command::Cubic([(217.0, 317.0), (162.0, 262.0), (87.0, 262.0)]),
    Command::Cubic([(80.0, 262.0), (75.0, 257.0), (75.0, 250.0)]),
    Command::Cubic([(75.0, 243.0), (80.0, 238.0), (87.0, 238.0)]),
    Command::Cubic([(162.0, 238.0), (217.0, 183.0), (217.0, 125.0)]),
    Command::Line((217.0, -125.0)),
    Command::Cubic([(217.0, -198.0), (308.0, -250.0), (413.0, -250.0)]),
    Command::Cubic([(420.0, -250.0), (425.0, -245.0), (425.0, -238.0)]),
    Command::Close,
];

pub(super) fn commands(fs: f64, dpi: f64, baseline: f64, right: bool) -> Vec<Command> {
    let origin = baseline - super::negation_geometry::math_origin_shift(fs, dpi);
    let map = |(x, y): (f64, f64)| {
        let x = if right { 500.0 - x } else { x };
        (
            fs * (113.0 + (x - 75.0) * 333.0 / 350.0) / 1024.0,
            origin - fs * (-203.0 + (y + 250.0) * 1024.0 / 1000.0) / 1024.0,
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
