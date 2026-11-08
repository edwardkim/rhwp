//! HWPX 편집 자기검증은 저장 대상 줄 캐시를 비교하고 원본 손실은 계속 검출한다.

use rhwp::model::{
    control::Control,
    document::{Document, Section},
    paragraph::{CharShapeRef, LineSeg, Paragraph},
    table::{Cell, Table},
};
use rhwp::serializer::hwpx::roundtrip::{diff_documents, diff_exported_hwpx_documents};

fn line(synthetic: bool) -> LineSeg {
    LineSeg {
        text_start: 0,
        vertical_pos: 100,
        line_height: 1000,
        text_height: 1000,
        baseline_distance: 850,
        line_spacing: 600,
        column_start: 0,
        segment_width: 3000,
        tag: 0x0006_0000
            | if synthetic {
                LineSeg::TAG_IMPLEMENTATION_PROPERTY
            } else {
                0
            },
    }
}

fn document(lines: Vec<LineSeg>) -> Document {
    let mut section = Section::default();
    section.paragraphs.push(Paragraph {
        line_segs: lines,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        ..Default::default()
    });
    Document {
        sections: vec![section],
        ..Default::default()
    }
}

#[test]
fn regenerated_runtime_cache_is_not_stored_source_loss() {
    let expected = document(vec![line(true)]);
    let mut actual = document(vec![line(true), line(true)]);
    actual.sections[0].paragraphs[0].line_segs[0].vertical_pos += 500;
    assert!(!diff_documents(&expected, &actual).is_empty());
    assert!(diff_exported_hwpx_documents(&expected, &actual).is_empty());
    assert_eq!(expected.sections[0].paragraphs[0].line_segs, [line(true)]);
    assert_eq!(actual.sections[0].paragraphs[0].line_segs.len(), 2);
}

#[test]
fn source_vertical_positions_remain_authoritative_after_relayout() {
    let expected = document(vec![line(false)]);
    let mut actual = expected.clone();
    let para = &mut actual.sections[0].paragraphs[0];
    para.source_line_seg_vertical_pos = Some(vec![line(false).vertical_pos]);
    para.line_segs[0].vertical_pos += 500;
    assert!(!diff_documents(&expected, &actual).is_empty());
    assert!(diff_exported_hwpx_documents(&expected, &actual).is_empty());
    actual.sections[0].paragraphs[0]
        .source_line_seg_vertical_pos
        .as_mut()
        .unwrap()[0] += 1;
    assert!(!diff_exported_hwpx_documents(&expected, &actual).is_empty());
}

#[test]
fn every_genuine_source_line_field_is_still_checked() {
    let expected = document(vec![line(false)]);
    for field in 0..9 {
        let mut actual = expected.clone();
        let line = &mut actual.sections[0].paragraphs[0].line_segs[0];
        match field {
            0 => line.text_start += 1,
            1 => line.vertical_pos += 1,
            2 => line.line_height += 1,
            3 => line.text_height += 1,
            4 => line.baseline_distance += 1,
            5 => line.line_spacing += 1,
            6 => line.column_start += 1,
            7 => line.segment_width += 1,
            _ => line.tag ^= 1,
        }
        assert!(
            !diff_exported_hwpx_documents(&expected, &actual).is_empty(),
            "원본 줄 속성 {field}의 손실을 검출해야 한다"
        );
    }
}

#[test]
fn synthetic_cache_must_not_hide_new_genuine_source_rows() {
    let expected = document(vec![line(true)]);
    let actual = document(vec![line(false)]);
    assert!(!diff_exported_hwpx_documents(&expected, &actual).is_empty());
}

#[test]
fn synthetic_parent_keeps_genuine_nested_cell_cache_validation() {
    let mut expected = document(vec![line(true)]);
    let mut cell = Cell::new_empty(0, 0, 3000, 2000, 0);
    cell.paragraphs[0].line_segs = vec![line(false)];
    expected.sections[0].paragraphs[0]
        .controls
        .push(Control::Table(Box::new(Table {
            row_count: 1,
            col_count: 1,
            cells: vec![cell],
            ..Default::default()
        })));
    let mut actual = expected.clone();
    let Control::Table(table) = &mut actual.sections[0].paragraphs[0].controls[0] else {
        panic!("표 전제");
    };
    table.cells[0].paragraphs[0].line_segs[0].segment_width += 1;
    let diff = diff_exported_hwpx_documents(&expected, &actual);
    assert!(!diff.is_empty());
    assert!(format!("{diff:?}").contains("tbl.cell[0].p[0]"));
}

#[test]
fn transient_cache_projection_keeps_character_style_validation() {
    let expected = document(vec![line(true)]);
    let mut actual = expected.clone();
    actual.sections[0].paragraphs[0].char_shapes[0].char_shape_id = 1;
    assert!(!diff_exported_hwpx_documents(&expected, &actual).is_empty());
}

#[test]
fn mismatched_source_snapshot_does_not_replace_runtime_rows() {
    let expected = document(vec![line(false)]);
    let mut actual = expected.clone();
    let para = &mut actual.sections[0].paragraphs[0];
    para.source_line_seg_vertical_pos = Some(vec![line(false).vertical_pos; 2]);
    para.line_segs[0].vertical_pos += 500;
    assert!(!diff_exported_hwpx_documents(&expected, &actual).is_empty());
}
