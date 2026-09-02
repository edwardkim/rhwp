//! [Issue #5720] 행별 셀 폭 합이 다른 표를 선언보다 넓게 그려 용지 밖으로 내보낸다.
//!
//! 2734559(25행×59열 가상 격자): 행마다 선언 셀 폭 합이 고르지 않아(세로 병합에
//! 덮인 불완전 행 다수) 병합 셀 제약이 서로 모순이고, 전역 grid 의 결핍 보정
//! ("뒤쪽 열 확장")이 누적돼 표가 선언 638.7px 대신 726.9px 로 그려졌다 — 용지
//! 오른쪽 10.7px 밖. 한글 2022 COM PDF 실측: 표는 선언 폭 그대로(76.4~716.7px).
//!
//! 수정: ① 전역 grid 합이 선언 폭을 넘으면 비례 축소(부족분을 마지막 열에 채우는
//! 기존 분기와 대칭). ② 행 선언 폭 합이 표 폭과 1% 이내로 어긋나는 완결 행도
//! 자기 구획으로 인정하고 표 폭으로 정규화.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5720/2734559_mixed_column_grid.hwpx";

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn issue_5720_table_width_stays_at_declaration() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);

    let table = nodes
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::Table(_)) && n.bbox.width > 300.0)
        .expect("본문 표");
    // 선언 폭 47,904 HWPUNIT = 638.72px. 종전 726.9px(+88.2).
    assert!(
        (table.bbox.width - 638.7).abs() < 1.0,
        "표 폭은 선언 폭이어야 한다: {:.1} (결함 시 726.9)",
        table.bbox.width
    );
    // 용지(793.7px) 안 — 종전에는 오른쪽 끝 804.4px.
    let right = table.bbox.x + table.bbox.width;
    assert!(
        right < 793.7,
        "표 오른쪽이 용지 안이어야 한다: {:.1}",
        right
    );
    // 셀 좌표도 표 밖으로 나가지 않는다.
    let max_cell_right = nodes
        .iter()
        .filter(|n| matches!(&n.node_type, RenderNodeType::TableCell(_)))
        .map(|n| n.bbox.x + n.bbox.width)
        .fold(0.0f64, f64::max);
    assert!(
        max_cell_right <= right + 1.0,
        "셀이 표 밖으로 나가면 안 된다: cell right {:.1} vs table right {:.1}",
        max_cell_right,
        right
    );
}
