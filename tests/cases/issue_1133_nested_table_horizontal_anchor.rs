//! 문단 기준의 자리차지 중첩 표는 선언한 왼쪽 앵커를 유지한다.
//! 같은 입력의 한컴 PDF2쪽 회색 표와 이어지는 글줄을 독립 기준으로 사용한다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn find_node<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, predicate))
}

fn assert_support_table_anchor(sample: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 파일")).expect("원본 문서 열기");
    assert_eq!(core.page_count(), 3, "원본의 쪽 소유 유지");
    let page = core.build_page_render_tree(1).expect("2쪽 실제 배치");
    let outer = find_node(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(29))
    })
    .expect("전형절차 바깥 표");
    let cell = outer
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 1 && cell.col == 1)
        })
        .expect("지원서 접수 설명 셀");
    let nested = find_node(cell, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(_))
    })
    .expect("회색 안내문 중첩 표");

    // 두 형식의 독립 PDF에서 같은 채움 상자 x190.457/폭469.508px(96dpi)이다.
    assert!(
        (nested.bbox.x - 190.457_336_425_781_25).abs() <= 0.5,
        "{sample}: 문단 왼쪽 앵커 대신 가운데 배치됨: x={}",
        nested.bbox.x
    );
    assert!(
        (nested.bbox.width - 469.508).abs() <= 0.5,
        "{sample}: 가로 원점을 고치며 선언 폭을 바꾸면 안 됨: width={}",
        nested.bbox.width
    );
    let continuation = find_node(nested, &|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == "합격 또는 채용이 취소됨")
    })
    .expect("안내문의 이어지는 글줄");
    // 독립 PDF의 같은 글줄 시작234.273px이며, 표와 함께 한 번만 이동해야 한다.
    assert!(
        (continuation.bbox.x - 234.273_030_598_958_34).abs() <= 0.5,
        "{sample}: 이어지는 글줄의 최종 시작 위치가 다름: x={}",
        continuation.bbox.x
    );
}

#[test]
fn issue_1133_native_support_table_keeps_para_left_anchor() {
    assert_support_table_anchor("samples/issue_1133.hwp");
}

#[test]
fn issue_1133_hwpx_support_table_keeps_para_left_anchor() {
    assert_support_table_anchor("samples/hwpx/issue_1133.hwpx");
}

#[test]
fn issue_6787_para_anchors_include_group_outer_margin() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6787/16774617-electronic-ballot-form.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("원본 투표용지"))
        .expect("투표용지 열기");
    let page = core.build_page_render_tree(0).expect("투표용지 1쪽");
    fn collect(node: &RenderNode, width: f64, xs: &mut Vec<f64>) {
        if matches!(node.node_type, RenderNodeType::Table(_))
            && (node.bbox.width - width).abs() < 0.6
        {
            xs.push(node.bbox.x);
        }
        for child in &node.children {
            collect(child, width, xs);
        }
    }
    // 독립 한컴 PDF의 카드·사진 칸·버튼 왼쪽 괘선(96dpi)이다.
    // 같은 줄 여부만으로 가로 원점과 자식 표 위치를 입증하지 않는다.
    for (width, expected) in [
        (253.213, vec![122.813, 418.655]),
        (96.227, vec![199.573, 497.492]),
        (196.627, vec![312.472]),
    ] {
        let mut actual = Vec::new();
        collect(&page.root, width, &mut actual);
        actual.sort_by(f64::total_cmp);
        assert_eq!(actual.len(), expected.len(), "표 폭{width}의 소유 개수");
        for (x, oracle) in actual.iter().zip(expected) {
            assert!(
                (x - oracle).abs() <= 0.6,
                "표 폭{width}: 최종 왼쪽 괘선{x}, 독립 PDF{oracle}"
            );
        }
    }
}
