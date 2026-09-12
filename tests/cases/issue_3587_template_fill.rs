//! Native repeat+fill contracts; save/reopen is not a Hancom visual verdict.
use rhwp::{
    document_core::{
        DocumentCore, ParagraphBlockLimits, ParagraphBlockPathStep as Step,
        RepeatParagraphBlockRequest, TemplateBinding, TemplateFillRequest, TemplateFillTarget,
    },
    model::{
        control::{Control, Field, FieldType},
        paragraph::{FieldRange, Paragraph},
    },
};
use std::collections::BTreeMap;

fn core() -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    c.document_mut().sections[0].paragraphs = ["before", "원😀형", "", "after"]
        .map(|s| {
            let mut p = Paragraph::default();
            p.insert_text_at(0, s);
            p
        })
        .to_vec();
    c
}
fn request() -> TemplateFillRequest {
    TemplateFillRequest {
        block: RepeatParagraphBlockRequest {
            section_index: 0,
            source_start: 1,
            source_end: 3,
            insert_before: 3,
            count: 2,
            limits: ParagraphBlockLimits::default(),
        },
        bindings: vec![TemplateBinding {
            key: "title".into(),
            target: TemplateFillTarget::TextRange {
                path: vec![Step::Paragraph(0)],
                start: 0,
                end: 3,
            },
        }],
        records: vec![
            BTreeMap::from([("title".into(), "첫😀번째".into())]),
            BTreeMap::from([("title".into(), "두번째\n본문".into())]),
        ],
    }
}
fn state(c: &DocumentCore) -> (String, String, String) {
    (
        format!("{:?}", c.document()),
        format!("{:?}", c.serialize_event_log()),
        format!("{:?}", c.get_clipboard_text_native()),
    )
}

#[test]
fn repeat_fill_preserves_source_neighbors_and_intentional_empty_paragraphs() {
    let mut c = core();
    let r = request();
    let clipboard = c.get_clipboard_text_native();
    let events: serde_json::Value = serde_json::from_str(&c.serialize_event_log()).unwrap();
    let result = c.repeat_and_fill_paragraph_block_native(&r).unwrap();
    let after: serde_json::Value = serde_json::from_str(&c.serialize_event_log()).unwrap();
    assert_eq!(
        after["events"].as_array().unwrap().len(),
        events["events"].as_array().unwrap().len() + 1
    );
    assert_eq!(result.inserted, 3..7);
    assert_eq!(result.copies.len(), 2);
    let texts: Vec<_> = c.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect();
    assert_eq!(
        texts,
        [
            "before",
            "원😀형",
            "",
            "첫😀번째",
            "",
            "두번째\n본문",
            "",
            "after"
        ]
    );
    assert_eq!(c.get_clipboard_text_native(), clipboard);
    c.repeat_and_fill_paragraph_block_native(&r).unwrap();
    assert_eq!(c.document().sections[0].paragraphs.len(), 12);
}

#[test]
fn nested_textbox_fill_preserves_its_owner_and_unselected_paragraphs() {
    use rhwp::model::shape::{RectangleShape, ShapeObject, TextBox};
    let mut c = core();
    let mut r = request();
    let source = c.document().sections[0].paragraphs[1].clone();
    let mut shape = RectangleShape::default();
    shape.common.instance_id = 50;
    shape.drawing.inst_id = 60;
    shape.drawing.text_box = Some(TextBox {
        paragraphs: vec![source, Paragraph::default()],
        ..Default::default()
    });
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Shape(Box::new(ShapeObject::Rectangle(shape))));
    r.bindings[0].target = TemplateFillTarget::TextRange {
        path: vec![
            Step::Paragraph(0),
            Step::Control(0),
            Step::Shape,
            Step::TextBox,
            Step::Paragraph(0),
        ],
        start: 0,
        end: 3,
    };
    c.repeat_and_fill_paragraph_block_native(&r).unwrap();
    for (pi, expected) in [(1, "원😀형"), (3, "첫😀번째"), (5, "두번째\n본문")] {
        let host = &c.document().sections[0].paragraphs[pi];
        assert_eq!(host.text, "원😀형");
        let Control::Shape(s) = &host.controls[0] else {
            panic!("owner lost")
        };
        let text = &s.drawing().unwrap().text_box.as_ref().unwrap().paragraphs;
        assert_eq!(text.len(), 2);
        assert_eq!(text[0].text, expected);
        assert!(text[1].text.is_empty());
    }
}

#[test]
fn final_record_error_and_zero_records_never_change_live_state() {
    let mut c = core();
    let mut r = request();
    let before = state(&c);
    r.records[1].clear();
    assert!(c.repeat_and_fill_paragraph_block_native(&r).is_err());
    assert_eq!(state(&c), before);
    r.records.clear();
    r.block.count = 0;
    assert!(c
        .repeat_and_fill_paragraph_block_native(&r)
        .unwrap()
        .copies
        .is_empty());
    assert_eq!(state(&c), before);
}

#[test]
fn scalar_ranges_are_resolved_before_longer_replacements() {
    let mut c = core();
    let mut r = request();
    r.bindings[0].target = TemplateFillTarget::TextRange {
        path: vec![Step::Paragraph(0)],
        start: 0,
        end: 1,
    };
    r.bindings.push(TemplateBinding {
        key: "tail".into(),
        target: TemplateFillTarget::TextRange {
            path: vec![Step::Paragraph(0)],
            start: 2,
            end: 3,
        },
    });
    r.records[0].insert("tail".into(), "끝".into());
    r.records[1].insert("title".into(), String::new());
    r.records[1].insert("tail".into(), "e\u{301}".into());
    c.repeat_and_fill_paragraph_block_native(&r).unwrap();
    assert_eq!(c.document().sections[0].paragraphs[3].text, "첫😀번째😀끝");
    assert_eq!(c.document().sections[0].paragraphs[5].text, "😀e\u{301}");
}

#[test]
fn field_value_range_and_filled_state_are_preserved_on_copies() {
    let mut c = core();
    let p = &mut c.document_mut().sections[0].paragraphs[1];
    p.controls.push(Control::Field(Field {
        field_type: FieldType::ClickHere,
        field_id: 99,
        ctrl_id: u32::from_le_bytes(*b"klc%"),
        ctrl_data_name: Some("title".into()),
        ..Default::default()
    }));
    p.field_ranges.push(FieldRange {
        start_char_idx: 0,
        end_char_idx: 3,
        control_idx: 0,
        ..Default::default()
    });
    p.char_offsets = vec![8, 9, 11];
    p.char_count = 21;
    let mut r = request();
    r.bindings[0].target = TemplateFillTarget::Field {
        path: vec![Step::Paragraph(0)],
        field_range_index: 0,
    };
    c.repeat_and_fill_paragraph_block_native(&r).unwrap();
    for (pi, value) in [(3, "첫😀번째"), (5, "두번째\n본문")] {
        let p = &c.document().sections[0].paragraphs[pi];
        assert_eq!(p.text, value);
        assert_eq!(p.field_ranges[0].end_char_idx, value.chars().count());
        let Control::Field(f) = &p.controls[0] else {
            panic!("field lost")
        };
        assert_ne!(f.field_id, 99);
        assert_ne!(f.properties & (1 << 15), 0);
    }
    assert_eq!(c.document().sections[0].paragraphs[1].text, "원😀형");
}

#[test]
fn real_labnote_filled_copies_save_and_reopen_in_both_formats() {
    let mut c =
        DocumentCore::from_bytes(&std::fs::read("samples/rnote/labnote-001.hwp").unwrap()).unwrap();
    let mut r = request();
    r.block.source_start = 12;
    r.block.source_end = 13;
    r.block.insert_before = 13;
    r.bindings[0].target = TemplateFillTarget::TextRange {
        path: vec![
            Step::Paragraph(0),
            Step::Control(1),
            Step::Cell(5),
            Step::Paragraph(0),
        ],
        start: 0,
        end: 0,
    };
    c.repeat_and_fill_paragraph_block_native(&r).unwrap();
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        for (pi, text) in [(13, "첫😀번째"), (14, "두번째\n본문")] {
            let Control::Table(t) = &reopened.document().sections[0].paragraphs[pi].controls[1]
            else {
                panic!("table lost")
            };
            assert_eq!(t.cells[5].paragraphs[0].text, text);
        }
        let Control::Table(t) = &reopened.document().sections[0].paragraphs[12].controls[1] else {
            panic!("source table lost")
        };
        assert!(t.cells[5].paragraphs[0].text.is_empty());
    }
}

#[test]
fn derived_growth_budget_failure_precedes_insertion() {
    let mut c = core();
    let mut r = request();
    r.block.limits.max_structure_bytes = 64 * 1024;
    r.records[1].insert("title".into(), "x".repeat(4096));
    c.validate_template_fill_native(&r).unwrap();
    let before = state(&c);
    assert!(c
        .repeat_and_fill_paragraph_block_native(&r)
        .unwrap_err()
        .to_string()
        .contains("budget"));
    assert_eq!(state(&c), before);
}

#[test]
fn late_staging_failure_discards_earlier_filled_copy() {
    let mut c = core();
    let mut r = request();
    let mut p = Paragraph::default();
    p.insert_text_at(0, &"a".repeat(100));
    c.document_mut().sections[0].paragraphs[1] = p;
    r.block.limits.max_structure_bytes = 800_000;
    r.bindings = (0..100)
        .map(|i| TemplateBinding {
            key: i.to_string(),
            target: TemplateFillTarget::TextRange {
                path: vec![Step::Paragraph(0)],
                start: i,
                end: i + 1,
            },
        })
        .collect();
    r.records = vec![
        (0..100).map(|i| (i.to_string(), "a".into())).collect(),
        (0..100).map(|i| (i.to_string(), "x".repeat(200))).collect(),
    ];
    c.validate_template_fill_native(&r).unwrap();
    // The first copy alone fits and is actually editable.
    let mut first = r.clone();
    first.block.count = 1;
    first.records.truncate(1);
    core_for_first_copy(&first);
    let before = state(&c);
    let error = c.repeat_and_fill_paragraph_block_native(&r).unwrap_err();
    assert!(error.to_string().contains("fill byte budget"), "{error}");
    assert!(error.to_string().contains("fill copy 1"), "{error}");
    assert_eq!(state(&c), before);
}

fn core_for_first_copy(r: &TemplateFillRequest) {
    let mut c = core();
    let mut p = Paragraph::default();
    p.insert_text_at(0, &"a".repeat(100));
    c.document_mut().sections[0].paragraphs[1] = p;
    c.repeat_and_fill_paragraph_block_native(r).unwrap();
    assert_eq!(c.document().sections[0].paragraphs[3].text, "a".repeat(100));
}
