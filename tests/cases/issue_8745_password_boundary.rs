//! 관계형: 암호 비노출, 인증 실패 시 기존 문서 보존, 저장 크기와 실제 파일의 일치.
//! 주석 갱신: 2026-11-07T19:55:10+09:00.
//! 시각 측정 비해당: 열기·저장·진단 계약이며 절대 좌표·쪽수 기대값이 없다.

use rhwp::service::OpenOptions;
use rhwp::{wasm_api::HwpDocument, DocumentCore};

fn fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn paragraphs(core: &DocumentCore) -> Vec<String> {
    core.document()
        .sections
        .iter()
        .flat_map(|section| {
            section
                .paragraphs
                .iter()
                .map(|paragraph| paragraph.text.clone())
        })
        .collect()
}

#[test]
fn open_options_debug_redacts_credentials_without_hiding_limits() {
    let synthetic_value = "synthetic-8745-debug-probe";
    let options = OpenOptions::new()
        .with_password(synthetic_value)
        .with_max_bytes(4096);
    for text in [
        format!("{options:?}"),
        format!("{options:#?}"),
        format!("{:?}", Some(options.clone())),
    ] {
        assert!(
            !text.contains(synthetic_value),
            "Debug exposed a credential"
        );
        assert!(text.contains("[redacted]"));
        assert!(text.contains("4096"));
    }
    assert_eq!(options.password.as_deref(), Some(synthetic_value));
    assert_eq!(options.max_bytes, Some(4096));
    let plain = format!("{:?}", OpenOptions::new());
    assert!(plain.contains("None"));
    assert!(!plain.contains("[redacted]"));
}

#[test]
fn document_load_is_atomic_and_retry_preserves_authenticated_content() {
    // Existing public fixture credentials, not an application secret.
    let encrypted = fixture("samples/HWP5-password-123456.hwpx");
    let plain = fixture("samples/HWP5-nopassword-123456.hwpx");
    let mut core = DocumentCore::from_bytes(&plain).unwrap();
    let expected = paragraphs(&core);
    core.document_mut().sections[0].paragraphs[0]
        .text
        .push_str(" existing unsaved content");
    let retained = paragraphs(&core);
    let retained_pages = core.page_count();
    for credential in [None, Some(b"wrong-8745-synthetic".as_slice())] {
        let error = core.load_bytes(&encrypted, credential).unwrap_err();
        assert!(!error.to_string().contains("wrong-8745-synthetic"));
        assert_eq!(paragraphs(&core), retained);
        assert_eq!(core.page_count(), retained_pages);
    }
    assert!(core.load_bytes(b"invalid document", None).is_err());
    assert_eq!(paragraphs(&core), retained);
    core.load_bytes(&encrypted, Some(b"123456")).unwrap();
    assert_eq!(paragraphs(&core), expected);
    // Supplying a credential for a plain document keeps the existing parser contract.
    core.load_bytes(&plain, Some(b"unused-8745-synthetic"))
        .unwrap();
    assert_eq!(paragraphs(&core), expected);
}

#[test]
fn native_wrapper_command_keeps_constructor_compatibility() {
    let input = fixture("samples/HWP5-password-123456.hwpx");
    let compatibility = HwpDocument::from_bytes_with_password(&input, b"123456").unwrap();
    let mut document = HwpDocument::from(DocumentCore::new_empty());
    document.load_bytes(&input, Some(b"123456")).unwrap();
    assert_eq!(paragraphs(&document), paragraphs(&compatibility));
    assert_eq!(
        document.document().header.encrypted,
        compatibility.document().header.encrypted
    );
}

#[test]
fn cli_export_reports_persisted_size_without_exposing_output_credentials() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = root.join("output/tests/issue8745").join(format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let synthetic_value = "synthetic-open-8745";
    for (command, extension) in [("convert", "hwp"), ("export-hwpx", "hwpx")] {
        for encrypted in [false, true] {
            let path = directory.join(format!("document-{encrypted}.{extension}"));
            let binary = std::env::var_os("CARGO_BIN_EXE_rhwp")
                .unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
            let mut cli = Command::new(binary);
            if encrypted {
                cli.arg("--output-password-stdin");
            }
            let mut child = cli
                .arg(command)
                .arg(root.join("saved/blank2010.hwp"))
                .arg(&path)
                .arg("--json")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            if encrypted {
                let mut input = child.stdin.take().unwrap();
                writeln!(input, "{synthetic_value}").unwrap();
            } else {
                drop(child.stdin.take());
            }
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success(), "export failed for {command}");
            for channel in [&output.stdout, &output.stderr] {
                assert!(!String::from_utf8_lossy(channel).contains(synthetic_value));
            }
            let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(
                report["bytes"].as_u64(),
                Some(std::fs::metadata(&path).unwrap().len())
            );
            assert_eq!(report["passwordProtected"], encrypted);
            let bytes = std::fs::read(&path).unwrap();
            let mut core = DocumentCore::new_empty();
            if encrypted {
                assert!(core.load_bytes(&bytes, None).is_err());
            }
            core.load_bytes(&bytes, encrypted.then_some(synthetic_value.as_bytes()))
                .unwrap();
            assert!(!core.document().sections.is_empty());
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "pdf-import")]
#[test]
fn authenticated_pdf_load_keeps_images_and_failure_is_atomic() {
    // Public independently generated controls: pdf/issue8415/authentication/README.md.
    let plain = fixture("pdf/issue8415/authentication/password-control.pdf");
    let mut core = DocumentCore::from_bytes(&plain).unwrap();
    let expected = paragraphs(&core);
    let expected_image = core.document().bin_data_content[0].data.load().to_vec();
    for revision in 2..=6 {
        let encrypted = fixture(&format!(
            "pdf/issue8415/authentication/password-r{revision}.pdf"
        ));
        for credential in [None, Some(b"incorrect-8745".as_slice())] {
            let error = core.load_bytes(&encrypted, credential).unwrap_err();
            assert!(!error.to_string().contains("incorrect-8745"));
            assert_eq!(paragraphs(&core), expected);
            assert_eq!(
                core.document().bin_data_content[0].data.load(),
                expected_image
            );
        }
        for credential in [b"synthetic-open".as_slice(), b"synthetic-owner".as_slice()] {
            core.load_bytes(&encrypted, Some(credential)).unwrap();
            assert_eq!(paragraphs(&core), expected);
            assert_eq!(
                core.document().bin_data_content[0].data.load(),
                expected_image
            );
        }
    }
}

#[test]
fn hwp5_lazy_image_debug_hides_retained_credential() {
    let original = fixture("samples/hwp3-sample16-hwp5-2024-password-123456.hwp");
    let document = rhwp::parse_document_with_password(&original, b"123456").unwrap();
    let synthetic_value = b"synthetic-8745-lazy-image-secret";
    let encrypted =
        rhwp::serializer::serialize_hwp_with_password(&document, synthetic_value).unwrap();
    let mut core = DocumentCore::new_empty();
    core.load_bytes(&encrypted, Some(synthetic_value)).unwrap();
    let binaries = &core.document().bin_data_content;
    assert!(
        !binaries.is_empty(),
        "fixture must exercise lazy image resources"
    );
    let debug = format!("{binaries:?}");
    assert!(
        debug.contains("Hwp5BinResolver"),
        "must exercise the real lazy resolver"
    );
    assert!(!debug.contains(std::str::from_utf8(synthetic_value).unwrap()));
    assert!(
        !debug.contains(&format!("{synthetic_value:?}")),
        "byte-vector credential leaked"
    );
    let before: Vec<_> = document
        .bin_data_content
        .iter()
        .map(|item| item.data.load())
        .collect();
    let after: Vec<_> = binaries.iter().map(|item| item.data.load()).collect();
    assert_eq!(
        before, after,
        "deferred authenticated images must remain usable"
    );
}
