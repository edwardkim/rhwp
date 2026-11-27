//! 삽입·편집이 저장할 수식 개체 프레임(폭·높이·기준선).
//!
//! 한컴 2020(11.0.0.9166)과 한컴 2022(12.0.0.4637·4645)는 파일을 열 때 저장된 수식
//! 크기·기준선을 그대로 쓰고 다시 조판하지 않는다(#8927). 그래서 rhwp가 저장한 값이
//! 한컴의 조판 값과 다르면 수식이 뒤 글자와 겹치거나 글줄에서 뜬다.
//! 측정·표시가 쓰는 원본 face 조판(`EqLayout::for_equation`)의 루트 프레임을 그대로
//! 저장해, rhwp가 그리는 상자와 한컴이 여는 상자를 같은 결과로 맞춘다.

use crate::model::control::Equation;
use crate::renderer::{hwpunit_to_px, DEFAULT_DPI, HWPUNIT_PER_INCH};

/// 저장할 프레임. `baseline`이 `None`이면 기존 기준선을 유지한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProducedFrame {
    pub width: u32,
    pub height: u32,
    pub baseline: Option<i16>,
}

fn to_hwpunit(px: f64) -> u32 {
    // 저장 정수 HWPUNIT으로 반올림한다. 4375HU(58.333…px)처럼 격자 값을 px로 만든
    // 부동소수점 손실을 절삭하면 한컴 값보다 1HU 작아진다.
    (px * HWPUNIT_PER_INCH / DEFAULT_DPI).round().max(1.0) as u32
}

/// 수식 값만으로 저장 프레임을 정한다.
///
/// - 버전 문자열이 없는 HFT 수식은 기존 HFT 논리 프레임 경로를 유지한다.
/// - 원본 수식 face(HYhwpEQ·HancomEQN)이고, 그 글자 크기의 한컴 글줄 프레임이 측정돼
///   있고, 루트 높이·기준선이 한컴 대조로 확인한 구조 프레임(`hancom_frame::metrics`)이면
///   원본 face 조판의 루트 프레임과 그 기준선을 쓴다.
/// - 그 밖은 측정 근거가 없으므로 기존 추정 크기를 쓰고 기준선은 바꾸지 않는다.
///   #8927 첨부 실측에서 표준 밖 글자 크기 1,215건은 원본 face 조판의 자연 기준선이
///   한컴과 더 멀었고, 확인되지 않은 구조(`{sum} _{a} ^{b}` 등)는 자연 높이가 더 낮았다.
pub(crate) fn produced_frame(equation: &Equation) -> ProducedFrame {
    if super::uses_legacy_hft_metrics(equation) {
        let (width, height) = super::edited_size_hwp(equation);
        return ProducedFrame {
            width,
            height,
            baseline: None,
        };
    }
    let source_face =
        equation.uses_hyhwpeq_font() || equation.font_name.eq_ignore_ascii_case("HancomEQN");
    let font_size = hwpunit_to_px(equation.font_size.max(1) as i32, DEFAULT_DPI);
    if source_face
        && super::font_frame_metrics::line_at_dpi(font_size, DEFAULT_DPI).is_some()
        && !equation.script.trim().is_empty()
    {
        let ast = super::document_script::parse(equation);
        let frame = super::layout::EqLayout::for_equation(equation, DEFAULT_DPI).layout(&ast);
        if super::hancom_frame::metrics(&frame, font_size, DEFAULT_DPI).is_some()
            && frame.width.is_finite()
            && frame.height.is_finite()
            && frame.baseline.is_finite()
            && frame.width > 0.0
            && frame.height > 0.0
        {
            let percent = (frame.baseline / frame.height * 100.0)
                .round()
                .clamp(0.0, 100.0);
            return ProducedFrame {
                width: to_hwpunit(frame.width),
                height: to_hwpunit(frame.height),
                baseline: Some(percent as i16),
            };
        }
    }
    let (width, height) = super::intrinsic_size_hwp(&equation.script, equation.font_size);
    ProducedFrame {
        width,
        height,
        baseline: None,
    }
}
