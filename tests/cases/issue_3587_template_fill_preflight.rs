//! C1 selection/data preflight. These assertions do not certify mutation or rendering.
use rhwp::{
    document_core::{
        DocumentCore, ParagraphBlockLimits, ParagraphBlockPathStep as Step,
        RepeatParagraphBlockRequest, TemplateBinding, TemplateFillRequest, TemplateFillTarget,
    },
    model::{
        control::{Control, Field, FieldType},
        paragraph::{FieldRange, Paragraph},
        shape::{RectangleShape, ShapeObject, TextBox},
    },
};
use std::collections::BTreeMap;

fn core() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.document_mut().sections[0].paragraphs = ["before", "원😀형", "", "after"]
        .map(|text| Paragraph {
            text: text.into(),
            ..Default::default()
        })
        .to_vec();
    core
}

fn binding(key: &str, start: usize, end: usize) -> TemplateBinding {
    TemplateBinding {
        key: key.into(),
        target: TemplateFillTarget::TextRange {
            path: vec![Step::Paragraph(0)],
            start,
            end,
        },
    }
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
        bindings: vec![binding("title", 0, 3)],
        records: vec![
            BTreeMap::from([("title".into(), "첫번째".into())]),
            BTreeMap::from([("title".into(), "두번째\n본문".into())]),
        ],
    }
}

fn unchanged(core: &DocumentCore, request: &TemplateFillRequest, code: Option<&str>) {
    let before = format!("{:?}", core.document());
    let events = core.serialize_event_log();
    let clipboard = core.get_clipboard_text_native();
    let result = core.validate_template_fill_native(request);
    if let Some(code) = code {
        assert_eq!(result.unwrap_err().code, code);
    } else {
        result.unwrap();
    }
    assert_eq!(format!("{:?}", core.document()), before);
    assert_eq!(core.serialize_event_log(), events);
    assert_eq!(core.get_clipboard_text_native(), clipboard);
}

#[test]
fn all_records_are_checked_without_copying_or_changing_originals() {
    let core = core();
    let request = request();
    let preview = core.validate_template_fill_native(&request).unwrap();
    assert_eq!(preview.target_count, 2);
    assert_eq!(preview.block.added_paragraphs, 4);
    assert_eq!(preview.replacement_text_bytes, "첫번째두번째\n본문".len());
    unchanged(&core, &request, None);
}

#[test]
fn final_record_missing_extra_and_empty_values_are_distinct() {
    for mode in 0..3 {
        let mut request = request();
        match mode {
            0 => {
                request.records[1].clear();
            }
            1 => {
                request.records[1].insert("extra".into(), "wrong".into());
            }
            _ => {
                request.records[1].insert("title".into(), String::new());
            }
        }
        unchanged(&core(), &request, (mode != 2).then_some("fillKey"));
    }
}

#[test]
fn unicode_ranges_use_scalars_not_utf16_or_byte_offsets() {
    let mut request = request();
    request.bindings[0] = binding("title", 1, 2); // Only the emoji.
    unchanged(&core(), &request, None);
    request.bindings[0] = binding("title", 0, 4); // UTF-16 length, not scalar count.
    unchanged(&core(), &request, Some("fillRange"));
}

#[test]
fn adjacent_ranges_are_valid_but_overlaps_and_duplicate_insertions_are_not() {
    for (start, end, expected) in [
        (2, 3, None),
        (1, 3, Some("fillOverlap")),
        (2, 2, Some("fillOverlap")),
    ] {
        let mut request = request();
        request.bindings = vec![binding("title", 0, 2), binding("body", start, end)];
        for record in &mut request.records {
            record.insert("body".into(), "value".into());
        }
        unchanged(&core(), &request, expected);
    }
}

#[test]
fn duplicate_keys_bad_paths_and_record_counts_are_rejected() {
    let mut request = request();
    request.bindings.push(binding("title", 1, 2));
    unchanged(&core(), &request, Some("fillKey"));
    request.bindings.pop();
    request.bindings[0].target = TemplateFillTarget::TextRange {
        path: vec![Step::Paragraph(9)],
        start: 0,
        end: 0,
    };
    unchanged(&core(), &request, Some("fillPath"));
    request.records.pop();
    unchanged(&core(), &request, Some("fillCardinality"));
}

#[test]
fn zero_records_skip_source_support_but_validate_address_and_options() {
    let mut core = core();
    // Source unsupported by nonzero B, but a zero request must not clone/scan it.
    core.document_mut().sections[0].paragraphs[1].raw_break_type = 1;
    let mut request = request();
    request.records.clear();
    request.block.count = 0;
    unchanged(&core, &request, None);
    request.block.insert_before = 99;
    unchanged(&core, &request, Some("budgetOrAddress"));
}

#[test]
fn input_depth_expanded_count_and_text_budget_are_bounded() {
    let mut request = request();
    request.bindings[0].target = TemplateFillTarget::TextRange {
        path: vec![Step::Paragraph(0); 65],
        start: 0,
        end: 0,
    };
    unchanged(&core(), &request, Some("fillBudget"));
    request = self::request();
    request.block.count = 1000;
    request.records = vec![request.records[0].clone(); 1000];
    request.bindings = vec![binding("title", 0, 0); 11];
    unchanged(&core(), &request, Some("fillBudget"));
    request = self::request();
    request.records[1].insert("title".into(), "x".repeat(8 * 1024 * 1024));
    unchanged(&core(), &request, Some("fillBudget"));
}

fn field_core() -> DocumentCore {
    let mut core = core();
    let p = &mut core.document_mut().sections[0].paragraphs[1];
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
    core
}

#[test]
fn explicit_closed_field_is_distinct_from_plain_text_range() {
    let core = field_core();
    let mut request = request();
    unchanged(&core, &request, Some("fillField"));
    request.bindings[0].target = TemplateFillTarget::Field {
        path: vec![Step::Paragraph(0)],
        field_range_index: 0,
    };
    unchanged(&core, &request, None);
}

#[test]
fn nested_textbox_selection_uses_owned_path_not_global_paragraph_number() {
    let mut core = core();
    let mut rectangle = RectangleShape::default();
    rectangle.common.instance_id = 50;
    rectangle.drawing.inst_id = 60;
    rectangle.drawing.text_box = Some(TextBox {
        paragraphs: vec![Paragraph {
            text: "원😀형".into(),
            ..Default::default()
        }],
        ..Default::default()
    });
    core.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Shape(Box::new(ShapeObject::Rectangle(rectangle))));
    let mut request = request();
    request.bindings[0].target = TemplateFillTarget::TextRange {
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
    unchanged(&core, &request, None);
    request.bindings[0] = binding("title", 0, 3);
    unchanged(&core, &request, Some("fillControl"));
}

#[test]
fn real_labnote_cell_target_preflight_preserves_template() {
    let bytes = std::fs::read("samples/rnote/labnote-001.hwp").unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let mut request = request();
    request.block.source_start = 12;
    request.block.source_end = 13;
    request.block.insert_before = 13;
    request.bindings[0].target = TemplateFillTarget::TextRange {
        path: vec![
            Step::Paragraph(0),
            Step::Control(1),
            Step::Cell(5),
            Step::Paragraph(0),
        ],
        start: 0,
        end: 0,
    };
    unchanged(&core, &request, None);
}
