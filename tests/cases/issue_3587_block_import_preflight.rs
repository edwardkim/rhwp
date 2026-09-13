//! Foreign block inspection: source/target namespaces and failure immutability.
use rhwp::{
    document_core::{DocumentCore, ImportParagraphBlockLimits, ImportParagraphBlockRequest},
    model::{
        bin_data::{BinDataBytes, BinDataContent, BinDataResolver},
        control::Control,
        document::DocInfo,
        image::Picture,
        paragraph::{CharShapeRef, Paragraph},
        style::{CharShape, Font, ParaShape, Style, TabDef},
    },
};
use std::sync::Arc;

fn core(count: usize) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.document_mut().doc_info = DocInfo {
        font_faces: vec![vec![Font::default()]; 7],
        char_shapes: vec![CharShape::default()],
        para_shapes: vec![ParaShape::default()],
        styles: vec![Style::default()],
        tab_defs: vec![TabDef::default()],
        ..Default::default()
    };
    core.document_mut().sections[0].paragraphs = vec![Paragraph::default(); count];
    core
}

fn request() -> ImportParagraphBlockRequest {
    ImportParagraphBlockRequest {
        source_section: 0,
        source_start: 1,
        source_end: 3,
        target_section: 0,
        insert_before: 1,
        count: 2,
        limits: ImportParagraphBlockLimits::default(),
    }
}

#[test]
fn foreign_source_range_does_not_have_to_fit_the_target() {
    let source = core(5);
    let target = core(1);
    let before = format!("{:?}", target.document());
    let source_before = format!("{:?}", source.document());
    let events = target.serialize_event_log();
    let preview = target
        .inspect_paragraph_block_import_native(source.document(), &request())
        .unwrap();
    assert_eq!(preview.inserted, 1..5);
    assert_eq!(preview.added_paragraphs, 4);
    assert!(!preview.resources_prepared);
    assert!(preview.source_document_nodes > preview.target_document_nodes);
    assert_eq!(format!("{:?}", target.document()), before);
    assert_eq!(format!("{:?}", source.document()), source_before);
    assert_eq!(target.serialize_event_log(), events);
}

#[test]
fn target_boundary_can_numerically_fall_inside_the_foreign_range() {
    let source = core(5);
    let target = core(5);
    let mut req = request();
    req.insert_before = 2;
    assert_eq!(
        target
            .inspect_paragraph_block_import_native(source.document(), &req)
            .unwrap()
            .inserted,
        2..6
    );
}

#[test]
fn invalid_source_target_addresses_and_limits_fail_without_mutation() {
    let source = core(5);
    let target = core(2);
    let before = format!("{:?}", target.document());
    let events = target.serialize_event_log();
    for case in 0..9 {
        let mut req = request();
        match case {
            0 => req.source_section = 5,
            1 => req.source_start = req.source_end,
            2 => req.source_end = 6,
            3 => req.target_section = 5,
            4 => req.insert_before = 3,
            5 => req.count = usize::MAX,
            6 => req.limits.max_binary_bytes = 0,
            7 => req.limits.max_resource_metadata_bytes += 1,
            _ => req.limits.block.max_depth = 0,
        }
        assert!(
            target
                .inspect_paragraph_block_import_native(source.document(), &req)
                .is_err(),
            "case {case}"
        );
        assert_eq!(format!("{:?}", target.document()), before);
        assert_eq!(target.serialize_event_log(), events);
    }
}

#[test]
fn zero_count_skips_source_content_but_not_address_validation() {
    let mut source = core(5);
    source.document_mut().sections[0].paragraphs[1].style_id = 255;
    let target = core(2);
    let mut req = request();
    req.count = 0;
    let preview = target
        .inspect_paragraph_block_import_native(source.document(), &req)
        .unwrap();
    assert_eq!(preview.inserted, 1..1);
    assert_eq!(preview.added_nodes, 0);
    assert_eq!(
        preview.source_document_nodes + preview.target_document_nodes,
        0
    );
    req.source_end = 100;
    assert!(target
        .inspect_paragraph_block_import_native(source.document(), &req)
        .is_err());
}

#[test]
fn source_reference_is_resolved_in_source_not_target_docinfo() {
    let mut source = core(5);
    let mut target = core(2);
    target
        .document_mut()
        .doc_info
        .char_shapes
        .push(CharShape::default());
    source.document_mut().sections[0].paragraphs[1].char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 1,
    }];
    let error = target
        .inspect_paragraph_block_import_native(source.document(), &request())
        .unwrap_err();
    assert!(error.to_string().contains("source:"));
    assert!(error.to_string().contains("missingResource"));
    source
        .document_mut()
        .doc_info
        .char_shapes
        .push(CharShape::default());
    target.document_mut().doc_info.char_shapes.truncate(1);
    assert!(target
        .inspect_paragraph_block_import_native(source.document(), &request())
        .is_ok());
}

#[test]
fn combined_document_budget_covers_both_documents() {
    let source = core(5);
    let target = core(100);
    let mut req = request();
    let p = target
        .inspect_paragraph_block_import_native(source.document(), &req)
        .unwrap();
    req.limits.block.max_document_nodes = p.source_document_nodes.max(p.target_document_nodes);
    assert!(target
        .inspect_paragraph_block_import_native(source.document(), &req)
        .is_err());
}

#[derive(Debug)]
struct NoLoad;
impl BinDataResolver for NoLoad {
    fn resolve(&self, _: &str) -> Vec<u8> {
        panic!("preflight must not load resource bytes")
    }
    fn resolved_len(&self, _: &str) -> usize {
        panic!("preflight must not ask lazy byte length")
    }
}

#[test]
fn structural_preflight_does_not_claim_lazy_resources_are_prepared() {
    let mut source = core(5);
    let mut picture = Picture::default();
    picture.image_attr.bin_data_id = 1;
    source.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Picture(Box::new(picture)));
    source.document_mut().bin_data_content.push(BinDataContent {
        id: 17,
        extension: "png".into(),
        data: BinDataBytes::Lazy {
            key: "unloaded".into(),
            resolver: Arc::new(NoLoad),
        },
    });
    let target = core(2);
    let p = target
        .inspect_paragraph_block_import_native(source.document(), &request())
        .unwrap();
    assert!(!p.resources_prepared);
}

#[test]
fn input_json_rejects_unknown_keys_and_resource_limits_above_ceiling() {
    let mut json = serde_json::to_value(request()).unwrap();
    json["extra"] = true.into();
    assert!(serde_json::from_value::<ImportParagraphBlockRequest>(json).is_err());
    let mut req = request();
    req.limits.max_binary_bytes += 1;
    assert!(core(2)
        .inspect_paragraph_block_import_native(core(5).document(), &req)
        .is_err());
}

#[test]
fn labnote_hwp_and_derived_hwpx_are_inspected_without_pasting() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rnote/labnote-001.hwp");
    let hwp = std::fs::read(path).expect("required labnote fixture");
    let source = DocumentCore::from_bytes(&hwp).unwrap();
    let hwpx = source.export_hwpx_native().unwrap();
    for bytes in [hwp, hwpx] {
        let source = DocumentCore::from_bytes(&bytes).unwrap();
        let mut target = DocumentCore::new_empty();
        target.create_blank_document_native().unwrap();
        let before = target.export_hwpx_native().unwrap();
        let req = ImportParagraphBlockRequest {
            source_start: 12,
            source_end: 13,
            insert_before: 1,
            ..request()
        };
        let p = target
            .inspect_paragraph_block_import_native(source.document(), &req)
            .unwrap();
        assert_eq!(p.added_paragraphs, 2);
        assert!(!p.resources_prepared);
        let again = target
            .inspect_paragraph_block_import_native(source.document(), &req)
            .unwrap();
        assert_eq!(p, again);
        assert_eq!(target.export_hwpx_native().unwrap(), before);
    }
}
