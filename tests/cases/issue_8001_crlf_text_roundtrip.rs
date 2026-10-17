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
    for input in [
        "1. 일정\r\n2. 예산\r\n",
        "1. 일정\r2. 예산\r",
        "1. 일정\n2. 예산\n",
    ] {
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
        }
        .unwrap();
        let result: serde_json::Value = serde_json::from_str(&result).unwrap();
        if matches!(operation, "range" | "one") {
            assert_eq!(result["newLength"], 7);
        } else {
            assert_eq!(result["count"], 1);
        }
        check_saved(doc, "앞 일정\n예산\n끝 뒤😀");
    }
}

#[test]
fn multiline_cell_edits_preserve_text_after_save() {
    for nested in [false, true] {
        let cell = |content: &str| {
            format!(
                r#"<TABLE RowCount="1" ColCount="1"><SHAPEOBJECT><SIZE Width="5000" Height="2000"/></SHAPEOBJECT><ROW><CELL ColAddr="0" RowAddr="0" Width="5000" Height="2000"><PARALIST><P><TEXT>{content}</TEXT></P></PARALIST></CELL></ROW></TABLE>"#
            )
        };
        let text = "<CHAR>TARGET 뒤😀</CHAR>";
        let table = cell(&if nested { cell(text) } else { text.into() });
        let hml = format!(
            r#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT>{table}</TEXT></P></SECTION></BODY><TAIL/></HWPML>"#
        );
        for replace in [false, true] {
            let mut doc = DocumentCore::from_bytes(hml.as_bytes()).unwrap();
            if replace {
                doc.replace_all_native("TARGET", "일정\r\n예산", true)
                    .unwrap();
            } else {
                let path = vec![(0, 0, 0); if nested { 2 } else { 1 }];
                let result: serde_json::Value = serde_json::from_str(
                    &doc.insert_text_in_cell_by_path(0, 0, &path, 0, "일정\r\n예산")
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(result["charOffset"], 5);
            }
            let expected = if replace {
                "일정\n예산 뒤😀"
            } else {
                "일정\n예산TARGET 뒤😀"
            };
            assert_eq!(doc.grep("일정", true, None)[0].text, expected);
            for bytes in [
                doc.export_hwp_native().unwrap(),
                doc.export_hwpx_native().unwrap(),
            ] {
                let reopened = DocumentCore::from_bytes(&bytes).unwrap();
                assert_eq!(reopened.grep("일정", true, None)[0].text, expected);
            }
        }
    }
}

#[test]
fn cli_insert_reports_normalized_text_including_dry_run() {
    let dir = std::env::temp_dir().join(format!("rhwp-8001-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("input.hwp");
    let output = dir.join("output.hwp");
    std::fs::write(&input, document("원문 끝.").export_hwp_native().unwrap()).unwrap();
    for dry_run in [true, false] {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_rhwp"));
        command
            .args(["edit", "insert-text"])
            .arg(&input)
            .args([
                "--section",
                "0",
                "--para",
                "0",
                "--offset",
                "0",
                "--text",
                "일정\r\n예산",
                "--json",
                "-o",
            ])
            .arg(&output);
        if dry_run {
            command.arg("--dry-run");
        }
        let result = command.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(value["insertedChars"], 5);
        assert_eq!(value["text"], "일정\n예산");
        if !dry_run {
            let reopened = DocumentCore::from_bytes(&std::fs::read(&output).unwrap()).unwrap();
            assert_eq!(
                reopened.document().sections[0].paragraphs[0].text,
                "일정\n예산원문 끝."
            );
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}
