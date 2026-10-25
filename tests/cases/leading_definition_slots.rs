//! 빠진 구역·단 정의를 보강해도 기존 머리말·누름틀의 슬롯을 빼앗지 않는다.

use rhwp::document_core::converters::hwpx_to_hwp::convert_hwpx_to_hwp_ir;
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Field, FieldType};
use rhwp::model::paragraph::{CharShapeRef, FieldRange, LineSeg, Paragraph, RangeTag, TitleMark};
use rhwp::model::table::{Cell, Table};

fn document(para: Paragraph) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.document_mut().sections[0].raw_stream = None;
    core.document_mut().sections[0].paragraphs = vec![para];
    core
}

fn table() -> Control {
    Control::Table(Box::new(Table {
        row_count: 1,
        col_count: 1,
        row_sizes: vec![1],
        cells: vec![Cell {
            paragraphs: vec![Paragraph {
                text: "첫 표".into(),
                char_count: 4,
                ..Paragraph::new_empty()
            }],
            ..Cell::new_empty(0, 0, 1000, 1000, 0)
        }],
        ..Default::default()
    }))
}

fn assert_table_position(para: &Paragraph, position: usize) {
    let index = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .unwrap();
    assert_eq!(para.control_text_positions()[index], position, "{para:?}");
    let Control::Table(table) = &para.controls[index] else {
        unreachable!()
    };
    assert_eq!(table.cells[0].paragraphs[0].text, "첫 표");
    for (control, position) in para.controls.iter().zip(para.control_text_positions()) {
        if matches!(control, Control::Header(_) | Control::Footer(_)) {
            assert_eq!(position, 0);
        }
    }
}

#[test]
fn hwp_save_keeps_table_between_text_after_leading_headers() {
    for hidden in 0..=2 {
        for existing_defs in 0..=2 {
            let mut core = document(Paragraph::new_empty());
            let section_def = core.document().sections[0].section_def.clone();
            let mut controls = Vec::new();
            if existing_defs == 1 {
                controls.push(Control::ColumnDef(Default::default()));
            }
            if existing_defs == 2 {
                controls.push(Control::SectionDef(Box::new(section_def)));
            }
            if hidden > 0 {
                controls.push(Control::Header(Default::default()));
            }
            if hidden > 1 {
                controls.push(Control::Footer(Default::default()));
            }
            let leading = controls.len() as u32 * 8;
            controls.push(table());
            core.document_mut().sections[0].paragraphs[0] = Paragraph {
                text: "앞😊\u{fffc}뒤".into(),
                char_offsets: vec![leading, leading + 1, leading + 3, leading + 11],
                char_count: leading + 13,
                controls,
                ..Paragraph::new_empty()
            };
            let before = serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap();
            assert_table_position(&core.document().sections[0].paragraphs[0], 2);
            let saved = core.export_hwp_native().unwrap();
            assert_eq!(
                serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap(),
                before
            );
            let reopened = DocumentCore::from_bytes(&saved).unwrap();
            let para = &reopened.document().sections[0].paragraphs[0];
            assert_eq!(para.text, "앞😊뒤");
            assert_table_position(para, 2);
            let again = DocumentCore::from_bytes(&reopened.export_hwp_native().unwrap()).unwrap();
            assert_table_position(&again.document().sections[0].paragraphs[0], 2);
        }
    }
}

#[test]
fn adapter_keeps_owned_slots_metadata_and_unused_reservation() {
    for controls in [vec![], vec![Control::Header(Default::default())]] {
        let occupied = !controls.is_empty();
        let mut core = document(Paragraph {
            text: "가나".into(),
            char_offsets: vec![8, 9],
            char_count: 11,
            ctrl_data_records: if occupied {
                vec![Some(vec![1, 2, 3])]
            } else {
                vec![]
            },
            controls,
            char_shapes: vec![
                CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                },
                CharShapeRef {
                    start_pos: 9,
                    char_shape_id: 0,
                },
            ],
            line_segs: vec![
                LineSeg {
                    text_start: 0,
                    ..Default::default()
                },
                LineSeg {
                    text_start: 9,
                    ..Default::default()
                },
            ],
            range_tags: vec![RangeTag {
                start: 8,
                end: 10,
                tag: 0,
            }],
            ..Paragraph::new_empty()
        });
        convert_hwpx_to_hwp_ir(core.document_mut());
        let para = &core.document().sections[0].paragraphs[0];
        let shift = if occupied { 8 } else { 0 };
        assert_eq!(para.char_offsets, [8 + shift, 9 + shift]);
        assert_eq!(para.char_shapes[0].start_pos, 0);
        assert_eq!(para.char_shapes[1].start_pos, 9 + shift);
        assert_eq!(para.line_segs[0].text_start, 0);
        assert_eq!(para.line_segs[1].text_start, 9 + shift);
        assert_eq!(
            (para.range_tags[0].start, para.range_tags[0].end),
            (8 + shift, 10 + shift)
        );
        if occupied {
            assert_eq!(para.ctrl_data_records[1], Some(vec![1, 2, 3]));
        }
        let before = serde_json::to_value(para).unwrap();
        convert_hwpx_to_hwp_ir(core.document_mut());
        assert_eq!(
            serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap(),
            before
        );
    }
}

#[test]
fn definition_insertion_preserves_empty_field_and_title_slots() {
    let mut core = document(Paragraph {
        text: "앞\u{fffc}뒤".into(),
        char_offsets: vec![24, 25, 33],
        char_count: 35,
        controls: vec![
            Control::Field(Field {
                field_type: FieldType::ClickHere,
                ctrl_id: rhwp::parser::tags::FIELD_CLICKHERE,
                field_id: 42,
                ..Default::default()
            }),
            table(),
        ],
        field_ranges: vec![FieldRange {
            control_idx: 0,
            ..Default::default()
        }],
        title_marks: vec![TitleMark {
            char_idx: 0,
            ignore: false,
        }],
        ..Paragraph::new_empty()
    });
    convert_hwpx_to_hwp_ir(core.document_mut());
    let para = &core.document().sections[0].paragraphs[0];
    assert_eq!(para.field_ranges[0].control_idx, 1);
    assert_eq!(para.char_offsets, [32, 33, 41]);
    let saved = core.export_hwp_native().unwrap();
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    let para = &reopened.document().sections[0].paragraphs[0];
    assert_eq!(para.text, "앞뒤");
    assert_table_position(para, 1);
    assert_eq!(para.title_marks.len(), 1);
    assert_eq!(para.field_ranges.len(), 1);
    let range = &para.field_ranges[0];
    assert_eq!((range.start_char_idx, range.end_char_idx), (0, 0));
    assert!(
        matches!(&para.controls[range.control_idx], Control::Field(field) if field.field_id == 42)
    );
}
