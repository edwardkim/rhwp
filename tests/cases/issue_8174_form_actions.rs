//! Script/address/update contracts; these do not assert rendering fidelity or pixel placement.
use rhwp::document_core::{queries::form_action::FormAddress, DocumentCore};
use rhwp::model::{
    control::{Control, FormType},
    paragraph::Paragraph,
};
use serde_json::{json, Value};

fn sample() -> DocumentCore {
    DocumentCore::from_bytes(include_bytes!("../../samples/hwpx/form-02.hwpx"))
        .expect("real Hancom form sample")
}
fn button(core: &DocumentCore) -> FormAddress {
    fn walk(paragraph: &Paragraph, target: FormAddress) -> Option<FormAddress> {
        for (index, control) in paragraph.controls.iter().enumerate() {
            if matches!(control, Control::Form(form) if form.form_type == FormType::PushButton && form.enabled)
            {
                return Some(FormAddress {
                    control: index,
                    ..target
                });
            }
            if let Control::Table(table) = control {
                for (cell, data) in table.cells.iter().enumerate() {
                    for (para, child) in data.paragraphs.iter().enumerate() {
                        let mut next = target.clone();
                        next.cell_path.push((index, cell, para));
                        if let Some(found) = walk(child, next) {
                            return Some(found);
                        }
                    }
                }
            }
        }
        None
    }
    for (section, data) in core.document().sections.iter().enumerate() {
        for (para, paragraph) in data.paragraphs.iter().enumerate() {
            if let Some(found) = walk(
                paragraph,
                FormAddress {
                    section,
                    para,
                    control: 0,
                    cell_path: Vec::new(),
                },
            ) {
                return found;
            }
        }
    }
    panic!("sample needs a real enabled push button")
}
fn context(core: &DocumentCore, address: &FormAddress) -> Value {
    serde_json::from_str(&core.form_action_context_native(address).unwrap()).unwrap()
}

#[test]
fn real_hwp_hwpx_scripts_and_form_addresses_survive_both_serializers() {
    let original = sample();
    let address = button(&original);
    let expected = context(&original, &address);
    assert!(expected["header"]
        .as_str()
        .unwrap()
        .contains("ItemFromName"));
    assert!(expected["source"]
        .as_str()
        .unwrap()
        .contains("OnPushButton_Click"));
    for bytes in [
        original.export_hwp_native().unwrap(),
        original.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let actual = context(&reopened, &button(&reopened));
        assert_eq!(actual["header"], expected["header"]);
        assert_eq!(actual["source"], expected["source"]);
        assert_eq!(
            actual["forms"].as_array().unwrap().len(),
            expected["forms"].as_array().unwrap().len()
        );
    }
}

#[test]
fn invalid_late_update_is_atomic_and_missing_cell_never_falls_back_to_body() {
    let mut core = sample();
    let address = button(&core);
    let before = context(&core, &address);
    let form = &before["forms"][0];
    let valid = json!({"address":form["address"],"text":"must not leak", "caption":"caption", "value":1,"enabled":true,"items":[]});
    let mut invalid = valid.clone();
    invalid["address"]["cellPath"] = json!([[999, 999, 999]]);
    assert!(core
        .apply_form_action_updates_native(&json!([valid, invalid]).to_string())
        .is_err());
    assert_eq!(context(&core, &address)["forms"], before["forms"]);
}

#[test]
fn form_updates_are_preserved_in_saved_hwp_and_hwpx() {
    let mut core = sample();
    let address = button(&core);
    let original = context(&core, &address);
    let target = original["forms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["formType"] == "Edit")
        .expect("real edit control");
    let update = json!({"address":target["address"],"text":"calculation result 42", "caption":target["caption"], "value":target["value"],"enabled":true,"items":[]});
    core.apply_form_action_updates_native(&json!([update]).to_string())
        .unwrap();
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let after = context(&reopened, &button(&reopened));
        assert!(after["forms"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["name"] == target["name"] && f["text"] == "calculation result 42"));
        assert_eq!(after["source"], original["source"]);
    }
}

#[test]
fn hwp_origin_scripts_convert_to_standard_hwpx_parts_and_package_references() {
    let original = DocumentCore::from_bytes(include_bytes!("../../samples/form-02.hwp")).unwrap();
    let expected = context(&original, &button(&original));
    let bytes = original.export_hwpx_native().unwrap();
    let reopened = DocumentCore::from_bytes(&bytes).unwrap();
    let actual = context(&reopened, &button(&reopened));
    assert_eq!(actual["header"], expected["header"]);
    assert_eq!(actual["source"], expected["source"]);
    assert!(!actual["source"].as_str().unwrap().is_empty());
    let package = reopened
        .document()
        .hwpx_aux_entry("Contents/content.hpf")
        .unwrap();
    let package = std::str::from_utf8(package).unwrap();
    for (id, path) in [
        ("rhwpHeaderScript", "Scripts/headerScripts"),
        ("rhwpSourceScript", "Scripts/sourceScripts"),
    ] {
        assert!(reopened.document().hwpx_aux_entry(path).is_some());
        assert!(package.contains(&format!("href=\"{path}\"")));
        assert!(package.contains(&format!("idref=\"{id}\"")));
    }
}

#[test]
fn excessive_or_corrupt_scripts_are_rejected_before_runtime_exposure() {
    let mut core = sample();
    let address = button(&core);
    for (path, bytes) in &mut core.document_mut().hwpx_aux_entries {
        if path == "Scripts/sourceScripts" {
            *bytes = vec![0; 1_048_578];
        }
    }
    assert!(core.form_action_context_native(&address).is_err());
    let mut core = sample();
    core.document_mut()
        .hwpx_aux_entries
        .retain(|(p, _)| !p.starts_with("Scripts/"));
    for (path, bytes) in &mut core.document_mut().extra_streams {
        if path.trim_start_matches('/') == "Scripts/DefaultJScript" {
            *bytes = vec![0xff; 30];
        }
    }
    assert!(core.form_action_context_native(&button(&core)).is_err());
}

#[test]
fn distribution_script_envelope_survives_editable_conversion_and_hwpx_export() {
    // Public Hancom distribution document: its Scripts stream has the same
    // DISTRIBUTE_DOC_DATA envelope as ViewText. This is a source-preservation
    // contract, not a page-layout or fixed-coordinate expectation.
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/한글문서파일형식_5.0_revision1.3.hwp"
    ))
    .unwrap();
    assert!(core.document().header.distribution);
    assert!(core.document_mut().convert_to_editable());
    let bytes = core.export_hwpx_native().unwrap();
    let document = rhwp::parse_document(&bytes).unwrap();
    for (path, expected) in [
        (
            "Scripts/headerScripts",
            "var Documents = XHwpDocuments;\r\nvar Document = Documents.Active_XHwpDocument;\r\n",
        ),
        (
            "Scripts/sourceScripts",
            "function OnDocument_New()\r\n{\r\n\t//todo : \r\n}\r\n\r\n",
        ),
    ] {
        let expected: Vec<u8> = expected.encode_utf16().flat_map(u16::to_le_bytes).collect();
        assert_eq!(document.hwpx_aux_entry(path).unwrap(), expected);
    }
}
