//! 관계형 — [#8935] 저장 줄 없는 칸의 재조판 성장분은 TAC 표 비례 축소로 다른 행이 흡수하지 않는다.
//! 주석 갱신: 2026-11-11 14:05 KST (새 파일)
//!
//! 입력: 공개 결재문서 36454139(한컴오피스 2020 저장) 원본. 1쪽 17×5 글자처럼 취급 표(sz 높이
//! 36261HU)의 모든 단일 행 칸 cellSz 높이는 2133HU 다. 마지막 열 칸들은 저장 줄(linesegarray)이
//! 없고, 별표 15자가 칸 폭에 맞춰 두 줄로 재조판된다. 그중 한 행짜리 칸은 r1·r9 두 개다.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8935/36454139-reflowed-cell-row-growth-2020.pdf`(2쪽).
//! 한/글은 r1·r9 만 두 줄 높이로 키우고 나머지 행은 선언 높이 그대로 두어 표가 선언보다 커진다.
//! - 관계형: 저장 줄 칸만 있는 행(r1·r9 밖의 단일 행 칸 행) 높이 = 선언 2133HU.
//! - 관계형: r1·r9 높이 = 두 줄 내용 + 칸 위아래 여백. 줄 메트릭은 같은 표 저장 줄 칸의 값(줄 높이
//!   1000·줄 간격 600HU), 여백은 cellMargin 142+142HU 다: 1000 + 600 + 1000 + 284 = 2884HU.
//! - 관계형: 표 높이 − 선언 표 높이 = 두 행 성장분 합.
//!
//! 반례: 같은 원본에서 별표 15자 칸을 한 글자로 줄인 변형. 재조판 성장분이 없으므로 모든
//! 단일 행 칸 행이 선언 높이이고 표 높이가 선언 표 높이와 같다.
//!
//! 측정(2026-11-11, 코드 head a8751a1d07(측정 커밋 892bcadcb9 와 트리 동일), `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`,
//! raw = `silhouette_raw_match_percent`):
//! - Native: 2/2쪽, p1 100.00 · p2 100.00, 전쪽 최저 100.00.
//! - fresh WASM: 2/2쪽, p1 100.00 · p2 100.00, 전쪽 최저 100.00.
//! - 수정 전(base 3cac26e599 코드): Native p1 63.71 · fresh WASM p1 63.18, p2 100.00.
//! - 증거: `mydocs/pr/assets/issue_8935/`(TSV 원본·review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8935/36454139-reflowed-cell-row-growth.hwpx";
const TOLERANCE_PX: f64 = 0.5;
const DECLARED_ROW_HU: i32 = 2133;
const DECLARED_TABLE_HU: i32 = 36261;
const REFLOWED_TEXT: &str = "<hp:t>***************</hp:t>";
const GROWN_ROWS: [u16; 2] = [1, 9];
/// 두 줄 재조판 칸의 행: 줄 높이 1000 + 줄 간격 600 + 줄 높이 1000 + 칸 위아래 여백 142 + 142.
const TWO_LINE_ROW_HU: i32 = 1000 + 600 + 1000 + 142 + 142;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn sample_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// 저장 줄 없는 별표 15자 칸을 한 글자로 줄여 재조판 성장이 없게 한 HWPX.
fn with_single_line_reflowed_cells() -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(sample_bytes())).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("entry");
        let name = entry.name().expect("name").to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).expect("utf-8");
            assert!(
                xml.matches(REFLOWED_TEXT).count() >= 2,
                "원본 재조판 칸 글자"
            );
            data = xml.replace(REFLOWED_TEXT, "<hp:t>*</hp:t>").into_bytes();
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        out.start_file(name, options).expect("start");
        out.write_all(&data).expect("write");
    }
    out.finish().expect("finish").into_inner()
}

fn find_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.row_count == 17 && t.col_count == 5)
    {
        return Some(node);
    }
    node.children.iter().find_map(find_table)
}

struct TableRows {
    table: BoundingBox,
    /// 행마다 그 행에서 시작하는 단일 행 칸의 높이.
    rows: Vec<f64>,
}

fn table_rows(bytes: &[u8], label: &str) -> TableRows {
    let core = DocumentCore::from_bytes(bytes).unwrap_or_else(|e| panic!("open {label}: {e:?}"));
    let page = core.build_page_render_tree(0).expect("1쪽");
    let table = find_table(&page.root).unwrap_or_else(|| panic!("{label}: 1쪽 17×5 표"));
    let mut rows = vec![f64::NAN; 17];
    for child in &table.children {
        if let RenderNodeType::TableCell(cell) = &child.node_type {
            if cell.row_span == 1 && rows[cell.row as usize].is_nan() {
                rows[cell.row as usize] = child.bbox.height;
            }
        }
    }
    assert!(
        rows.iter().all(|h| !h.is_nan()),
        "{label}: 모든 행에 단일 행 칸"
    );
    TableRows {
        table: table.bbox,
        rows,
    }
}

#[test]
fn reflowed_cell_rows_grow_without_shrinking_stored_rows() {
    let layout = table_rows(&sample_bytes(), "원본");
    let declared = hu_to_px(DECLARED_ROW_HU);
    let mut growth = 0.0;
    for (row, height) in layout.rows.iter().enumerate() {
        if GROWN_ROWS.contains(&(row as u16)) {
            let two_lines = hu_to_px(TWO_LINE_ROW_HU);
            assert!(
                (height - two_lines).abs() <= TOLERANCE_PX,
                "r{row}: 두 줄로 재조판되는 칸의 행 {height:.2}px 이 두 줄 내용+여백 {two_lines:.2}px 이어야 한다"
            );
            growth += height - declared;
        } else {
            assert!(
                (height - declared).abs() <= TOLERANCE_PX,
                "r{row}: 저장 줄 칸 행 {height:.2}px 이 선언 {declared:.2}px 과 달라졌다"
            );
        }
    }
    let table_growth = layout.table.height - hu_to_px(DECLARED_TABLE_HU);
    assert!(
        (table_growth - growth).abs() <= TOLERANCE_PX,
        "표가 선언보다 {table_growth:.2}px 커졌다 — 재조판 행 성장분 합 {growth:.2}px 이어야 한다"
    );
}

#[test]
fn table_without_reflowed_growth_keeps_declared_rows() {
    let layout = table_rows(&with_single_line_reflowed_cells(), "한 줄 재조판 칸 변형");
    let declared = hu_to_px(DECLARED_ROW_HU);
    for (row, height) in layout.rows.iter().enumerate() {
        assert!(
            (height - declared).abs() <= TOLERANCE_PX,
            "r{row}: 행 {height:.2}px 이 선언 {declared:.2}px 과 달라졌다"
        );
    }
    assert!(
        (layout.table.height - hu_to_px(DECLARED_TABLE_HU)).abs() <= TOLERANCE_PX,
        "표 높이 {:.2}px 가 선언 {:.2}px 과 달라졌다",
        layout.table.height,
        hu_to_px(DECLARED_TABLE_HU)
    );
}
