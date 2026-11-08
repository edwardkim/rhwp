//! 공개 Latin Modern Math의 MATH 둥근 괄호 조립 데이터.
//! Copyright 2012--2014 B. Jackowski, P. Strzelczyk, P. Pianowski.
//! GUST Font License: assets/fonts/LatinModernMath-GUST-FONT-LICENSE.txt.
//! 원본 SHA-256: 1177d9025d1e797c54b3c7676976365abd90f903423091de7f0f4e7a4b372162.
//! fontTools RecordingPen으로 추출. 사설 글꼴의 윤곽은 포함하지 않는다.

use super::Part;
use crate::renderer::equation::GlyphOutlineCommand as Command;

const OUTLINE_0: &[Command] = &[
    Command::Move((823.0, 21.0)),
    Command::Cubic([(823.0, 28.0), (820.0, 34.0), (814.0, 37.0)]),
    Command::Cubic([(559.0, 230.0), (379.0, 776.0), (379.0, 1245.0)]),
    Command::Line((379.0, 1495.0)),
    Command::Line((277.0, 1495.0)),
    Command::Line((277.0, 1245.0)),
    Command::Cubic([(277.0, 755.0), (522.0, 205.0), (790.0, 4.0)]),
    Command::Cubic([(793.0, 2.0), (797.0, 0.0), (802.0, 0.0)]),
    Command::Cubic([(813.0, 0.0), (823.0, 9.0), (823.0, 21.0)]),
    Command::Close,
];

const OUTLINE_1: &[Command] = &[
    Command::Move((379.0, 0.0)),
    Command::Line((379.0, 498.0)),
    Command::Line((277.0, 498.0)),
    Command::Line((277.0, 0.0)),
    Command::Close,
];

const OUTLINE_2: &[Command] = &[
    Command::Move((823.0, 1474.0)),
    Command::Cubic([(823.0, 1485.0), (813.0, 1495.0), (802.0, 1495.0)]),
    Command::Cubic([(797.0, 1495.0), (793.0, 1493.0), (790.0, 1491.0)]),
    Command::Cubic([(522.0, 1289.0), (277.0, 740.0), (277.0, 250.0)]),
    Command::Line((277.0, 0.0)),
    Command::Line((379.0, 0.0)),
    Command::Line((379.0, 250.0)),
    Command::Cubic([(379.0, 718.0), (559.0, 1265.0), (814.0, 1457.0)]),
    Command::Cubic([(820.0, 1461.0), (823.0, 1467.0), (823.0, 1474.0)]),
    Command::Close,
];

pub(super) const MAX_VARIANT: f64 = 2991.0;
pub(super) const MIN_OVERLAP: f64 = 20.0;
pub(super) const PARTS: &[Part] = &[
    Part {
        outline: OUTLINE_0,
        full: 1495.0,
        start: 0.0,
        end: 249.0,
        extender: false,
    },
    Part {
        outline: OUTLINE_1,
        full: 498.0,
        start: 498.0,
        end: 498.0,
        extender: true,
    },
    Part {
        outline: OUTLINE_2,
        full: 1495.0,
        start: 249.0,
        end: 0.0,
        extender: false,
    },
];
