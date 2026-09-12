//! Public height measurement contract for stored inline-table rows.
//! These are synthetic IR boundaries, not Hancom fixture evidence.
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::{
    composer::compose_section, height_measurer::HeightMeasurer, style_resolver::resolve_styles,
};

fn nested(height: u32) -> Control {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        ..Default::default()
    };
    table.common.width = 1500;
    table.common.height = height;
    table.common.treat_as_char = true;
    table.cells = vec![Cell {
        row_span: 1,
        col_span: 1,
        width: 1500,
        height,
        ..Default::default()
    }];
    table.rebuild_grid();
    Control::Table(Box::new(table))
}
fn height(mut para: Paragraph) -> f64 {
    let native_stored = para.hwpx_axis_shift == 0;
    let width = para
        .line_segs
        .first()
        .map_or(20000, |seg| seg.segment_width.max(1) as u32);
    para.controls = if para.hwpx_axis_shift == 16 {
        vec![
            Control::SectionDef(Box::default()),
            Control::ColumnDef(Default::default()),
            nested(3000),
            nested(4500),
        ]
    } else {
        vec![nested(3000), nested(4500)]
    };
    let mut table = Table {
        row_count: 1,
        col_count: 2,
        ..Default::default()
    };
    table.common.width = width * 2;
    table.common.height = 1;
    table.common.treat_as_char = false;
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.common.vert_rel_to = VertRelTo::Para;
    table.cells = vec![Cell {
        row_span: 1,
        col_span: 1,
        width,
        height: 1,
        paragraphs: vec![para],
        ..Default::default()
    }];
    // Use a real two-cell host, so the unrelated 1x1 wrapper shortcut is not selected.
    table.cells.push(Cell {
        col: 1,
        row_span: 1,
        col_span: 1,
        width,
        height: 1,
        ..Default::default()
    });
    table.rebuild_grid();
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
        .with_native_hwp5(native_stored)
        .measure_section(&section.paragraphs, &composed, &styles, Some(300.0))
        .tables[0]
        .total_height
}
fn para(second_line: bool, non_bmp: bool, shift: u32) -> Paragraph {
    let split = if non_bmp { 10 } else { 9 };
    let mut lines = vec![LineSeg {
        line_height: 4500,
        baseline_distance: 3825,
        segment_width: 20000,
        ..Default::default()
    }];
    if second_line {
        lines.push(LineSeg {
            text_start: split,
            vertical_pos: 6000,
            line_height: 4500,
            baseline_distance: 3825,
            segment_width: 20000,
            ..Default::default()
        });
    }
    Paragraph {
        text: if non_bmp { "😀B" } else { "AB" }.into(),
        char_offsets: vec![8 + shift, split + 8 + shift],
        char_count: split + 10 + shift,
        hwpx_axis_shift: shift,
        line_segs: lines,
        ..Default::default()
    }
}
#[test]
fn separate_stored_rows_keep_their_control_slots_and_vertical_gap() {
    for non_bmp in [false, true] {
        for shift in [0, 16] {
            let same = height(para(false, non_bmp, shift));
            let separate = height(para(true, non_bmp, shift));
            assert!(
                same < 90.0,
                "same row must reserve the maximum, not sum: {same}"
            );
            assert!(separate >= 140.0, "second row y=80 plus table height=60: {separate}, shift={shift}, non_bmp={non_bmp}");
        }
    }
}
#[test]
fn missing_stored_rows_do_not_assume_one_inline_row() {
    let mut p = para(false, false, 0);
    p.line_segs.clear();
    assert!(
        height(p) >= 100.0,
        "legacy NO_LS fallback must not group TAC tables without row evidence"
    );
}

#[test]
fn text_free_controls_keep_distinct_raw_slots() {
    let mut p = para(true, false, 0);
    p.text.clear();
    p.char_offsets.clear();
    p.char_count = 17;
    p.line_segs[1].text_start = 8;
    for seg in &mut p.line_segs {
        seg.line_height = 750;
        seg.baseline_distance = 600;
    }
    assert!(
        height(p) >= 140.0,
        "raw slots 0 and 8 belong to separate stored rows"
    );
}

#[test]
fn visible_object_markers_keep_their_source_offsets_without_control_gaps() {
    let mut p = para(true, false, 0);
    p.text = "\u{FFFC}A\u{FFFC}B".into();
    p.char_offsets = vec![0, 1, 2, 3];
    p.char_count = 5;
    p.line_segs[1].text_start = 2;
    assert!(
        height(p) >= 140.0,
        "visible one-unit object markers retain separate stored rows"
    );
}

#[test]
fn explicit_line_break_preserves_each_nested_tables_stored_row() {
    let mut p = para(true, false, 0);
    p.text = "A\nB".into();
    p.char_offsets = vec![8, 9, 18];
    p.char_count = 20;
    p.line_segs[1].text_start = 10;
    assert!(
        height(p) >= 140.0,
        "a line break retains the second row's y=80 and height=60"
    );
}

#[test]
fn stored_width_wrap_keeps_a_second_table_on_its_own_row() {
    let mut p = para(true, false, 0);
    // Each table fits 3000 HU, but their combined 3000 HU plus text does not.
    for line in &mut p.line_segs {
        line.segment_width = 3000;
    }
    assert!(
        height(p) >= 140.0,
        "a stored width wrap is not one row merely because both tables are inline"
    );
}
