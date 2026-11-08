//! 공개 Latin Modern Math 연산자를 독립 수학 잉크 프레임에 배치한다.
//! Copyright 2012--2014 B. Jackowski, P. Strzelczyk, P. Pianowski.
//! GUST Font License: assets/fonts/LatinModernMath-GUST-FONT-LICENSE.txt.
//! 공개 원본 SHA-256: 1177d9025d1e797c54b3c7676976365abd90f903423091de7f0f4e7a4b372162.
//! 사설 글리프에서는 UPM 1024의 메트릭만 소비하며 윤곽·바이트를 복제하지 않는다.

use super::GlyphOutlineCommand as Command;

// 공개 plus, UPM 1000, bbox (56, -83, 722, 583).
const PLUS: &[Command] = &[
    Command::Move((722.0, 250.0)),
    Command::Cubic([(722.0, 261.0), (713.0, 270.0), (702.0, 270.0)]),
    Command::Line((409.0, 270.0)),
    Command::Line((409.0, 563.0)),
    Command::Cubic([(409.0, 574.0), (400.0, 583.0), (389.0, 583.0)]),
    Command::Cubic([(378.0, 583.0), (369.0, 574.0), (369.0, 563.0)]),
    Command::Line((369.0, 270.0)),
    Command::Line((76.0, 270.0)),
    Command::Cubic([(65.0, 270.0), (56.0, 261.0), (56.0, 250.0)]),
    Command::Cubic([(56.0, 239.0), (65.0, 230.0), (76.0, 230.0)]),
    Command::Line((369.0, 230.0)),
    Command::Line((369.0, -63.0)),
    Command::Cubic([(369.0, -74.0), (378.0, -83.0), (389.0, -83.0)]),
    Command::Cubic([(400.0, -83.0), (409.0, -74.0), (409.0, -63.0)]),
    Command::Line((409.0, 230.0)),
    Command::Line((702.0, 230.0)),
    Command::Cubic([(713.0, 230.0), (722.0, 239.0), (722.0, 250.0)]),
    Command::Close,
];

// 공개 minus, UPM 1000, bbox (56, 230, 722, 270).
const MINUS: &[Command] = &[
    Command::Move((722.0, 250.0)),
    Command::Cubic([(722.0, 261.0), (713.0, 270.0), (702.0, 270.0)]),
    Command::Line((76.0, 270.0)),
    Command::Cubic([(65.0, 270.0), (56.0, 261.0), (56.0, 250.0)]),
    Command::Cubic([(56.0, 239.0), (65.0, 230.0), (76.0, 230.0)]),
    Command::Line((702.0, 230.0)),
    Command::Cubic([(713.0, 230.0), (722.0, 239.0), (722.0, 250.0)]),
    Command::Close,
];

// 공개 equal, UPM 1000, bbox (56, 133, 722, 367).
const EQUAL: &[Command] = &[
    Command::Move((722.0, 347.0)),
    Command::Cubic([(722.0, 358.0), (713.0, 367.0), (702.0, 367.0)]),
    Command::Line((76.0, 367.0)),
    Command::Cubic([(65.0, 367.0), (56.0, 358.0), (56.0, 347.0)]),
    Command::Cubic([(56.0, 336.0), (65.0, 327.0), (76.0, 327.0)]),
    Command::Line((702.0, 327.0)),
    Command::Cubic([(713.0, 327.0), (722.0, 336.0), (722.0, 347.0)]),
    Command::Close,
    Command::Move((722.0, 153.0)),
    Command::Cubic([(722.0, 164.0), (713.0, 173.0), (702.0, 173.0)]),
    Command::Line((76.0, 173.0)),
    Command::Cubic([(65.0, 173.0), (56.0, 164.0), (56.0, 153.0)]),
    Command::Cubic([(56.0, 142.0), (65.0, 133.0), (76.0, 133.0)]),
    Command::Line((702.0, 133.0)),
    Command::Cubic([(713.0, 133.0), (722.0, 142.0), (722.0, 153.0)]),
    Command::Close,
];

pub(super) fn commands(text: &str, fs: f64, dpi: f64, baseline: f64) -> Option<Vec<Command>> {
    let (outline, source, target) = match text {
        "+" => (
            PLUS,
            (56.0, -83.0, 722.0, 583.0),
            (57.0, -58.0, 738.0, 623.0),
        ),
        "−" => (
            MINUS,
            (56.0, 230.0, 722.0, 270.0),
            (57.0, 267.0, 718.0, 308.0),
        ),
        "=" => (
            EQUAL,
            (56.0, 133.0, 722.0, 367.0),
            (57.0, 164.0, 738.0, 403.0),
        ),
        _ => return None,
    };
    let origin = baseline - super::negation_geometry::math_origin_shift(fs, dpi);
    let map = |(x, y): (f64, f64)| {
        (
            fs * (target.0 + (x - source.0) * (target.2 - target.0) / (source.2 - source.0))
                / 1024.0,
            origin
                - fs * (target.1 + (y - source.1) * (target.3 - target.1) / (source.3 - source.1))
                    / 1024.0,
        )
    };
    Some(
        outline
            .iter()
            .map(|command| match *command {
                Command::Move(p) => Command::Move(map(p)),
                Command::Line(p) => Command::Line(map(p)),
                Command::Cubic(p) => Command::Cubic(p.map(map)),
                Command::Close => Command::Close,
            })
            .collect(),
    )
}
