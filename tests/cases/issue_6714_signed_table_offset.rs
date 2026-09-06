//! Negative raw HWP offsets must not enter positive-offset paginator branches.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::page::{ColumnDef, PageDef};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{CommonObjAttr, TextWrap, VertRelTo};
use rhwp::model::table::{Table, TablePageBreak};
use rhwp::renderer::height_measurer::{MeasuredParagraph, MeasuredSection, MeasuredTable};
use rhwp::renderer::pagination::{PageItem, PaginationResult, Paginator};

fn paginate(offset: i32, prefix: f64, text: &str, row_heights: Vec<f64>) -> PaginationResult {
    let total_height: f64 = row_heights.iter().sum();
    let para_index = usize::from(prefix > 0.0);
    let table = Table {
        row_count: row_heights.len() as u16,
        col_count: 1,
        page_break: TablePageBreak::RowBreak,
        common: CommonObjAttr {
            treat_as_char: false,
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Para,
            vertical_offset: offset as u32,
            width: 7500,
            height: (total_height * 75.0) as u32,
            ..Default::default()
        },
        ..Default::default()
    };
    let measure_para = |para_index, height, has_table| MeasuredParagraph {
        para_index,
        total_height: height,
        line_heights: vec![height],
        line_spacings: vec![0.0],
        spacing_before: 0.0,
        spacing_after: 0.0,
        has_table,
    };
    let mut paragraphs = Vec::new();
    let mut measured = Vec::new();
    if prefix > 0.0 {
        paragraphs.push(Paragraph {
            text: "prefix".into(),
            ..Default::default()
        });
        measured.push(measure_para(0, prefix, false));
    }
    paragraphs.push(Paragraph {
        text: text.into(),
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    });
    measured.push(measure_para(
        para_index,
        if text.is_empty() { 0.0 } else { 20.0 },
        true,
    ));
    let mut cumulative_heights = vec![0.0];
    for height in &row_heights {
        cumulative_heights.push(cumulative_heights.last().unwrap() + height);
    }
    let section = MeasuredSection {
        fallback_paragraphs: measured,
        tables: vec![MeasuredTable {
            para_index,
            control_index: 0,
            total_height,
            row_heights,
            caption_height: 0.0,
            cell_spacing: 0.0,
            cumulative_heights,
            repeat_header: false,
            has_header_cells: false,
            cells: Vec::new(),
            page_break: TablePageBreak::RowBreak,
            row_block_start: Vec::new(),
            row_block_end: Vec::new(),
        }],
    };
    Paginator::new(96.0).paginate_with_measured(
        &paragraphs,
        &section,
        &PageDef {
            width: 15000,
            height: 15000,
            ..Default::default()
        },
        &ColumnDef::default(),
        0,
        &[],
    )
}

fn first_page_order(result: &PaginationResult) -> Vec<&'static str> {
    result.pages[0].column_contents[0]
        .items
        .iter()
        .map(|item| match item {
            PageItem::Table { .. } => "table",
            PageItem::PartialParagraph { .. } => "text",
            _ => "other",
        })
        .collect()
}

#[test]
fn negative_offset_does_not_move_host_text_before_the_table() {
    let zero = paginate(0, 0.0, "host", vec![80.0]);
    assert_eq!(first_page_order(&zero), ["table", "text"]);
    for offset in [-1, -750, i32::MIN] {
        let negative = paginate(offset, 0.0, "host", vec![80.0]);
        assert_eq!(
            first_page_order(&negative),
            first_page_order(&zero),
            "offset={offset}"
        );
        assert_eq!(
            negative.pages[0].column_contents[0].used_height,
            zero.pages[0].column_contents[0].used_height,
            "offset={offset}"
        );
    }
}

#[test]
fn positive_offset_keeps_pre_table_text_order() {
    let positive = paginate(750, 0.0, "host", vec![80.0]);
    assert_eq!(first_page_order(&positive), ["text", "table"]);
    assert_eq!(positive.pages.len(), 1);
}

fn first_fragment_end(result: &PaginationResult) -> usize {
    result.pages[0].column_contents[0]
        .items
        .iter()
        .find_map(|item| match item {
            PageItem::PartialTable {
                start_row: 0,
                end_row,
                ..
            } => Some(*end_row),
            _ => None,
        })
        .expect("table must split on the first page")
}

#[test]
fn negative_offset_does_not_increase_room_for_split_rows() {
    let zero = paginate(0, 100.0, "", vec![55.0; 3]);
    assert_eq!(first_fragment_end(&zero), 1);
    let negative = paginate(-750, 100.0, "", vec![55.0; 3]);
    assert_eq!(first_fragment_end(&negative), 1);
    let positive = paginate(750, 100.0, "", vec![55.0; 3]);
    assert_eq!(first_fragment_end(&positive), 1);
}
