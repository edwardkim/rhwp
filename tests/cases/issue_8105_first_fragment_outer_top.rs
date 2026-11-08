//! [#8105] 쪽 위에서 시작하는 문단 기준 RowBreak 표의 첫 조각은 바깥 위 여백을 연다.
//!
//! 표 위끝 = 문단 윗변(본문 위) + 세로 오프셋 + 바깥 위 여백. 예산은 첫 조각 host_before 로 이
//! 여백을 예약하는데, 그리기는 그 밖의 앵커(글자 없는 앵커의 첫 표)에서 여백을 열지 않았다.
//! 1480000-201900698(정본 `pdf/issue5941-1480000-201900698-2020.pdf`) 11쪽 구역 3 문단 0 셋째 개체
//! (97×4 RowBreak 표, 바깥 위 283HU, 정본 위 괘선 134px): 수정 전 rhwp 는 여백 없이 3.8px 높게 그려 쪽 전체가 어긋났다
//! (Visual Sweep 46.2 → 100.0). 반례: 칸 단위로 나누는(CellBreak) 표는 이 원점에 여백을 넣지 않는다
//! (같은 문서 156쪽 정본 = 여백 없는 원점).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5941/1480000-201900698-native-neartop-reset.hwp";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("parse 1480000-201900698")
}

/// `(본문 위, 표 (문단, 개체) 위)` — 쪽의 본문 영역과 지정한 표 노드의 윗변.
fn body_and_table_top(core: &DocumentCore, page: u32, table: (usize, usize)) -> (f64, f64) {
    fn find<'a>(
        node: &'a RenderNode,
        pred: &dyn Fn(&RenderNode) -> bool,
    ) -> Option<&'a RenderNode> {
        if pred(node) {
            return Some(node);
        }
        node.children.iter().find_map(|child| find(child, pred))
    }
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let body = find(&tree.root, &|node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .expect("본문 영역");
    let table_node = find(&tree.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(table.0) && t.control_index == Some(table.1))
    })
    .unwrap_or_else(|| panic!("{}쪽 표 {table:?}", page + 1));
    (body.bbox.y, table_node.bbox.y)
}

#[test]
fn page_top_rowbreak_first_fragment_opens_the_outer_top() {
    let core = core();
    let Control::Table(table) = &core.document().sections[3].paragraphs[0].controls[2] else {
        panic!("구역 3 문단 0 셋째 개체는 표");
    };
    assert_eq!(table.outer_margin_top, 283, "바깥 위 여백 283HU");
    let (body, top) = body_and_table_top(&core, 10, (0, 2));
    // 정본 11쪽 표 위 괘선 134px(96dpi 래스터) − 본문 위 86.9px = 47.1px. 바깥 위 여백(3.77px)을
    // 빼면 수정 전 rhwp 의 원점(43.9px)과 같다 — 차이는 여백 하나다.
    let gap = top - body;
    let outer = table.outer_margin_top as f64 / 75.0;
    assert!(
        (gap - 47.7).abs() <= 0.8,
        "11쪽 표 위 − 본문 위 = 정본 47.1–47.7px(여백 {outer:.2}px 포함), 실제 {gap:.2}"
    );
}
