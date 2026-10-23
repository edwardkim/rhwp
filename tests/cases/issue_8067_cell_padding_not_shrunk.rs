//! [#8067] 칸 위·아래 안 여백 합이 칸 선언 높이를 넘어도 한/글은 여백을 줄이지 않는다.
//!
//! 입력 `samples/issue8067/*.hwpx` 는 생성기 `gen_pad_guard.py` 의 출력이고, 정본
//! `pdf/issue8067/*-2024.pdf` 는 한/글 2024(13.0.0.564) Print 다.
//!
//! | 입력 | 칸 선언 | 유효 안 여백(위/아래) | 한/글 첫 줄 위치 |
//! |---|---|---|---|
//! | `{flow,tac}_L{1,3}` | 1000HU | 칸 여백 1500/1500HU (`hasMargin=1`) | 칸 위 +20px |
//! | `flow_L{1,3}_tablepad` | 566HU | 표 여백 368/368HU (`hasMargin=0`) | 칸 위 +4.9px |
//!
//! 수정 전 rhwp 는 #501 가드로 여백을 선언 높이의 절반까지 줄여 첫 줄을 칸 위 +3.4px /
//! +1.9px 에 두고, 한 줄 칸(`*_L1`)은 선언 높이(13.3px)로 눌렀다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const DPI: f64 = 96.0;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

/// (칸 상자, 칸 안 글줄들, 유효 위·아래 안 여백 px, 칸 안 줄 높이 합 px)
fn cell_geometry(name: &str) -> (f64, f64, Vec<(f64, f64)>, f64, f64) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8067")
        .join(format!("{name}.hwpx"));
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse");
    let table = core
        .document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|para| &para.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("표");
    let cell = &table.cells[0];
    let padding = cell.effective_padding(&table.padding);
    let root = core.build_page_render_tree(0).expect("render tree").root;
    let mut nodes = Vec::new();
    collect(&root, &mut nodes);
    let cell_node = nodes
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .expect("칸");
    let mut lines = Vec::new();
    collect(cell_node, &mut lines);
    let lines: Vec<(f64, f64)> = lines
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .map(|node| (node.bbox.y, node.bbox.y + node.bbox.height))
        .collect();
    (
        cell_node.bbox.y,
        cell_node.bbox.y + cell_node.bbox.height,
        lines,
        hu_to_px(i32::from(padding.top)),
        hu_to_px(i32::from(padding.bottom)),
    )
}

#[test]
fn oversized_vertical_padding_is_kept_like_hancom() {
    for name in [
        "flow_L1",
        "flow_L3",
        "tac_L1",
        "tac_L3",
        "flow_L1_tablepad",
        "flow_L3_tablepad",
    ] {
        let (top, bottom, lines, pad_top, pad_bottom) = cell_geometry(name);
        assert!(!lines.is_empty(), "{name}: 칸 안 글줄");
        let first_offset = lines[0].0 - top;
        assert!(
            (first_offset - pad_top).abs() <= 0.5,
            "{name}: 첫 줄은 칸 위에서 저장 위 안 여백({pad_top:.2}px)만큼 아래 — 실제 {first_offset:.2}px"
        );
        let last_bottom = lines.iter().map(|line| line.1).fold(f64::MIN, f64::max);
        assert!(
            bottom - last_bottom + 0.5 >= pad_bottom,
            "{name}: 칸 아래 괘선은 마지막 줄 아래로 아래 안 여백({pad_bottom:.2}px) 이상 — 실제 {:.2}px",
            bottom - last_bottom
        );
    }
}
