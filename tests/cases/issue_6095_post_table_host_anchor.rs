//! #6095: 종료 호스트의 저장 좌표로 중첩 표 높이를 다시 앞에 놓지 않는다.
//! #5884와 원본 SHA가 같은 문서의 독립 Print 전2쪽·Native/fresh WASM 및
//! 확대 판독 결과를 재사용한다. 제목은1쪽, 계산식과 그림은2쪽에 이어진다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue6095/3090867_icepack_levy_criteria.hwpx";

fn words(node: &RenderNode) -> String {
    let mut text = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        text.push_str(&words(child));
    }
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn find<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find(child, predicate))
}
fn deepest_table<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
    for child in &node.children {
        if let Some(found) = deepest_table(child, needle) {
            return Some(found);
        }
    }
    (matches!(node.node_type, RenderNodeType::Table(_)) && words(node).contains(needle))
        .then_some(node)
}
#[test]
fn issue_6095_nested_host_anchor_below_table_not_double_charged() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("열기");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 2, "독립 한컴 Print 전체 쪽수");
    let p1 = core.build_page_render_tree(0).expect("앞 조각");
    let p2 = core.build_page_render_tree(1).expect("뒤 조각");
    let intro = find(&p1.root, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
            && words(node).ends_with("직선보간법을적용")
    })
    .expect("표 앞 안내 문단의 마지막 줄");
    let weights = deepest_table(&p1.root, "0.103").expect("규격별 중량 표");
    let note = find(&p1.root, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_)) && words(node).contains("1)특대형")
    })
    .expect("표 뒤 첫 주석 줄");
    let gap = weights.bbox.y - (intro.bbox.y + intro.bbox.height);
    assert!(
        gap >= 0.0 && gap < weights.bbox.height,
        "표는 안내 문단 뒤에 오고, 표 높이를 그 앞에 한 번 더 예약하지 않는다: gap={gap}, table={}",
        weights.bbox.height
    );
    assert!(
        weights.bbox.y + weights.bbox.height <= note.bbox.y,
        "중량 표와 뒤 주석은 순서대로 배치된다"
    );
    let first = words(&p1.root);
    let second = words(&p2.root);
    for title in ["예1)", "예2)"] {
        assert_eq!(
            first.matches(title).count(),
            1,
            "제목은 앞 조각에서 한 번 보존"
        );
        assert!(
            !second.contains(title),
            "제목을 뒤 계산식 조각에 중복하지 않는다"
        );
    }
    for result in ["0.066", "0.273"] {
        assert!(!first.contains(result));
        assert_eq!(
            second.matches(result).count(),
            1,
            "계산 결과는 뒤 조각에서 한 번 보존"
        );
    }
    assert!(second.contains("직선보간법적용"));
    let examples = deepest_table(&p2.root, "0.066").expect("계산식 소유 표");
    assert!(
        words(examples).contains("0.273"),
        "두 계산식은 같은 중첩 표 조각에 속한다"
    );
}
