//! 공개 Latin Modern Math 관계 기호·둥근 괄호의 독립 잉크 프레임.
//! Copyright 2012--2014 B. Jackowski, P. Strzelczyk, P. Pianowski.
//! GUST Font License: assets/fonts/LatinModernMath-GUST-FONT-LICENSE.txt.
//! 공개 원본 SHA-256: 1177d9025d1e797c54b3c7676976365abd90f903423091de7f0f4e7a4b372162.
//! 사설 face에서는 UPM 1024의 bbox 메트릭만 소비한다. 윤곽·바이트는 복제하지 않는다.

use super::GlyphOutlineCommand as Command;

// 공개 less, UPM 1000, bbox (77.25, -47.06528420403273, 700.5583390178637, 547.0652842040328).
const LESS: &[Command] = &[
    Command::Move((689.0, -9.0)),
    Command::Line((141.0, 250.0)),
    Command::Line((689.0, 509.0)),
    Command::Cubic([(713.0, 520.0), (696.0, 556.0), (671.0, 545.0)]),
    Command::Line((90.0, 270.0)),
    Command::Cubic([(73.0, 262.0), (73.0, 238.0), (90.0, 230.0)]),
    Command::Line((671.0, -45.0)),
    Command::Cubic([(696.0, -56.0), (713.0, -20.0), (689.0, -9.0)]),
    Command::Close,
];

// 공개 greater, UPM 1000, bbox (77.44166098213633, -47.06528420403272, 700.75, 547.0652842040328).
const GREATER: &[Command] = &[
    Command::Move((688.0, 230.0)),
    Command::Cubic([(705.0, 238.0), (705.0, 262.0), (688.0, 270.0)]),
    Command::Line((107.0, 545.0)),
    Command::Cubic([(82.0, 556.0), (65.0, 520.0), (89.0, 509.0)]),
    Command::Line((637.0, 250.0)),
    Command::Line((89.0, -9.0)),
    Command::Cubic([(65.0, -20.0), (82.0, -56.0), (107.0, -45.0)]),
    Command::Close,
];

// 공개 lessequal, UPM 1000, bbox (76, -119, 702.5583390178637, 631.0652842040328).
const LESS_EQUAL: &[Command] = &[
    Command::Move((691.0, 75.0)),
    Command::Line((143.0, 334.0)),
    Command::Line((691.0, 593.0)),
    Command::Cubic([(715.0, 604.0), (698.0, 640.0), (673.0, 629.0)]),
    Command::Line((92.0, 354.0)),
    Command::Cubic([(75.0, 346.0), (75.0, 322.0), (92.0, 314.0)]),
    Command::Line((673.0, 39.0)),
    Command::Cubic([(698.0, 28.0), (715.0, 64.0), (691.0, 75.0)]),
    Command::Close,
    Command::Move((702.0, -99.0)),
    Command::Cubic([(702.0, -88.0), (693.0, -79.0), (682.0, -79.0)]),
    Command::Line((96.0, -79.0)),
    Command::Cubic([(85.0, -79.0), (76.0, -88.0), (76.0, -99.0)]),
    Command::Cubic([(76.0, -110.0), (85.0, -119.0), (96.0, -119.0)]),
    Command::Line((682.0, -119.0)),
    Command::Cubic([(693.0, -119.0), (702.0, -110.0), (702.0, -99.0)]),
    Command::Close,
];

// 공개 greaterequal, UPM 1000, bbox (75.44166098213633, -119, 702, 631.0652842040328).
const GREATER_EQUAL: &[Command] = &[
    Command::Move((686.0, 314.0)),
    Command::Cubic([(703.0, 322.0), (703.0, 346.0), (686.0, 354.0)]),
    Command::Line((105.0, 629.0)),
    Command::Cubic([(80.0, 640.0), (63.0, 604.0), (87.0, 593.0)]),
    Command::Line((635.0, 334.0)),
    Command::Line((87.0, 75.0)),
    Command::Cubic([(63.0, 64.0), (80.0, 28.0), (105.0, 39.0)]),
    Command::Close,
    Command::Move((702.0, -99.0)),
    Command::Cubic([(702.0, -88.0), (693.0, -79.0), (682.0, -79.0)]),
    Command::Line((96.0, -79.0)),
    Command::Cubic([(85.0, -79.0), (76.0, -88.0), (76.0, -99.0)]),
    Command::Cubic([(76.0, -110.0), (85.0, -119.0), (96.0, -119.0)]),
    Command::Line((682.0, -119.0)),
    Command::Cubic([(693.0, -119.0), (702.0, -110.0), (702.0, -99.0)]),
    Command::Close,
];

// 공개 parenleft, UPM 1000, bbox (101, -248, 332, 748).
const LEFT_PAREN: &[Command] = &[
    Command::Move((332.0, -238.0)),
    Command::Cubic([(332.0, -235.0), (330.0, -232.0), (328.0, -230.0)]),
    Command::Cubic([(224.0, -152.0), (155.0, 48.0), (155.0, 208.0)]),
    Command::Line((155.0, 292.0)),
    Command::Cubic([(155.0, 452.0), (224.0, 652.0), (328.0, 730.0)]),
    Command::Cubic([(330.0, 732.0), (332.0, 735.0), (332.0, 738.0)]),
    Command::Cubic([(332.0, 743.0), (327.0, 748.0), (322.0, 748.0)]),
    Command::Cubic([(320.0, 748.0), (318.0, 747.0), (316.0, 746.0)]),
    Command::Cubic([(206.0, 663.0), (101.0, 463.0), (101.0, 292.0)]),
    Command::Line((101.0, 208.0)),
    Command::Cubic([(101.0, 37.0), (206.0, -163.0), (316.0, -246.0)]),
    Command::Cubic([(318.0, -247.0), (320.0, -248.0), (322.0, -248.0)]),
    Command::Cubic([(327.0, -248.0), (332.0, -243.0), (332.0, -238.0)]),
    Command::Close,
];

// 공개 parenright, UPM 1000, bbox (57, -248, 288, 748).
const RIGHT_PAREN: &[Command] = &[
    Command::Move((288.0, 208.0)),
    Command::Line((288.0, 292.0)),
    Command::Cubic([(288.0, 463.0), (183.0, 663.0), (73.0, 746.0)]),
    Command::Cubic([(71.0, 747.0), (69.0, 748.0), (67.0, 748.0)]),
    Command::Cubic([(62.0, 748.0), (57.0, 743.0), (57.0, 738.0)]),
    Command::Cubic([(57.0, 735.0), (59.0, 732.0), (61.0, 730.0)]),
    Command::Cubic([(165.0, 652.0), (234.0, 452.0), (234.0, 292.0)]),
    Command::Line((234.0, 208.0)),
    Command::Cubic([(234.0, 48.0), (165.0, -152.0), (61.0, -230.0)]),
    Command::Cubic([(59.0, -232.0), (57.0, -235.0), (57.0, -238.0)]),
    Command::Cubic([(57.0, -243.0), (62.0, -248.0), (67.0, -248.0)]),
    Command::Cubic([(69.0, -248.0), (71.0, -247.0), (73.0, -246.0)]),
    Command::Cubic([(183.0, -163.0), (288.0, 37.0), (288.0, 208.0)]),
    Command::Close,
];

pub(super) fn commands(text: &str, fs: f64, dpi: f64, baseline: f64) -> Option<Vec<Command>> {
    let (outline, source, target) = match text {
        "<" => (
            LESS,
            (77.2, -47.1, 700.6, 547.1),
            (103.0, -38.0, 762.0, 555.0),
        ),
        ">" => (
            GREATER,
            (77.4, -47.1, 700.8, 547.1),
            (97.0, -38.0, 756.0, 555.0),
        ),
        "≤" => (
            LESS_EQUAL,
            (76.0, -119.0, 702.6, 631.1),
            (154.0, -2.0, 845.0, 634.0),
        ),
        "≥" => (
            GREATER_EQUAL,
            (75.4, -119.0, 702.0, 631.1),
            (165.0, -2.0, 855.0, 634.0),
        ),
        "(" => (
            LEFT_PAREN,
            (101.0, -248.0, 332.0, 748.0),
            (102.0, -207.0, 345.0, 826.0),
        ),
        ")" => (
            RIGHT_PAREN,
            (57.0, -248.0, 288.0, 748.0),
            (52.0, -208.0, 295.0, 826.0),
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

/// 한컴 Print는 작은 확장 괄호의 세로 em을 본체 높이에 맞추고 가로 em은 유지한다.
/// 공개 기본 괄호의 닫힌 윤곽을 같은 잉크 프레임에 투영한다.
pub(super) fn small_round_commands(
    height: f64,
    fs: f64,
    dpi: f64,
    right: bool,
) -> Option<Vec<Command>> {
    if !height.is_finite() || height <= 0.0 || !fs.is_finite() || fs <= 0.0 {
        return None;
    }
    let text = if right { ")" } else { "(" };
    let origin = -super::negation_geometry::math_origin_shift(fs, dpi);
    let top = origin - fs * 826.0 / 1024.0;
    let descent = if right { 208.0 } else { 207.0 };
    let bottom = origin + fs * descent / 1024.0;
    let scale_y = height / fs;
    let inset = (height - (bottom - top) * scale_y) / 2.0;
    let map = |(x, y): (f64, f64)| (x, inset + (y - top) * scale_y);
    Some(
        commands(text, fs, dpi, 0.0)?
            .into_iter()
            .map(|command| match command {
                Command::Move(p) => Command::Move(map(p)),
                Command::Line(p) => Command::Line(map(p)),
                Command::Cubic(p) => Command::Cubic(p.map(map)),
                Command::Close => Command::Close,
            })
            .collect(),
    )
}

/// 한컴 Print의 E101은 UPM 1024에서 가로 bbox 225..286인 세로 막대다.
/// 폰트 윤곽 대신 논리 구분자 높이를 소비하는 공통 채움 사각형을 발행한다.
pub(super) fn vertical_delimiter_commands(height: f64, fs: f64) -> Option<Vec<Command>> {
    if !height.is_finite() || height <= 0.0 || !fs.is_finite() || fs <= 0.0 {
        return None;
    }
    let left = fs * 225.0 / 1024.0;
    let right = fs * 286.0 / 1024.0;
    Some(vec![
        Command::Move((left, 0.0)),
        Command::Line((right, 0.0)),
        Command::Line((right, height)),
        Command::Line((left, height)),
        Command::Close,
    ])
}
