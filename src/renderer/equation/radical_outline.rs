//! 공개 Latin Modern Math의 근호 윤곽을 공통 점유 상자에 연결한다.
//! Copyright 2012--2014 B. Jackowski, P. Strzelczyk, P. Pianowski.
//! GUST Font License: assets/fonts/LatinModernMath-GUST-FONT-LICENSE.txt.
//! LatinModernMath-Regular.woff2 SHA-256:
//! 1177d9025d1e797c54b3c7676976365abd90f903423091de7f0f4e7a4b372162.
//! U+221A radical, UPM 1000, 전진 833, bbox (73,-960,853,40).
//! 사설 글꼴 윤곽이나 문서의 절대 위치를 포함하지 않는다.

use super::layout::LayoutBox;
use super::GlyphOutlineCommand as Command;

const OUTLINE: &[Command] = &[
    Command::Move((853.0, 20.0)),
    Command::Cubic([(853.0, 31.0), (845.0, 40.0), (833.0, 40.0)]),
    Command::Cubic([(821.0, 40.0), (818.0, 33.0), (812.0, 21.0)]),
    Command::Line((389.0, -855.0)),
    Command::Line((211.0, -463.0)),
    Command::Cubic([(207.0, -455.0), (205.0, -449.0), (198.0, -449.0)]),
    Command::Cubic([(195.0, -449.0), (193.0, -449.0), (184.0, -457.0)]),
    Command::Line((85.0, -532.0)),
    Command::Cubic([(74.0, -541.0), (73.0, -542.0), (73.0, -546.0)]),
    Command::Cubic([(73.0, -553.0), (77.0, -557.0), (83.0, -557.0)]),
    Command::Cubic([(86.0, -557.0), (88.0, -557.0), (97.0, -549.0)]),
    Command::Line((149.0, -510.0)),
    Command::Line((347.0, -945.0)),
    Command::Cubic([(353.0, -959.0), (355.0, -960.0), (366.0, -960.0)]),
    Command::Cubic([(382.0, -960.0), (384.0, -956.0), (391.0, -941.0)]),
    Command::Line((847.0, 3.0)),
    Command::Cubic([(852.0, 14.0), (853.0, 15.0), (853.0, 20.0)]),
    Command::Close,
];

pub(crate) struct RadicalGeometry {
    pub outline: Vec<Command>,
    pub bar_start: (f64, f64),
    pub bar_end: (f64, f64),
    pub rule_thickness: f64,
}

pub(crate) fn geometry(value: &LayoutBox, body: &LayoutBox, fs: f64) -> RadicalGeometry {
    // 공개 근호의 전진 끝을 실제 몸체 원점에 연결한다. 근 지수가 예약한
    // 추가 공간도 body.x에 이미 포함되어 있으므로 별도로 추측하지 않는다.
    let origin_x = body.x - fs * 833.0 / 1000.0;
    let map = |(a, b): (f64, f64)| {
        (
            origin_x + a * fs / 1000.0,
            (40.0 - b) * value.height / 1000.0,
        )
    };
    let outline = OUTLINE
        .iter()
        .map(|command| match *command {
            Command::Move(p) => Command::Move(map(p)),
            Command::Line(p) => Command::Line(map(p)),
            Command::Cubic(p) => Command::Cubic(p.map(map)),
            Command::Close => Command::Close,
        })
        .collect();
    // glyph의 상단 접합 중심은 y=20이다. 세 경로가 같은 접합과 끝을 쓴다.
    let bar_y = value.height * 20.0 / 1000.0;
    RadicalGeometry {
        outline,
        bar_start: (body.x, bar_y),
        bar_end: (value.width, bar_y),
        rule_thickness: fs * 40.0 / 1000.0,
    }
}
