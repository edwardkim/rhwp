//! #8060: 저장된 첫 물리 상자와 마지막 중첩 조각의 경계를 보호한다.
//!
//! 입력: samples/issue8060/71352_lpg_regulatory_impact.hwp.
//! 독립 기준: pdf/issue8060/71352_lpg_regulatory_impact-2024.pdf, 19·20·31·47·48쪽.
//! 기대 좌표는 한컴 Print의 괘선을 72pt에서 96dpi로 환산했다.
//! 원본 소유·내용으로 대상을 선택하며 포함·순서·단일 소유 검사도 유지한다.
//! 전 48쪽 Native110/fresh WASM042 최저 95.24545/95.25170%,
//! 선택 쪽은 양쪽 모두 99% 이상이며 review·standalone overlay를 직접 판독했다.
//! 입력/PDF 해시·TSV·producer와 수정 전후 실행은
//! mydocs/working/task_semanticist21_20261022_float_host_order.md에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8060/71352_lpg_regulatory_impact.hwp");
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("원본 파싱");
    core.set_hangul2024_compat(true);
    core
}

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn table(root: &RenderNode, para: usize, rows: u16, cols: u16) -> &RenderNode {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    let selected: Vec<_> = nodes
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
                if t.para_index == Some(para) && t.control_index == Some(0)
                    && t.row_count == rows && t.col_count == cols)
        })
        .collect();
    assert_eq!(selected.len(), 1, "원본 표 조각의 단일 배치");
    selected[0]
}

fn body(node: &RenderNode) -> String {
    let mut value = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        value.push_str(&body(child));
    }
    value.chars().filter(|c| !c.is_whitespace()).collect()
}

fn edges(node: &RenderNode, expected: [f64; 4]) {
    let b = node.bbox;
    for ((actual, expected), label) in [b.x, b.y, b.x + b.width, b.y + b.height]
        .into_iter()
        .zip(expected)
        .zip(["왼쪽", "위쪽", "오른쪽", "아래쪽"])
    {
        assert!(
            (actual - expected).abs() <= 2.0,
            "{label}: 실제 {actual:.6}, 독립 Print {expected:.6}, 허용 ±2px"
        );
    }
}

fn nested_owner<'a>(node: &'a RenderNode, target: &RenderNode) -> Option<&'a RenderNode> {
    for child in &node.children {
        if std::ptr::eq(child, target) && matches!(node.node_type, RenderNodeType::TableCell(_)) {
            return Some(node);
        }
        if let Some(owner) = nested_owner(child, target) {
            return Some(owner);
        }
    }
    None
}

fn contained(child: &RenderNode, parent: &RenderNode) {
    let c = child.bbox;
    let p = parent.bbox;
    assert!(c.width > 0.0 && c.height > 0.0, "빈 중첩 조각");
    assert!(
        c.x >= p.x - 0.05
            && c.y >= p.y - 0.05
            && c.x + c.width <= p.x + p.width + 0.05
            && c.y + c.height <= p.y + p.height + 0.05,
        "중첩 조각은 실제 소유 셀 안에 있어야 한다: {c:?}, {p:?}"
    );
}

#[test]
fn nested_opening_fragment_keeps_its_independent_physical_border() {
    let core = load();
    assert_eq!(core.page_count(), 48, "독립 Print 전체 쪽수");
    let first = core.build_page_render_tree(18).unwrap().root;
    let last = core.build_page_render_tree(19).unwrap().root;
    let parent = table(&first, 245, 7, 4);
    let child = table(parent, 3, 2, 3);
    // 빈 첫 물리 띠도 원본 프레임의 일부다. 최종 paint에서 다시 접지 않는다.
    edges(parent, [99.146667, 774.033386, 699.464050, 1042.700073]);
    edges(child, [271.374695, 958.152039, 687.790710, 1036.306641]);
    let owner = nested_owner(parent, child).expect("중첩 표 소유 셀");
    assert!(matches!(&owner.node_type, RenderNodeType::TableCell(c) if c.row == 5 && c.col == 3));
    contained(child, owner);
    contained(owner, parent);
    let first_text = body(child);
    let last_text = body(table(table(&last, 245, 7, 4), 3, 2, 3));
    for needle in ["제재대상", "규제사례", "및시행규칙제16"] {
        assert_eq!(
            first_text.matches(needle).count(),
            1,
            "앞 조각 내용의 단일 소유"
        );
        assert!(
            !last_text.contains(needle),
            "소비한 앞 조각 내용을 반복하지 않는다"
        );
    }
    assert!(first_text.find("제재대상") < first_text.find("및시행규칙제16"));
}

#[test]
fn nested_terminal_fragment_keeps_its_origin_and_following_body() {
    let core = load();
    let first = core.build_page_render_tree(18).unwrap().root;
    let last = core.build_page_render_tree(19).unwrap().root;
    let parent = table(&last, 245, 7, 4);
    let child = table(parent, 3, 2, 3);
    // 앞 조각의 빈 물리 공간을 논리 소비량으로 빼서 상단 패딩을 다시 넣지 않는다.
    edges(parent, [99.146667, 77.514648, 699.464050, 365.520020]);
    edges(child, [271.374695, 83.109299, 687.790710, 118.909348]);
    let owner = nested_owner(parent, child).expect("마지막 중첩 조각 소유 셀");
    assert!(matches!(&owner.node_type, RenderNodeType::TableCell(c) if c.row == 5 && c.col == 3));
    contained(child, owner);
    contained(owner, parent);
    let child_text = body(child);
    for needle in ["판매자의공급자의무", "2년1회이상정기점검실시"] {
        assert_eq!(
            child_text.matches(needle).count(),
            1,
            "이어받은 내용의 단일 소유"
        );
        assert!(!body(table(&first, 245, 7, 4)).contains(needle));
    }
    let parent_text = body(parent);
    assert!(parent_text.find("2년1회이상정기점검실시") < parent_text.find("차등화대상결정"));
    let following = table(&last, 258, 6, 3);
    assert!(
        following.bbox.y >= parent.bbox.y + parent.bbox.height,
        "후속 표와 겹치지 않는다"
    );
    assert_eq!(body(following).matches("포괄적개념정의").count(), 1);
    let page_text = body(&last);
    for needle in ["기타고려사항", "시장유인적규제설계", "일몰설정여부"] {
        assert_eq!(page_text.matches(needle).count(), 1, "후속 본문 보존");
    }
    assert!(page_text.find("차등화대상결정") < page_text.find("기타고려사항"));
    assert!(page_text.find("기타고려사항") < page_text.find("포괄적개념정의"));
}

#[test]
fn completed_nested_row_keeps_its_source_minimum_and_next_row_order() {
    let core = load();
    let root = core.build_page_render_tree(30).unwrap().root;
    let parent = table(&root, 392, 7, 4);
    let child = table(parent, 3, 2, 3);
    edges(parent, [99.146667, 245.172043, 699.464050, 802.642700]);
    let owner = nested_owner(parent, child).expect("완료 행의 중첩 표 소유 셀");
    assert!(matches!(&owner.node_type, RenderNodeType::TableCell(c) if c.row == 5 && c.col == 3));
    // 독립 Print의 이중 세로선 안쪽과 행 경계를 사용한다.
    edges(owner, [241.949341, 366.0, 699.464050, 576.169373]);
    contained(child, owner);
    contained(owner, parent);
    let next = parent
        .children
        .iter()
        .find(|node| matches!(&node.node_type, RenderNodeType::TableCell(c) if c.row == 6 && c.col == 3))
        .expect("후속 원본 행");
    assert!(next.bbox.y >= owner.bbox.y + owner.bbox.height - 0.05);
    let subject = child
        .children
        .iter()
        .find(|node| matches!(&node.node_type, RenderNodeType::TableCell(c) if c.row == 1 && c.col == 0))
        .expect("제재 대상 원본 셀");
    assert_eq!(body(subject), "정기검사범위");
    assert_eq!(body(next).matches("차등화대상결정").count(), 1);
    let page_text = body(&root);
    assert!(page_text.find("차등화대상결정") < page_text.find("기타고려사항"));
}

#[test]
fn wrapping_single_cell_keeps_child_flow_across_the_last_two_pages() {
    let core = load();
    assert_eq!(core.page_count(), 48, "원본·독립 Print의 전체 쪽수");
    let first = core.build_page_render_tree(46).unwrap().root;
    let last = core.build_page_render_tree(47).unwrap().root;
    let opening = table(&first, 631, 1, 1);
    let closing = table(&last, 631, 1, 1);
    edges(opening, [77.238670, 146.878662, 712.257324, 1032.310791]);
    edges(closing, [77.238670, 77.514648, 712.257324, 322.368011]);
    for frame in [opening, closing] {
        let child = table(frame, 0, 10, 5);
        let owner = nested_owner(frame, child).expect("래퍼의 단일 소유 셀");
        assert!(
            matches!(&owner.node_type, RenderNodeType::TableCell(c) if c.row == 0 && c.col == 0)
        );
        contained(child, owner);
        contained(owner, frame);
    }
    let opening_text = body(opening);
    let closing_text = body(closing);
    for needle in ["안전점검자의자격과인원", "가스누출검지기"] {
        assert_eq!(opening_text.matches(needle).count(), 1);
        assert!(!closing_text.contains(needle));
    }
    for needle in [
        "일산화탄소경보기설치여부",
        "저장설비가시설기준에적합한지여부",
    ] {
        assert_eq!(closing_text.matches(needle).count(), 1);
        assert!(!opening_text.contains(needle));
    }
    assert!(
        closing_text.find("일산화탄소경보기설치여부")
            < closing_text.find("저장설비가시설기준에적합한지여부")
    );
}
