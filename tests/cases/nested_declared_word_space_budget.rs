//! 검사 유형: 첫 검사는 고정형(독립 Print 숫자 원점 ±2 CSSpx), 둘째 검사는 관계형(명시 예산·대조군).
//! 주석 갱신: 2026-11-09T05:42:04+09:00; 신규 파일로 직전 파일 변경 커밋 없음.
//! 2026-11-09 canonical raw 2px, 96 DPI, compat2022; 독립 한컴2020 Print의 Linux PDFium PNG를 SHA 검증 후 재사용.
//! 원본 전17쪽 Native 최저98.36%, fresh WASM 최저98.37%; 보호 표시값 하락·쪽 누락·중복 0.
//! 고정하는 3쪽은 양쪽 모두100.00%; 같은 쪽 review·overlay와 낱말·리더 원점을 확대 판독했다.
//! HWP5 표44의 공백 최소값(bit9~15)과 독립 Print를 대조하며 설치 글꼴·비공개 파일을 요구하지 않는다.
//! 근거: mydocs/pr/assets/planet6897_20261106/nested-declared-word-space-budget/validation.json.

use rhwp::{
    model::control::Control,
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

fn owned_digit_origin(use_default_budget: bool) -> f64 {
    fn collect(node: &RenderNode, positions: &mut Vec<f64>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text == "3"
                && run.section_index == Some(0)
                && run.cell_context.as_ref().is_some_and(|context| {
                    context.parent_para_index == 2
                        && context.path.len() == 2
                        && context.path[0].control_index == 1
                        && context.path[0].cell_index == 1
                        && context.path[0].cell_para_index == 3
                        && context.path[1].control_index == 0
                        && context.path[1].cell_index == 15
                        && context.path[1].cell_para_index == 0
                })
            {
                positions.push(node.bbox.x);
            }
        }
        for child in &node.children {
            collect(child, positions);
        }
    }

    let mut core = DocumentCore::from_bytes(
        &std::fs::read("samples/basic/issue2007_nested_cell_pagination_42065.hwp").unwrap(),
    )
    .unwrap();
    assert_eq!(
        (core.document().doc_info.para_shapes[25].attr1 >> 9) & 127,
        25
    );
    if use_default_budget {
        // 원본과 동일한 문단 모양을 복제하고 이 문단의 명시 공백 예산만 지운다.
        // 원본 모양을 공유하는 다른 문단의 조판은 바꾸지 않는다.
        let mut document = core.document().clone();
        let shape_index = document.doc_info.para_shapes.len();
        let mut shape = document.doc_info.para_shapes[25].clone();
        shape.attr1 &= !(127 << 9);
        document.doc_info.para_shapes.push(shape);
        let Control::Table(outer) = &mut document.sections[0].paragraphs[2].controls[1] else {
            panic!("원본 바깥 표 소유");
        };
        let Control::Table(inner) = &mut outer.cells[1].paragraphs[3].controls[0] else {
            panic!("원본 안쪽 표 소유");
        };
        inner.cells[15].paragraphs[0].para_shape_id = u16::try_from(shape_index).unwrap();
        core.set_document(document);
    }
    core.set_hangul2024_compat(false);
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"approved-print-gulim-model","substitutions":{"휴먼명조":"Gulim"},"trueTypeFaces":["Gulim"],"preserveSourceMetrics":true}"#,
    ).unwrap())).unwrap();
    let tree = core.build_page_render_tree(2).unwrap();
    let mut positions = Vec::new();
    collect(&tree.root, &mut positions);
    assert_eq!(positions.len(), 1, "원본 중첩 셀의 숫자 소유·누락·중복");
    positions[0]
}

/// 고정형: 독립 한컴 Print의 숫자 원점을 보존한다.
#[test]
fn declared_condense_keeps_the_independent_print_digit_origin() {
    let origin = owned_digit_origin(false);
    assert!(
        (origin - 563.5345865885416).abs() <= 2.0,
        "독립 한컴 Print 3쪽 제3항 숫자 원점: {origin}"
    );
}

/// 관계형: 명시 예산을 지운 대조군과 실제 원문의 배치를 구분한다.
#[test]
fn larger_declared_budget_is_not_replaced_by_the_default_limit() {
    assert!(
        owned_digit_origin(false) < owned_digit_origin(true),
        "자연 대시의 부족을 수용할 수 있는 원문의 명시 공백 예산을 사용한다"
    );
}
