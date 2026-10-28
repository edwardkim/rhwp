//! 구형 표시의 논리 전진과 선언 프레임에서 실제 글리프 원점을 함께 생산한다.
//!
//! 독립 영문·숫자 62자×1/2/3회 Print: 8d0992d1-ea91-4413-82ba-1f77d8e676ea.
//! 전진은 root 여백을 1로 정규화한 무차원 비율이다. 글리프 d1 폭이나 화면 px가 아니다.
//! 별도 너비·높이·크기·혼재 문자 44개 대조의 최대 x 오차는 96dpi에서 0.82987px다.
//! 명시적 그룹의 전진은 AST 소유를 읽어 계산하며, 글리프 크기와 세로 정렬은 보존한다.

use super::ast::EqNode;
use super::layout::{LayoutBox, LayoutKind};
use super::symbols::FontStyleKind;

#[path = "legacy_coordinate_plan.rs"]
mod plan;

// 독립 Print에서 얻은 root 여백 대비 논리 전진. 생성 근거와 교차 대조는
// task_semanticist21_20261024_legacy_equation_paint.md의 2931·2932·2959에 있다.
const UPPER: [f64; 26] = [
    3.92902788, 3.29155200, 3.29155200, 3.29155200, 3.29155200, 3.29155200, 3.29155200, 3.58579968,
    1.64839447, 1.64839447, 3.58579968, 3.29154957, 3.92902109, 3.29154957, 3.29154957, 3.03858903,
    3.29154957, 3.29154957, 2.78826602, 3.58579968, 3.58579968, 3.58579968, 3.58579968, 3.92902109,
    3.58579968, 2.78826602,
];
const LOWER: [f64; 26] = [
    2.49795890, 2.25592965, 2.25592965, 2.49795890, 2.25592965, 1.64839443, 2.49795890, 2.49795890,
    1.39494582, 1.39494582, 2.25592965, 1.39494582, 3.92902788, 2.78826414, 2.25592965, 2.25592965,
    2.25592965, 2.25592965, 1.64839443, 1.39494582, 2.78826414, 2.25592965, 3.29155200, 2.25592965,
    2.25592965, 2.25592965,
];
const DIGITS: [f64; 10] = [
    2.25593095, 2.25593095, 2.25593095, 2.25593095, 2.25593095, 2.25593095, 2.25593095, 2.25593095,
    2.25593095, 2.25593095,
];

fn cjk_letter(character: char) -> bool {
    character.is_alphabetic() && super::text_has_cjk(&character.to_string())
}

fn advance(character: char) -> Option<f64> {
    match character {
        'A'..='Z' => Some(UPPER[character as usize - 'A' as usize]),
        'a'..='z' => Some(LOWER[character as usize - 'a' as usize]),
        '0'..='9' => Some(DIGITS[character as usize - '0' as usize]),
        // 독립 literal108조건과 CJK/연산자72조건의 root 단위 대비 전진이다.
        '%' | '△' | '․' | '=' | '+' | '-' => Some(9.0 / 2.0),
        '/' => Some(9.0 / 4.0),
        c if cjk_letter(c) => Some(9.0 / 2.0),
        _ => None,
    }
}

/// 같은 glyph로 단순화돼도 명시적 Group·인용·함수는 별도의 계약이다.
fn linear_source(node: &EqNode) -> bool {
    match node {
        EqNode::Text(text) => !text.is_empty() && text.bytes().all(|ch| ch.is_ascii_alphabetic()),
        EqNode::Number(text) => !text.is_empty() && text.bytes().all(|ch| ch.is_ascii_digit()),
        EqNode::Row(items) => items.iter().all(linear_source),
        EqNode::FontStyle {
            style: FontStyleKind::Italic,
            body,
        } => linear_source(body),
        EqNode::Empty => true,
        _ => false,
    }
}

fn observe_line(font_size: f64, baseline: f64, frame: &mut Option<(f64, f64)>) -> Option<()> {
    if !font_size.is_finite() || font_size <= 0.0 || !baseline.is_finite() {
        return None;
    }
    let expected = frame.get_or_insert((font_size, baseline));
    ((font_size - expected.0).abs() <= 1e-8 && (baseline - expected.1).abs() <= 1e-8).then_some(())
}

fn inspect(
    value: &LayoutBox,
    parent_y: f64,
    frame: &mut Option<(f64, f64)>,
    count: &mut usize,
) -> Option<f64> {
    let y = parent_y + value.y;
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            let characters = run.glyphs.chars().count();
            if run.font_family != super::legacy_glyphs::FONT_FAMILY
                || run.positions.len() != characters
                || run.shears.len() != characters
            {
                return None;
            }
            let baseline = y + value.baseline + run.baseline_shift;
            observe_line(value.font_size, baseline, frame)?;
            let mut width = 0.0;
            for (character, shear) in run.glyphs.chars().zip(&run.shears) {
                // 이 표본의 영문은 기본 이탤릭이고 숫자는 정체다.
                let expected_shear = if character.is_ascii_alphabetic() {
                    0.333
                } else {
                    0.0
                };
                if (*shear - expected_shear).abs() > 1e-8 {
                    return None;
                }
                width += advance(character)?;
            }
            *count += characters;
            Some(width)
        }
        LayoutKind::Text(text) if !text.is_empty() && text.chars().all(cjk_letter) => {
            observe_line(value.font_size, y + value.baseline, frame)?;
            *count += text.chars().count();
            text.chars()
                .try_fold(0.0, |width, ch| Some(width + advance(ch)?))
        }
        LayoutKind::Row(children) => children.iter().try_fold(0.0, |width, child| {
            Some(width + inspect(child, y, frame, count)?)
        }),
        LayoutKind::FontStyle {
            style: FontStyleKind::Italic,
            body,
        } => inspect(body, y, frame, count),
        LayoutKind::Empty => Some(0.0),
        _ => None,
    }
}

/// 자식의 논리 폭·원점도 같은 평면에 옮긴다. 세로 정렬·실제 font_size·shear는 유지한다.
fn place(value: &mut LayoutBox, scale: f64) -> f64 {
    let width = match &mut value.kind {
        LayoutKind::HancomGlyphs(run) => {
            let mut cursor = 0.0;
            for (character, position) in run.glyphs.chars().zip(&mut run.positions) {
                *position = cursor;
                cursor += advance(character).expect("검사한 논리 문자") * scale;
            }
            run.width = cursor;
            cursor
        }
        LayoutKind::Row(children) => {
            let mut cursor = 0.0;
            for child in children {
                child.x = cursor;
                cursor += place(child, scale);
            }
            cursor
        }
        LayoutKind::FontStyle { body, .. } => {
            body.x = 0.0;
            place(body, scale)
        }
        LayoutKind::Empty => 0.0,
        _ => unreachable!("검사한 선형 표시 트리"),
    };
    value.width = width;
    width
}

pub(super) fn project_linear(node: &EqNode, value: &mut LayoutBox, width: f64) -> bool {
    if !linear_source(node) || !width.is_finite() || width <= 0.0 {
        return false;
    }
    let mut count = 0;
    let Some(advance) = inspect(value, 0.0, &mut None, &mut count) else {
        return false;
    };
    if count == 0 || !advance.is_finite() || advance <= 0.0 {
        return false;
    }
    let scale = width / (1.0 + advance);
    let mut body = value.clone();
    place(&mut body, scale);
    body.x = scale;
    body.y = 0.0;
    value.width = width;
    value.kind = LayoutKind::Row(vec![body]);
    true
}

/// 모든 잉크가 같은 글줄에 있는 문자·그룹을 하나의 가로 좌표계에 둔다.
/// 자식의 원점과 폭도 그 좌표계를 사용한다. font_size·baseline·y는 바꾸지 않는다.
fn place_plan(
    value: &mut LayoutBox,
    positions: &mut impl Iterator<Item = (char, f64)>,
    scale: f64,
) {
    value.x = 0.0;
    value.width = match &mut value.kind {
        LayoutKind::HancomGlyphs(run) => {
            let mut end = 0.0;
            for (character, position) in run.glyphs.chars().zip(&mut run.positions) {
                let (expected, x) = positions.next().expect("대조한 문자 소유");
                debug_assert_eq!(character, expected);
                *position = (1.0 + x) * scale;
                end = *position + advance(character).expect("대조한 논리 전진") * scale;
            }
            run.width = end;
            end
        }
        LayoutKind::Text(text) => {
            let mut offsets = Vec::with_capacity(text.chars().count());
            let mut end = 0.0;
            for character in text.chars() {
                let (expected, x) = positions.next().expect("대조한 CJK 문자 소유");
                debug_assert_eq!(character, expected);
                let origin = (1.0 + x) * scale;
                offsets.push(origin);
                end = origin + advance(character).expect("대조한 CJK 전진") * scale;
            }
            let glyphs = text.clone();
            let shears = vec![0.0; offsets.len()];
            // 측정의 자연 Text는 그대로 두고, 수용한 같은 CHAR 평면에서만
            // 독립 Print의 face·원점·물리 baseline을 모든 painter에 발행한다.
            value.kind = LayoutKind::HancomGlyphs(super::hancom_glyphs::GlyphRun {
                font_family: "Haansoft Batang",
                glyphs,
                positions: offsets,
                shears,
                width: end,
                baseline_shift: value.font_size * 0.06,
            });
            end
        }
        LayoutKind::Row(children) => {
            let mut end: f64 = 0.0;
            for child in children {
                place_plan(child, positions, scale);
                end = end.max(child.width);
            }
            end
        }
        LayoutKind::FontStyle { body, .. } => {
            place_plan(body, positions, scale);
            body.width
        }
        LayoutKind::Empty => 0.0,
        _ => unreachable!("대조한 단일 글줄의 표시 트리"),
    };
}

fn displayed_characters(value: &LayoutBox, out: &mut Vec<char>) {
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => out.extend(run.glyphs.chars()),
        LayoutKind::Text(text) => out.extend(text.chars()),
        LayoutKind::Row(children) => {
            for child in children {
                displayed_characters(child, out);
            }
        }
        LayoutKind::FontStyle { body, .. } => displayed_characters(body, out),
        _ => {}
    }
}

pub(super) fn project(node: &EqNode, value: &mut LayoutBox, width: f64) -> bool {
    let Some(plan) = plan::Plan::source(node) else {
        return project_linear(node, value, width);
    };
    if !width.is_finite() || width <= 0.0 || !plan.width.is_finite() || plan.width <= 0.0 {
        return false;
    }
    let mut count = 0;
    if inspect(value, 0.0, &mut None, &mut count).is_none() || count == 0 {
        return false;
    }
    let mut actual = Vec::with_capacity(count);
    displayed_characters(value, &mut actual);
    if !actual
        .iter()
        .copied()
        .eq(plan.glyphs.iter().map(|(ch, _)| *ch))
    {
        return false;
    }
    let scale = width / (1.0 + plan.width);
    place_plan(value, &mut plan.glyphs.into_iter(), scale);
    value.width = width;
    true
}
