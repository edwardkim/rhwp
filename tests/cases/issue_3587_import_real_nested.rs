//! Real nested resource probes. Successful serialization is not a Hancom visual verdict.
use rhwp::document_core::{DocumentCore, ImportParagraphBlockRequest};
use rhwp::model::{control::Control, image::Picture, table::Table};
use serde_json::json;

fn table(core: &DocumentCore, pi: usize) -> &Table {
    let Control::Table(table) = &core.document().sections[0].paragraphs[pi].controls[0] else {
        panic!("expected fixture table");
    };
    table
}

fn picture(table: &Table) -> &Picture {
    let Control::Picture(picture) = &table.cells[1].paragraphs[0].controls[0] else {
        panic!("expected fixture picture in cell (0,1)");
    };
    picture
}

fn picture_bytes(core: &DocumentCore, pi: usize) -> Vec<u8> {
    let id = picture(table(core, pi)).image_attr.bin_data_id;
    assert!(id > 0);
    core.document().bin_data_content[usize::from(id - 1)]
        .data
        .load_limited(1024 * 1024)
        .unwrap()
}

fn assert_table_content(source: &DocumentCore, target: &DocumentCore, pi: usize) {
    let original = table(source, 2);
    let imported = table(target, pi);
    assert_eq!(
        (original.row_count, original.col_count, original.cells.len()),
        (6, 4, 15)
    );
    assert_eq!(
        (imported.row_count, imported.col_count, imported.cells.len()),
        (6, 4, 15)
    );
    assert_eq!(original.page_break, imported.page_break);
    for (a, b) in original.cells.iter().zip(&imported.cells) {
        assert_eq!(
            (a.row, a.col, a.row_span, a.col_span, a.width, a.height),
            (b.row, b.col, b.row_span, b.col_span, b.width, b.height)
        );
        assert_eq!(a.paragraphs.len(), b.paragraphs.len());
        for (a, b) in a.paragraphs.iter().zip(&b.paragraphs) {
            assert_eq!(a.text, b.text);
            assert_eq!(a.controls.len(), b.controls.len());
        }
    }
    let a = picture(original);
    let b = picture(imported);
    assert_eq!(
        (a.common.width, a.common.height),
        (b.common.width, b.common.height)
    );
    assert_eq!(
        serde_json::to_value(a.crop).unwrap(),
        serde_json::to_value(b.crop).unwrap()
    );
    let bytes = picture_bytes(source, 2);
    assert!(!bytes.is_empty());
    assert_eq!(bytes, picture_bytes(target, pi));
}

#[test]
fn real_cell_picture_survives_foreign_import_reuse_and_both_exports() {
    let source =
        DocumentCore::from_bytes(&std::fs::read("samples/table-in-tbox.hwp").unwrap()).unwrap();
    let source_before = (
        format!("{:?}", source.document()),
        source.serialize_event_log(),
    );
    let mut target = target_with_source_page(&source);
    let prefix = format!("{:?}", target.document().sections[0].paragraphs[0]);
    let page = serde_json::to_value(&target.document().sections[0].section_def.page_def).unwrap();
    let first = target
        .import_paragraph_block_native(source.document(), &request(2))
        .unwrap();
    assert_eq!(first.resources.binaries_added, 1);
    assert_table_content(&source, &target, 1);
    let info = format!("{:?}", target.document().doc_info);
    let second = target
        .import_paragraph_block_native(source.document(), &request(2))
        .unwrap();
    assert_eq!(second.resources.added, 0);
    assert_eq!(second.resources.binaries_added, 0);
    assert_eq!(second.resources.binaries_reused, 1);
    assert_eq!(info, format!("{:?}", target.document().doc_info));
    assert_eq!(
        prefix,
        format!("{:?}", target.document().sections[0].paragraphs[0])
    );
    assert_eq!(
        page,
        serde_json::to_value(&target.document().sections[0].section_def.page_def).unwrap()
    );
    assert_ne!(
        table(&target, 1).common.instance_id,
        table(&target, 2).common.instance_id
    );
    assert_ne!(
        picture(table(&target, 1)).instance_id,
        picture(table(&target, 2)).instance_id
    );
    for bytes in [
        target.export_hwp_native().unwrap(),
        target.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_table_content(&source, &reopened, 1);
        assert_table_content(&source, &reopened, 2);
    }
    assert_eq!(
        source_before,
        (
            format!("{:?}", source.document()),
            source.serialize_event_log()
        )
    );
}

#[test]
fn real_uninterpreted_textbox_is_rejected_without_partial_import() {
    let source =
        DocumentCore::from_bytes(&std::fs::read("samples/table-in-tbox.hwp").unwrap()).unwrap();
    let mut target = target_with_source_page(&source);
    let before = (
        format!("{:?}", target.document()),
        target.serialize_event_log(),
    );
    let preview = target
        .preview_paragraph_block_import_native(source.document(), &request(4))
        .unwrap_err();
    let execute = target
        .import_paragraph_block_native(source.document(), &request(4))
        .unwrap_err();
    assert!(preview
        .to_string()
        .contains("uninterpreted textbox LIST_HEADER tail"));
    assert_eq!(preview.to_string(), execute.to_string());
    assert_eq!(
        before,
        (
            format!("{:?}", target.document()),
            target.serialize_event_log()
        )
    );
}

fn target_with_source_page(source: &DocumentCore) -> DocumentCore {
    let mut target = DocumentCore::new_empty();
    target.create_blank_document_native().unwrap();
    let pd = &source.document().sections[0].section_def.page_def;
    let binding = match pd.binding {
        rhwp::model::page::BindingMethod::SingleSided => 0,
        rhwp::model::page::BindingMethod::DuplexSided => 1,
        rhwp::model::page::BindingMethod::TopFlip => 2,
    };
    target
        .set_page_def_native(
            0,
            &json!({
                "width": pd.width, "height": pd.height,
                "marginLeft": pd.margin_left, "marginRight": pd.margin_right,
                "marginTop": pd.margin_top, "marginBottom": pd.margin_bottom,
                "marginHeader": pd.margin_header, "marginFooter": pd.margin_footer,
                "marginGutter": pd.margin_gutter, "landscape": pd.landscape, "binding": binding,
            })
            .to_string(),
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(&target.document().sections[0].section_def.page_def).unwrap(),
        serde_json::to_value(pd).unwrap()
    );
    target
}

fn request(pi: usize) -> ImportParagraphBlockRequest {
    ImportParagraphBlockRequest {
        source_section: 0,
        source_start: pi,
        source_end: pi + 1,
        target_section: 0,
        insert_before: 1,
        count: 1,
        limits: Default::default(),
    }
}

#[test]
#[ignore = "diagnostic candidate inspection; set RHWP_3587_NESTED_OUTPUT"]
fn inspect_real_nested_import_candidates() {
    let output = std::path::PathBuf::from(std::env::var("RHWP_3587_NESTED_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let bytes = std::fs::read("samples/table-in-tbox.hwp").unwrap();
    let source = DocumentCore::from_bytes(&bytes).unwrap();
    let source_before = format!("{:?}", source.document());
    let mut cases = Vec::new();
    for pi in [0, 2, 4] {
        let mut target = target_with_source_page(&source);
        let before = format!("{:?}", target.document());
        let preview = target.preview_paragraph_block_import_native(source.document(), &request(pi));
        assert_eq!(before, format!("{:?}", target.document()));
        match preview {
            Ok(preview) => {
                let result = target
                    .import_paragraph_block_native(source.document(), &request(pi))
                    .unwrap();
                assert_eq!(preview, result);
                let hwp = target.export_hwp_native().unwrap();
                let hwpx = target.export_hwpx_native().unwrap();
                DocumentCore::from_bytes(&hwp).unwrap();
                DocumentCore::from_bytes(&hwpx).unwrap();
                std::fs::write(output.join(format!("pi{pi}-import.hwp")), hwp).unwrap();
                std::fs::write(output.join(format!("pi{pi}-import.hwpx")), hwpx).unwrap();
                cases.push(json!({"sourceParagraph":pi, "status":"serialized-and-reopened", "result":result}));
            }
            Err(error) => {
                let execute_error = target
                    .import_paragraph_block_native(source.document(), &request(pi))
                    .unwrap_err();
                assert_eq!(error.to_string(), execute_error.to_string());
                assert_eq!(before, format!("{:?}", target.document()));
                cases.push(
                    json!({"sourceParagraph":pi, "status":"rejected", "error":error.to_string()}),
                );
            }
        }
    }
    assert_eq!(source_before, format!("{:?}", source.document()));
    std::fs::write(
        output.join("candidates.json"),
        serde_json::to_vec_pretty(&cases).unwrap(),
    )
    .unwrap();
}
