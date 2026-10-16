//! RowBreak(«나눔») 표의 0행 rowspan 묶음을 쪽 끝에서 0행 뒤로 자르는 경계.
//!
//! #1145 는 작은 rowspan 묶음을 행 중간에서 자르지 않는다. 그 규칙은 쪽 끝에 묶음의
//! 무엇도 온전히 놓이지 않을 때 맞다(#1145 표본: 잔여 15.2px < 0행 17.3px). 그런데 0행이
//! 통째로 들고, 0행에서 시작하는 rowspan 칸의 내용도 0행 안에 다 들면 0행 뒤의 컷은 어떤
//! 칸의 글자도 자르지 않는다 — 한/글 2020 PDF 는 1480000-201900698 표 9·17 의 0행만
//! 앞 쪽 끝에 두고 1·2행을 다음 쪽에서 잇는다. rhwp 는 표를 통째로 넘겨 뒤 쪽들이
//! 두 행씩 밀렸다.
//!
//! 대조군: rowspan 칸 내용이 0행을 넘는 경우, 0행이 제목 행인 경우, 묶음이 잔여에
//! 통째로 드는 경우는 기존처럼 자르지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table, TablePageBreak};
use rhwp::model::Padding;

const ROW_HEIGHTS: [u32; 3] = [3_526, 3_928, 3_402];
const WHOLE: u64 = u64::MAX;

fn line(vertical_pos: i32, line_height: i32, line_spacing: i32) -> LineSeg {
    LineSeg {
        text_start: 0,
        vertical_pos,
        line_height,
        text_height: line_height,
        baseline_distance: line_height * 85 / 100,
        line_spacing,
        segment_width: 40_000,
        ..Default::default()
    }
}

fn cell_text(text: &str, lines: usize) -> Vec<Paragraph> {
    (0..lines)
        .map(|i| Paragraph {
            text: text.to_string(),
            char_count: text.encode_utf16().count() as u32,
            char_offsets: (0..text.encode_utf16().count() as u32).collect(),
            line_segs: vec![line((i as i32) * 1_600, 1_000, 600)],
            ..Default::default()
        })
        .collect()
}

fn cell(row: u16, col: u16, row_span: u16, text: &str, lines: usize, header: bool) -> Cell {
    let height = (row..row + row_span)
        .map(|r| ROW_HEIGHTS[r as usize])
        .sum::<u32>();
    Cell {
        row,
        col,
        row_span,
        col_span: 1,
        width: 20_000,
        height,
        padding: Padding {
            left: 510,
            right: 510,
            top: 141,
            bottom: 141,
        },
        paragraphs: cell_text(text, lines),
        is_header: header,
        ..Default::default()
    }
}

/// 2열 3행 표: (0,0) 이 0~1행에 걸친 rowspan 칸, (0,1)·(1,1) 은 제목, 2행은 값.
fn table_paragraph(anchor_vpos: i32, rowspan_lines: usize, header: bool) -> Paragraph {
    let mut table = Table {
        row_count: 3,
        col_count: 2,
        row_sizes: vec![2, 1, 2],
        cells: vec![
            cell(0, 0, 2, "Name", rowspan_lines, header),
            cell(0, 1, 1, "Raw", 1, header),
            cell(1, 1, 1, "Reads", 1, false),
            cell(2, 0, 1, "값", 1, false),
            cell(2, 1, 1, "9,768,330", 1, false),
        ],
        page_break: TablePageBreak::RowBreak,
        repeat_header: header,
        ..Default::default()
    };
    table.common.width = 40_000;
    table.common.height = ROW_HEIGHTS.iter().sum();
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.common.vert_rel_to = VertRelTo::Para;
    table.rebuild_grid();

    Paragraph {
        line_segs: vec![line(anchor_vpos, 1_100, 660)],
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    }
}

/// 본문 70,018 HU. 앞 문단 뒤 표 머리까지 남는 높이를 `filler_height` 로 정한다.
fn document(filler_height: i32, rowspan_lines: usize, header: bool) -> DocumentCore {
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    let filler = Paragraph {
        text: "앞".to_string(),
        char_count: 1,
        char_offsets: vec![0],
        line_segs: vec![line(0, filler_height, 660)],
        ..Default::default()
    };
    let after = Paragraph {
        text: "뒤".to_string(),
        char_count: 1,
        char_offsets: vec![0],
        line_segs: vec![line(0, 1_100, 660)],
        ..Default::default()
    };
    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59_528,
        height: 84_188,
        margin_left: 5_669,
        margin_right: 5_669,
        margin_top: 4_535,
        margin_bottom: 3_685,
        margin_header: 3_118,
        margin_footer: 2_834,
        ..Default::default()
    };
    section.paragraphs = vec![
        filler,
        table_paragraph(filler_height + 660, rowspan_lines, header),
        after,
    ];
    doc.sections.push(section);
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    core
}

/// (kind, startRow, endRow) — 쪽 `page` 에 놓인 문단 `para_index` 의 표 항목.
/// 통째 배치(`table`)는 행 범위가 없으므로 `(0, WHOLE)` 로 적는다.
fn table_items(core: &DocumentCore, page: u32, para_index: u64) -> Vec<(String, u64, u64)> {
    core.dump_page_items_json(Some(page))
        .as_array()
        .and_then(|pages| pages.first())
        .and_then(|page| page.get("columns"))
        .and_then(|columns| columns.as_array())
        .into_iter()
        .flatten()
        .filter_map(|column| column.get("items").and_then(|items| items.as_array()))
        .flatten()
        .filter(|item| item.get("paraIndex").and_then(|v| v.as_u64()) == Some(para_index))
        .filter_map(|item| {
            let kind = item.get("kind")?.as_str()?.to_string();
            (kind == "table" || kind == "partialTable").then(|| {
                (
                    kind,
                    item.get("startRow").and_then(|v| v.as_u64()).unwrap_or(0),
                    item.get("endRow").and_then(|v| v.as_u64()).unwrap_or(WHOLE),
                )
            })
        })
        .collect()
}

// 앞 문단 63,000 HU → 표 머리 뒤 잔여는 약 5,000 HU(0행 3,526 은 들고 0~1행 7,454 는 안 든다).
const TIGHT_FILLER: i32 = 63_000;

#[test]
fn opening_rowspan_block_cuts_after_the_first_row() {
    let core = document(TIGHT_FILLER, 1, false);

    assert_eq!(
        table_items(&core, 0, 1),
        vec![("partialTable".to_string(), 0, 1)],
        "0행이 들고 rowspan 칸 내용이 0행 안에 들면 첫 쪽은 0행만 받는다"
    );
    assert_eq!(
        table_items(&core, 1, 1),
        vec![("partialTable".to_string(), 1, 3)],
        "다음 쪽은 rowspan 칸의 이어짐과 1·2행을 받는다"
    );
}

#[test]
fn rowspan_content_taller_than_the_first_row_keeps_the_block_whole() {
    // 3줄(약 4,600 HU) 내용은 0행 3,526 HU 에 다 안 든다 — 묶음을 통째로 넘긴다.
    let core = document(TIGHT_FILLER, 3, false);

    assert!(
        table_items(&core, 0, 1).is_empty(),
        "rowspan 칸 글자를 자르는 0행 컷은 하지 않는다"
    );
    assert_eq!(
        table_items(&core, 1, 1),
        vec![("table".to_string(), 0, WHOLE)]
    );
}

#[test]
fn header_first_row_is_not_left_alone_at_the_page_end() {
    let core = document(TIGHT_FILLER, 1, true);

    assert!(
        table_items(&core, 0, 1).is_empty(),
        "제목 행만 담은 조각을 쪽 끝에 두지 않는다"
    );
    assert_eq!(
        table_items(&core, 1, 1),
        vec![("table".to_string(), 0, WHOLE)]
    );
}

#[test]
fn block_that_fits_the_remaining_space_stays_whole() {
    // 잔여가 0~1행 묶음보다 크면 행 중간에서 자르지 않는다(#1145).
    let core = document(57_000, 1, false);

    let first = table_items(&core, 0, 1);
    assert!(
        first.iter().all(|(_, _, end)| *end >= 2),
        "묶음이 들어가는 잔여에서는 0행 뒤 컷이 생기지 않는다: {first:?}"
    );
}

#[test]
fn sample_1480000_tables_9_and_17_open_with_their_first_row() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5941/1480000-201900698-native-neartop-reset.hwp");
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("parse 1480000-201900698");

    // 표 9(구역 4 pi=246): 한/글 2020 PDF 35쪽 끝에 0행(Name|Raw|Assembly), 36쪽에 1·2행.
    // 표 17(pi=289): 50쪽 끝에 0행, 51쪽에 1·2행. 쪽 번호는 0-기준 전역 쪽.
    for (page, para, label) in [(34, 246, "표 9"), (49, 289, "표 17")] {
        assert_eq!(
            table_items(&core, page, para),
            vec![("partialTable".to_string(), 0, 1)],
            "{label} 은 앞 쪽 끝에 0행만 둔다"
        );
        assert_eq!(
            table_items(&core, page + 1, para),
            vec![("partialTable".to_string(), 1, 3)],
            "{label} 의 1·2행은 다음 쪽에서 잇는다"
        );
    }
}

#[test]
fn sample_task1725_block_that_fits_the_page_end_stays_whole() {
    // pi=631(4×3, 0~1행 rowspan 묶음 84.5px)은 잔여 87.3px 에 묶음이 통째로 든다. 행 스캔의
    // 예산은 그보다 작아도, 이월 게이트가 «묶음이 넘칠 때만 자른다»를 정하므로 0행 뒤에서
    // 자르지 않는다 — 한/글 2024 PDF 29쪽도 0·1행을 함께 둔다.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task1725/text_footnote_tail_overpagination.hwp");
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("parse text_footnote_tail_overpagination");

    assert_eq!(
        table_items(&core, 28, 631),
        vec![("partialTable".to_string(), 0, 2)]
    );
    assert_eq!(
        table_items(&core, 29, 631),
        vec![("partialTable".to_string(), 2, 4)]
    );
}
