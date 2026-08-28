//! 회귀 검증: 보기 > 투명선 켠 상태에서 셀 병합으로 사라진 내부 경계에
//! 더 이상 존재하지 않는 안내선이 남아있으면 안 된다.
//!
//! `render_transparent_borders`는 `h_edges`/`v_edges` 그리드에서 `None`인 위치에
//! 점선 안내선을 그린다. 병합된 셀 내부의 옛 경계 위치도 `collect_cell_borders`가
//! 다시 방문하지 않으므로 `None`으로 남는데, 이는 "테두리 없음으로 편집됨"과
//! 구분되지 않아 병합 후에도 안내선이 그려졌다. `mark_cell_span_interior_covered`가
//! 이 두 경우를 구분해 병합된 span 내부를 커버리지로 표시한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn table_for_paragraph(node: &RenderNode, para_index: usize) -> Option<&RenderNode> {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table) if table.para_index == Some(para_index)
    ) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table_for_paragraph(child, para_index))
}

/// 편집 전용(투명선 안내선 등)으로 표시된 Line 노드 개수.
fn count_editor_only_lines(node: &RenderNode) -> usize {
    let mut n = if node.editor_only && matches!(node.node_type, RenderNodeType::Line(_)) {
        1
    } else {
        0
    };
    n += node
        .children
        .iter()
        .map(count_editor_only_lines)
        .sum::<usize>();
    n
}

#[test]
fn issue_6301_merged_cell_interior_has_no_transparent_border_guide() {
    let mut doc = HwpDocument::create_empty();

    let create_json = doc.create_table(0, 0, 0, 1, 3).expect("1x3 표 생성");
    let v: serde_json::Value = serde_json::from_str(&create_json).expect("표 생성 JSON 파싱");
    let para_idx = v["paraIdx"].as_u64().expect("paraIdx") as u32;
    let ctrl_idx = v["controlIdx"].as_u64().expect("controlIdx") as u32;

    doc.set_show_transparent_borders(true);

    // 병합 전: 기본 실선 테두리 표라 편집 전용 안내선이 없어야 한다 (기준선).
    let before_tree = doc
        .build_page_render_tree(0)
        .expect("병합 전 페이지 렌더 트리");
    let before_table =
        table_for_paragraph(&before_tree.root, para_idx as usize).expect("표 노드(병합 전)");
    let before_count = count_editor_only_lines(before_table);
    assert_eq!(
        before_count, 0,
        "병합 전 기본 실선 표에 편집 전용 안내선이 있으면 테스트 전제가 틀림"
    );

    // (0,0)~(0,1) 셀 병합: 옛 col0/col1 경계가 병합 셀 내부로 사라진다.
    doc.merge_table_cells(0, para_idx, ctrl_idx, 0, 0, 0, 1)
        .expect("셀 병합");

    let after_tree = doc
        .build_page_render_tree(0)
        .expect("병합 후 페이지 렌더 트리");
    let after_table =
        table_for_paragraph(&after_tree.root, para_idx as usize).expect("표 노드(병합 후)");
    let after_count = count_editor_only_lines(after_table);
    assert_eq!(
        after_count, 0,
        "병합으로 사라진 셀 내부 경계에 투명선 안내선이 남아있음 (회귀)"
    );
}
