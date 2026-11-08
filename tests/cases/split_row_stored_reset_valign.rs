//! 저장 쪽 재시작에서 끝나는 분할 행 조각의 세로 정렬.
//!
//! 1342000 교육과정 지도(구역 1 `pi=0`, 47×9 RowBreak) 31쪽 끝 행 24 의 경남 칸(셀 75,
//! valign Center, 선언 15662HU)은 8문단 11줄이고 p[4] 첫 줄이 저장 vpos 0 으로 다시 시작한다.
//! 한/글 2020 PDF(`existing-hwp-hancom2020-print.pdf`) 31쪽은 이 칸의 앞 6줄을 조각 안에서
//! 가운데 정렬해 첫 줄을 조각 위 9.8px 에 둔다(위 정렬이면 위 패딩 3.8px 자리). 문단 안 재시작
//! (`row_plain_restart_capacity_cut`)만 원본 조각으로 인정하던 수정 전 rhwp 는 문단 경계 재시작
//! 칸을 위 정렬했다.
//!
//! 반례: rowbreak-problem-pages 13쪽 1×1 표(구역 1 `pi=13`, valign Center)도 저장 쪽 재시작에서
//! 끝난다. 이 칸 조각은 수정 전에도 위 정렬보다 17.9px 아래에 그려지고 한/글 2024 PDF 와 맞는다
//! (첫 글줄 잉크 rhwp 497 / PDF 495). 같은 정렬을 한 번 더 얹으면 내용이 조각 아래로 밀린다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE_1342000: &str =
    "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp";
const SAMPLE_ROWBREAK_PAGES: &str = "samples/rowbreak-problem-pages.hwp";

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

/// 쪽 `page`(0-기준) 표 칸 (row, col) 의 상자와 그 안 글줄들의 (위, 아래).
fn cell_lines(core: &DocumentCore, page: u32, row: u16, col: u16) -> ((f64, f64), Vec<(f64, f64)>) {
    fn lines(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            out.push((node.bbox.y, node.bbox.y + node.bbox.height));
        }
        for child in &node.children {
            lines(child, out);
        }
    }
    fn find<'a>(node: &'a RenderNode, row: u16, col: u16, out: &mut Vec<&'a RenderNode>) {
        if let RenderNodeType::TableCell(cell) = &node.node_type {
            if cell.row == row && cell.col == col && cell.model_cell_index.is_some() {
                out.push(node);
            }
        }
        for child in &node.children {
            find(child, row, col, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut cells = Vec::new();
    find(&tree.root, row, col, &mut cells);
    // 같은 칸 좌표의 표가 여럿이면 쪽 끝에서 나뉘는 가장 아래 표의 칸이다.
    let cell = cells
        .into_iter()
        .max_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
        .unwrap_or_else(|| panic!("{}쪽 칸 ({row},{col})", page + 1));
    let mut out = Vec::new();
    lines(cell, &mut out);
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    ((cell.bbox.y, cell.bbox.y + cell.bbox.height), out)
}

/// 같은 문서에서 칸 하나만 위 정렬로 바꾼 대조본. 쪽 나눔·행 높이는 그대로이고 칸 정렬만 다르다.
fn top_aligned(
    sample: &str,
    section: usize,
    para: usize,
    control: usize,
    cell: usize,
) -> DocumentCore {
    let mut core = core(sample);
    let mut doc = core.document().clone();
    match &mut doc.sections[section].paragraphs[para].controls[control] {
        rhwp::model::control::Control::Table(table) => {
            table.cells[cell].vertical_align = rhwp::model::table::VerticalAlign::Top;
        }
        _ => panic!("{sample} 구역 {section} 문단 {para} 개체 {control} 는 표가 아니다"),
    }
    core.set_document(doc);
    core
}

#[test]
fn stored_paragraph_reset_fragment_keeps_cell_center_alignment() {
    let source = core(SAMPLE_1342000);
    let ((top, bottom), lines) = cell_lines(&source, 30, 24, 3);
    let (_, top_lines) = cell_lines(&top_aligned(SAMPLE_1342000, 1, 0, 2, 75), 30, 24, 3);
    assert_eq!(lines.len(), 6, "31쪽 경남 칸은 저장 재시작 앞 6줄을 그린다");
    assert_eq!(top_lines.len(), 6);
    let lowered = lines[0].0 - top_lines[0].0;
    assert!(
        lowered > 3.0,
        "가운데 정렬 칸은 조각 안 남는 높이의 절반만큼 위 정렬보다 아래에 놓인다: {lowered:.2}px"
    );
    assert!(
        top <= lines[0].0 && lines[5].1 <= bottom,
        "내용은 조각 안에 있다"
    );
}

#[test]
fn single_cell_stored_reset_fragment_is_aligned_once() {
    let source = core(SAMPLE_ROWBREAK_PAGES);
    let (_, lines) = cell_lines(&source, 12, 0, 0);
    let ((_, top_bottom), top_lines) =
        cell_lines(&top_aligned(SAMPLE_ROWBREAK_PAGES, 1, 13, 0, 0), 12, 0, 0);
    assert_eq!(lines.len(), top_lines.len());
    let pad = 141.0 / 75.0;
    let lowered = lines[0].0 - top_lines[0].0;
    let slack = top_bottom - pad - top_lines.last().expect("끝 줄").1;
    assert!(
        lowered <= slack + 0.5,
        "1×1 Center 칸 조각은 한 번만 정렬한다: 내려간 {lowered:.2}px, 남는 높이 {slack:.2}px"
    );
}
