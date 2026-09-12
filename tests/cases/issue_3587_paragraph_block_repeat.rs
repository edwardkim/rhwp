//! B2 core contracts, not Hancom visual acceptance or the B3 save matrix.
use rhwp::{
    document_core::{
        DocumentCore, ParagraphBlockLimits, ParagraphBlockPathStep as Step,
        RepeatParagraphBlockRequest,
    },
    model::{
        control::{Control, Field, FieldType},
        paragraph::{OrphanFieldEnd, Paragraph},
        shape::{RectangleShape, ShapeObject, TextBox},
    },
};
use std::collections::BTreeSet;

fn core() -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    c.document_mut().sections[0].paragraphs = ["before", "body", "", "after"]
        .into_iter()
        .map(|text| Paragraph {
            text: text.into(),
            ..Default::default()
        })
        .collect();
    c
}
fn request() -> RepeatParagraphBlockRequest {
    RepeatParagraphBlockRequest {
        section_index: 0,
        source_start: 1,
        source_end: 3,
        insert_before: 3,
        count: 2,
        limits: ParagraphBlockLimits::default(),
    }
}
fn texts(c: &DocumentCore) -> Vec<&str> {
    c.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect()
}

#[test]
fn boundaries_keep_originals_and_exact_empty_paragraph_count() {
    for at in [0, 1, 3, 4] {
        for count in [1, 2, 5] {
            let mut c = core();
            let original = c.document().sections[0].paragraphs.clone();
            let mut r = request();
            r.insert_before = at;
            r.count = count;
            let result = c.repeat_paragraph_block_native(&r).unwrap();
            let mut expected = vec!["before", "body", "", "after"];
            expected.splice(at..at, (0..count).flat_map(|_| ["body", ""]));
            assert_eq!(texts(&c), expected);
            assert_eq!(result.inserted, at..at + count * 2);
            assert_eq!(result.copies.len(), count);
            for (index, p) in original.iter().enumerate() {
                let after = index + if index >= at { count * 2 } else { 0 };
                assert_eq!(
                    format!("{p:?}"),
                    format!("{:?}", c.document().sections[0].paragraphs[after])
                );
            }
            assert_eq!(
                result.source_after,
                if at <= 1 {
                    1 + count * 2..3 + count * 2
                } else {
                    1..3
                }
            );
            assert_eq!(
                result.copies[0].mappings[0].source,
                vec![Step::Paragraph(0)]
            );
            assert_eq!(
                result.copies[0].mappings[0].destination,
                vec![Step::Paragraph(at)]
            );
        }
    }
}

#[test]
fn zero_and_rejections_leave_document_clipboard_and_events_unchanged() {
    for kind in 0..4 {
        let mut c = core();
        let mut r = request();
        match kind {
            0 => r.count = 0,
            1 => r.insert_before = 2,
            2 => r.count = usize::MAX,
            _ => r.limits.max_structure_bytes = 1,
        }
        let before = format!("{:?}", c.document());
        let events = c.serialize_event_log();
        let clipboard = c.get_clipboard_text_native();
        let result = c.repeat_paragraph_block_native(&r);
        if kind == 0 {
            assert!(result.unwrap().copies.is_empty());
        } else {
            assert!(result.is_err());
        }
        assert_eq!(format!("{:?}", c.document()), before);
        assert_eq!(c.serialize_event_log(), events);
        assert_eq!(c.get_clipboard_text_native(), clipboard);
    }
}

#[test]
fn field_end_references_are_copy_local_across_repeated_requests() {
    let mut c = core();
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Field(Field {
            field_type: FieldType::ClickHere,
            field_id: 99,
            ctrl_id: u32::from_le_bytes(*b"klc%"),
            ctrl_data_name: Some("unchanged-name".into()),
            ..Default::default()
        }));
    c.document_mut().sections[0].paragraphs[2]
        .orphan_field_ends
        .push(OrphanFieldEnd {
            begin_id_ref: 99,
            ..Default::default()
        });
    for _ in 0..2 {
        c.repeat_paragraph_block_native(&request()).unwrap();
    }
    let mut ids = BTreeSet::new();
    for pair in c.document().sections[0].paragraphs[1..11].chunks_exact(2) {
        let Control::Field(f) = &pair[0].controls[0] else {
            panic!()
        };
        assert!(ids.insert(f.field_id));
        assert_eq!(pair[1].orphan_field_ends[0].begin_id_ref, f.field_id);
        assert_eq!(f.ctrl_data_name.as_deref(), Some("unchanged-name"));
    }
    assert_eq!(ids.len(), 5);
}

#[test]
fn nested_textbox_paths_and_independent_content_are_preserved() {
    let mut c = core();
    let mut rectangle = RectangleShape::default();
    rectangle.common.instance_id = 50;
    rectangle.drawing.inst_id = 60;
    rectangle.drawing.text_box = Some(TextBox {
        paragraphs: vec![Paragraph {
            text: "inside".into(),
            ..Default::default()
        }],
        ..Default::default()
    });
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Shape(Box::new(ShapeObject::Rectangle(rectangle))));
    let result = c.repeat_paragraph_block_native(&request()).unwrap();
    assert!(result.copies[1].mappings.iter().any(|m| m.destination
        == vec![
            Step::Paragraph(5),
            Step::Control(0),
            Step::Shape,
            Step::TextBox,
            Step::Paragraph(0)
        ]));
    let Control::Shape(s) = &mut c.document_mut().sections[0].paragraphs[3].controls[0] else {
        panic!()
    };
    let ShapeObject::Rectangle(r) = s.as_mut() else {
        panic!()
    };
    r.drawing.text_box.as_mut().unwrap().paragraphs[0].text = "edited copy".into();
    let mut ids = BTreeSet::new();
    for pi in [1, 5] {
        let Control::Shape(s) = &c.document().sections[0].paragraphs[pi].controls[0] else {
            panic!()
        };
        let ShapeObject::Rectangle(r) = s.as_ref() else {
            panic!()
        };
        assert_eq!(
            r.drawing.text_box.as_ref().unwrap().paragraphs[0].text,
            "inside"
        );
        assert!(ids.insert(r.common.instance_id));
    }
}

#[test]
fn real_table_blocks_preserve_original_box_and_have_distinct_copy_ids() {
    for (file, pi) in [
        ("samples/hwp_table_test.hwp", 3),
        ("samples/rnote/labnote-001.hwp", 12),
    ] {
        let mut c = DocumentCore::from_bytes(&std::fs::read(file).unwrap()).unwrap();
        let r = RepeatParagraphBlockRequest {
            source_start: pi,
            source_end: pi + 1,
            insert_before: pi + 1,
            ..request()
        };
        let original = &c.document().sections[0].paragraphs[pi];
        let table_index = original
            .controls
            .iter()
            .position(|x| matches!(x, Control::Table(_)))
            .unwrap();
        let Control::Table(t) = &original.controls[table_index] else {
            panic!()
        };
        let pointer = t.as_ref() as *const _ as usize;
        let before = format!("{t:?}");
        c.repeat_paragraph_block_native(&r).unwrap();
        let mut ids = BTreeSet::new();
        let mut pointers = BTreeSet::new();
        for index in pi..=pi + 2 {
            let Control::Table(t) =
                &c.document().sections[0].paragraphs[index].controls[table_index]
            else {
                panic!()
            };
            assert!(ids.insert(t.common.instance_id));
            assert!(pointers.insert(t.as_ref() as *const _ as usize));
            assert_eq!(
                u32::from_le_bytes(t.raw_ctrl_data[32..36].try_into().unwrap()),
                t.common.instance_id
            );
            if index == pi {
                assert_eq!(t.as_ref() as *const _ as usize, pointer);
                assert_eq!(format!("{t:?}"), before);
            }
        }
    }
}

#[test]
fn successful_repeat_does_not_replace_clipboard_and_emits_one_event() {
    let mut c = core();
    let clipboard = c.get_clipboard_text_native();
    c.begin_batch_native().unwrap();
    let before: serde_json::Value = serde_json::from_str(&c.serialize_event_log()).unwrap();
    c.repeat_paragraph_block_native(&request()).unwrap();
    let after: serde_json::Value = serde_json::from_str(&c.serialize_event_log()).unwrap();
    assert_eq!(
        after["events"].as_array().unwrap().len(),
        before["events"].as_array().unwrap().len() + 1
    );
    assert_eq!(c.get_clipboard_text_native(), clipboard);
    c.end_batch_native().unwrap();
    assert_eq!(
        texts(&c),
        ["before", "body", "", "body", "", "body", "", "after"]
    );
}

#[test]
fn connector_targets_follow_each_copy_not_the_original_or_other_copy() {
    use rhwp::model::shape::{ConnectorData, LineShape};
    let mut c = core();
    let mut rectangle = RectangleShape::default();
    rectangle.common.instance_id = 100;
    rectangle.drawing.inst_id = 200;
    let mut line = LineShape::default();
    line.common.instance_id = 300;
    line.drawing.inst_id = 400;
    line.connector = Some(ConnectorData {
        start_subject_id: 200,
        ..Default::default()
    });
    c.document_mut().sections[0].paragraphs[1].controls = vec![
        Control::Shape(Box::new(ShapeObject::Rectangle(rectangle))),
        Control::Shape(Box::new(ShapeObject::Line(line))),
    ];
    c.repeat_paragraph_block_native(&request()).unwrap();
    let mut targets = BTreeSet::new();
    for pi in [1, 3, 5] {
        let controls = &c.document().sections[0].paragraphs[pi].controls;
        let Control::Shape(target) = &controls[0] else {
            panic!()
        };
        let Control::Shape(connector) = &controls[1] else {
            panic!()
        };
        let ShapeObject::Line(line) = connector.as_ref() else {
            panic!()
        };
        let id = target.drawing().unwrap().inst_id;
        assert!(targets.insert(id));
        assert_eq!(line.connector.as_ref().unwrap().start_subject_id, id);
    }
}

#[test]
fn ids_owned_by_ole_fallback_outside_source_are_reserved() {
    use rhwp::model::shape::OleShape;
    let mut c = core();
    let mut fallback = OleShape::default();
    fallback.common.instance_id = 1;
    fallback.drawing.inst_id = 3;
    fallback.hwpx_ole_id = Some(4);
    let mut ole = OleShape::default();
    ole.common.instance_id = 500;
    ole.chart_switch_fallback = Some(Box::new(fallback));
    c.document_mut().sections[0].paragraphs[0]
        .controls
        .push(Control::Shape(Box::new(ShapeObject::Ole(Box::new(ole)))));
    let mut rectangle = RectangleShape::default();
    rectangle.common.instance_id = 100;
    rectangle.drawing.inst_id = 200;
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Shape(Box::new(ShapeObject::Rectangle(rectangle))));
    let before = format!("{:?}", c.document().sections[0].paragraphs[0]);
    c.repeat_paragraph_block_native(&request()).unwrap();
    assert_eq!(
        format!("{:?}", c.document().sections[0].paragraphs[0]),
        before
    );
    for pi in [3, 5] {
        let Control::Shape(shape) = &c.document().sections[0].paragraphs[pi].controls[0] else {
            panic!()
        };
        assert!(![0, 1, 2, 3, 4].contains(&shape.common().instance_id));
        assert!(![0, 1, 2, 3, 4].contains(&shape.drawing().unwrap().inst_id));
    }
}

#[test]
fn existing_snapshot_remains_restorable_after_success_and_rejection() {
    let mut c = core();
    let before = format!("{:?}", c.document());
    let snapshot = c.save_snapshot_native();
    c.repeat_paragraph_block_native(&request()).unwrap();
    let mut invalid = request();
    invalid.insert_before = usize::MAX;
    assert!(c.repeat_paragraph_block_native(&invalid).is_err());
    c.restore_snapshot_native(snapshot).unwrap();
    assert_eq!(format!("{:?}", c.document()), before);
    assert_eq!(c.save_snapshot_native(), snapshot + 1);
}

#[test]
fn editing_one_table_copy_through_core_keeps_original_and_other_copy() {
    let mut c =
        DocumentCore::from_bytes(&std::fs::read("samples/hwp_table_test.hwp").unwrap()).unwrap();
    // Mark an actual source as edited before copying; exercise the provenance
    // inheritance path as well as subsequent public cell editing.
    c.insert_text_in_cell_native(0, 3, 0, 0, 0, 0, "source ")
        .unwrap();
    c.copy_control_native(0, 3, &[], 0).unwrap();
    let clipboard = c.get_clipboard_text_native();
    let r = RepeatParagraphBlockRequest {
        source_start: 3,
        source_end: 4,
        insert_before: 4,
        ..request()
    };
    c.repeat_paragraph_block_native(&r).unwrap();
    let original = format!("{:?}", c.document().sections[0].paragraphs[3].controls);
    let other = format!("{:?}", c.document().sections[0].paragraphs[5].controls);
    c.insert_text_in_cell_native(0, 4, 0, 0, 0, 0, "copy only ")
        .unwrap();
    assert_eq!(
        format!("{:?}", c.document().sections[0].paragraphs[3].controls),
        original
    );
    assert_eq!(
        format!("{:?}", c.document().sections[0].paragraphs[5].controls),
        other
    );
    assert_eq!(c.get_clipboard_text_native(), clipboard);
    let Control::Table(t) = &c.document().sections[0].paragraphs[4].controls[0] else {
        panic!()
    };
    assert!(t.cells[0].paragraphs[0]
        .text
        .starts_with("copy only source "));
}
