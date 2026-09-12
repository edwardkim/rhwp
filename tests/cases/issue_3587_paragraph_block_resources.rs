//! Reachable resource contracts; real-document observations are not visual verdicts.
use rhwp::{
    document_core::{
        DocumentCore, ParagraphBlockLimits, ParagraphBlockPathStep as Step,
        RepeatParagraphBlockRequest,
    },
    model::{
        bin_data::{BinDataBytes, BinDataContent, BinDataResolver},
        control::Control,
        image::Picture,
        paragraph::{CharShapeRef, Paragraph},
        shape::{RectangleShape, ShapeObject, TextBox},
        style::{BorderFill, Fill, HeadType, ImageFill, Numbering},
    },
};
use std::sync::Arc;

fn core() -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    c.document_mut().sections[0].paragraphs = vec![Paragraph::default(); 3];
    c
}
fn request() -> RepeatParagraphBlockRequest {
    RepeatParagraphBlockRequest {
        section_index: 0,
        source_start: 1,
        source_end: 2,
        insert_before: 2,
        count: 1,
        limits: ParagraphBlockLimits::default(),
    }
}
fn check(c: &DocumentCore) -> Result<(), rhwp::document_core::ParagraphBlockValidationError> {
    c.validate_paragraph_block_native(&request()).map(|_| ())
}
fn picture(c: &mut DocumentCore, id: u16) {
    let mut p = Picture::default();
    p.image_attr.bin_data_id = id;
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Picture(Box::new(p)));
}

#[test]
fn zero_based_style_graph_and_self_cycle_are_valid_without_mutation() {
    let c = core();
    let before = format!("{:?}", c.document());
    let events = c.serialize_event_log();
    check(&c).unwrap();
    assert_eq!(format!("{:?}", c.document()), before);
    assert_eq!(c.serialize_event_log(), events);
}

#[test]
fn missing_style_para_run_and_transitive_next_style_are_rejected() {
    for kind in 0..4 {
        let mut c = core();
        match kind {
            0 => c.document_mut().sections[0].paragraphs[1].style_id = 255,
            1 => c.document_mut().sections[0].paragraphs[1].para_shape_id = u16::MAX,
            2 => c.document_mut().sections[0].paragraphs[1]
                .char_shapes
                .push(CharShapeRef {
                    start_pos: 0,
                    char_shape_id: u32::MAX,
                }),
            _ => c.document_mut().doc_info.styles[0].next_style_id = 255,
        }
        let before = format!("{:?}", c.document());
        let e = check(&c).unwrap_err();
        assert_eq!(e.code, "missingResource");
        assert_eq!(e.path, vec![Step::Paragraph(0)]);
        assert_eq!(format!("{:?}", c.document()), before);
    }
}

#[test]
fn nested_textbox_resource_error_has_owned_path() {
    let mut c = core();
    let mut r = RectangleShape::default();
    r.drawing.text_box = Some(TextBox {
        paragraphs: vec![Paragraph {
            style_id: 255,
            ..Default::default()
        }],
        ..Default::default()
    });
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Shape(Box::new(ShapeObject::Rectangle(r))));
    assert_eq!(
        check(&c).unwrap_err().path,
        vec![
            Step::Paragraph(0),
            Step::Control(0),
            Step::Shape,
            Step::TextBox,
            Step::Paragraph(0)
        ]
    );
}

#[test]
fn border_zero_is_none_but_positive_border_and_its_image_must_resolve() {
    let mut c = core();
    c.document_mut().doc_info.char_shapes[0].border_fill_id = 0;
    check(&c).unwrap();
    c.document_mut().doc_info.border_fills = vec![BorderFill {
        fill: Fill {
            image: Some(ImageFill {
                bin_data_id: 1,
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }];
    c.document_mut().doc_info.char_shapes[0].border_fill_id = 1;
    assert_eq!(check(&c).unwrap_err().code, "missingResource");
    c.document_mut().bin_data_content.push(content(77));
    check(&c).unwrap();
    c.document_mut().doc_info.char_shapes[0].border_fill_id = 2;
    assert_eq!(check(&c).unwrap_err().code, "missingResource");
}

#[derive(Debug)]
struct NeverLoad;
impl BinDataResolver for NeverLoad {
    fn resolve(&self, _: &str) -> Vec<u8> {
        panic!("preflight must not materialize images")
    }
}
fn content(id: u16) -> BinDataContent {
    BinDataContent {
        id,
        data: BinDataBytes::Lazy {
            resolver: Arc::new(NeverLoad),
            key: "unread".into(),
        },
        extension: "png".into(),
    }
}

#[test]
fn image_ordinal_and_sparse_fallback_do_not_load_or_change_storage() {
    let mut c = core();
    c.document_mut().bin_data_content.push(content(500));
    picture(&mut c, 1);
    picture(&mut c, 500);
    check(&c).unwrap();
    assert_eq!(c.document().bin_data_content[0].id, 500);
    picture(&mut c, 2);
    assert_eq!(check(&c).unwrap_err().code, "missingResource");
}

#[test]
fn font_definition_and_embedded_storage_are_distinct_from_image_ordinals() {
    let mut c = core();
    c.document_mut().bin_data_content.push(content(500));
    let font = &mut c.document_mut().doc_info.font_faces[0][0];
    font.is_embedded = true;
    font.resolved_bin_data_id = Some(500);
    check(&c).unwrap();
    c.document_mut().doc_info.font_faces[0][0].resolved_bin_data_id = Some(1);
    assert_eq!(check(&c).unwrap_err().code, "missingResource");
    c.document_mut().doc_info.font_faces[0][0].is_embedded = false;
    c.document_mut().doc_info.char_shapes[0].font_ids[0] = u16::MAX;
    assert_eq!(check(&c).unwrap_err().code, "missingResource");
}

#[test]
fn numbering_sentinel_and_builtin_outline_are_not_missing_resources() {
    let mut c = core();
    c.document_mut().doc_info.para_shapes[0].head_type = HeadType::Outline;
    c.document_mut().sections[0].outline_numbering_id = 0;
    check(&c).unwrap();
    c.document_mut().sections[0].outline_numbering_id = 1;
    assert_eq!(check(&c).unwrap_err().code, "missingResource");
    let mut n = Numbering::default();
    for head in &mut n.heads {
        head.char_shape_id = u32::MAX;
    }
    c.document_mut().doc_info.numberings = vec![n];
    check(&c).unwrap();
}

#[test]
fn count_zero_skips_resources_and_bounded_resource_walk_stops_without_editing() {
    let mut c = core();
    let mut r = request();
    r.limits.max_document_nodes = 5; // Owned tree fits; reachable style edges do not.
    assert_eq!(
        c.validate_paragraph_block_native(&r).unwrap_err().code,
        "budgetOrAddress"
    );
    c.document_mut().doc_info.styles.clear();
    r.count = 0;
    assert!(c.validate_paragraph_block_native(&r).is_ok());
}

#[test]
fn unrelated_broken_resources_do_not_block_the_source() {
    let mut c = core();
    let mut unused = c.document().doc_info.char_shapes[0].clone();
    unused.font_ids = [u16::MAX; 7];
    c.document_mut().doc_info.char_shapes.push(unused);
    c.document_mut().sections[0].paragraphs[2].style_id = 255;
    check(&c).unwrap();
}

#[test]
fn repository_blocks_are_observed_without_editing_or_claiming_support() {
    for (file, pi) in [
        ("samples/hwp_table_test.hwp", 3),
        ("samples/table-in-tbox.hwp", 0),
        ("samples/rnote/labnote-001.hwp", 0),
    ] {
        let bytes = std::fs::read(file).unwrap();
        let c = DocumentCore::from_bytes(&bytes).unwrap();
        let before = format!("{:?}", c.document());
        let mut r = request();
        r.source_start = pi;
        r.source_end = pi + 1;
        r.insert_before = r.source_end;
        let result = c.validate_paragraph_block_native(&r);
        println!(
            "BLOCK_OBSERVATION {}",
            serde_json::json!({"file": file, "section": 0, "start": pi, "end": pi+1, "result": result})
        );
        // This assertion proves read-only operation, NOT acceptance or visual fidelity.
        assert_eq!(format!("{:?}", c.document()), before);
    }
}
