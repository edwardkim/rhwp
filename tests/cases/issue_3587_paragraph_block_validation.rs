//! Strict block checks are separate from legacy clipboard external-ref behavior.
use rhwp::{
    document_core::{
        DocumentCore, ParagraphBlockLimits, ParagraphBlockPathStep as Step,
        RepeatParagraphBlockRequest,
    },
    model::{
        control::{Control, Field, FieldType},
        paragraph::{ColumnBreakType, FieldRange, OrphanFieldEnd, Paragraph},
        shape::{ConnectorData, DrawingObjAttr, LineShape, RectangleShape, ShapeObject, TextBox},
    },
};

fn core() -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    c.document_mut().sections[0].paragraphs = vec![Paragraph::default(); 4];
    c
}
fn request() -> RepeatParagraphBlockRequest {
    RepeatParagraphBlockRequest {
        section_index: 0,
        source_start: 1,
        source_end: 3,
        insert_before: 3,
        count: 1,
        limits: ParagraphBlockLimits::default(),
    }
}
fn begin(id: u32) -> Control {
    Control::Field(Field {
        field_type: FieldType::ClickHere,
        field_id: id,
        ctrl_id: u32::from_le_bytes(*b"klc%"),
        ctrl_data_name: Some("same-name".into()),
        ..Default::default()
    })
}
fn end(id: u32) -> OrphanFieldEnd {
    OrphanFieldEnd {
        begin_id_ref: id,
        ..Default::default()
    }
}
fn add_closed(c: &mut DocumentCore, pi: usize, id: u32) {
    let p = &mut c.document_mut().sections[0].paragraphs[pi];
    let control_idx = p.controls.len();
    p.controls.push(begin(id));
    p.field_ranges.push(FieldRange {
        control_idx,
        ..Default::default()
    });
}
fn rectangle(common: u32, drawing: u32) -> ShapeObject {
    let mut r = RectangleShape::default();
    r.common.instance_id = common;
    r.drawing.inst_id = drawing;
    ShapeObject::Rectangle(r)
}
fn line(target: u32) -> ShapeObject {
    let mut l = LineShape::default();
    l.common.instance_id = 900;
    l.drawing.inst_id = 901;
    l.connector = Some(ConnectorData {
        start_subject_id: target,
        ..Default::default()
    });
    ShapeObject::Line(l)
}
fn shape(c: &mut DocumentCore, pi: usize, s: ShapeObject) {
    c.document_mut().sections[0].paragraphs[pi]
        .controls
        .push(Control::Shape(Box::new(s)));
}
fn code(c: &DocumentCore) -> String {
    c.validate_paragraph_block_native(&request())
        .unwrap_err()
        .code
}

#[test]
fn plain_and_closed_fields_preserve_same_names_and_shared_fieldid() {
    let mut c = core();
    add_closed(&mut c, 1, 10);
    add_closed(&mut c, 2, 20);
    for pi in [1, 2] {
        let Control::Field(f) = &mut c.document_mut().sections[0].paragraphs[pi].controls[0] else {
            panic!()
        };
        f.instance_id = Some(99);
    }
    let before = format!("{:?}", c.document());
    assert!(c.validate_paragraph_block_native(&request()).is_ok());
    assert_eq!(format!("{:?}", c.document()), before);
}

#[test]
fn cross_paragraph_field_must_close_inside_the_same_owned_list() {
    let mut c = core();
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(begin(10));
    c.document_mut().sections[0].paragraphs[2]
        .orphan_field_ends
        .push(end(10));
    assert!(c.validate_paragraph_block_native(&request()).is_ok());
    c.document_mut().sections[0].paragraphs[2]
        .orphan_field_ends
        .clear();
    c.document_mut().sections[0].paragraphs[3]
        .orphan_field_ends
        .push(end(10));
    assert_eq!(code(&c), "externalField");
}

#[test]
fn inside_end_with_outside_begin_is_rejected() {
    let mut c = core();
    c.document_mut().sections[0].paragraphs[0]
        .controls
        .push(begin(10));
    c.document_mut().sections[0].paragraphs[2]
        .orphan_field_ends
        .push(end(10));
    assert_eq!(code(&c), "externalField");
}

#[test]
fn missing_duplicate_and_reversed_ends_are_rejected() {
    let mut c = core();
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(begin(10));
    assert_eq!(code(&c), "fieldClosure");
    c.document_mut().sections[0].paragraphs[2].orphan_field_ends = vec![end(10), end(10)];
    assert_eq!(code(&c), "fieldClosure");
    c.document_mut().sections[0].paragraphs[2]
        .orphan_field_ends
        .clear();
    c.document_mut().sections[0].paragraphs.swap(1, 2);
    c.document_mut().sections[0].paragraphs[1]
        .orphan_field_ends
        .push(end(10));
    assert_eq!(code(&c), "fieldOrder");
}

#[test]
fn duplicate_begin_outside_source_is_not_silently_resolved() {
    let mut c = core();
    add_closed(&mut c, 1, 10);
    add_closed(&mut c, 0, 10);
    assert_eq!(code(&c), "ambiguousField");
}

#[test]
fn invalid_local_field_control_and_text_positions_are_rejected() {
    let mut c = core();
    add_closed(&mut c, 1, 10);
    c.document_mut().sections[0].paragraphs[1].field_ranges[0].control_idx = 9;
    assert_eq!(code(&c), "invalidFieldRange");
    c.document_mut().sections[0].paragraphs[1].field_ranges[0].control_idx = 0;
    c.document_mut().sections[0].paragraphs[1].field_ranges[0].end_char_idx = 1;
    assert_eq!(code(&c), "invalidFieldRange");
}

#[test]
fn nested_field_cannot_end_in_outer_paragraph_list() {
    let mut c = core();
    let mut p = Paragraph::default();
    p.controls.push(begin(10));
    let r = RectangleShape {
        drawing: DrawingObjAttr {
            text_box: Some(TextBox {
                paragraphs: vec![p],
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    shape(&mut c, 1, ShapeObject::Rectangle(r));
    c.document_mut().sections[0].paragraphs[2]
        .orphan_field_ends
        .push(end(10));
    assert_eq!(code(&c), "fieldOrder");
}

#[test]
fn connectors_resolve_common_alias_and_drawing_identity() {
    for target in [100, 101, 200] {
        let mut c = core();
        shape(&mut c, 1, rectangle(100, 200));
        shape(&mut c, 2, line(target));
        assert!(
            c.validate_paragraph_block_native(&request()).is_ok(),
            "target {target}"
        );
    }
}

#[test]
fn outbound_and_inbound_connector_boundaries_are_rejected() {
    let mut c = core();
    shape(&mut c, 0, rectangle(100, 200));
    shape(&mut c, 1, line(200));
    assert_eq!(code(&c), "externalConnector");
    let mut c = core();
    shape(&mut c, 1, rectangle(100, 200));
    shape(&mut c, 3, line(200));
    assert_eq!(code(&c), "externalConnector");
}

#[test]
fn ambiguous_global_connector_target_is_rejected() {
    let mut c = core();
    shape(&mut c, 0, rectangle(100, 200));
    shape(&mut c, 1, rectangle(300, 200));
    shape(&mut c, 2, line(200));
    assert_eq!(code(&c), "ambiguousConnector");
}

#[test]
fn unsupported_control_returns_exact_nested_typed_path() {
    let mut c = core();
    let mut p = Paragraph::default();
    p.controls.push(Control::Form(Default::default()));
    let r = RectangleShape {
        drawing: DrawingObjAttr {
            text_box: Some(TextBox {
                paragraphs: vec![p],
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    shape(&mut c, 1, ShapeObject::Rectangle(r));
    let err = c.validate_paragraph_block_native(&request()).unwrap_err();
    assert_eq!(err.code, "unsupported");
    assert_eq!(err.detail, "Form");
    assert_eq!(
        err.path,
        [
            Step::Paragraph(0),
            Step::Control(0),
            Step::Shape,
            Step::TextBox,
            Step::Paragraph(0),
            Step::Control(0)
        ]
    );
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(json["path"][3]["kind"], "textBox");
}

#[test]
fn page_and_column_breaks_are_allowed_but_section_bits_are_not() {
    let mut c = core();
    for (kind, raw, ok) in [
        (ColumnBreakType::Page, 4, true),
        (ColumnBreakType::Column, 8, true),
        (ColumnBreakType::Section, 1, false),
        (ColumnBreakType::Page, 5, false),
    ] {
        let p = &mut c.document_mut().sections[0].paragraphs[1];
        p.column_type = kind;
        p.raw_break_type = raw;
        assert_eq!(c.validate_paragraph_block_native(&request()).is_ok(), ok);
    }
}

#[test]
fn zero_copies_skip_unsupported_content_but_not_bad_addresses() {
    let mut c = core();
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Unknown(Default::default()));
    let mut r = request();
    r.count = 0;
    c.validate_paragraph_block_native(&r).unwrap();
    r.insert_before = 2;
    assert!(c.validate_paragraph_block_native(&r).is_err());
}

#[test]
fn raw_header_tracking_and_opaque_payloads_are_rejected_not_erased() {
    let mut c = core();
    for len in [0, 10, 12] {
        c.document_mut().sections[0].paragraphs[1].raw_header_extra = vec![0; len];
        assert!(c.validate_paragraph_block_native(&request()).is_ok());
    }
    c.document_mut().sections[0].paragraphs[1].raw_header_extra[10] = 1;
    let before = format!("{:?}", c.document());
    assert_eq!(code(&c), "unsupported");
    assert_eq!(format!("{:?}", c.document()), before);
}

#[test]
fn reference_scan_arrays_are_bounded_even_outside_the_source() {
    let mut c = core();
    c.document_mut().sections[0].paragraphs[0].orphan_field_ends = vec![end(99); 20];
    let mut r = request();
    r.limits.max_document_nodes = 10;
    let e = c.validate_paragraph_block_native(&r).unwrap_err();
    assert!(e.detail.contains("reference scan budget"));
}

#[test]
fn success_and_rejection_leave_editor_and_clipboard_unchanged() {
    let mut c = core();
    c.create_table_native(0, 1, 0, 1, 1).unwrap();
    let (pi, ci) = c.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, p)| {
            p.controls
                .iter()
                .position(|x| matches!(x, Control::Table(_)))
                .map(|ci| (pi, ci))
        })
        .unwrap();
    c.copy_control_native(0, pi, &[], ci).unwrap();
    let mut r = request();
    r.source_start = pi;
    r.source_end = pi + 1;
    r.insert_before = pi;
    let before = format!("{:?}", c.document());
    let events = c.serialize_event_log();
    let clipboard = c.get_clipboard_text_native();
    assert!(c.validate_paragraph_block_native(&r).is_ok());
    r.limits.max_nodes = 1;
    assert!(c.validate_paragraph_block_native(&r).is_err());
    assert_eq!(format!("{:?}", c.document()), before);
    assert_eq!(c.serialize_event_log(), events);
    assert_eq!(c.get_clipboard_text_native(), clipboard);
    assert!(c.has_internal_clipboard_native());
}

#[test]
fn name_only_ctrl_data_is_preserved_but_extra_items_are_rejected() {
    let mut c = core();
    add_closed(&mut c, 1, 10);
    let name: Vec<u16> = "same-name".encode_utf16().collect();
    let mut raw = vec![0x1b, 2, 1, 0, 0, 0, 0, 0x40, 1, 0];
    raw.extend_from_slice(&(name.len() as u16).to_le_bytes());
    for ch in name {
        raw.extend_from_slice(&ch.to_le_bytes());
    }
    c.document_mut().sections[0].paragraphs[1].ctrl_data_records = vec![Some(raw)];
    let before = format!("{:?}", c.document());
    c.validate_paragraph_block_native(&request()).unwrap();
    assert_eq!(format!("{:?}", c.document()), before);
    c.document_mut().sections[0].paragraphs[1].ctrl_data_records[0]
        .as_mut()
        .unwrap()
        .push(0);
    assert_eq!(code(&c), "unsupported");
}

#[test]
fn table_common_and_cell_raw_boundaries_follow_existing_writer() {
    let mut c = core();
    c.create_table_native(0, 1, 0, 1, 1).unwrap();
    let (pi, ci) = c.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, p)| {
            p.controls
                .iter()
                .position(|x| matches!(x, Control::Table(_)))
                .map(|ci| (pi, ci))
        })
        .unwrap();
    let mut r = request();
    r.source_start = pi;
    r.source_end = pi + 1;
    r.insert_before = pi;
    for len in [36, 38, 40, 42] {
        let Control::Table(t) = &mut c.document_mut().sections[0].paragraphs[pi].controls[ci]
        else {
            panic!()
        };
        t.raw_ctrl_data = vec![0; len];
        t.cells[0].raw_list_extra = vec![0; 13];
        c.validate_paragraph_block_native(&r).unwrap();
    }
    let Control::Table(t) = &mut c.document_mut().sections[0].paragraphs[pi].controls[ci] else {
        panic!()
    };
    t.cells[0].raw_list_extra.push(1);
    let err = c.validate_paragraph_block_native(&r).unwrap_err();
    assert_eq!(err.code, "unsupported");
    assert_eq!(err.path.last(), Some(&Step::Cell(0)));
}

#[test]
fn unvalidated_range_tags_and_field_parameter_extensions_are_rejected() {
    let mut c = core();
    c.document_mut().sections[0].paragraphs[1]
        .range_tags
        .push(Default::default());
    assert_eq!(code(&c), "unsupported");
    c.document_mut().sections[0].paragraphs[1]
        .range_tags
        .clear();
    add_closed(&mut c, 1, 10);
    let Control::Field(f) = &mut c.document_mut().sections[0].paragraphs[1].controls[0] else {
        panic!()
    };
    f.raw_parameters_xml = Some("<parameters/>".into());
    assert_eq!(code(&c), "unsupported");
}
