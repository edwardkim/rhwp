//! 버전 문자열이 빈 옛 수식의 한글·숫자 줄과 단순 분수 프레임.
//!
//! 독립 빈 문서의 2024 저장/Print: 두 face·다섯 크기와 버전 대조를 따른다.
//! 원본 Type3 프로그램은 호출자가 공급한 `RHWP Legacy Equation`에서 읽는다.
//! 영문·그리스 문자와 첨자 등 미확인 구조는 기존 경로를 유지한다.

use super::{EqLayout, EqNode, FontGlyphPaint, FontStyleKind, LayoutBox, LayoutKind};
use crate::model::control::Equation;

const FAMILY: &str = "RHWP Legacy Equation, HyhwpEQ, serif";

pub(super) fn matches_version(equation: &Equation) -> bool {
    if equation.script.trim().is_empty()
        || !equation.version_info.is_empty()
        || equation.attr & 1 != 0
        || equation.font_size == 0
        || !(equation.font_name.eq_ignore_ascii_case("HYhwpEQ")
            || equation.font_name.eq_ignore_ascii_case("HancomEQN"))
    {
        return false;
    }
    // 옛 엔진의 한쪽에만 붙은 일반 공백은 연산자 간격을 바꾼다.
    // 해당 미확인 조합은 공백을 버린 AST만으로 새 간격을 추정하지 않는다.
    // 토큰화는 값 조건을 통과한 수식에만 한다.
    let tokens = super::super::document_script::tokenize(&equation.script);
    !tokens.windows(2).enumerate().any(|(index, pair)| {
        index > 0
            && pair[0].ty == super::super::tokenizer::TokenType::Symbol
            && matches!(pair[0].value.as_str(), "+" | "-" | "=" | "<" | ">")
            && !pair[0].space_before
            && pair[1].space_before
    })
}

fn glyph_width(character: char) -> Option<f64> {
    match character {
        '0'..='9' | '.' | '<' | '>' => Some(0.5),
        '+' | '-' | '−' | '=' | '%' => Some(1.0),
        '×' => Some(0.616),
        '÷' => Some(0.623),
        _ => None,
    }
}

fn cjk(character: char) -> bool {
    super::super::text_has_cjk(&character.to_string())
}

fn line_node(node: &EqNode) -> bool {
    match node {
        EqNode::Text(text) | EqNode::Number(text) => {
            text.chars().all(|ch| cjk(ch) || glyph_width(ch).is_some())
        }
        EqNode::Symbol(text) | EqNode::MathSymbol(text) => {
            text.chars().count() == 1 && text.chars().all(|ch| glyph_width(ch).is_some())
        }
        EqNode::Row(items) => items.iter().all(line_node),
        EqNode::FontStyle {
            style: FontStyleKind::Roman,
            body,
        } => line_node(body),
        EqNode::Color { body, .. } => line_node(body),
        EqNode::Empty => true,
        _ => false,
    }
}

pub(super) fn supports(node: &EqNode) -> bool {
    match node {
        EqNode::Fraction { numer, denom } => line_node(numer) && line_node(denom),
        EqNode::Paren { left, right, body } => {
            matches!((left.as_str(), right.as_str()), ("(", ")") | ("[", "]")) && supports(body)
        }
        EqNode::Row(items) => items.iter().all(supports),
        EqNode::FontStyle {
            style: FontStyleKind::Roman,
            body,
        } => supports(body),
        EqNode::Color { body, .. } => supports(body),
        _ => line_node(node),
    }
}

// 단독 HFT 글줄·분수는 기존의 등록 원점과 선언 프레임을 소유한다.
// 괄호가 감싼 복합 분수만 독립 Print로 확인한 옛 편집기의 연속 전진을 쓴다.
// 문자열이나 문서 식별자가 아니라 AST의 프레임 소유 구조로 구분한다.
pub(super) fn compound_frame(node: &EqNode) -> bool {
    fn fraction(node: &EqNode) -> bool {
        match node {
            EqNode::Fraction { .. } => true,
            EqNode::Row(items) => items.iter().any(fraction),
            EqNode::Paren { body, .. }
            | EqNode::FontStyle { body, .. }
            | EqNode::Color { body, .. } => fraction(body),
            _ => false,
        }
    }
    match node {
        EqNode::Paren { body, .. } => fraction(body),
        EqNode::Row(items) => items.iter().any(compound_frame),
        EqNode::FontStyle { body, .. } | EqNode::Color { body, .. } => compound_frame(body),
        _ => false,
    }
}

fn line_frame(fs: f64) -> (f64, f64) {
    let height = fs * 1.2;
    (height, height * 0.84)
}

fn row(fs: f64, width: f64, height: f64, baseline: f64, children: Vec<LayoutBox>) -> LayoutBox {
    LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width,
        height,
        baseline,
        kind: LayoutKind::Row(children),
    }
}

fn font_glyph(text: String, fs: f64, width: f64, height: f64, baseline: f64) -> LayoutBox {
    LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width,
        height,
        baseline,
        kind: LayoutKind::FontGlyph {
            text,
            font_family: FAMILY,
            outline: None,
            paint: None,
        },
    }
}

pub(super) fn text(text: &str, fs: f64) -> LayoutBox {
    let (height, baseline) = line_frame(fs);
    let mut children = Vec::new();
    let mut x = 0.0;
    for ch in text.chars() {
        let width = fs
            * if cjk(ch) {
                1.0
            } else {
                glyph_width(ch).unwrap_or(0.0)
            };
        let mut child = if cjk(ch) {
            LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width,
                height,
                baseline,
                kind: LayoutKind::Text(ch.to_string()),
            }
        } else {
            // 옛 수학 프로그램은 한글 대체 글자보다 0.06em 높은 기준선에서 그린다.
            font_glyph(
                if ch == '−' {
                    "-".to_owned()
                } else {
                    ch.to_string()
                },
                fs,
                width,
                height,
                baseline - fs * 0.06,
            )
        };
        child.x = x;
        x += width;
        children.push(child);
    }
    row(fs, x, height, baseline, children)
}

fn operator(node: &EqNode) -> Option<&str> {
    match node {
        EqNode::Symbol(text) | EqNode::MathSymbol(text) => Some(text),
        EqNode::FontStyle { body, .. } | EqNode::Color { body, .. } => operator(body),
        _ => None,
    }
}

pub(super) fn row_children(layout: &EqLayout, nodes: &[EqNode], fs: f64) -> Vec<LayoutBox> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let mut value = layout.layout_node(node, fs);
            let Some(op) = operator(node) else {
                return value;
            };
            let (left, right) = match op {
                "+" | "-" | "−" => (
                    if index > 0 { fs / 9.0 } else { 0.0 },
                    if index + 1 < nodes.len() && !matches!(nodes[index + 1], EqNode::Paren { .. })
                    {
                        fs / 9.0
                    } else {
                        // 늘인 괄호의 자체 앞 여백과 연산자 여백을 중복하지 않는다.
                        0.0
                    },
                ),
                "=" | "<" | ">" => (
                    if index > 0 { fs * 2.0 / 15.0 } else { 0.0 },
                    if index + 1 < nodes.len() {
                        fs / 6.0
                    } else {
                        0.0
                    },
                ),
                _ => (0.0, 0.0),
            };
            let width = left + value.width + right;
            let (height, baseline) = (value.height, value.baseline);
            value.x = left;
            row(fs, width, height, baseline, vec![value])
        })
        .collect()
}

pub(super) fn root(mut value: LayoutBox, fs: f64, dpi: f64) -> LayoutBox {
    // 옛 편집기의 바깥 전진은 각 HWPUNIT 항을 절삭한 뒤 합산한다.
    let pad_hu = (fs * 7200.0 / dpi * 2.0 / 9.0).floor();
    let pad = pad_hu * dpi / 7200.0;
    value.x = pad;
    row(
        fs,
        value.width + pad,
        value.height,
        value.baseline,
        vec![value],
    )
}

pub(super) fn fraction(mut numer: LayoutBox, mut denom: LayoutBox, fs: f64) -> LayoutBox {
    let (line_height, line_baseline) = line_frame(fs);
    let height = fs * 2.34;
    let baseline = height * 0.67;
    let width = numer.width.max(denom.width) + fs * 5.0 / 6.0;
    numer.x = (width - numer.width) / 2.0;
    denom.x = (width - denom.width) / 2.0;
    numer.y = line_baseline - numer.baseline;
    denom.y = height - line_height + line_baseline - denom.baseline;
    LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width,
        height,
        baseline,
        kind: LayoutKind::Fraction {
            numer: Box::new(numer),
            denom: Box::new(denom),
            axis_height: fs * 5.0 / 12.0,
            rule_thickness: fs * 0.04,
            rule_start: fs * 0.125,
            rule_end: width - fs * 0.125,
        },
    }
}

pub(super) fn parentheses(left: &str, right: &str, mut body: LayoutBox, fs: f64) -> LayoutBox {
    let scale = if body.height > fs * 1.2 { 1.421 } else { 1.0 };
    let glyph_size = fs * scale;
    let slot = glyph_size * if left == "[" { 0.387 } else { 0.375 };
    let (height, baseline) = (body.height, body.baseline);
    let mut opening = font_glyph(
        left.to_owned(),
        glyph_size,
        slot,
        height,
        baseline + fs * 0.15,
    );
    opening.x = if scale != 1.0 { fs / 9.0 } else { 0.0 };
    body.x = if scale != 1.0 { fs * 0.6 } else { slot };
    let width = body.width + slot * 2.0;
    let mut closing = font_glyph(
        right.to_owned(),
        glyph_size,
        slot,
        height,
        baseline + fs * 0.15,
    );
    closing.x = body.x + body.width;
    if scale != 1.0 {
        // 독립 Print의 원시 Tf/Tz: 12pt의 괄호는 Tf=203, Tz=49%다.
        // 추출기의 span size(기하평균 1.421em)를 균일 확대 비율로 쓰지 않는다.
        for glyph in [&mut opening, &mut closing] {
            glyph.font_size = fs * 2.03;
            glyph.width = glyph.font_size * 0.49 * if left == "[" { 0.387 } else { 0.375 };
            if let LayoutKind::FontGlyph { paint, .. } = &mut glyph.kind {
                *paint = Some(FontGlyphPaint {
                    scale_x: 0.49,
                    stroke_width: 0.0,
                });
            }
        }
    }
    row(fs, width, height, baseline, vec![opening, body, closing])
}
