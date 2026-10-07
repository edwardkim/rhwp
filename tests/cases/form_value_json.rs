//! 양식 값 JSON은 이스케이프를 한 번 해석하고, 본문·셀 및 저장본에 같은 값을 남긴다.

use rhwp::document_core::DocumentCore;
use rhwp::error::HwpError;
use rhwp::model::control::{Control, FormObject, FormType};
use serde_json::{json, Value};

const BODY_FORMS: [(usize, usize, FormType); 5] = [
    (0, 2, FormType::PushButton),
    (2, 0, FormType::CheckBox),
    (4, 0, FormType::ComboBox),
    (6, 0, FormType::RadioButton),
    (8, 0, FormType::Edit),
];

fn load(in_cell: bool) -> DocumentCore {
    let path = if in_cell {
        "samples/hwpx/form-002.hwpx"
    } else {
        "samples/form-01.hwp"
    };
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .expect("공개 양식 표본");
    DocumentCore::from_bytes(&bytes).expect("양식 표본 열기")
}

fn body_form(doc: &DocumentCore, para: usize, ci: usize) -> &FormObject {
    let Control::Form(form) = &doc.document().sections[0].paragraphs[para].controls[ci] else {
        panic!("본문 양식 표본 주소가 바뀌었습니다")
    };
    form
}

fn form(doc: &DocumentCore, in_cell: bool) -> &FormObject {
    if !in_cell {
        return body_form(doc, 8, 0);
    }
    let Control::Table(table) = &doc.document().sections[0].paragraphs[0].controls[3] else {
        panic!("셀 양식의 표가 없습니다")
    };
    let Control::Form(form) = &table.cells[6].paragraphs[0].controls[0] else {
        panic!("셀 양식이 없습니다")
    };
    assert_eq!(form.form_type, FormType::CheckBox);
    form
}

fn set(doc: &mut DocumentCore, in_cell: bool, payload: &str) -> Result<String, HwpError> {
    if in_cell {
        doc.set_form_value_in_cell_native(0, 0, 3, 6, 0, 0, payload)
    } else {
        doc.set_form_value_native(0, 8, 0, payload)
    }
}

#[test]
fn body_form_json_escapes_survive_live_hwp_and_hwpx_values() {
    for value in [
        "plain",
        "한글🦦",
        "A\"B",
        "C:\\tmp",
        "tail\\",
        "A\nB\tC",
        "",
    ] {
        let mut doc = load(false);
        for (para, ci, kind) in BODY_FORMS {
            assert_eq!(body_form(&doc, para, ci).form_type, kind);
            let key = if matches!(kind, FormType::ComboBox | FormType::Edit) {
                "text"
            } else {
                "caption"
            };
            let payload = json!({key: value, "value": 0}).to_string();
            let result = doc.set_form_value_native(0, para, ci, &payload).unwrap();
            assert_eq!(serde_json::from_str::<Value>(&result).unwrap()["ok"], true);
            let live: Value =
                serde_json::from_str(&doc.get_form_value_native(0, para, ci).unwrap()).unwrap();
            assert_eq!(live[key], value, "live {kind:?}");
        }
        for bytes in [
            doc.export_hwp_native().unwrap(),
            doc.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            for (para, ci, kind) in BODY_FORMS {
                let actual = body_form(&reopened, para, ci);
                assert_eq!(actual.form_type, kind);
                let actual_value = if matches!(kind, FormType::ComboBox | FormType::Edit) {
                    &actual.text
                } else {
                    &actual.caption
                };
                assert_eq!(actual_value, value, "reopen {kind:?}");
                assert_eq!(actual.value, 0);
            }
        }
    }
}

#[test]
fn cell_form_json_escapes_survive_save_and_snapshot_restore() {
    for caption in [
        "plain",
        "한글🦦",
        "A\"B",
        "C:\\tmp",
        "tail\\",
        "A\nB\tC",
        "",
    ] {
        let mut doc = load(true);
        let original = form(&doc, true).caption.clone();
        let snapshot = doc.save_snapshot_native();
        set(
            &mut doc,
            true,
            &json!({"caption": caption, "value": 1}).to_string(),
        )
        .unwrap();
        assert_eq!(form(&doc, true).caption, caption);
        assert_eq!(form(&doc, true).value, 1);
        for bytes in [
            doc.export_hwp_native().unwrap(),
            doc.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(form(&reopened, true).caption, caption);
            assert_eq!(form(&reopened, true).value, 1);
        }
        doc.restore_snapshot_native(snapshot).unwrap();
        assert_eq!(form(&doc, true).caption, original);
        assert_eq!(form(&doc, true).value, 0);
    }
}

#[test]
fn form_json_handles_whitespace_unicode_and_only_top_level_typed_fields() {
    for in_cell in [false, true] {
        let mut doc = load(in_cell);
        set(
            &mut doc,
            in_cell,
            r#"{ "value" : 1, "text" : "\ud55c\uae00\ud83e\udda6", "caption" : "old" }"#,
        )
        .unwrap();
        assert_eq!(form(&doc, in_cell).text, "한글🦦");
        assert_eq!(form(&doc, in_cell).value, 1);
        let before = serde_json::to_value(form(&doc, in_cell)).unwrap();
        for payload in [
            r#"{}"#,
            r#"{"value":null,"text":null,"caption":null}"#,
            r#"{"value":true,"text":2,"caption":[]}"#,
            r#"{"value":2147483648}"#,
            r#"{"value":1.5}"#,
            r#"{"unknown":{"value":0,"text":"wrong","caption":"wrong"}}"#,
        ] {
            set(&mut doc, in_cell, payload).unwrap();
            assert_eq!(
                serde_json::to_value(form(&doc, in_cell)).unwrap(),
                before,
                "{payload}"
            );
        }
        set(&mut doc, in_cell, r#"{"value":0,"text":"","caption":""}"#).unwrap();
        assert_eq!(form(&doc, in_cell).value, 0);
        assert!(form(&doc, in_cell).text.is_empty());
        assert!(form(&doc, in_cell).caption.is_empty());
    }
}

#[test]
fn invalid_form_json_never_partially_mutates_or_invalidates_source() {
    for in_cell in [false, true] {
        let mut doc = load(in_cell);
        let before = format!("{:?}", doc.document());
        for payload in [
            r#"{"value":1,"text":"unfinished}"#,
            r#"{"value":1,"text":"bad\q"}"#,
            r#"{"text":"\ud800"}"#,
            r#"{"value":1} trailing"#,
            r#"[{"value":1}]"#,
            r#"null"#,
            r#""text""#,
        ] {
            assert!(
                matches!(
                    set(&mut doc, in_cell, payload),
                    Err(HwpError::InvalidField(_))
                ),
                "{payload}"
            );
            assert_eq!(format!("{:?}", doc.document()), before, "{payload}");
        }
    }
}

#[test]
fn form_queries_escape_all_decoded_json_control_characters() {
    // XML에서 허용하지 않는 제어 문자는 JSON 조회 계약만 검사한다.
    let mut doc = load(false);
    let text = "\u{0}\u{8}\u{c}\r\n\t";
    set(&mut doc, false, &json!({"text": text}).to_string()).unwrap();
    assert_eq!(form(&doc, false).text, text);
    for raw in [
        doc.get_form_value_native(0, 8, 0).unwrap(),
        doc.get_form_object_info_native(0, 8, 0).unwrap(),
    ] {
        assert_eq!(serde_json::from_str::<Value>(&raw).unwrap()["text"], text);
    }
}
