//! 관계형·고정형: 원본 그림 줄의 상자와 셀 안 여백을 같은 행에서 소비한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 86쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 86쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 근거: mydocs/report/issue_8153_inline_picture_row_frame_20261026.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn visit<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        visit(child, nodes);
    }
}

fn text(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    visit(node, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

fn paint<'a>(node: &'a LayerNode, ops: &mut Vec<&'a PaintOp>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                paint(child, ops);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            paint(child, ops)
        }
        LayerNodeKind::Leaf { ops: leaf } => ops.extend(leaf),
    }
}

fn near(actual: f64, expected: f64, label: &str) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 2.0,
        "{label}: {actual}, 독립 기준 {expected}, 허용 ±2px"
    );
}

fn verify(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print 전체 쪽수");
    let Control::Table(outer) = &core.document().sections[2].paragraphs[599].controls[0] else {
        panic!("원본 바깥 표");
    };
    let Control::Table(original) = &outer.cells[0].paragraphs[0].controls[1] else {
        panic!("원본 중첩 표");
    };
    assert_eq!(
        (
            original.row_count,
            original.col_count,
            original.common.height
        ),
        (6, 3, 11849)
    );
    let logo = original
        .cells
        .iter()
        .find(|cell| cell.row == 1 && cell.col == 0)
        .unwrap();
    assert_eq!(logo.height, 3673);
    let padding = logo.effective_padding(&original.padding);
    assert_eq!((padding.top, padding.bottom), (140, 140));
    let pictures: Vec<_> = logo
        .paragraphs
        .iter()
        .flat_map(|para| &para.controls)
        .filter_map(|control| match control {
            Control::Picture(pic) => Some(pic),
            _ => None,
        })
        .collect();
    assert_eq!(pictures.len(), 1);
    assert!(pictures[0].common.treat_as_char && pictures[0].caption.is_none());
    assert_eq!(pictures[0].common.height, 3717);
    assert_eq!(logo.paragraphs[0].line_segs[0].line_height, 3717);

    let semantic = core.build_page_render_tree(85).unwrap();
    let mut nodes = Vec::new();
    visit(&semantic.root, &mut nodes);
    let tables: Vec<_> = nodes.iter().copied().filter(|node| {
        matches!(&node.node_type, RenderNodeType::Table(t) if t.row_count == 6 && t.col_count == 3)
            && text(node).contains("국무조정실장")
    }).collect();
    assert_eq!(tables.len(), 1, "소유 쪽·누락·중복");
    let table = tables[0];
    let cells: Vec<_> = table
        .children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell) => Some((node, cell)),
            _ => None,
        })
        .collect();
    let row = |index| {
        cells
            .iter()
            .find(|(_, cell)| cell.row == index && cell.col == 0)
            .unwrap()
            .0
    };
    for pair in [1, 2, 3, 4, 5].windows(2) {
        assert!(
            row(pair[0]).bbox.y + row(pair[0]).bbox.height <= row(pair[1]).bbox.y + 1.0 / 75.0,
            "로고 뒤 행의 순서·비겹침"
        );
    }
    assert!(text(row(3)).contains("수신"));
    assert!(text(row(4)).contains("경유"));
    let layer = core
        .build_page_layer_tree_with_profile(85, RenderProfile::Print)
        .unwrap();
    let mut ops = Vec::new();
    paint(&layer.root, &mut ops);
    for (needle, expected, owner) in [("수신", 315.359985, row(3)), ("경유", 341.599976, row(4))]
    {
        let runs: Vec<_> = ops
            .iter()
            .filter_map(|op| match op {
                PaintOp::TextRun { bbox, run, .. } if run.text.contains(needle) => {
                    Some((bbox, run))
                }
                _ => None,
            })
            .collect();
        assert_eq!(runs.len(), 1, "출력 글줄 누락·중복 금지");
        let (bounds, run) = runs[0];
        assert_eq!((run.section_index, run.para_index), (Some(2), Some(0)));
        let context = run.cell_context.as_ref().expect("원본 중첩 셀 소유 보존");
        assert_eq!(context.parent_para_index, 599);
        assert_eq!(context.path.len(), 2);
        assert_eq!(context.outermost_control(), Some(0));
        assert_eq!(context.outermost_cell(), Some(0));
        assert_eq!(context.innermost().unwrap().control_index, 1);
        let source_cell = &original.cells[context.innermost().unwrap().cell_index];
        assert_eq!(source_cell.row, if needle == "수신" { 3 } else { 4 });
        near(bounds.y + run.baseline, expected, needle);
        assert!(
            bounds.y >= owner.bbox.y
                && bounds.y + bounds.height <= owner.bbox.y + owner.bbox.height,
            "출력 글줄의 셀 내부 포함"
        );
    }
    // 원본 그림 3717HU와 활성 여백 280HU. 뒤 행에 재분배하지 않는다.
    near(row(1).bbox.height, 3997.0 / 75.0, "그림 행의 물리 상자");
    near(row(5).bbox.height, 1964.0 / 75.0, "마지막 행의 원본 상자");
    near(table.bbox.height, 11849.0 / 75.0, "원본 전체 프레임 보존");
    assert!(!text(row(5)).trim().is_empty(), "후속 제목 보존");
}

#[test]
fn hwp_inline_picture_and_padding_own_the_same_print_row() {
    verify("hwp");
}

#[test]
fn hwpx_inline_picture_and_padding_own_the_same_print_row() {
    verify("hwpx");
}
