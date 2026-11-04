//! 관계형·고정형: 합성 HWPUNIT 첫 프레임·여러 rowspan 높이 보존과 내용 하한·저장 유효성.
//! 주석 갱신: 2026-11-03T10:49:58+09:00.
//! 공개 HeightMeasurer의 행 원장을 검사하는 합성 계약이다. 실물의 절대 px/golden을 고정하지 않는다.
//! 이 합성 입력의 시각 raw 측정은 비해당이다. 실제 호출·출력의 별도 KPS 전쪽 근거는
//! mydocs/pr/pr_8187_review.md와 assets/pr8187/에 있으며 다른 원본의 raw를 이 검사의 측정값으로 쓰지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{
    control::Control,
    document::{Document, Section},
    paragraph::{LineSeg, Paragraph},
    shape::{CommonObjAttr, TextWrap},
    table::{Cell, Table, TablePageBreak},
};
use rhwp::renderer::{
    composer::compose_section, height_measurer::HeightMeasurer, style_resolver::resolve_styles,
};

fn source_table() -> Table {
    let mut table = Table {
        row_count: 4,
        col_count: 4,
        page_break: TablePageBreak::RowBreak,
        common: CommonObjAttr {
            width: 24_000,
            height: 11_250,
            text_wrap: TextWrap::TopAndBottom,
            ..Default::default()
        },
        ..Default::default()
    };
    for (column, row, span, height) in [(0, 0, 3, 22_500), (1, 0, 4, 30_000), (2, 1, 2, 15_000)] {
        let mut cell = Cell::new_empty(column, row, 6000, height, 0);
        cell.row_span = span;
        table.cells.push(cell);
    }
    for row in 0..4 {
        table.cells.push(Cell::new_empty(3, row, 6000, 7500, 0));
    }
    table.cells[0].paragraphs = [0, 1200, 0]
        .into_iter()
        .map(|vertical_pos| Paragraph {
            text: "x".into(),
            char_offsets: vec![0],
            char_count: 2,
            has_para_text: true,
            line_segs: vec![LineSeg {
                vertical_pos,
                line_height: 1200,
                text_height: 1200,
                baseline_distance: 1020,
                segment_width: 6000,
                ..Default::default()
            }],
            ..Default::default()
        })
        .collect();
    table.rebuild_grid();
    table
}

fn measured_rows(table: Table) -> Vec<f64> {
    let section = Section {
        paragraphs: vec![Paragraph {
            controls: vec![Control::Table(Box::new(table))],
            ..Default::default()
        }],
        ..Default::default()
    };
    let styles = resolve_styles(&Document::default().doc_info, 96.0);
    let composed = compose_section(&section);
    HeightMeasurer::new(96.0)
        .with_native_hwp5(true)
        .measure_section(&section.paragraphs, &composed, &styles, Some(400.0))
        .tables[0]
        .row_heights
        .clone()
}

#[test]
fn shared_rowspan_end_reserves_carried_physical_height_once() {
    let table = source_table();
    let spans: Vec<_> = table
        .cells
        .iter()
        .filter(|cell| cell.row_span > 1)
        .map(|cell| {
            (
                usize::from(cell.row),
                usize::from(cell.row + cell.row_span),
                f64::from(cell.height) / 75.0,
            )
        })
        .collect();
    let rows = measured_rows(table);
    assert_eq!(rows, [100.0, 50.0, 150.0, 100.0]);
    assert_eq!(rows[..2].iter().sum::<f64>(), 150.0, "독립 저장 첫 프레임");
    for (start, end, height) in spans {
        assert_eq!(
            rows[start..end].iter().sum::<f64>(),
            height,
            "같은 행에서 끝나는 두 rowspan과 뒤에서 끝나는 rowspan의 높이 보존"
        );
    }
}

#[test]
fn content_floors_and_missing_or_dirty_source_witness_keep_the_existing_ledger() {
    let table = source_table();
    let mut high_content = table.clone();
    let paragraph = &mut high_content.cells[4].paragraphs[0];
    paragraph.text = "x".into();
    paragraph.char_offsets = vec![0];
    paragraph.char_count = 2;
    paragraph.has_para_text = true;
    paragraph.line_segs = vec![LineSeg {
        line_height: 5625,
        text_height: 5625,
        baseline_distance: 4781,
        segment_width: 6000,
        ..Default::default()
    }];
    assert_eq!(
        measured_rows(high_content),
        [100.0; 4],
        "닫는 행의 75px 내용은 50px로 줄이지 않는다"
    );
    let mut ending = table.clone();
    ending.col_count = 5;
    ending.common.width = 30_000;
    let mut cell = Cell::new_empty(4, 0, 6000, 15_000, 0);
    cell.row_span = 2;
    cell.paragraphs[0].text = "x".into();
    cell.paragraphs[0].has_para_text = true;
    cell.paragraphs[0].char_offsets = vec![0];
    cell.paragraphs[0].char_count = 2;
    cell.paragraphs[0].line_segs = vec![LineSeg {
        line_height: 13_500,
        text_height: 13_500,
        baseline_distance: 11_475,
        segment_width: 6000,
        ..Default::default()
    }];
    ending.cells.push(cell);
    ending.rebuild_grid();
    assert_eq!(
        measured_rows(ending),
        [100.0; 4],
        "이 조각에서 끝나는 다른 병합 셀의 180px 내용 보존"
    );
    let mut no_rewind = table.clone();
    no_rewind.cells[0].paragraphs[2].line_segs[0].vertical_pos = 2400;
    let ordinary = measured_rows(no_rewind);
    assert!(
        ordinary
            .windows(2)
            .all(|pair| (pair[0] - pair[1]).abs() < 1e-6),
        "되감김 없는 일반 표에 새 조각 높이를 배정하지 않는다"
    );
    let mut dirty = table;
    dirty.cells[0].dirty_flag = true;
    assert_eq!(
        measured_rows(dirty),
        [100.0; 4],
        "편집한 셀에는 이전 저장 프레임을 적용하지 않는다"
    );
}
