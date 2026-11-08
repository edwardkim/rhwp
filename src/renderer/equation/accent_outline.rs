//! 한컴 Print의 장식 잉크 프레임에 공개 MATH 화살촉을 배치한다.
//! 공개 Latin Modern Math 1.959의 arrowright.rt / arrowboth.lft / arrowboth.rt만 사용한다.
//! Copyright 2012--2014 B. Jackowski, P. Strzelczyk, P. Pianowski.
//! GUST Font License: assets/fonts/LatinModernMath-GUST-FONT-LICENSE.txt.
//! 원본 LatinModernMath-Regular.woff2 SHA-256:
//! 1177d9025d1e797c54b3c7676976365abd90f903423091de7f0f4e7a4b372162.
//! 사설 글꼴의 곡선·바이트는 포함하지 않는다. 독립 Print의 bbox·논리 크기만 대조한다.

use super::layout::LayoutBox;
use super::symbols::DecoKind;
use super::GlyphOutlineCommand as Command;

const RIGHT_HEAD: &[Command] = &[
    Command::Move((507.0, 250.0)),
    Command::Cubic([(507.0, 256.0), (503.0, 261.0), (498.0, 263.0)]),
    Command::Cubic([(441.0, 282.0), (397.0, 322.0), (361.0, 367.0)]),
    Command::Cubic([(333.0, 403.0), (312.0, 447.0), (303.0, 494.0)]),
    Command::Cubic([(301.0, 503.0), (293.0, 510.0), (283.0, 510.0)]),
    Command::Cubic([(272.0, 510.0), (263.0, 501.0), (263.0, 490.0)]),
    Command::Cubic([(263.0, 489.0), (264.0, 487.0), (264.0, 486.0)]),
    Command::Cubic([(274.0, 433.0), (297.0, 384.0), (330.0, 342.0)]),
    Command::Cubic([(352.0, 314.0), (378.0, 290.0), (408.0, 270.0)]),
    Command::Line((0.0, 270.0)),
    Command::Line((0.0, 230.0)),
    Command::Line((408.0, 230.0)),
    Command::Cubic([(378.0, 210.0), (352.0, 186.0), (330.0, 158.0)]),
    Command::Cubic([(297.0, 116.0), (274.0, 67.0), (264.0, 14.0)]),
    Command::Cubic([(264.0, 13.0), (263.0, 11.0), (263.0, 10.0)]),
    Command::Cubic([(263.0, -1.0), (272.0, -10.0), (283.0, -10.0)]),
    Command::Cubic([(293.0, -10.0), (301.0, -3.0), (303.0, 6.0)]),
    Command::Cubic([(312.0, 53.0), (333.0, 97.0), (361.0, 133.0)]),
    Command::Cubic([(397.0, 178.0), (441.0, 218.0), (498.0, 237.0)]),
    Command::Cubic([(503.0, 239.0), (507.0, 244.0), (507.0, 250.0)]),
    Command::Close,
];

const LEFT_HEAD: &[Command] = &[
    Command::Move((499.0, 270.0)),
    Command::Line((99.0, 270.0)),
    Command::Cubic([(128.0, 290.0), (155.0, 314.0), (177.0, 342.0)]),
    Command::Cubic([(209.0, 384.0), (232.0, 433.0), (243.0, 486.0)]),
    Command::Line((243.0, 490.0)),
    Command::Cubic([(243.0, 501.0), (234.0, 510.0), (223.0, 510.0)]),
    Command::Cubic([(214.0, 510.0), (206.0, 503.0), (204.0, 494.0)]),
    Command::Cubic([(194.0, 447.0), (174.0, 403.0), (145.0, 367.0)]),
    Command::Cubic([(110.0, 322.0), (65.0, 282.0), (9.0, 263.0)]),
    Command::Cubic([(4.0, 261.0), (0.0, 256.0), (0.0, 250.0)]),
    Command::Cubic([(0.0, 244.0), (4.0, 239.0), (9.0, 237.0)]),
    Command::Cubic([(65.0, 218.0), (110.0, 178.0), (145.0, 133.0)]),
    Command::Cubic([(174.0, 97.0), (194.0, 53.0), (204.0, 6.0)]),
    Command::Cubic([(206.0, -3.0), (214.0, -10.0), (223.0, -10.0)]),
    Command::Cubic([(234.0, -10.0), (243.0, -1.0), (243.0, 10.0)]),
    Command::Line((243.0, 14.0)),
    Command::Cubic([(232.0, 67.0), (209.0, 116.0), (177.0, 158.0)]),
    Command::Cubic([(155.0, 186.0), (128.0, 210.0), (99.0, 230.0)]),
    Command::Line((499.0, 230.0)),
    Command::Close,
];

const BOTH_RIGHT_HEAD: &[Command] = &[
    Command::Move((499.0, 250.0)),
    Command::Cubic([(499.0, 256.0), (495.0, 261.0), (490.0, 263.0)]),
    Command::Cubic([(433.0, 282.0), (389.0, 322.0), (353.0, 367.0)]),
    Command::Cubic([(325.0, 403.0), (304.0, 447.0), (295.0, 494.0)]),
    Command::Cubic([(293.0, 503.0), (285.0, 510.0), (275.0, 510.0)]),
    Command::Cubic([(264.0, 510.0), (255.0, 501.0), (255.0, 490.0)]),
    Command::Cubic([(255.0, 489.0), (256.0, 487.0), (256.0, 486.0)]),
    Command::Cubic([(266.0, 433.0), (289.0, 384.0), (322.0, 342.0)]),
    Command::Cubic([(344.0, 314.0), (370.0, 290.0), (400.0, 270.0)]),
    Command::Line((0.0, 270.0)),
    Command::Line((0.0, 230.0)),
    Command::Line((400.0, 230.0)),
    Command::Cubic([(370.0, 210.0), (344.0, 186.0), (322.0, 158.0)]),
    Command::Cubic([(289.0, 116.0), (266.0, 67.0), (256.0, 14.0)]),
    Command::Cubic([(256.0, 13.0), (255.0, 11.0), (255.0, 10.0)]),
    Command::Cubic([(255.0, -1.0), (264.0, -10.0), (275.0, -10.0)]),
    Command::Cubic([(285.0, -10.0), (293.0, -3.0), (295.0, 6.0)]),
    Command::Cubic([(304.0, 53.0), (325.0, 97.0), (353.0, 133.0)]),
    Command::Cubic([(389.0, 178.0), (433.0, 218.0), (490.0, 237.0)]),
    Command::Cubic([(495.0, 239.0), (499.0, 244.0), (499.0, 250.0)]),
    Command::Close,
];

/// 크기·장평은 배치 단계에서 한 번 발행하며 backend는 이 채우기 윤곽만 소비한다.
pub(super) fn commands(
    kind: DecoKind,
    body: &LayoutBox,
    fs: f64,
    dpi: f64,
) -> Option<Vec<Command>> {
    if body.width <= 0.0 || !matches!(kind, DecoKind::Bar | DecoKind::Vec | DecoKind::Dyad) {
        return None;
    }
    // 독립 Print의 600dpi 글꼴 크기 격자. 출력 DPI가 바뀌어도 같은 물리 기하다.
    let em = (fs * 600.0 / dpi).round() * dpi / 600.0;
    let center = body.y - fs * if kind == DecoKind::Vec { 0.25 } else { 0.15 };
    // HYhwpEQ의 shaft bbox 높이는 UPM 1024에서 41이다.
    let thickness = em * 41.0 / 1024.0;
    let head_width = em * 400.0 / 1024.0;
    let head_height = em * 265.0 / 1024.0;
    let right_tip = body.x + body.width + em * 0.075;
    let left_tip = body.x - em * 0.10;
    // 공개 화살촉의 직선 stem 끝까지 shaft를 겹친다. 화살촉 bbox를 맞춘
    // 세로 배율은 stem도 가늘게 하므로 최소 connector까지만 겹치면
    // 접합 뒤에 더 가는 선이 남는다. 공통 shaft의 굵기를 이 끝까지 보존한다.
    let right_stem_end = if kind == DecoKind::Vec {
        408.0 / 507.0
    } else {
        400.0 / 499.0
    };
    let tip_to_stem = head_width * (1.0 - right_stem_end);
    let (start, end) = match kind {
        DecoKind::Bar => (body.x, body.x + body.width),
        DecoKind::Vec => (body.x - em * 0.025, right_tip - tip_to_stem),
        DecoKind::Dyad => (
            left_tip + head_width * 99.0 / 499.0,
            right_tip - tip_to_stem,
        ),
        _ => unreachable!(),
    };
    let mut result = vec![
        Command::Move((start, center - thickness / 2.0)),
        Command::Line((end, center - thickness / 2.0)),
        Command::Line((end, center + thickness / 2.0)),
        Command::Line((start, center + thickness / 2.0)),
        Command::Close,
    ];
    if kind == DecoKind::Vec || kind == DecoKind::Dyad {
        let (outline, width) = if kind == DecoKind::Vec {
            (RIGHT_HEAD, 507.0)
        } else {
            (BOTH_RIGHT_HEAD, 499.0)
        };
        append_head(
            &mut result,
            outline,
            width,
            right_tip - head_width,
            center,
            head_width,
            head_height,
        );
    }
    if kind == DecoKind::Dyad {
        append_head(
            &mut result,
            LEFT_HEAD,
            499.0,
            left_tip,
            center,
            head_width,
            head_height,
        );
    }
    Some(result)
}

fn append_head(
    result: &mut Vec<Command>,
    outline: &[Command],
    source_width: f64,
    x: f64,
    center: f64,
    width: f64,
    height: f64,
) {
    // 공개 부품 bbox는 y=-10..510이며 중심은 250이다.
    let map = |(sx, sy): (f64, f64)| {
        (
            x + sx * width / source_width,
            center - (sy - 250.0) * height / 520.0,
        )
    };
    result.extend(outline.iter().map(|command| match *command {
        Command::Move(p) => Command::Move(map(p)),
        Command::Line(p) => Command::Line(map(p)),
        Command::Cubic(p) => Command::Cubic(p.map(map)),
        Command::Close => Command::Close,
    }));
}
