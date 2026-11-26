//! 관계형(쪽 소속·구조), 고정형(쪽수·기준선 간격).
//! 주석 갱신: 2026-11-24 08:36 KST (Asia/Seoul).
//! 제주 관광 교육 계획 299쪽의 Native와 fresh WASM raw 실루엣은 각각 100.00%다.
//! 96 DPI, 2px 실루엣, compat 2022. 기준 PDF는 한컴 2020 Print
//! `pdf/issue8759/6500000-201700118_jeju_tourism_education_plan-2020.pdf`다.
//! 독립 PDF의 한글 분수 분자와 분모의 glyph origin 차이 15.6pt를 20.8px로 환산한다.
//! 313쪽과 299쪽 수식의 문단 소속을 유지하면서 두 기준선 사이의 간격을 ±1px로 검사한다.

use rhwp::renderer::equation::layout::LayoutKind;
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn collect<'a>(node: &'a RenderNode, equations: &mut Vec<&'a EquationNode>) {
    if let RenderNodeType::Equation(equation) = &node.node_type {
        equations.push(equation);
    }
    for child in &node.children {
        collect(child, equations);
    }
}

#[test]
fn jeju_korean_fraction_keeps_independent_print_row_pitch() {
    let path = "samples/issue8759/6500000-201700118_jeju_tourism_education_plan.hwp";
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 313);
    let page = core.build_page_render_tree(298).unwrap();
    let mut equations = Vec::new();
    collect(&page.root, &mut equations);
    let [equation] = equations.as_slice() else {
        panic!("299쪽의 수식 하나가 필요합니다");
    };
    assert_eq!(equation.section_index, Some(0));
    assert_eq!(equation.para_index, Some(3531));
    let LayoutKind::Fraction { numer, denom, .. } = &equation.layout_box.kind else {
        panic!("299쪽 수식의 분수 소속이 바뀌었습니다");
    };
    let numerator_baseline = numer.y + numer.baseline;
    let denominator_baseline = denom.y + denom.baseline;
    assert!(denominator_baseline > numerator_baseline);
    let pitch = denominator_baseline - numerator_baseline;
    assert!(
        (pitch - 20.8).abs() <= 1.0,
        "독립 인쇄 기준선 간격 20.8px, 실제 {pitch:.5}px"
    );
}
