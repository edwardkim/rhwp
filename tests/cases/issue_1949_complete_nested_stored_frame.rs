//! #1949: 통째로 배치하는 중첩 표의 저장 프레임과 내용 소유를 보호한다.
//!
//! Native176/fresh WASM049 전쪽 115+115쪽이 각각 90% 이상이고, 이 4쪽은
//! Native 99.99055% / WASM 100% 및 독립 Print 확대 판독을 완료했다.
//! 사전 증거와 source/PDF SHA는 task_semanticist21_20261024_edit_frame_coordinate_space.md.
//! 독립 PDF의 괘선을 72pt→96dpi로 환산하며 ±2px 좌표 검사와 관계 검사를 함께 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn descendants<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        descendants(child, out);
    }
}

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn inside(child: &RenderNode, parent: &RenderNode) {
    let c = child.bbox;
    let p = parent.bbox;
    assert!(c.width > 0.0 && c.height > 0.0);
    assert!(
        c.x >= p.x - 0.05
            && c.y >= p.y - 0.05
            && c.x + c.width <= p.x + p.width + 0.05
            && c.y + c.height <= p.y + p.height + 0.05,
        "원본 내용은 물리 소유 상자 안에 있어야 한다: {c:?}, {p:?}"
    );
}

fn check_complete_frame(extension: &str, compatibility_2024: bool) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "samples/issue1949_giant_cell_nested_tables_perf.{extension}"
    ));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(compatibility_2024);
    let root = core.build_page_render_tree(3).unwrap().root;
    let mut nodes = Vec::new();
    descendants(&root, &mut nodes);
    let selected: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
                if table.para_index == Some(81) && table.control_index == Some(0)
                    && table.row_count == 3 && table.col_count == 3)
        })
        .collect();
    assert_eq!(selected.len(), 1, "원본 문단·컨트롤이 소유하는 표 하나");
    let table = selected[0];
    let b = table.bbox;
    // 동일 입력의 독립 한컴 2024/2020 Print p4 벡터 경계다.
    // 이전 Native175는 마지막 행의 빈 띠를 회수하지 않아 아래 경계가 이 범위를 벗어났다.
    for (actual, expected) in [b.x, b.y, b.x + b.width, b.y + b.height].into_iter().zip([
        154.797_333,
        969.658_691,
        669.079_997,
        1_071.308_024,
    ]) {
        assert!(
            (actual - expected).abs() <= 2.0,
            "{extension}: 표 경계 {actual}, 독립 Print {expected}, 허용 ±2px"
        );
    }
    let owners: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .filter(|node| {
            let mut children = Vec::new();
            descendants(node, &mut children);
            children.into_iter().any(|child| std::ptr::eq(child, table))
        })
        .collect();
    assert_eq!(owners.len(), 1, "통째 표의 원본 바깥 셀 소유");
    inside(table, owners[0]);

    let content = text(table);
    for needle in [
        "종류가합쳐진재료",
        "안쪽역류",
        "바깥쪽역류",
        "스테인리스강-청동",
        "스테인리스강-고무",
        "스테인리스강-카프롤론",
    ] {
        assert_eq!(content.matches(needle).count(), 1, "누락·중복: {needle}");
    }
    let material_rows: Vec<_> = ["청동", "고무", "카프롤론"]
        .into_iter()
        .map(|material| {
            let mut lines = Vec::new();
            descendants(table, &mut lines);
            let lines: Vec<_> = lines
                .into_iter()
                .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
                .filter(|node| text(node).contains(material))
                .collect();
            assert_eq!(lines.len(), 1, "재료별 본문 줄 소유");
            lines[0]
        })
        .collect();
    for pair in material_rows.windows(2) {
        assert!(
            pair[0].bbox.y + pair[0].bbox.height <= pair[1].bbox.y,
            "재료 순서와 줄 비겹침"
        );
    }
    for cell in &table.children {
        if !matches!(cell.node_type, RenderNodeType::TableCell(_)) {
            continue;
        }
        let mut contents = Vec::new();
        descendants(cell, &mut contents);
        for line in contents {
            if matches!(line.node_type, RenderNodeType::TextLine(_)) && !text(line).is_empty() {
                inside(line, cell);
            }
        }
    }
}

#[test]
fn hwp_complete_nested_frame_keeps_print_edges_and_material_ownership() {
    check_complete_frame("hwp", true);
}

#[test]
fn hwpx_complete_nested_frame_keeps_print_edges_and_material_ownership() {
    check_complete_frame("hwpx", false);
}
