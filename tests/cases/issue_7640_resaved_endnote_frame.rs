//! 검사 유형: 관계형. 독립 한컴 Print가 같은 원본·재저장본의 수식 단 소속을 검사한다.
//! 작성 시 Native 전 21쪽 중 7쪽 미달·11쪽 53.60031%. 보정 후보의 11쪽은 99.66679%다.
//! 전쪽 중 18·19·20쪽 미달이 남고 fresh WASM은 아직 미실행이므로 절대 px 핀을 두지 않는다.
//! 주석 갱신: 2026-11-05 23:14:42 KST.
//! 근거: mydocs/plans/task_m100_7640_plan.md 및 같은 입력의 한컴 2020 Print 11쪽.
//! 수식 프레임의 저장 캐시 차이로 앞 단에 들어갈 마지막 수식이 다음 단으로 밀리면 실패한다.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn equation_columns(node: &RenderNode, column: Option<u16>, out: &mut Vec<u16>) {
    let column = match node.node_type {
        RenderNodeType::Column(index) => Some(index),
        _ => column,
    };
    if let RenderNodeType::Equation(equation) = &node.node_type {
        if equation.note_ref.as_ref().is_some_and(|source| {
            source.kind == "endnote"
                && source.section_index == 0
                && source.para_index == 99
                && source.control_index == 0
                && source.note_para_index == 8
        }) {
            out.push(column.expect("미주 수식의 실제 단 소속"));
        }
    }
    for child in &node.children {
        equation_columns(child, column, out);
    }
}

#[test]
fn original_and_resaved_last_integral_remain_in_the_first_column() {
    for input in [
        "samples/3-11월_실전_통합_2022.hwp",
        "tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp",
    ] {
        let mut document = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
        document.set_hangul2024_compat(false);
        assert_eq!(document.page_count(), 21, "독립 Print의 전체 쪽수: {input}");
        let page = document.build_page_render_tree(10).unwrap();
        let mut columns = Vec::new();
        equation_columns(&page.root, None, &mut columns);
        assert_eq!(
            columns,
            [0, 0],
            "한컴 Print 11쪽 왼쪽 단 끝의 g(x)와 적분식이 누락·중복 없이 같은 단에 남는다: {input}"
        );
    }
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn host_height(input: &str, protected: bool, fraction: bool) -> f64 {
    let mut document = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    let mut source = document.document().clone();
    let Control::Endnote(note) = &mut source.sections[0].paragraphs[88].controls[0] else {
        panic!("원본 미주 11의 소유");
    };
    let Control::Equation(equation) = &mut note.paragraphs[2].controls[0] else {
        panic!("코사인 법칙 앞의 원본 수식");
    };
    equation.common.size_protect = protected;
    if fraction {
        // 별도 복합 수식의 선언 프레임을 단순 글자 줄의 논리 프레임으로 바꾸지 않는다.
        equation.script = "1 over 2".into();
        equation.common.height = equation.font_size;
        equation.baseline = 86;
    }
    document.set_document(source);
    document.set_hangul2024_compat(false);
    let page = document.build_page_render_tree(10).unwrap();
    let mut all = Vec::new();
    walk(&page.root, &mut all);
    let owners: Vec<_> = all
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Equation(equation)
                if equation.note_ref.as_ref().is_some_and(|source| {
                    source.kind == "endnote"
                        && source.para_index == 88
                        && source.note_para_index == 2
                }) =>
            {
                equation.para_index
            }
            _ => None,
        })
        .collect();
    assert_eq!(owners.len(), 1, "실제 수식의 소유가 유일해야 한다");
    let rows: Vec<_> = all.iter().filter(|node| matches!(&node.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(owners[0]) && line.line_index == Some(0)))
        .collect();
    assert_eq!(rows.len(), 1, "실제 호스트 줄이 유일해야 한다");
    rows[0].bbox.height
}

#[test]
fn protected_and_complex_frames_keep_their_separate_stored_row() {
    let original = "samples/3-11월_실전_통합_2022.hwp";
    let resaved = "tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp";
    let normal = host_height(original, false, false);
    assert_eq!(
        normal,
        host_height(resaved, false, false),
        "같은 현재 수식의 줄 점유"
    );
    let protected = host_height(original, true, false);
    assert!(
        protected > normal,
        "보호된 저장 예약 높이는 자연 글자 줄보다 크다"
    );
    assert_eq!(protected, host_height(resaved, true, false));
    assert_eq!(
        protected,
        host_height(resaved, false, true),
        "복합 수식의 저장 줄 예약 유지"
    );
}
