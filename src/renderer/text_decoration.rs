//! 글자 장식(밑줄) 기하 — 한글 2022 실측 계약. (#5730)
//!
//! 종전에는 밑줄을 기준선 아래 **고정 2.0px** 에 그렸다. 한글 2022 COM 프로브
//! (9/12/15/18/24/36pt, 실선)로 실측하면 밑줄은 글꼴 크기에 비례해
//! **기준선 + 0.17em** 에 온다 (1.44/9=0.160, 2.04/12=0.170, 2.52/15=0.168,
//! 3.00/18=0.167, 4.07/24=0.170, 6.11/36=0.170). 고정 2.0px 은 11.8px 글꼴에서만
//! 우연히 맞고, 제목처럼 큰 글꼴에서는 밑줄이 디센더를 가로지른다
//! (156467175 실측: 24px 제목에서 한글 4.2px vs rhwp 0.8px).
//!
//! 이중/삼중선(shape 7~10)도 같은 프로브(15pt·24pt)로 선별 위치·굵기가 전부
//! em 비례임을 실측했다. 이 모듈이 그 계약의 단일 출처다 — SVG/Skia 백엔드가
//! 같은 표를 소비한다.

/// 단일선(및 파선 등 shape 0~6, 11)의 기준선 아래 오프셋 비율 (em).
pub(crate) const UNDERLINE_BASELINE_RATIO: f64 = 0.17;

/// 밑줄 다중선(shape 7~10)의 (기준선 아래 오프셋 em, 선 굵기 em) 목록.
///
/// 한글 2022 실측 (15pt/24pt 프로브, PDF 드로잉 좌표):
/// - 7 이중선: 0.160(0.032) + 0.250(0.032)
/// - 8 가는+굵은: 0.175(0.048) + 0.300(0.100)
/// - 9 굵은+가는: 0.200(0.100) + 0.325(0.048)
/// - 10 삼중선: 0.165(0.032) + 0.250(0.080) + 0.335(0.032)
///
/// 그 외 shape 는 `None` — 호출부가 단일선 경로(0.17em, 기존 선 모양 유지)로 그린다.
pub(crate) fn underline_multi_lines(shape: u8) -> Option<&'static [(f64, f64)]> {
    match shape {
        7 => Some(&[(0.160, 0.032), (0.250, 0.032)]),
        8 => Some(&[(0.175, 0.048), (0.300, 0.100)]),
        9 => Some(&[(0.200, 0.100), (0.325, 0.048)]),
        10 => Some(&[(0.165, 0.032), (0.250, 0.080), (0.335, 0.032)]),
        _ => None,
    }
}

/// 밑줄의 독립 Print 계약. 좌표는 기준선에 대한 오프셋이며 길이 단위는 레이아웃 px다.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DecorationStroke {
    pub offset: f64,
    pub width: f64,
    pub dash: Vec<f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DecorationDots {
    pub offset: f64,
    pub radius: f64,
    pub step: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct TopUnderlineGeometry {
    pub phase: f64,
    pub strokes: Vec<DecorationStroke>,
    pub dots: Option<DecorationDots>,
}

impl TopUnderlineGeometry {
    /// 기준선에 대한 잉크의 위·아래 경계. 모든 출력과 clip이 같은 기하를 사용한다.
    pub fn vertical_bounds(&self) -> (f64, f64) {
        self.strokes.iter().fold(
            self.dots
                .as_ref()
                .map_or((f64::INFINITY, f64::NEG_INFINITY), |dot| {
                    (dot.offset - dot.radius, dot.offset + dot.radius)
                }),
            |(top, bottom), stroke| {
                (
                    top.min(stroke.offset - stroke.width / 2.0),
                    bottom.max(stroke.offset + stroke.width / 2.0),
                )
            },
        )
    }
}

/// 한컴 Print의 12/24/36/48pt 대조군에서 확인한 위 밑줄이다.
/// 같은 줄의 비장식 큰 글자도 높이·굵기의 기준이며, 아래 밑줄의 표와 구분한다.
pub(crate) fn top_underline_geometry(
    style: &super::TextStyle,
    font_size: f64,
) -> Option<TopUnderlineGeometry> {
    use crate::model::style::UnderlineType;
    if style.underline != UnderlineType::Top || style.underline_shape > 10 {
        return None;
    }
    let size = style
        .top_underline_font_size
        .filter(|size| size.is_finite() && *size > 0.0)
        .unwrap_or(font_size);
    if !(size * 2.0).is_finite() || size <= 0.0 {
        return None;
    }
    let shape = style.underline_shape;
    let phase = style
        .top_underline_phase
        .filter(|phase| phase.is_finite())
        .unwrap_or(0.0);
    if shape == 6 {
        return Some(TopUnderlineGeometry {
            phase,
            strokes: Vec::new(),
            dots: Some(DecorationDots {
                offset: -0.875 * size,
                radius: 0.03 * size,
                step: size / 7.0,
            }),
        });
    }
    let bands: &[(f64, f64)] = match shape {
        7 => &[(0.955, 0.030), (0.865, 0.030)],
        8 => &[(1.019, 0.048), (0.900, 0.100)],
        9 => &[(0.993, 0.100), (0.874, 0.048)],
        10 => &[(1.035, 0.030), (0.950, 0.080), (0.865, 0.030)],
        _ => &[(0.870, 0.040)],
    };
    // 짧은 선·긴 선과 간격의 공통 비율. Print 장치의 600dpi 반올림은 넣지 않는다.
    let pattern: &[f64] = match shape {
        1 => &[2.0, 3.0],
        2 => &[10.0, 6.0],
        3 => &[20.0, 6.0, 2.0, 6.0],
        4 => &[20.0, 6.0, 2.0, 6.0, 2.0, 6.0],
        5 => &[20.0, 6.0],
        _ => &[],
    };
    Some(TopUnderlineGeometry {
        phase,
        strokes: bands
            .iter()
            .map(|&(offset, width)| DecorationStroke {
                offset: -offset * size,
                width: width * size,
                dash: pattern.iter().map(|value| value * size / 35.0).collect(),
            })
            .collect(),
        dots: None,
    })
}

/// 아래 일점쇄선도 같은 출력 기하를 소비한다. 위 밑줄의 기존 계약은 그대로 유지한다.
pub(crate) fn underline_geometry(
    style: &super::TextStyle,
    font_size: f64,
) -> Option<TopUnderlineGeometry> {
    use crate::model::style::UnderlineType;
    if style.underline != UnderlineType::Bottom || style.underline_shape != 3 {
        return top_underline_geometry(style, font_size);
    }
    if !(font_size * 2.0).is_finite() || font_size <= 0.0 {
        return None;
    }
    // 9~48pt Print에서 점은 600dpi 한 칸, 간격은 두 칸 격자에 맞는다.
    let scale = font_size / 35.0;
    let dot = (scale * 2.0 / 0.16).round().max(1.0) * 0.16;
    let gap = (scale * 6.0 / 0.32).round().max(1.0) * 0.32;
    Some(TopUnderlineGeometry {
        phase: style
            .top_underline_phase
            .filter(|phase| phase.is_finite())
            .unwrap_or(0.0),
        strokes: vec![DecorationStroke {
            offset: font_size * UNDERLINE_BASELINE_RATIO,
            width: font_size * 0.04,
            dash: vec![scale * 20.0, gap, dot, gap],
        }],
        dots: None,
    })
}

/// 언어별 렌더링 런 경계는 원본 글자 모양 구간의 펜 경계가 아니다.
/// 글꼴·크기·색을 실제로 바꾼 원본 구간과 비장식 간격에서는 위상을 다시 시작한다.
pub(crate) fn resolve_page_phases(root: &mut super::render_tree::RenderNode) {
    use super::render_tree::{RenderNodeType, TextRunNode};
    use super::text_direction::VisualTextPositions;
    use crate::model::style::UnderlineType;
    for child in &mut root.children {
        resolve_page_phases(child);
    }
    if !matches!(root.node_type, RenderNodeType::TextLine(_)) {
        return;
    }
    type Key = (u32, u8, u32, bool);
    struct Group {
        key: Key,
        baseline: f64,
        end: f64,
        origin: f64,
        runs: Vec<usize>,
    }
    let mut groups: Vec<Group> = Vec::new();
    let mut active = false;
    for (index, node) in root.children.iter().enumerate() {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            active = false;
            continue;
        };
        let TextRunNode { style, .. } = run;
        let Some(id) = run.char_shape_id.filter(|_| {
            (style.underline == UnderlineType::Top
                || (style.underline == UnderlineType::Bottom && style.underline_shape == 3))
                && run.rotation == 0.0
                && !run.is_vertical
                && run.char_overlap.is_none()
                && !run.display_or_text().contains('\u{fffc}')
        }) else {
            active = false;
            continue;
        };
        let text = run.display_or_text();
        let positions = run.replay_positions_for(text);
        let (Some(&start), Some(&end)) = (positions.first(), positions.last()) else {
            active = false;
            continue;
        };
        let (_, baseline) =
            style.script_draw_metrics(style.effective_font_size(), node.bbox.y + run.baseline);
        let start = node.bbox.x + start;
        let end = node.bbox.x + end;
        let origin = VisualTextPositions::for_style(text, &positions, style)
            .map(|v| {
                v.segments(text.chars().count())
                    .into_iter()
                    .map(|(left, _)| node.bbox.x + left)
                    .fold(f64::INFINITY, f64::min)
            })
            .unwrap_or(start);
        if ![start, end, origin, baseline].iter().all(|x| x.is_finite()) || end <= start {
            active = false;
            continue;
        }
        let key = (
            id,
            style.underline_shape,
            style.underline_color,
            style.underline == UnderlineType::Top,
        );
        if active
            && groups.last().is_some_and(|g| {
                g.key == key && (g.baseline - baseline).abs() < 1e-6 && (g.end - start).abs() < 1e-6
            })
        {
            let g = groups.last_mut().unwrap();
            g.end = end;
            g.origin = g.origin.min(origin);
            g.runs.push(index);
        } else {
            groups.push(Group {
                key,
                baseline,
                end,
                origin,
                runs: vec![index],
            });
        }
        active = true;
    }
    for group in groups {
        for index in group.runs {
            let node = &mut root.children[index];
            if let RenderNodeType::TextRun(run) = &mut node.node_type {
                run.style.top_underline_phase = Some(node.bbox.x - group.origin);
            }
        }
    }
}

/// 글줄 상자 위로 올라가는 장식(위 밑줄·강조점)의 잉크 범위다.
///
/// 첫 줄의 이 잉크는 글줄 상자 밖, 본문 영역 위에 놓여 본문 clip에 잘린다.
/// 글자·글줄 상자를 키우면 캐럿/선택도 바뀌므로 본문 clip 계산에만 쓴다.
/// 위 밑줄과 강조점의 공통 기하를 소비한다.
/// 회전·세로쓰기·글자겹침은 다루지 않는다.
pub(crate) fn raised_decoration_ink_bbox(
    bbox: super::render_tree::BoundingBox,
    run: &super::render_tree::TextRunNode,
) -> Option<super::render_tree::BoundingBox> {
    use crate::model::style::UnderlineType;

    let style = &run.style;
    let top_underline = style.underline == UnderlineType::Top && style.underline_shape <= 10;
    let emphasis = (1..=6).contains(&style.emphasis_dot);
    if !(top_underline || emphasis)
        || run.char_overlap.is_some()
        || run.rotation != 0.0
        || run.is_vertical
        || !run.display_or_text().chars().any(|ch| ch != '\u{FFFC}')
    {
        return None;
    }
    let base_size = if style.font_size > 0.0 {
        style.font_size
    } else {
        12.0
    };
    let (font_size, baseline) = style.script_draw_metrics(base_size, bbox.y + run.baseline);
    let mut top = f64::INFINITY;
    let mut bottom = f64::NEG_INFINITY;
    let mut left = bbox.x;
    let mut right = bbox.x + bbox.width;
    let geometry = top_underline_geometry(style, font_size);
    if let Some(geometry) = &geometry {
        let (ink_top, ink_bottom) = geometry.vertical_bounds();
        top = top.min(baseline + ink_top);
        bottom = bottom.max(baseline + ink_bottom);
    }
    if emphasis {
        let text = run.display_or_text();
        let positions = run.replay_positions_for(text);
        if let Some(ink) = super::emphasis::geometry(text, &positions, style, font_size)
            .and_then(|g| g.ink_bounds())
        {
            top = top.min(baseline + ink.y);
            bottom = bottom.max(baseline + ink.y + ink.height);
            left = left.min(bbox.x + ink.x);
            right = right.max(bbox.x + ink.x + ink.width);
        }
    }
    // 원형 점선(6)만 round cap 이라 선 양 끝이 획 반만큼 번진다.
    let end_pad = geometry
        .as_ref()
        .and_then(|g| g.dots.as_ref())
        .map_or(0.0, |d| d.radius);
    let ink = super::render_tree::BoundingBox::new(
        left - end_pad,
        top,
        right - left + 2.0 * end_pad,
        bottom - top,
    );
    [ink.x, ink.y, ink.width, ink.height]
        .iter()
        .all(|v| v.is_finite())
        .then_some(ink)
}
