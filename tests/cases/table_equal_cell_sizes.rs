//! 고정형(한컴 저장본 대조): 셀 블록의 너비·높이를 같게 맞추면 한컴처럼 격자를 다시 나눈다.
//!
//! 측정일 2026-10-30. 줄마다 칸 수가 다른 블록은 `uneven-*` 시험 문서와 한컴 저장본으로 따로 대조한다.
//! 원본 `tests/fixtures/table_equal_cell_sizes/original.hwp`(구역 0 문단 25의
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

/// 문단의 첫 표와 그 컨트롤 번호.
fn table_control(core: &DocumentCore, para: usize) -> (usize, &Table) {
    core.document().sections[0].paragraphs[para]
        .controls
        .iter()
        .enumerate()
        .find_map(|(i, c)| match c {
            Control::Table(table) => Some((i, table.as_ref())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("문단 {para}의 표"))
}

fn table_at(core: &DocumentCore, para: usize) -> &Table {
    table_control(core, para).1
}

fn table(core: &DocumentCore) -> &Table {
    table_at(core, PARA)
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
    check_at(ORIGINAL, PARA, axis, block, expected);
}

fn check_at(
    original: &str,
    para: usize,
    axis: CellSizeAxis,
    block: (u16, u16, u16, u16),
    expected: &str,
) {
    let mut core = open(original);
    let control = table_control(&core, para).0;
    core.equalize_table_cell_sizes_native(0, para, control, block, axis)
        .expect("크기 같게");
    let hancom = open(expected);
    let hancom = table_at(&hancom, para);
    assert_same_grid(table_at(&core, para), hancom);
    assert_eq!(
        cell_texts(table_at(&core, para)),
        cell_texts(hancom),
        "칸 내용"
    );

    // 저장한 HWP·HWPX를 다시 열어도 같은 격자다.
    let hwp = DocumentCore::from_bytes(&core.export_hwp_native().expect("hwp")).expect("reopen");
    assert_same_grid(table_at(&hwp, para), hancom);
    let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().expect("hwpx")).expect("reopen");
    assert_same_grid(table_at(&hwpx, para), hancom);
}

#[test]
fn equal_height_matches_hancom_regrid() {
    check(CellSizeAxis::Height, (0, 3, 13, 5), HANCOM_HEIGHT);
}

#[test]
fn equal_width_matches_hancom_regrid() {
    check(CellSizeAxis::Width, (2, 2, 5, 4), HANCOM_WIDTH);
}

/// 줄마다 칸 수가 다른 블록: 한컴은 줄마다 블록 길이를 그 줄의 칸 수로 나눈다.
/// 높이 시험 문서는 3행×2열(열 0의 행 0~1 병합, 행 높이 3000·4500·6000HU)이고, 한컴은
/// 4행으로 나눠 열 0의 두 칸을 6750HU, 열 1의 세 칸을 4500HU로 둔다. 표 높이 13500HU는 같다.
/// 한컴 저장본 표 아래 문단의 글자 "ㅅ으"는 저장 때 섞여 든 것이고 표와 무관하다.
#[test]
fn uneven_height_divides_each_column_by_its_own_cells() {
    check_at(
        "tests/fixtures/table_equal_cell_sizes/uneven-height.hwp",
        0,
        CellSizeAxis::Height,
        (0, 0, 2, 1),
        "tests/fixtures/table_equal_cell_sizes/uneven-height-hancom.hwp",
    );
}

/// 너비 시험 문서는 2행×3열(행 0의 열 0~1 병합, 칸 너비 6000·9000·12000HU)이고, 한컴은
/// 4열로 나눠 행 0의 두 칸을 13500HU, 행 1의 세 칸을 9000HU로 둔다. 표 너비 27000HU는 같다.
#[test]
fn uneven_width_divides_each_row_by_its_own_cells() {
    check_at(
        "tests/fixtures/table_equal_cell_sizes/uneven-width.hwp",
        0,
        CellSizeAxis::Width,
        (0, 0, 1, 2),
        "tests/fixtures/table_equal_cell_sizes/uneven-width-hancom.hwp",
    );
}

fn shape(core: &DocumentCore) -> Vec<(u16, u16, u16, u16, u32, u32)> {
    table(core)
        .cells
        .iter()
        .map(|c| (c.row, c.col, c.row_span, c.col_span, c.width, c.height))
        .collect()
}

#[test]
fn crossing_blocks_are_refused_unchanged() {
    let mut core = open(ORIGINAL);
    let before = shape(&core);
    // 행 0의 "국외" 머리 칸은 열 3~5를 합친 칸이라 열 3~4 블록에 반만 든다.
    let crossing = core
        .equalize_table_cell_sizes_native(0, PARA, 0, (0, 3, 3, 4), CellSizeAxis::Width)
        .expect_err("걸친 병합 칸");
    assert!(crossing.to_string().contains("걸친 병합 셀"), "{crossing}");
    // 행 0~1 × 열 0~2의 너비: 열 0 칸은 두 행에 걸치는데 행 0은 두 칸, 행 1은 세 칸이라
    // 그 칸의 오른쪽 경계가 행마다 다르다. 한컴의 결과를 확인하지 못해 맞추지 않는다.
    let lines = core
        .equalize_table_cell_sizes_native(0, PARA, 0, (0, 0, 1, 2), CellSizeAxis::Width)
        .expect_err("줄마다 다른 병합 칸 경계");
    assert!(lines.to_string().contains("줄마다 달라"), "{lines}");
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
