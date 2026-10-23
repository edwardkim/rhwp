//! 미계산 본문을 다시 조판해도 원본이 선언한 수식 프레임은 별도 소유자다.

use crate::model::control::{DeclaredEquationFrameSource, Equation, EQUATION_LINE_MODE_BIT};

fn frame_signature(equation: &Equation) -> [u8; 32] {
    // 색 변경·호스트 이동은 수식 내부의 선언 프레임을 바꾸지 않는다.
    // 내용·글꼴·범위·크기·기준선·보호 변경에는 이전 출처를 재사용하지 않는다.
    crate::model::raw_provenance::record_digest(&(
        &equation.script,
        &equation.font_name,
        equation.font_size,
        equation.attr,
        equation.eqedit,
        equation.common.width,
        equation.common.height,
        equation.baseline,
        equation.common.size_protect,
        equation.common.treat_as_char,
    ))
}

/// 호출자는 원본 본문의 저장 좌표가 미계산임을 확인하고 캐시를 비우기 전에 봉인한다.
pub(crate) fn retain_unpositioned_host_frame(equation: &mut Equation) {
    if equation.font_name.eq_ignore_ascii_case("HYhwpEQ")
        && equation.common.treat_as_char
        && equation.attr & EQUATION_LINE_MODE_BIT == 0
        && equation.common.width > 0
        && equation.common.width <= i32::MAX as u32
        && equation.common.height >= equation.font_size
        && equation.common.height <= i32::MAX as u32
        && equation.font_size > 0
        && !equation.script.trim().is_empty()
        && (1..=100).contains(&equation.baseline)
    {
        equation.unpositioned_host_frame_seal = Some(DeclaredEquationFrameSource {
            signature: frame_signature(equation),
            legacy_layout: true,
        });
    }
}

pub(super) fn retains_unpositioned_host_frame(equation: &Equation) -> bool {
    equation
        .unpositioned_host_frame_seal
        .as_ref()
        .is_some_and(|source| source.legacy_layout && source.signature == frame_signature(equation))
}

/// 원본 서명은 보존하고 명시적으로 선택한 목표 조판 세대만 전달한다.
pub(crate) fn set_generation(
    document: &mut crate::model::document::Document,
    hangul2024: bool,
) -> bool {
    use crate::model::control::Control;
    use crate::model::identity::walk::{walk, Node};

    let mut changed = false;
    for section in &mut document.sections {
        walk(&mut section.paragraphs, |node| {
            if let Node::Control(Control::Equation(equation)) = node {
                if let Some(source) = &mut equation.unpositioned_host_frame_seal {
                    changed |= source.legacy_layout == hangul2024;
                    source.legacy_layout = !hangul2024;
                }
            }
            Ok(())
        })
        .expect("목표 수식 세대 전달은 실패하지 않는다");
    }
    changed
}
