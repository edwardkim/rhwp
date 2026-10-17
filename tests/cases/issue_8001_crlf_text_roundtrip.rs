//! CRLF edit input must not become an HWP paragraph terminator.
use rhwp::DocumentCore;

fn document(text: &str) -> DocumentCore {
    let mut doc = DocumentCore::new_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, text).unwrap();
    doc
}

fn check_saved(mut doc: DocumentCore, expected: &str) {
    assert_eq!(doc.document().sections[0].paragraphs[0].text, expected);
    for bytes in [
        doc.export_hwp_native().unwrap(),
        doc.export_hwp_with_adapter().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(reopened.document().sections[0].paragraphs[0].text, expected);
    }
}

#[test]
fn multiline_insert_preserves_suffix_and_reports_normalized_caret() {
    for input in ["1. 일정\r\n2. 예산\r\n", "1. 일정\r2. 예산\r", "1. 일정\n2. 예산\n"] {
        let mut doc = document("원문 끝.");
        let result: serde_json::Value =
            serde_json::from_str(&doc.insert_text_native(0, 0, 0, input).unwrap()).unwrap();
        assert_eq!(result["charOffset"], 12);
        check_saved(doc, "1. 일정\n2. 예산\n원문 끝.");
    }
}

#[test]
fn multiline_replacements_preserve_suffix_and_normalized_length() {
    for operation in ["range", "one", "all", "nth"] {
        let mut doc = document("앞 TARGET 뒤😀");
        let input = "일정\r\n예산\r끝";
        let result = match operation {
            "range" => doc.replace_text_native(0, 0, 2, 6, input),
            "one" => doc.replace_one_native("TARGET", input, true),
            "all" => doc.replace_all_native("TARGET", input, true),
            _ => doc.replace_nth_native("TARGET", input, true, 0),
        }.unwrap();
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        if matches!(operation, "range" | "one") {
            assert_eq!(result["newLength"], 7);
        } else {
            assert_eq!(result["count"], 1);
        }
        check_saved(doc, "앞 일정\n예산\n끝 뒤😀");
    }
}
