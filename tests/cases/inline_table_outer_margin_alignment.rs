//! A stored line's alignment footprint must include inline table outer margins.
//! Related: #3396/#3410 (paint advance) and #6601/#6604 (separate table-only path).
//! Fixture dimensions, rather than the potentially expanded render-tree parent
//! bounds, define the expected frame. See the fixture README for provenance.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::style::Alignment;
use serde_json::Value;

const FIXTURE: &[u8] = include_bytes!("../fixtures/inline_table_outer_margins/two_digits.hwpx");
const FRAME_LEFT: f64 = (3600.0 + 7200.0) * 96.0 / 7200.0;
const FRAME_RIGHT: f64 = (3600.0 + 28800.0) * 96.0 / 7200.0;

fn digit_boxes(
    align: Alignment,
    left: i16,
    right: i16,
    stored: bool,
    paragraph_right_margin: i32,
) -> Vec<(f64, f64, f64)> {
    let source = DocumentCore::from_bytes(FIXTURE).expect("synthetic HWPX");
    let mut document = source.document().clone();
    let Control::Table(outer) = &mut document.sections[0].paragraphs[1].controls[0] else {
        panic!("fixture outer table");
    };
    let paragraph = &mut outer.cells[1].paragraphs[0];
    document.doc_info.para_shapes[paragraph.para_shape_id as usize].alignment = align;
    document.doc_info.para_shapes[paragraph.para_shape_id as usize].margin_right =
        paragraph_right_margin;
    // ParaShape margins use twice the LineSeg unit. The saved row already
    // excludes this paragraph margin, as in the public issue_1285 document.
    paragraph.line_segs[0].segment_width -= paragraph_right_margin / 2;
    if !stored {
        paragraph.line_segs.clear();
    }
    for control in &mut paragraph.controls {
        if let Control::Table(table) = control {
            table.outer_margin_left = left;
            table.outer_margin_right = right;
            table.common.margin.left = left;
            table.common.margin.right = right;
        }
    }
    let mut core = DocumentCore::new_empty();
    core.set_document(document);
    let layout: Value = serde_json::from_str(
        &core
            .get_page_control_layout_native(0)
            .expect("render digit line"),
    )
    .expect("control layout JSON");
    let mut boxes: Vec<_> = layout["controls"]
        .as_array()
        .expect("controls")
        .iter()
        .filter(|control| {
            control["stableIndex"]
                .as_array()
                .is_some_and(|path| path.len() > 3)
        })
        .map(|control| {
            let number = |name| control[name].as_f64().expect("table coordinate");
            (number("x"), number("y"), number("w"))
        })
        .collect();
    boxes.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(
        boxes.len(),
        2,
        "both independently editable digit tables remain"
    );
    assert!(
        (boxes[0].1 - boxes[1].1).abs() < 0.15,
        "digits share their line"
    );
    for (_, _, width) in &boxes {
        assert!(
            (width - 28.8).abs() < 0.15,
            "outer margins must not resize the table"
        );
    }
    boxes
}

#[test]
fn stored_line_aligns_the_whole_inline_table_footprint() {
    for align in [Alignment::Left, Alignment::Center, Alignment::Right] {
        for (left, right) in [(0, 0), (360, 720), (720, 360), (-180, 360)] {
            let boxes = digit_boxes(align, left, right, true, 0);
            let start = boxes[0].0 - f64::from(left) * 96.0 / 7200.0;
            let end = boxes[1].0 + boxes[1].2 + f64::from(right) * 96.0 / 7200.0;
            let (actual, expected) = match align {
                Alignment::Left => (start, FRAME_LEFT),
                Alignment::Center => ((start + end) / 2.0, (FRAME_LEFT + FRAME_RIGHT) / 2.0),
                Alignment::Right => (end, FRAME_RIGHT),
                _ => unreachable!(),
            };
            assert!(
                (actual - expected).abs() < 0.2,
                "{align:?} margins=({left},{right}): footprint {start:.2}..{end:.2}; expected alignment at {expected:.2}"
            );
        }
    }
}

#[test]
fn table_only_layout_keeps_counting_outer_margins_once() {
    // Removing stored line data selects the separate inline-table paragraph path.
    // Its widths already include margins; changing Table::flow_width_hu globally
    // would double-count them here.
    let boxes = digit_boxes(Alignment::Right, 360, 720, false, 0);
    let end = boxes[1].0 + boxes[1].2 + 720.0 * 96.0 / 7200.0;
    assert!(
        (end - FRAME_RIGHT).abs() < 0.2,
        "table-only footprint ends at {end}"
    );
}

#[test]
fn stored_inline_table_line_counts_paragraph_right_margin_once() {
    for margin in [600, 1800] {
        for (left, right) in [(0, 0), (360, 720)] {
            let boxes = digit_boxes(Alignment::Right, left, right, true, margin);
            let end = boxes[1].0 + boxes[1].2 + f64::from(right) * 96.0 / 7200.0;
            let expected = FRAME_RIGHT - f64::from(margin) / 2.0 * 96.0 / 7200.0;
            assert!(
                (end - expected).abs() < 0.2,
                "paragraph right margin {margin}: footprint ends at {end}, expected {expected}"
            );
        }
    }
}
