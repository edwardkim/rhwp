//! [#5712] 음수 vertOffset 어울림 중첩 표와 자리차지(TAC) 중첩 표가 같은 문단에
//! co-anchored 로 놓일 때 완전히 포개지던 회귀 가드.
//!
//! 3184241(병역판정신체검사장비 보유 현황, 별지 서식) 1쪽 실측 — 수정 전:
//! 비-TAC 표 A(y 351.9~629.2)와 TAC 표 B(y 369.0~612.7)가 완전 포개져 글자를
//! 읽을 수 없었다. 근인: 비-TAC 표는 para_y 커서를 전진시키는데 TAC 분기는
//! para_y_before_compose 기준 앵커를 써 전진분을 물려받지 못한다. 판별자는
//! "저장 줄이 서로 **부분 겹침**인 co-anchored 쌍" 한정 — 완전 동일 vpos 는
//! #3820 p144 의 가로 overlay 계약(같은 vpos·horzOffset 만 다른 표는 가로 배치
//! 유지)이라 제외한다. 수정 후 B 는 A 아래(631.1~874.8)로 순차 적층된다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5712/3184241_medical_exam_equipment.hwpx";

fn nested_tables(node: &RenderNode, depth: usize, acc: &mut Vec<(f64, f64)>) {
    let next = if matches!(node.node_type, RenderNodeType::Table(_)) {
        if depth == 1 {
            acc.push((node.bbox.y, node.bbox.y + node.bbox.height));
        }
        depth + 1
    } else {
        depth
    };
    for child in &node.children {
        nested_tables(child, next, acc);
    }
}

#[test]
fn issue_5712_coanchored_tables_stack_instead_of_overlapping() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5712 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5712 fixture");

    assert_eq!(core.page_count(), 1, "3184241 은 1쪽 서식이다");

    let page = core.build_page_render_tree(0).expect("render p1");
    let mut tables = Vec::new();
    nested_tables(&page.root, 0, &mut tables);
    assert_eq!(
        tables.len(),
        2,
        "p1 에 중첩 표 2개가 있어야 함; got {tables:?}"
    );

    tables.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let (a_top, a_bottom) = tables[0];
    let (b_top, b_bottom) = tables[1];
    // 수정 전엔 B(369.0~612.7)가 A(351.9~629.2) 안에 통째로 들어가 있었다.
    assert!(
        b_top + 0.5 >= a_bottom,
        "두 중첩 표는 세로로 적층되어야 함 — A {a_top:.1}~{a_bottom:.1} 뒤에 \
         B {b_top:.1}~{b_bottom:.1} (수정 전엔 완전 포개짐)"
    );
}
