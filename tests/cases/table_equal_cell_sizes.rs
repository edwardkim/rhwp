//! 고정형(한컴 저장본 대조): 셀 블록의 너비·높이를 같게 맞추면 한컴처럼 격자를 다시 나눈다.
//!
//! 측정일 2026-10-30. 원본 `tests/fixtures/table_equal_cell_sizes/original.hwp`(구역 0 문단 25의
//! 14행×6열 표, 머리 칸 병합)에 한컴 한글 12.30(Mac)이 기능을 적용해 저장한 두 문서가 기대값이다.
//! - 셀 높이를 같게: 블록 = 행 0~13 × 열 3~5. 한컴은 14행을 27행으로 나누고 블록 칸을 모두
//!   1500HU로 둔다. 블록 밖 칸은 선언 높이(282HU 등)를 그대로 두고 두 행씩 걸친다. 기대값
//!   `tests/fixtures/rowspan_staggered_heights/equal-height-partial-block.hwp`, 인쇄
//!   `pdf/rowspan_staggered_heights/equal-height-partial-block-hancom-mac-12.30.pdf`(96dpi에서
//!   적용 전후 표 높이 모두 280px, 블록 칸 20px 간격).
//! - 셀 너비를 같게: 블록 = 행 2~5 × 열 2~4(6855·7495·6646HU). 한컴은 6열을 8열로 나누고 블록
//!   칸을 모두 평균 6999HU로 둔다. 다른 행의 칸은 제 너비를 지키고 블록 끝 뒤 경계만 반올림
//!   차이 1HU를 함께 민다. 기대값 `tests/fixtures/table_equal_cell_sizes/equal-width-partial-block.hwp`,
//!   인쇄 `pdf/table_equal_cell_sizes/equal-width-partial-block-hancom-mac-12.30.pdf`.
//!
//! 높이의 블록 길이는 화면에 그린 행 높이(내용 높이 18.4px 행이 대부분)의 합 21000HU이고,
//! 한컴의 1500HU는 이 길이를 열마다 놓인 칸 수 14로 나눈 값이다. 칸 선언 높이의 합이나 칸
//! 화면 높이의 평균(약 1506HU)과는 다르다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::{CellSizeAxis, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const ORIGINAL: &str = "tests/fixtures/table_equal_cell_sizes/original.hwp";
const HANCOM_HEIGHT: &str =
    "tests/fixtures/rowspan_staggered_heights/equal-height-partial-block.hwp";
const HANCOM_WIDTH: &str = "tests/fixtures/table_equal_cell_sizes/equal-width-partial-block.hwp";
const PARA: usize = 25;

fn open(path: &str) -> DocumentCore {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("read");
    DocumentCore::from_bytes(&bytes).expect("open")
}

fn table(core: &DocumentCore) -> &Table {
    match &core.document().sections[0].paragraphs[PARA].controls[0] {
        Control::Table(table) => table,
        _ => panic!("문단 {PARA}의 표"),
    }
}

/// 행·열·병합과 크기(1HU 오차)를 한컴 저장본과 맞춘다.
fn assert_same_grid(ours: &Table, hancom: &Table) {
    assert_eq!(
        (ours.row_count, ours.col_count, ours.cells.len()),
        (hancom.row_count, hancom.col_count, hancom.cells.len()),
        "행·열·칸 수"
    );
    for (a, b) in ours.cells.iter().zip(&hancom.cells) {
        let at = format!("r{} c{}", b.row, b.col);
        assert_eq!(
            (a.row, a.col, a.row_span, a.col_span),
            (b.row, b.col, b.row_span, b.col_span),
            "{at} 자리·병합"
        );
        assert!(
            a.width.abs_diff(b.width) <= 1,
            "{at} 너비 {} / 한컴 {}",
            a.width,
            b.width
        );
        assert!(
            a.height.abs_diff(b.height) <= 1,
            "{at} 높이 {} / 한컴 {}",
            a.height,
            b.height
        );
        assert_eq!(a.paragraphs.len(), b.paragraphs.len(), "{at} 문단 수");
        assert_eq!(a.border_fill_id, b.border_fill_id, "{at} 테두리");
    }
    assert!(
        ours.common.width.abs_diff(hancom.common.width) <= 1,
        "표 너비"
    );
    assert!(
        ours.common.height.abs_diff(hancom.common.height) <= 1,
        "표 높이"
    );
    assert_eq!(ours.row_sizes, hancom.row_sizes, "행별 칸 수");
}

fn cell_texts(table: &Table) -> Vec<String> {
    table
        .cells
        .iter()
        .map(|c| {
            c.paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect()
}

fn check(axis: CellSizeAxis, block: (u16, u16, u16, u16), expected: &str) {
    let mut core = open(ORIGINAL);
    core.equalize_table_cell_sizes_native(0, PARA, 0, block, axis)
        .expect("크기 같게");
    let hancom = open(expected);
    assert_same_grid(table(&core), table(&hancom));
    assert_eq!(
        cell_texts(table(&core)),
        cell_texts(table(&hancom)),
        "칸 내용"
    );

    // 저장한 HWP·HWPX를 다시 열어도 같은 격자다.
    let hwp = DocumentCore::from_bytes(&core.export_hwp_native().expect("hwp")).expect("reopen");
    assert_same_grid(table(&hwp), table(&hancom));
    let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().expect("hwpx")).expect("reopen");
    assert_same_grid(table(&hwpx), table(&hancom));
}

#[test]
fn equal_height_matches_hancom_regrid() {
    check(CellSizeAxis::Height, (0, 3, 13, 5), HANCOM_HEIGHT);
}

#[test]
fn equal_width_matches_hancom_regrid() {
    check(CellSizeAxis::Width, (2, 2, 5, 4), HANCOM_WIDTH);
}

fn shape(core: &DocumentCore) -> Vec<(u16, u16, u16, u16, u32, u32)> {
    table(core)
        .cells
        .iter()
        .map(|c| (c.row, c.col, c.row_span, c.col_span, c.width, c.height))
        .collect()
}

#[test]
fn uneven_or_crossing_blocks_are_refused_unchanged() {
    let mut core = open(ORIGINAL);
    let before = shape(&core);
    // 행 0의 "국외" 머리 칸은 열 3~5를 합친 칸이라 열 3~4 블록에 반만 든다.
    let crossing = core
        .equalize_table_cell_sizes_native(0, PARA, 0, (0, 3, 3, 4), CellSizeAxis::Width)
        .expect_err("걸친 병합 칸");
    assert!(crossing.to_string().contains("걸친 병합 셀"), "{crossing}");
    // 행 0~2 × 열 0~2: 열 0에는 두 칸(두 행 병합 칸 + 한 칸), 열 1에는 세 칸이다.
    // 줄마다 칸 수가 다를 때 한컴의 결과는 아직 확인하지 못해 맞추지 않는다.
    let uneven = core
        .equalize_table_cell_sizes_native(0, PARA, 0, (0, 0, 2, 2), CellSizeAxis::Height)
        .expect_err("줄마다 다른 칸 수");
    assert!(uneven.to_string().contains("셀 수가 달라"), "{uneven}");
    assert_eq!(shape(&core), before, "거부하면 표를 바꾸지 않는다");
}

fn cell_heights(node: &RenderNode, para: usize, out: &mut Vec<f64>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && table.cell_context.is_none() {
            for child in &node.children {
                if let RenderNodeType::TableCell(_) = &child.node_type {
                    out.push(child.bbox.height);
                }
            }
            return;
        }
    }
    for child in &node.children {
        cell_heights(child, para, out);
    }
}

#[test]
fn rows_of_a_reopened_table_shrink_to_the_equal_height() {
    // 불러온 HWP5를 고치면 재측정이 직전 행 배분을 행별 하한으로 쓴다. 블록이 행 전체라
    // 격자가 그대로면 이 하한이 줄인 행(6000HU → 4500HU)을 옛 높이로 되돌린다.
    let mut seed = DocumentCore::new_empty();
    seed.create_blank_document_native().expect("빈 문서");
    let created: serde_json::Value =
        serde_json::from_str(&seed.create_table_native(0, 0, 0, 2, 2).expect("표")).unwrap();
    let para = created["paraIdx"].as_u64().unwrap() as usize;
    let ctrl = created["controlIdx"].as_u64().unwrap() as usize;
    for cell in 0..4 {
        let height = if cell < 2 { 3000 } else { 6000 };
        seed.set_cell_properties_native(0, para, ctrl, cell, &format!(r#"{{"height":{height}}}"#))
            .expect("칸 높이");
    }
    let mut core = DocumentCore::from_bytes(&seed.export_hwp_native().expect("hwp")).expect("open");
    core.equalize_table_cell_sizes_native(0, para, ctrl, (0, 0, 1, 1), CellSizeAxis::Height)
        .expect("높이 같게");
    let mut heights = Vec::new();
    cell_heights(
        &core.build_page_render_tree(0).expect("쪽").root,
        para,
        &mut heights,
    );
    assert_eq!(heights.len(), 4);
    for height in heights {
        assert!((height - 4500.0 / 75.0).abs() < 0.05, "칸 높이 {height}");
    }
}
