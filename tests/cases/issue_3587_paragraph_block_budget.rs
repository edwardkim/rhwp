//! #3587 B1: pre-clone request budgets do not mutate the editor.
use rhwp::{
    document_core::{DocumentCore, ParagraphBlockLimits, RepeatParagraphBlockRequest},
    model::{
        control::Control,
        paragraph::Paragraph,
        shape::{RectangleShape, ShapeObject, TextBox},
    },
};

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.document_mut().sections[0].paragraphs = vec![Paragraph::default(); 4];
    core
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

#[test]
fn pre_edit_boundary_coordinates_and_empty_paragraphs_are_preserved() {
    let core = blank();
    for (at, start, end) in [(0, 5, 7), (1, 5, 7), (3, 1, 3), (4, 1, 3)] {
        let mut r = request();
        r.insert_before = at;
        r.count = 2;
        let b = core.paragraph_block_budget_native(&r).unwrap();
        assert_eq!(b.added_paragraphs, 4);
        assert_eq!((b.source_start_after, b.source_end_after), (start, end));
        assert_eq!(b.inserted_end, at + 4);
        assert_eq!(core.document().sections[0].paragraphs.len(), 4);
    }
}

#[test]
fn invalid_addresses_are_rejected_even_for_zero_copies() {
    let core = blank();
    for count in [0, 1] {
        let base = RepeatParagraphBlockRequest { count, ..request() };
        for r in [
            RepeatParagraphBlockRequest {
                section_index: 1,
                ..base.clone()
            },
            RepeatParagraphBlockRequest {
                source_start: 3,
                ..base.clone()
            },
            RepeatParagraphBlockRequest {
                source_end: 5,
                ..base.clone()
            },
            RepeatParagraphBlockRequest {
                insert_before: 2,
                ..base.clone()
            },
            RepeatParagraphBlockRequest {
                insert_before: 5,
                ..base.clone()
            },
        ] {
            assert!(core.paragraph_block_budget_native(&r).is_err());
        }
    }
}

#[test]
fn zero_copies_skip_content_and_document_scans_without_mutating_state() {
    let mut core = blank();
    core.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Form(Default::default()));
    let before = format!("{:?}", core.document());
    let events = core.serialize_event_log();
    let clipboard = core.get_clipboard_text_native();
    let mut r = request();
    r.count = 0;
    r.limits.max_document_nodes = 1;
    let b = core.paragraph_block_budget_native(&r).unwrap();
    assert_eq!(
        (
            b.added_paragraphs,
            b.added_nodes,
            b.structure_bytes,
            b.mapping_bytes,
            b.document_nodes
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(format!("{:?}", core.document()), before);
    assert_eq!(core.serialize_event_log(), events);
    assert_eq!(core.get_clipboard_text_native(), clipboard);
    r.count = 1;
    assert!(core
        .paragraph_block_budget_native(&r)
        .unwrap_err()
        .to_string()
        .contains("Form"));
}

#[test]
fn caller_cannot_disable_or_raise_any_limit() {
    let core = blank();
    for name in [
        "maxCopies",
        "maxParagraphs",
        "maxNodes",
        "maxDepth",
        "maxStructureBytes",
        "maxMappingBytes",
        "maxDocumentNodes",
    ] {
        for value in [0, usize::MAX] {
            let mut json = serde_json::to_value(request()).unwrap();
            json["limits"][name] = value.into();
            let r = serde_json::from_value(json).unwrap();
            assert!(
                core.paragraph_block_budget_native(&r).is_err(),
                "{name}={value}"
            );
        }
    }
}

#[test]
fn copy_and_paragraph_limits_and_arithmetic_are_checked() {
    let core = blank();
    for count in [1001, usize::MAX] {
        assert!(core
            .paragraph_block_budget_native(&RepeatParagraphBlockRequest { count, ..request() })
            .is_err());
    }
    let mut r = request();
    r.count = 2;
    r.limits.max_paragraphs = 3;
    assert!(core.paragraph_block_budget_native(&r).is_err());
    r.limits.max_paragraphs = 4;
    assert!(core.paragraph_block_budget_native(&r).is_ok());
}

#[test]
fn byte_node_mapping_and_document_budgets_accept_exact_limit_reject_one_less() {
    let core = blank();
    let b = core.paragraph_block_budget_native(&request()).unwrap();
    for (name, cost) in [
        ("maxStructureBytes", b.structure_bytes),
        ("maxNodes", b.added_nodes),
        ("maxMappingBytes", b.mapping_bytes),
        ("maxDocumentNodes", b.document_nodes),
    ] {
        for delta in [0, 1] {
            let mut json = serde_json::to_value(request()).unwrap();
            json["limits"][name] = (cost - delta).into();
            let r = serde_json::from_value(json).unwrap();
            assert_eq!(
                core.paragraph_block_budget_native(&r).is_ok(),
                delta == 0,
                "{name}"
            );
        }
    }
}

#[test]
fn text_raw_and_serde_skipped_vpos_are_charged() {
    let mut core = blank();
    let original = core
        .paragraph_block_budget_native(&request())
        .unwrap()
        .structure_bytes;
    core.document_mut().sections[0].paragraphs[1].text = "한글".repeat(100);
    let text = core
        .paragraph_block_budget_native(&request())
        .unwrap()
        .structure_bytes;
    assert!(text >= original + 600);
    core.document_mut().sections[0].paragraphs[1].raw_header_extra = vec![0; 1024];
    let raw = core
        .paragraph_block_budget_native(&request())
        .unwrap()
        .structure_bytes;
    assert!(raw >= text + 1024);
    core.document_mut().sections[0].paragraphs[1].source_line_seg_vertical_pos =
        Some(vec![42; 512]);
    let skipped = core
        .paragraph_block_budget_native(&request())
        .unwrap()
        .structure_bytes;
    assert_eq!(skipped, raw + 512 * 4);
}

#[test]
fn nested_textbox_paragraphs_and_skipped_buffers_are_charged() {
    let mut core = blank();
    let shape = RectangleShape {
        drawing: rhwp::model::shape::DrawingObjAttr {
            text_box: Some(TextBox {
                paragraphs: vec![Paragraph::default()],
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    core.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Shape(Box::new(ShapeObject::Rectangle(shape))));
    let before = core.paragraph_block_budget_native(&request()).unwrap();
    // body paragraph -> control -> shape -> textbox -> nested paragraph
    assert_eq!(before.owned_depth, 5);
    assert_eq!(before.added_nodes, 6);
    let Control::Shape(shape) = &mut core.document_mut().sections[0].paragraphs[1].controls[0]
    else {
        panic!()
    };
    shape
        .drawing_mut()
        .unwrap()
        .text_box
        .as_mut()
        .unwrap()
        .paragraphs[0]
        .source_line_seg_vertical_pos = Some(vec![1; 16]);
    let after = core.paragraph_block_budget_native(&request()).unwrap();
    assert_eq!(after.structure_bytes, before.structure_bytes + 64);
    let mut r = request();
    r.limits.max_depth = 4;
    assert!(core.paragraph_block_budget_native(&r).is_err());
    r.limits.max_depth = 5;
    assert!(core.paragraph_block_budget_native(&r).is_ok());
}

#[test]
fn repeated_budget_is_linear_but_document_scan_is_once_per_request() {
    let core = blank();
    let one = core.paragraph_block_budget_native(&request()).unwrap();
    for count in [10, 100] {
        let b = core
            .paragraph_block_budget_native(&RepeatParagraphBlockRequest { count, ..request() })
            .unwrap();
        assert_eq!(b.structure_bytes, one.structure_bytes * count);
        assert_eq!(b.added_nodes, one.added_nodes * count);
        assert_eq!(b.mapping_bytes, one.mapping_bytes * count);
        assert_eq!(b.document_nodes, one.document_nodes);
    }
}

#[test]
fn success_and_failure_preserve_document_events_and_clipboard() {
    let mut core = blank();
    core.create_table_native(0, 1, 0, 2, 2).unwrap();
    let (pi, ci) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, p)| {
            p.controls
                .iter()
                .position(|c| matches!(c, Control::Table(_)))
                .map(|ci| (pi, ci))
        })
        .unwrap();
    core.copy_control_native(0, pi, &[], ci).unwrap();
    let before = format!("{:?}", core.document());
    let events = core.serialize_event_log();
    let clip = core.get_clipboard_text_native();
    let mut r = request();
    r.source_start = pi;
    r.source_end = pi + 1;
    r.insert_before = pi;
    let b = core.paragraph_block_budget_native(&r).unwrap();
    assert!(b.added_nodes >= 10); // table and its four cell/paragraph pairs
    r.limits.max_structure_bytes = 1;
    assert!(core.paragraph_block_budget_native(&r).is_err());
    assert_eq!(format!("{:?}", core.document()), before);
    assert_eq!(core.serialize_event_log(), events);
    assert_eq!(core.get_clipboard_text_native(), clip);
    assert!(core.has_internal_clipboard_native());
}
