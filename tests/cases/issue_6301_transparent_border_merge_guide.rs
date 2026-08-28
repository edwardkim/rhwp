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

fn create_table(doc: &mut HwpDocument, rows: u32, cols: u32) -> (u32, u32) {
    let create_json = doc.create_table(0, 0, 0, rows, cols).expect("표 생성");
    let value: serde_json::Value = serde_json::from_str(&create_json).expect("표 생성 JSON 파싱");
    (
        value["paraIdx"].as_u64().expect("paraIdx") as u32,
        value["controlIdx"].as_u64().expect("controlIdx") as u32,
    )
}

fn editor_only_line_count(doc: &HwpDocument, para_idx: u32) -> usize {
    let tree = doc.build_page_render_tree(0).expect("페이지 렌더 트리");
    let table = table_for_paragraph(&tree.root, para_idx as usize).expect("표 노드");
    count_editor_only_lines(table)
}

#[test]
fn issue_6301_merged_cell_span_interiors_have_no_transparent_border_guides() {
    let mut doc = HwpDocument::create_empty();
    let (para_idx, ctrl_idx) = create_table(&mut doc, 2, 3);

    doc.set_show_transparent_borders(true);

    // 병합 전: 기본 실선 테두리 표라 편집 전용 안내선이 없어야 한다 (기준선).
    assert_eq!(
        editor_only_line_count(&doc, para_idx),
        0,
        "병합 전 기본 실선 표에 편집 전용 안내선이 있으면 테스트 전제가 틀림"
    );

    // 가로·세로 병합 모두 각 방향의 옛 내부 경계를 제거한다.
    doc.merge_table_cells(0, para_idx, ctrl_idx, 0, 0, 0, 1)
        .expect("가로 셀 병합");
    doc.merge_table_cells(0, para_idx, ctrl_idx, 0, 2, 1, 2)
        .expect("세로 셀 병합");
    assert_eq!(
        editor_only_line_count(&doc, para_idx),
        0,
        "병합으로 사라진 셀 span 내부 경계에 투명선 안내선이 남아있음 (회귀)"
    );
}

#[test]
fn issue_6301_real_none_border_keeps_transparent_border_guide() {
    let mut doc = HwpDocument::create_empty();
    let (para_idx, ctrl_idx) = create_table(&mut doc, 1, 2);

    doc.set_show_transparent_borders(true);
    doc.set_cell_properties(
        0,
        para_idx,
        ctrl_idx,
        0,
        r##"{
            "borderLeft":{"type":1,"width":1,"color":"#000000"},
            "borderRight":{"type":0,"width":0,"color":"#000000"},
            "borderTop":{"type":1,"width":1,"color":"#000000"},
            "borderBottom":{"type":1,"width":1,"color":"#000000"}
        }"##,
    )
    .expect("셀 오른쪽 테두리를 없음으로 설정");

    assert!(
        editor_only_line_count(&doc, para_idx) > 0,
        "실제 테두리 없음 경계에도 투명선 안내선이 유지되어야 함"
    );
}
