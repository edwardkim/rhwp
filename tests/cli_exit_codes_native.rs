//! [#4132] native-skia `export-png` CLI 종료 코드 계약.
//!
//! 파일 전체를 native-skia로 게이트해 Native Skia job·classifier의 파일 게이트
//! 규약이 이 target의 배선을 자동으로 강제한다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

#[path = "support/cli_exit_code_support.rs"]
mod cli_exit_code_support;

use cli_exit_code_support::{assert_code, unique_temp_path};

#[test]
fn export_png_follows_the_same_contract() {
    let missing = unique_temp_path("missing-png.hwp");
    let missing = missing.to_str().expect("utf-8 경로").to_string();
    let out_dir = unique_temp_path("png-out");
    let out_dir = out_dir.to_str().expect("utf-8 경로").to_string();

    assert_code(&["export-png"], 2);
    assert_code(&["export-png", &missing, "-o", &out_dir], 1);
}

#[test]
fn png_json_metadata_and_render_preserve_feature_and_provenance_contracts() {
    use std::path::Path;

    // 출력 품질·좌표의 baseline 검사가 아닌 CLI 봉투·공급 feature 계약이다.
    // 같은 입력의 Native/fresh WASM 전쪽 증거는 PR #7903의 vpos-hwp TSV에 있다.
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/table-vpos-01.hwp");
    let sample = sample.to_str().expect("UTF-8 입력 경로");
    let capabilities: serde_json::Value =
        serde_json::from_slice(&assert_code(&["capabilities"], 0).stdout).unwrap();
    let png = capabilities["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|command| command["name"] == "export-png")
        .unwrap();
    assert_eq!(png["available"], true);
    assert_eq!(png["requiresFeature"], "native-skia");

    let manifest: serde_json::Value =
        serde_json::from_slice(&assert_code(&["capabilities", "--mcp"], 0).stdout).unwrap();
    let png_tool = manifest["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "hwp_export_png")
        .unwrap();
    assert_eq!(png_tool["available"], png["available"]);
    assert_eq!(png_tool["requiresFeature"], png["requiresFeature"]);

    struct OutputDirectory(std::path::PathBuf);
    impl Drop for OutputDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    for metadata_only in [true, false] {
        let folder = OutputDirectory(unique_temp_path("png-provenance"));
        let output = folder.0.to_str().expect("UTF-8 출력 경로");
        let mut args = vec![
            "export-png",
            sample,
            "--json",
            "--profile",
            "print",
            "-p",
            "0",
            "-o",
            output,
        ];
        if metadata_only {
            args.push("--metadata-only");
        }
        let result = assert_code(&args, 0);
        let envelope: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        // #3787의 실제 값 표지 계약: #8565가 추가한 글꼴 보고에는 문서의
        // 글꼴명·대체 이름도 실린다. 엔진 집계만인 이전 빈 목록으로 고정하지 않는다.
        // MAP 구현을 재인용하지 않고 이 CLI의 공개된 네 보고 필드를 검증한다.
        let document_font_fields = [
            "nativeFontSupply",
            "fontEnvironment",
            "observedFontFallbacks",
            "declaredFontEnvironment",
        ];
        let expected_fields: Vec<_> = document_font_fields
            .into_iter()
            .filter(|field| {
                envelope.get(*field).is_some_and(|value| match value {
                    serde_json::Value::Null => false,
                    serde_json::Value::Object(value) => !value.is_empty(),
                    serde_json::Value::Array(value) => !value.is_empty(),
                    _ => panic!("글꼴 보고는 객체나 배열이어야 함: {field}"),
                })
            })
            .collect();
        assert!(!expected_fields.is_empty(), "문서 글꼴 보고가 있어야 함");
        assert_eq!(envelope["untrustedContent"], true);
        let fields = envelope["untrustedFields"].as_array().unwrap();
        let actual: std::collections::BTreeSet<_> =
            fields.iter().map(|field| field.as_str().unwrap()).collect();
        assert_eq!(actual.len(), fields.len(), "출처 경로가 중복되면 안 됨");
        assert_eq!(
            actual,
            expected_fields
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
        );
        assert_eq!(envelope["schemaVersion"], "1.0");
        assert_eq!(envelope["backend"], "native-skia");
        assert!(envelope["pageCount"].as_u64().unwrap() > 0);
        if metadata_only {
            assert_eq!(envelope["metadataOnly"], true);
            assert_eq!(envelope["renderedCount"], 0);
            assert_eq!(envelope["pages"], serde_json::json!([]));
        } else {
            assert_eq!(envelope["renderedCount"], 1);
            assert_eq!(envelope["comparisonProfile"], "print");
            let pages = envelope["pages"].as_array().unwrap();
            assert_eq!(pages.len(), 1);
            assert_eq!(pages[0]["page"], 0);
            assert!(Path::new(pages[0]["path"].as_str().unwrap()).is_file());
        }
    }
}
