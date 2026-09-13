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
fn nondefault_textbox_is_rejected_without_partial_import() {
    let mut source =
        DocumentCore::from_bytes(&std::fs::read("samples/table-in-tbox.hwp").unwrap()).unwrap();
    // Deliberately nondefault test input, not a replacement visual fixture.
    textbox_mut(&mut source, 4).raw_list_header_extra[12] = 0xff;
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

fn textbox(core: &DocumentCore, pi: usize) -> &rhwp::model::shape::TextBox {
    let Control::Shape(shape) = &core.document().sections[0].paragraphs[pi].controls[0] else {
        panic!("expected textbox shape");
    };
    shape.drawing().unwrap().text_box.as_ref().unwrap()
}

#[test]
fn textbox_tail_and_hyperlink_rejection_boundaries_are_atomic() {
    // Mutated copies are negative contract inputs, never visual artifacts.
    for case in 0..11 {
        let mut source =
            DocumentCore::from_bytes(&std::fs::read("samples/table-in-tbox.hwp").unwrap()).unwrap();
        let tb = textbox_mut(&mut source, 4);
        let expected = match case {
            0 => {
                tb.raw_list_header_extra.pop();
                "textbox LIST_HEADER tail"
            }
            1 => {
                tb.raw_list_header_extra.push(0);
                "textbox LIST_HEADER tail"
            }
            2 => {
                tb.raw_list_header_extra[0] = 1;
                "textbox LIST_HEADER tail"
            }
            3 => {
                tb.raw_list_header_extra[8] = 1;
                "textbox LIST_HEADER tail"
            }
            4 => {
                tb.paragraphs[20].field_ranges.clear();
                "fieldClosure"
            }
            5 => {
                tb.paragraphs[20].field_ranges[0].end_char_idx = usize::MAX;
                "invalidFieldRange"
            }
            6 => {
                let Control::Field(f) = &mut tb.paragraphs[20].controls[0] else {
                    unreachable!()
                };
                f.raw_parameters_xml = Some("<unsupported/>".into());
                "unvalidated field parameters"
            }
            _ => {
                let Control::Field(f) = &mut tb.paragraphs[20].controls[0] else {
                    unreachable!()
                };
                f.parameters.items = vec![rhwp::model::control::Parameter::String {
                    name: Some("Command".into()),
                    value: f.command.clone(),
                    preserve_space: false,
                }];
                match case {
                    7 => f.parameters.items.push(f.parameters.items[0].clone()),
                    8 => f.command.push_str("mismatch"),
                    9 => f.raw_parameters_xml = Some("<hp:parameters unexpected=\"1\"/>".into()),
                    _ => f.field_type = rhwp::model::control::FieldType::Unknown,
                }
                if case == 10 {
                    "only plain ClickHere/Hyperlink"
                } else {
                    "unvalidated field parameters"
                }
            }
        };
        let source_before = format!("{:?}", source.document());
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
        assert!(
            execute.to_string().contains(expected),
            "case {case}: {execute}"
        );
        assert_eq!(preview.to_string(), execute.to_string());
        assert_eq!(
            before,
            (
                format!("{:?}", target.document()),
                target.serialize_event_log()
            )
        );
        assert_eq!(source_before, format!("{:?}", source.document()));
    }
}

#[test]
fn same_document_repeat_uses_the_same_textbox_hyperlink_contract() {
    let mut core =
        DocumentCore::from_bytes(&std::fs::read("samples/table-in-tbox.hwp").unwrap()).unwrap();
    let expected = evidence(&core, 4);
    let source_id = link(&core, 4).field_id;
    let end = core.document().sections[0].paragraphs.len();
    core.repeat_paragraph_block_native(&rhwp::document_core::RepeatParagraphBlockRequest {
        section_index: 0,
        source_start: 4,
        source_end: 5,
        insert_before: end,
        count: 2,
        limits: Default::default(),
    })
    .unwrap();
    assert_eq!(evidence(&core, end), expected);
    assert_eq!(evidence(&core, end + 1), expected);
    assert_ne!(link(&core, end).field_id, source_id);
    assert_ne!(link(&core, end + 1).field_id, source_id);
    assert_ne!(link(&core, end).field_id, link(&core, end + 1).field_id);
}

fn textbox_mut(core: &mut DocumentCore, pi: usize) -> &mut rhwp::model::shape::TextBox {
    let Control::Shape(shape) = &mut core.document_mut().sections[0].paragraphs[pi].controls[0]
    else {
        panic!("expected textbox shape");
    };
    shape.drawing_mut().unwrap().text_box.as_mut().unwrap()
}

fn link(core: &DocumentCore, pi: usize) -> &rhwp::model::control::Field {
    let Control::Field(field) = &textbox(core, pi).paragraphs[20].controls[0] else {
        panic!("expected hyperlink");
    };
    field
}

// Test-only owned-tree observation; IDs and format-specific raw are intentionally
// checked separately, not erased from the source to make it importable.
fn content_evidence(
    value: &serde_json::Value,
    core: &DocumentCore,
    out: &mut Vec<serde_json::Value>,
) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(picture) = map.get("Picture") {
                let id = picture["image_attr"]["bin_data_id"].as_u64().unwrap() as usize;
                let bytes = core.document().bin_data_content[id - 1]
                    .data
                    .load_limited(1024 * 1024)
                    .unwrap();
                out.push(json!({"pictureBytes":bytes,"crop":picture["crop"],
                    "width":picture["common"]["width"],"height":picture["common"]["height"]}));
            }
            for key in [
                "text",
                "command",
                "field_type",
                "row_count",
                "col_count",
                "row_span",
                "col_span",
            ] {
                if let Some(value) = map.get(key) {
                    out.push(json!({key:value}));
                }
            }
            for value in map.values() {
                content_evidence(value, core, out);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                content_evidence(value, core, out);
            }
        }
        _ => {}
    }
}

fn evidence(core: &DocumentCore, pi: usize) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    content_evidence(
        &serde_json::to_value(textbox(core, pi)).unwrap(),
        core,
        &mut out,
    );
    out
}

#[test]
fn whole_real_textbox_with_hyperlink_imports_and_survives_both_formats() {
    let source =
        DocumentCore::from_bytes(&std::fs::read("samples/table-in-tbox.hwp").unwrap()).unwrap();
    let before = format!("{:?}", source.document());
    assert_eq!(textbox(&source, 4).raw_list_header_extra, vec![0; 13]);
    assert_eq!(textbox(&source, 4).paragraphs.len(), 21);
    let expected = evidence(&source, 4);
    assert_eq!(
        expected
            .iter()
            .filter(|v| v.get("pictureBytes").is_some())
            .count(),
        8
    );
    assert_eq!(
        expected
            .iter()
            .filter(|v| v.get("row_count").is_some())
            .count(),
        4
    );
    let mut target = target_with_source_page(&source);
    let prefix = format!("{:?}", target.document().sections[0].paragraphs[0]);
    let page = serde_json::to_value(&target.document().sections[0].section_def.page_def).unwrap();
    let preview = target
        .preview_paragraph_block_import_native(source.document(), &request(4))
        .unwrap();
    let result = target
        .import_paragraph_block_native(source.document(), &request(4))
        .unwrap();
    assert_eq!(preview, result);
    assert_eq!(evidence(&target, 1), expected);
    let again = target
        .import_paragraph_block_native(source.document(), &request(4))
        .unwrap();
    assert_eq!(again.resources.added, 0);
    assert_eq!(again.resources.binaries_added, 0);
    assert_ne!(link(&target, 1).field_id, link(&target, 2).field_id);
    assert_eq!(
        prefix,
        format!("{:?}", target.document().sections[0].paragraphs[0])
    );
    assert_eq!(
        page,
        serde_json::to_value(&target.document().sections[0].section_def.page_def).unwrap()
    );
    for bytes in [
        target.export_hwp_native().unwrap(),
        target.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        for pi in [1, 2] {
            assert_eq!(evidence(&reopened, pi), expected);
            let a = &textbox(&source, 4).paragraphs[20].field_ranges[0];
            let b = &textbox(&reopened, pi).paragraphs[20].field_ranges[0];
            assert_eq!(
                (
                    a.start_char_idx,
                    a.end_char_idx,
                    a.control_idx,
                    a.end_field_id
                ),
                (
                    b.start_char_idx,
                    b.end_char_idx,
                    b.control_idx,
                    b.end_field_id
                )
            );
        }
        // Default tails written during export must remain importable after reopen.
        let mut destination = target_with_source_page(&source);
        destination
            .import_paragraph_block_native(reopened.document(), &request(1))
            .unwrap();
    }
    assert_eq!(before, format!("{:?}", source.document()));
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
