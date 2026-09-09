//! #6887 / PR #6926: 문단 상대 Square 표의 왼쪽 바깥 여백을 한 번만 반영한다.
//!
//! issue4090의 기존 공개 최소 샘플은 원본 section/header XML을 보존한다.
//! 동일 표(pi=44, ci=1)의 정렬과 여백만 바꾼 대조군으로 폰트·본문 쪽수와 분리한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{HorzAlign, HorzRelTo, TextWrap};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue4090/156492236_규제샌드박스_min.hwpx";
const PARA: usize = 44;
const CONTROL: usize = 1;
const MARGIN_HU: i16 = 1417;

fn table_box(align: HorzAlign, margin: i16) -> (f64, f64, f64, f64) {
    let bytes = std::fs::read(SAMPLE).expect("공개 최소 샘플");
    let source = DocumentCore::from_bytes(&bytes).expect("샘플 파싱");
    let mut doc = source.document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[PARA].controls[CONTROL] else {
        panic!("원본의 대상 표가 사라졌다");
    };
    assert!(!table.common.treat_as_char);
    assert_eq!(table.common.horz_rel_to, HorzRelTo::Para);
    assert_eq!(table.common.horz_align, HorzAlign::Left);
    assert_eq!(table.common.text_wrap, TextWrap::Square);
    assert_eq!(table.outer_margin_left, MARGIN_HU);
    table.common.horz_align = align;
    table.outer_margin_left = margin;

    // document_mut만 사용하면 파생 캐시가 남을 수 있다. 변형마다 새 core에 설정한다.
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    fn find(node: &RenderNode) -> Option<(f64, f64, f64, f64)> {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.section_index == Some(0)
                && table.para_index == Some(PARA)
                && table.control_index == Some(CONTROL)
                && table.cell_context.is_none()
            {
                return Some((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
        }
        node.children.iter().find_map(find)
    }
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("변형 문서 조판");
        if let Some(bbox) = find(&tree.root) {
            return bbox;
        }
    }
    panic!("변형 뒤 대상 표의 렌더 노드가 사라졌다");
}

#[test]
fn left_and_inside_apply_the_declared_margin_once() {
    let expected = f64::from(MARGIN_HU) * 96.0 / 7200.0;
    for align in [HorzAlign::Left, HorzAlign::Inside] {
        let before = table_box(align, 0);
        let after = table_box(align, MARGIN_HU);
        assert!(
            (after.0 - before.0 - expected).abs() < 0.15,
            "{align:?}: 선언 여백 {expected}px와 다른 이동: {before:?} -> {after:?}"
        );
        // x 이동 뒤 bbox 뺄셈의 f64 반올림만 허용한다. 픽셀 단위 크기 변화는 금지한다.
        assert!(
            (before.2 - after.2).abs() <= 1e-9 && (before.3 - after.3).abs() <= 1e-9,
            "여백 보정이 표 크기를 바꿨다: {before:?} -> {after:?}"
        );
    }
}

#[test]
fn center_right_and_outside_do_not_add_the_left_margin() {
    for align in [HorzAlign::Center, HorzAlign::Right, HorzAlign::Outside] {
        let before = table_box(align, 0);
        let after = table_box(align, MARGIN_HU);
        assert!(
            (after.0 - before.0).abs() < 0.01,
            "{align:?}: 비대상 정렬에 왼쪽 여백을 더했다: {before:?} -> {after:?}"
        );
    }
}

#[test]
fn negative_margin_does_not_shift_the_para_left_table() {
    let zero = table_box(HorzAlign::Left, 0);
    let negative = table_box(HorzAlign::Left, -MARGIN_HU);
    assert!(
        (negative.0 - zero.0).abs() < 0.01,
        "음수 여백이 표를 이동시켰다: {zero:?} -> {negative:?}"
    );
}
