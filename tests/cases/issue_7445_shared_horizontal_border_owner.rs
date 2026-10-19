//! 동일한 수평선 충돌의 색상은 아래 셀의 상단이 소유한다.
//! 현재 전66쪽 Native/fresh WASM 최저94.58132/94.60117%와 쪽수66/66을 확인했다.
//! 같은 경계의28쪽은 양쪽 raw100% 및 review/overlay 직접 판독 후 보호한다.
//! 독립 Print는 검은 하단 위에 흰 상단을 덧그린다. 논리 경계 한 개와 원본
//! 순서를 보존하고, Native PNG에서도 회색 가장자리를 잃지 않아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::control::Control,
    paint::{
        builder::LayerBuilder,
        layer_tree::{LayerNode, LayerNodeKind},
        paint_op::PaintOp,
        RenderProfile,
    },
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
    assert_eq!(edges.len(), 1, "논리 경계와 셀 소유는 하나다");
    edges[0]
}

fn matching_paint_strokes<'a>(node: &'a LayerNode, source: &LineNode, out: &mut Vec<&'a LineNode>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                matching_paint_strokes(child, source, out);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => matching_paint_strokes(child, source, out),
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Line { line, .. } = op {
                    // 원본 소유 노드에서 paint로 전달한 동일 기하를 찾는다.
                    // 독립 좌표 공차가 아니라 같은 값의 전달 여부다.
                    if (line.x1, line.y1, line.x2, line.y2)
                        == (source.x1, source.y1, source.x2, source.y2)
                    {
                        out.push(line);
                    }
                }
            }
        }
    }
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
            let layers = LayerBuilder::new(RenderProfile::Print).build(&tree);
            let mut strokes = Vec::new();
            matching_paint_strokes(&layers.root, white, &mut strokes);
            assert_eq!(
                strokes
                    .iter()
                    .map(|line| line.style.color)
                    .collect::<Vec<_>>(),
                [0, 0x00ffffff],
                "독립 Print의 검은 하단 → 흰 상단 순서를 실제 공통 paint가 소비"
            );
            assert!(
                strokes.iter().all(|line| line.style.compound_component),
                "겹치는 획을 각각 픽셀 중심으로 옮겨 원본 coverage를 지우지 않는다"
            );
            let mut normal = Vec::new();
            matching_paint_strokes(&layers.root, black, &mut normal);
            assert_eq!(
                normal.len(),
                1,
                "옆의 단일 검은 경계는 중복 paint하지 않는다"
            );
            assert!(!normal[0].style.compound_component);
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

#[cfg(feature = "native-skia")]
#[test]
fn shared_horizontal_overpaint_keeps_the_independent_print_ink_in_native_png() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx")).unwrap(),
    )
    .unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(include_str!(
        "../../mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json"
    )).unwrap())).unwrap();
    assert_eq!(core.page_count(), 66, "독립 Print 전체 쪽수와 내용 소속");
    let tree = core.build_page_render_tree(27).unwrap();
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let tables: Vec<_> = nodes
        .into_iter()
        .filter(|node| {
            matches!(
                &node.node_type, RenderNodeType::Table(table)
                    if table.para_index == Some(280) && table.control_index == Some(0)
            )
        })
        .collect();
    assert_eq!(tables.len(), 1, "독립 Print28쪽의 원본 표");
    let blank = cell(tables[0], 3, 1);
    let edge = top_edge(tables[0], blank);
    assert_eq!(edge.style.color, 0x00ffffff);
    let label = cell(tables[0], 3, 2);
    let mut label_nodes = Vec::new();
    walk(label, &mut label_nodes);
    let text: String = label_nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(text.split_whitespace().collect::<String>(), "판단근거");
    let png = core
        .render_page_png_native_with_profile_and_export_options(
            27,
            RenderProfile::Print,
            &rhwp::document_core::queries::rendering::PngExportOptions {
                dpi: Some(192.0),
                scale: Some(2.0),
                ..Default::default()
            },
        )
        .expect("실제 Native Print PNG");
    let pixels = image::load_from_memory(&png).unwrap().to_rgb8();
    // 독립 PDF28쪽의 짧은 선:132.089335..177.025330CSSpx, y418.582682.
    // 양 끝2CSSpx를 제외한 내부의 연속 잉크를 ±2CSSpx에서 확인한다.
    // 색 임계값232는 정본 Visual Sweep과 같고 캡처는192DPI(2배)다.
    let left = ((132.089335_f64 + 2.0) * 2.0).ceil() as u32;
    let right = ((177.025330_f64 - 2.0) * 2.0).floor() as u32;
    let top = ((418.582682_f64 - 2.0) * 2.0).floor() as u32;
    let bottom = ((418.582682_f64 + 2.0) * 2.0).ceil() as u32;
    for x in left..=right {
        assert!(
            (top..=bottom).any(|y| pixels.get_pixel(x, y).0.into_iter().any(|c| c < 232)),
            "독립 Print의 연속 회색 잉크가 Native PNG에서 사라짐: x={x}"
        );
    }
}
