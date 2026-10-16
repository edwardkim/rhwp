//! 동일한 수평선 충돌의 색상은 아래 셀의 상단이 소유한다.
//! 개인정보 Native/fresh WASM 전66쪽 최저93.77937% 확인 뒤 추가했다.
//! 같은 경계의9/28/52쪽 중 양 backend 최솟값이 가장 높은28쪽
//! (Native99.90314/WASM99.90397%)에서 독립 Print의 선 끝을 ±2CSSpx로 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::control::Control,
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{LineNode, RenderNode, RenderNodeType},
    },
    DocumentCore,
};

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn cell(table: &RenderNode, row: u16, col: u16) -> &RenderNode {
    let mut nodes = Vec::new();
    walk(table, &mut nodes);
    let cells: Vec<_> = nodes
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == row && cell.col == col)
        })
        .collect();
    assert_eq!(cells.len(), 1, "원본 셀 주소의 단일 배치");
    cells[0]
}

fn top_edge<'a>(table: &'a RenderNode, cell: &RenderNode) -> &'a LineNode {
    let mut nodes = Vec::new();
    walk(table, &mut nodes);
    let middle = cell.bbox.x + cell.bbox.width / 2.0;
    let edges: Vec<_> = nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Line(line)
                if (line.y1 - cell.bbox.y).abs() < 0.1
                    && (line.y2 - line.y1).abs() < 0.1
                    && line.x1.min(line.x2) < middle
                    && line.x1.max(line.x2) > middle =>
            {
                Some(line)
            }
            _ => None,
        })
        .collect();
    assert_eq!(edges.len(), 1, "겹쳐 그리는 두 선으로 충돌을 숨기지 않는다");
    edges[0]
}

#[test]
fn shared_horizontal_edge_uses_the_lower_cell_color_in_either_cell_order() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx"))
        .expect("정식 개인정보 원본");
    for (page, paragraph) in [(9, 91), (28, 280)] {
        for reverse in [false, true] {
            let mut core = DocumentCore::from_bytes(&bytes).unwrap();
            if reverse {
                let mut document = core.document().clone();
                let Control::Table(table) =
                    &mut document.sections[0].paragraphs[paragraph].controls[0]
                else {
                    panic!("원본 규제 차등화 표");
                };
                // 셀 주소·스타일·내용은 그대로이며 저장 배열 순서만 바꾼다.
                table.cells.reverse();
                table.rebuild_grid();
                core.set_document(document);
            }
            let environment = FontEnvironment::from_json(include_str!(
                "../../mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json"
            ))
            .unwrap();
            core.set_font_environment(Some(environment)).unwrap();
            assert_eq!(core.page_count(), 66, "배열 순서로 쪽 소속을 바꾸지 않는다");
            let tree = core.build_page_render_tree(page - 1).unwrap();
            let mut nodes = Vec::new();
            walk(&tree.root, &mut nodes);
            let tables: Vec<_> = nodes
                .into_iter()
                .filter(|node| {
                    matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(paragraph) && table.control_index == Some(0))
                })
                .collect();
            assert_eq!(tables.len(), 1, "원본 표의 단일 쪽 소속");
            let table = tables[0];
            let blank = cell(table, 3, 1);
            let white = top_edge(table, blank);
            assert_eq!(white.style.color, 0x00ffffff, "독립 Print의 흰 상단");
            assert!((white.x1 - blank.bbox.x).abs() < 0.1);
            assert!((white.x2 - blank.bbox.x - blank.bbox.width).abs() < 0.1);
            let label = cell(table, 3, 2);
            let black = top_edge(table, label);
            assert_eq!(black.style.color, 0, "옆의 정상 검은 경계는 보존");
            assert!(white.x2 <= black.x1 + 0.1, "각 선의 셀 소유 범위 보존");
            if page == 28 {
                // 독립 Print /72pt →96CSSpx. 화면 캡처는192DPI.
                for (actual, expected) in [
                    (white.x1, 132.0893351236979),
                    (white.x2, 177.02532958984375),
                    (white.y1, 418.5826822916667),
                ] {
                    assert!((actual - expected).abs() <= 2.0, "독립 Print 경계 ±2CSSpx");
                }
            }
        }
    }
}
