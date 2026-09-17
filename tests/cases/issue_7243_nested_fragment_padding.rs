//! 한컴 2024 정상 86712의 p26: 이어받은 중첩 표·빈 Enter·부모 여백의 점유 높이.
//! 기준은 tracked HWP/HWPX의 각 직접 변환 PDF와 원본 저장 메트릭이다.
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn table(node: &RenderNode, pi: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(pi)) {
        return Some(node);
    }
    node.children.iter().find_map(|c| table(c, pi))
}
fn body_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y + node.bbox.height);
    }
    node.children.iter().find_map(body_bottom)
}

fn check(path: &str) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    let doc = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(doc.page_count(), 64);
    let page = doc.build_page_render_tree(25).unwrap();
    let first = table(&page.root, 161).expect("continued cost table");
    // PDF 상/하단 선 77.515/145.760px => 68.245px.
    // source: (100+1800+1200) + child padding 282 + Enter 1300 + parent padding 446 = 5128 HU.
    let expected = 5128.0 / 75.0;
    assert!(
        (first.bbox.height - expected).abs() < 0.5,
        "{path}: source/PDF continuation height {expected:.3}, actual {:.3}",
        first.bbox.height
    );
    let next = table(&page.root, 172).expect("benefit table");
    // 다음 표로 전진하는 거리. PDF y=193.868, 첫 표 y=77.515.
    assert!(
        (next.bbox.y - first.bbox.y - (193.868 - 77.515)).abs() < 0.6,
        "{path}: next table relative y {}",
        next.bbox.y - first.bbox.y
    );
    // p28의 자식 끝 빈 줄은 보이지 않아도 이미 공간을 예약한다.
    // 그 높이를 자식 물리 상자 차이로 다시 더하면 뒤 표가 본문 바닥을 넘는다.
    let continuation = doc.build_page_render_tree(27).unwrap();
    let body_end = body_bottom(&continuation.root).expect("body");
    fn check_tables(node: &RenderNode, bottom: f64, path: &str) {
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            assert!(
                node.bbox.y + node.bbox.height <= bottom + 0.5,
                "{path}: p28 table bottom {} exceeds body {bottom}",
                node.bbox.y + node.bbox.height
            );
        }
        for child in &node.children {
            check_tables(child, bottom, path);
        }
    }
    check_tables(&continuation.root, body_end, path);
}
#[test]
fn native_continuation_reserves_nested_padding_and_host_enter() {
    check("samples/86712_regulatory_analysis.hwp");
}
#[test]
fn hwpx_continuation_reserves_nested_padding_and_host_enter() {
    check("samples/issue1891/86712_regulatory_analysis.hwpx");
}
