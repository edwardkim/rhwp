//! Native foreign template import contracts; synthetic inputs are not Hancom visual oracles.
use rhwp::{document_core::{DocumentCore, ImportParagraphBlockLimits, ImportParagraphBlockRequest}, model::{
    bin_data::{BinData, BinDataBytes, BinDataContent, BinDataResolver, BinDataType},
    control::Control, document::DocInfo, image::Picture, paragraph::{CharShapeRef, Paragraph},
    style::{CharShape, Font, ParaShape, Style, TabDef},
}};
use std::sync::Arc;

fn core(texts: &[&str]) -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    c.document_mut().doc_info = DocInfo {
        font_faces: vec![vec![Font { name: "바탕".into(), ..Default::default() }]; 7],
        char_shapes: vec![CharShape::default()], para_shapes: vec![ParaShape::default()],
        styles: vec![Style::default()], tab_defs: vec![TabDef::default()], ..Default::default()
    };
    c.document_mut().sections[0].paragraphs = texts.iter().map(|text| Paragraph {
        text: (*text).into(), char_count: text.encode_utf16().count() as u32 + 1,
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 0 }], ..Default::default()
    }).collect();
    c
}
fn request() -> ImportParagraphBlockRequest {
    ImportParagraphBlockRequest { source_section: 0, source_start: 1, source_end: 2,
        target_section: 0, insert_before: 1, count: 2, limits: ImportParagraphBlockLimits::default() }
}
fn image(c: &mut DocumentCore, storage: u16, bytes: &[u8]) {
    c.document_mut().doc_info.bin_data_list.push(BinData { storage_id: storage, data_type: BinDataType::Embedding,
        extension: Some("png".into()), ..Default::default() });
    c.document_mut().bin_data_content.push(BinDataContent { id: storage, extension: "png".into(), data: bytes.to_vec().into() });
    let mut pic = Picture::default();
    pic.common.width = 1000; pic.common.height = 1000;
    pic.image_attr.bin_data_id = 1;
    c.document_mut().sections[0].paragraphs[1].controls.push(Control::Picture(Box::new(pic)));
}
fn unchanged(c: &DocumentCore) -> (String, String) { (format!("{:?}", c.document()), c.serialize_event_log()) }

#[test]
fn imports_only_reachable_styles_and_preserves_target_boundaries_and_source() {
    let mut source = core(&["source before", "template", "source after"]);
    source.document_mut().doc_info.font_faces[0][0].name = "돋움".into();
    source.document_mut().doc_info.char_shapes[0].base_size = 1700;
    source.document_mut().doc_info.char_shapes.push(CharShape { font_ids: [400; 7], ..Default::default() });
    let source_before = unchanged(&source);
    let mut target = core(&["before", "after"]);
    let before_font = serde_json::to_value(&target.document().doc_info.font_faces).unwrap();
    let before_page = format!("{:?}", target.document().sections[0].section_def);
    let result = target.import_paragraph_block_native(source.document(), &request()).unwrap();
    assert_eq!(result.inserted, 1..3);
    assert_eq!(result.copies.len(), 2);
    assert_eq!(target.document().sections[0].paragraphs.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(), ["before", "template", "template", "after"]);
    assert_eq!(format!("{:?}", target.document().sections[0].section_def), before_page);
    let info = &target.document().doc_info;
    assert_eq!(serde_json::to_value(&info.font_faces[0][..1]).unwrap(), before_font[0]);
    assert_eq!(info.char_shapes.len(), 2, "unreachable invalid char shape must not be imported");
    let copied = &target.document().sections[0].paragraphs[1];
    let shape = &info.char_shapes[copied.char_shapes[0].char_shape_id as usize];
    assert_eq!(shape.base_size, 1700);
    assert_eq!(info.font_faces[0][shape.font_ids[0] as usize].name, "돋움");
    assert_eq!(unchanged(&source), source_before);
    let before = format!("{:?}", target.document().doc_info);
    let again = target.import_paragraph_block_native(source.document(), &request()).unwrap();
    assert_eq!(again.resources.added, 0);
    assert_eq!(format!("{:?}", target.document().doc_info), before);
}

#[test]
fn cyclic_styles_are_linked_after_reservation_and_reused_on_later_calls() {
    let mut source = core(&["unused", "template"]);
    source.document_mut().doc_info.styles = vec![
        Style { local_name: "A".into(), next_style_id: 1, ..Default::default() },
        Style { local_name: "B".into(), next_style_id: 0, ..Default::default() },
    ];
    let mut target = core(&["before", "after"]);
    target.document_mut().doc_info.styles[0].local_name = "keep".into();
    let existing = serde_json::to_value(&target.document().doc_info.styles[0]).unwrap();
    target.import_paragraph_block_native(source.document(), &request()).unwrap();
    let styles = &target.document().doc_info.styles;
    assert_eq!(styles.len(), 3);
    assert_eq!(styles[1].next_style_id, 2);
    assert_eq!(styles[2].next_style_id, 1);
    assert_eq!(serde_json::to_value(&styles[0]).unwrap(), existing);
    let second = target.import_paragraph_block_native(source.document(), &request()).unwrap();
    assert_eq!(second.resources.added, 0);
    assert_eq!(target.document().doc_info.styles.len(), 3);
}

#[test]
fn full_preview_matches_execution_and_has_no_mutation() {
    let mut source = core(&["unused", "template"]);
    source.document_mut().doc_info.styles[0].local_name = "foreign".into();
    let mut target = core(&["before", "after"]);
    let before = unchanged(&target);
    let preview = target.preview_paragraph_block_import_native(source.document(), &request()).unwrap();
    assert_eq!(unchanged(&target), before);
    assert_eq!(preview, target.import_paragraph_block_native(source.document(), &request()).unwrap());
}

#[test]
fn style_capacity_and_metadata_failures_leave_no_partial_resources() {
    let mut source = core(&["unused", "template"]);
    source.document_mut().doc_info.styles[0].local_name = "new".into();
    let mut target = core(&["before", "after"]);
    target.document_mut().doc_info.styles = (0..256).map(|i| Style { local_name: format!("existing-{i}"), ..Default::default() }).collect();
    let before = unchanged(&target);
    assert!(target.import_paragraph_block_native(source.document(), &request()).unwrap_err().to_string().contains("style ID space"));
    assert_eq!(unchanged(&target), before);
    let mut req = request(); req.limits.max_resource_metadata_bytes = 1;
    assert!(target.import_paragraph_block_native(source.document(), &req).is_err());
    assert_eq!(unchanged(&target), before);
}

#[test]
fn image_ordinal_is_not_confused_with_source_storage_id_and_copies_have_unique_ids() {
    let mut source = core(&["unused", "template"]); image(&mut source, 17, &[1,2,3,4]);
    let mut target = core(&["before", "after"]); image(&mut target, 1, &[9,8,7,6]);
    let source_before = unchanged(&source);
    let result = target.import_paragraph_block_native(source.document(), &request()).unwrap();
    assert_eq!(result.resources.binaries_added, 1);
    assert_eq!(target.document().bin_data_content.len(), 2);
    assert_eq!(target.document().bin_data_content[1].data.load(), [1,2,3,4]);
    let pictures: Vec<_> = target.document().sections[0].paragraphs[1..3].iter().map(|p| {
        let Control::Picture(pic) = &p.controls[0] else { panic!("picture") }; pic
    }).collect();
    assert_eq!(pictures[0].image_attr.bin_data_id, 2);
    assert_eq!(pictures[1].image_attr.bin_data_id, 2);
    assert_ne!(pictures[0].instance_id, pictures[1].instance_id);
    assert_eq!(unchanged(&source), source_before);
    let again = target.import_paragraph_block_native(source.document(), &request()).unwrap();
    assert_eq!(again.resources.binaries_added, 0);
    assert_eq!(again.resources.binaries_reused, 1);
    assert_eq!(target.document().bin_data_content.len(), 2);
}

#[test]
fn binary_limit_counts_source_and_target_comparison_reads() {
    let mut source = core(&["unused", "template"]); image(&mut source, 7, &[1,2,3,4]);
    let mut target = core(&["before", "after"]); image(&mut target, 1, &[1,2,3,4]);
    let before = unchanged(&target);
    let mut req = request(); req.limits.max_binary_bytes = 7;
    assert!(target.import_paragraph_block_native(source.document(), &req).is_err());
    assert_eq!(unchanged(&target), before);
    req.limits.max_binary_bytes = 8;
    let result = target.import_paragraph_block_native(source.document(), &req).unwrap();
    assert_eq!(result.resources.binary_bytes_read, 8);
    assert_eq!(result.resources.binaries_added, 0);
}

#[test]
fn unbounded_resolver_external_link_and_missing_metadata_are_rejected_without_mutation() {
    #[derive(Debug)] struct Unbounded;
    impl BinDataResolver for Unbounded {
        fn resolve(&self, _: &str) -> Vec<u8> { panic!("unbounded load forbidden") }
        fn resolved_len(&self, _: &str) -> usize { panic!("unbounded length forbidden") }
    }
    let mut source = core(&["unused", "template"]); image(&mut source, 17, &[1]);
    let mut target = core(&["before", "after"]);
    let before = unchanged(&target);
    source.document_mut().bin_data_content[0].data = BinDataBytes::Lazy { resolver: Arc::new(Unbounded), key: "forbidden".into() };
    assert!(target.import_paragraph_block_native(source.document(), &request()).is_err());
    assert_eq!(unchanged(&target), before);
    source.document_mut().doc_info.bin_data_list[0].data_type = BinDataType::Link;
    assert!(target.import_paragraph_block_native(source.document(), &request()).is_err());
    assert_eq!(unchanged(&target), before);
    source.document_mut().doc_info.bin_data_list.clear();
    assert!(target.import_paragraph_block_native(source.document(), &request()).is_err());
    assert_eq!(unchanged(&target), before);
}

#[test]
fn zero_count_does_not_prepare_resources_or_emit_events() {
    let mut source = core(&["unused", "template"]); source.document_mut().doc_info.styles.clear();
    let mut target = core(&["before", "after"]); let before = unchanged(&target);
    let mut req = request(); req.count = 0;
    assert_eq!(target.import_paragraph_block_native(source.document(), &req).unwrap().resources.added, 0);
    assert_eq!(unchanged(&target), before);
}

#[test]
fn real_labnote_block_imports_into_another_document_and_reopens_in_both_formats() {
    let source_bytes = std::fs::read("samples/rnote/labnote-001.hwp").unwrap();
    let mut source = DocumentCore::new_empty(); source.load_document(&source_bytes).unwrap();
    let before = format!("{:?}", source.document());
    let mut target = DocumentCore::new_empty(); target.create_blank_document_native().unwrap();
    let req = ImportParagraphBlockRequest { source_start: 12, source_end: 13, ..request() };
    let result = target.import_paragraph_block_native(source.document(), &req).unwrap();
    assert_eq!(result.inserted, 1..3);
    for bytes in [target.export_hwp_native().unwrap(), target.export_hwpx_native().unwrap()] {
        let mut reopened = DocumentCore::new_empty(); reopened.load_document(&bytes).unwrap();
        let tables: Vec<_> = reopened.document().sections[0].paragraphs.iter().flat_map(|p| &p.controls)
            .filter_map(|c| if let Control::Table(t) = c { Some(t) } else { None }).collect();
        assert_eq!(tables.len(), 2);
        assert_ne!(tables[0].common.instance_id, tables[1].common.instance_id);
        assert_eq!(tables[0].row_count, tables[1].row_count);
        assert_eq!(tables[0].col_count, tables[1].col_count);
    }
    assert_eq!(format!("{:?}", source.document()), before);
}
