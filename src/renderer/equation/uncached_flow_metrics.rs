//! 아직 저장 크기가 발행되지 않은 수식도 실제 표시 상자를 흐름에서 소비한다.

use crate::model::control::Equation;
use crate::renderer::{hwpunit_to_px, px_to_hwpunit, DEFAULT_DPI};

pub(super) fn metrics(equation: &Equation) -> Option<(u32, u32, i32)> {
    // 보호된 크기는 명시적인 표시 계약이다. 완전한 저장 캐시는 기존
    // 수용·정규화 경로를 따르고, 없는 축을 임의의 작은 상수로 채우지 않는다.
    if equation.common.size_protect
        || (equation.common.width > 0 && equation.common.height > 0)
        || equation.font_size == 0
        || equation.script.trim().is_empty()
    {
        return None;
    }
    let ast =
        super::parser::EqParser::for_document(super::document_script::tokenize(&equation.script))
            .parse();
    let layout = super::layout::EqLayout::for_equation(equation, DEFAULT_DPI).layout(&ast);
    if !layout.width.is_finite()
        || !layout.height.is_finite()
        || layout.width <= 0.0
        || layout.height <= 0.0
    {
        return None;
    }
    let font_size = hwpunit_to_px(equation.font_size as i32, DEFAULT_DPI);
    let (height, baseline) = (equation.attr & 1 == 0)
        .then(|| super::logical_font_frame::metrics(&layout, font_size))
        .flatten()
        .unwrap_or((layout.height, layout.baseline));
    Some((
        px_to_hwpunit(layout.width, DEFAULT_DPI) as u32,
        px_to_hwpunit(height, DEFAULT_DPI) as u32,
        px_to_hwpunit(baseline, DEFAULT_DPI),
    ))
}
