//! 공개 Latin Modern Math의 곱셈 윤곽을 원 face의 잉크 메트릭에 배치한다.
//! Copyright 2012--2014 B. Jackowski, P. Strzelczyk, P. Pianowski.
//! GUST Font License: assets/fonts/LatinModernMath-GUST-FONT-LICENSE.txt.
//! 원본: LatinModernMath-Regular.woff2, multiply, bbox (148,9,630,491), UPM 1000.
//! 원 face의 bbox (97,38,736,603), UPM 1024는 메트릭만 소비한다.
//! 사설 glyph의 윤곽·바이트를 포함하지 않는다. 값은 화면 px 좌표가 아니다.

use super::GlyphOutlineCommand as Command;

const OUTLINE: &[Command] = &[
    Command::Move((624.0, 15.0)),
    Command::Cubic([(632.0, 22.0), (632.0, 35.0), (624.0, 43.0)]),
    Command::Line((417.0, 250.0)),
    Command::Line((624.0, 457.0)),
    Command::Cubic([(632.0, 465.0), (632.0, 478.0), (624.0, 485.0)]),
    Command::Cubic([(617.0, 493.0), (604.0, 493.0), (596.0, 485.0)]),
    Command::Line((389.0, 278.0)),
    Command::Line((182.0, 485.0)),
    Command::Cubic([(174.0, 493.0), (161.0, 493.0), (154.0, 485.0)]),
    Command::Cubic([(146.0, 478.0), (146.0, 465.0), (154.0, 457.0)]),
    Command::Line((361.0, 250.0)),
    Command::Line((154.0, 43.0)),
    Command::Cubic([(146.0, 35.0), (146.0, 22.0), (154.0, 15.0)]),
    Command::Cubic([(161.0, 7.0), (174.0, 7.0), (182.0, 15.0)]),
    Command::Line((389.0, 222.0)),
    Command::Line((596.0, 15.0)),
    Command::Cubic([(604.0, 7.0), (617.0, 7.0), (624.0, 15.0)]),
    Command::Close,
];

pub(super) fn commands(fs: f64, advance: f64, baseline: f64) -> Vec<Command> {
    let width = fs * 639.0 / 1024.0;
    let height = fs * 565.0 / 1024.0;
    let center_y = baseline - fs * super::layout::AXIS_HEIGHT;
    let map = |(x, y): (f64, f64)| {
        (
            advance * 0.5 + (x - 389.0) * width / 482.0,
            center_y + (250.0 - y) * height / 482.0,
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

pub(crate) fn path_data(commands: &[Command]) -> String {
    use std::fmt::Write;
    let mut path = String::new();
    for command in commands {
        match *command {
            Command::Move((x, y)) => {
                let _ = write!(path, "M{x:.4},{y:.4}");
            }
            Command::Line((x, y)) => {
                let _ = write!(path, "L{x:.4},{y:.4}");
            }
            Command::Cubic([(a, b), (c, d), (e, f)]) => {
                let _ = write!(path, "C{a:.4},{b:.4} {c:.4},{d:.4} {e:.4},{f:.4}");
            }
            Command::Close => path.push('Z'),
        }
    }
    path
}
