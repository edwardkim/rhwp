//! 한컴 수식 스크립트 파싱 및 렌더링
//!
//! 수식 스크립트(버전 6.0)를 토큰화하고 AST로 변환한 뒤 SVG로 렌더링한다.
//! 참조: openhwp/docs/hwpx/appendix-i-formula.md
//!
//! 명령 분기는 [`dispatch`] 가 가족으로 분류하고, [`parser`] 가 같은 핸들러로
//! 소비한다. 동작은 바꾸지 않는다. 모듈 지도는 `README.md`, 매뉴얼은
//! `mydocs/manual/equation_module.md`.

pub mod ast;
#[cfg(target_arch = "wasm32")]
pub mod canvas_render;
pub(crate) mod dispatch;
pub mod layout;
pub(crate) mod legacy_hwpeq;
pub mod parser;
pub mod svg_render;
pub mod symbols;
pub mod tokenizer;

/// 한글 수식은 수학 글꼴의 누락 글리프 대신 본문 명조 계열을 사용한다.
pub(crate) const CJK_EQUATION_FONT_FAMILY: &str = "Haansoft Batang, 한컴바탕, Batang, 바탕, serif";

pub(crate) fn text_has_cjk(text: &str) -> bool {
    text.chars().any(|ch| {
        matches!(ch,
        '\u{3000}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' | '\u{AC00}'..='\u{D7AF}')
    })
}

/// 기본 글자보다 낮은 저장 수식 상자는 압축된 표시 프레임이다.
/// 일반 수식 높이에는 줄 여백이 포함될 수 있으므로 기존 글꼴 비율을 유지한다.
/// 압축 프레임에서는 배치 기준선과 모든 출력 backend가 같은 세로 비율을 쓴다.
pub(crate) fn stored_vertical_scale(height: f64, intrinsic_height: f64, font_size: f64) -> f64 {
    if height > 0.0 && height < font_size && intrinsic_height > 0.0 {
        height / intrinsic_height
    } else {
        1.0
    }
}

/// 수식 스크립트와 BaseUnit에서 레이아웃이 소비할 intrinsic HWPUNIT 크기를 계산한다.
pub fn intrinsic_size_hwp(script: &str, font_size: u32) -> (u32, u32) {
    let font_size_px = super::hwpunit_to_px(font_size.max(1) as i32, super::DEFAULT_DPI);
    let tokens = tokenizer::tokenize(script);
    let ast = parser::EqParser::new(tokens).parse();
    let layout = layout::EqLayout::new(font_size_px).layout(&ast);
    (
        super::px_to_hwpunit(layout.width, super::DEFAULT_DPI).max(1) as u32,
        super::px_to_hwpunit(layout.height, super::DEFAULT_DPI).max(1) as u32,
    )
}

/// 수학 글꼴 HYhwpEQ에는 한글 cmap이 없으므로 한글은 본문 대체 글꼴로 그린다.
/// 보호된 크기와 한글 대체가 없는 수식은 저장 상자 계약을 유지한다.
/// 대체 경로는 폭·높이·기준선을 같은 AST에서 얻어 측정과 paint가 함께 소비한다.
pub(crate) fn flow_metrics_hwp(equation: &crate::model::control::Equation) -> (u32, u32, i32) {
    if !equation.common.size_protect
        && equation.font_name.eq_ignore_ascii_case("HYhwpEQ")
        && equation
            .script
            .chars()
            .any(|ch| matches!(ch, '\u{AC00}'..='\u{D7A3}'))
    {
        let ast = parser::EqParser::new(tokenizer::tokenize(&equation.script)).parse();
        let layout = layout::EqLayout::for_equation(equation, super::DEFAULT_DPI).layout(&ast);
        (
            super::px_to_hwpunit(layout.width, super::DEFAULT_DPI).max(1) as u32,
            super::px_to_hwpunit(layout.height, super::DEFAULT_DPI).max(1) as u32,
            super::px_to_hwpunit(layout.baseline, super::DEFAULT_DPI),
        )
    } else {
        (
            equation.common.width,
            equation.common.height,
            (equation.common.height as i32)
                .saturating_mul(i32::from(equation.baseline))
                .saturating_div(100),
        )
    }
}

pub(crate) fn flow_width_hwp(equation: &crate::model::control::Equation) -> u32 {
    flow_metrics_hwp(equation).0
}

pub(crate) fn flow_height_hwp(equation: &crate::model::control::Equation) -> u32 {
    flow_metrics_hwp(equation).1
}

/// 측정한 한글 전진폭을 그대로 글자 원점에 적용한다. 글리프 자체 크기는 바꾸지 않는다.
/// CJK가 아닌 문자는 기존 추정 폭을 보존한다.
pub(crate) fn positioned_cjk_text(text: &str, font_size: f64, width: f64) -> Vec<(char, f64)> {
    let count = text
        .chars()
        .filter(|ch| text_has_cjk(&ch.to_string()))
        .count();
    let other_width: f64 = text
        .chars()
        .filter(|ch| !text_has_cjk(&ch.to_string()))
        .map(|ch| layout::estimate_text_width(&ch.to_string(), font_size, false))
        .sum();
    let advance = if count > 0 {
        (width - other_width) / count as f64
    } else {
        0.0
    };
    let mut x = 0.0;
    text.chars()
        .map(|ch| {
            let origin = x;
            x += if text_has_cjk(&ch.to_string()) {
                advance
            } else {
                layout::estimate_text_width(&ch.to_string(), font_size, false)
            };
            (ch, origin)
        })
        .collect()
}
