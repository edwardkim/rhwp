//! Issue #1914 — 확장자-실체 불일치 파일의 매직 바이트 스니핑.
//!
//! 정부 배포 채널에서 HWP5(OLE/CFB) 실체 파일이 `.hwpx` 확장자로 다수 유통된다
//! (10k 서베이: .hwpx 4,580건 중 49건). 제품 로드 경로(`parse_document`/
//! `DocumentCore::from_bytes`)는 내용 기반 감지로 이미 열리며(한글 정합),
//! 확장자를 신뢰하던 roundtrip 게이트 CLI 는 PARSE_FAIL("ZIP EOCD") 오분류
//! 대신 FORMAT_SKIP(실체·올바른 게이트 안내)으로 분류한다.

use std::fs;
use std::path::Path;
use std::process::Command;

use rhwp::parser::{detect_format, parse_document, FileFormat};

fn sample_bytes(rel: &str) -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    fs::read(Path::new(repo_root).join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// 제품 로드 경로 핀: 세 포맷 모두 파일명/확장자와 무관하게 내용으로 감지·파싱된다.
#[test]
fn issue_1914_parse_document_sniffs_content_for_all_formats() {
    let cases = [
        ("samples/hwpers_test4_complex_table.hwp", FileFormat::Hwp),
        (
            "samples/issue1892_hwp3_drawing_group_roundtrip.hwp",
            FileFormat::Hwp3,
        ),
        (
            "samples/issue1893_clickhere_field_roundtrip.hwpx",
            FileFormat::Hwpx,
        ),
    ];
    for (rel, expected) in cases {
        let bytes = sample_bytes(rel);
        assert_eq!(
            detect_format(&bytes),
            expected,
            "{rel}: 매직 바이트 감지 포맷 불일치"
        );
        // parse_document 는 바이트만 받는다 — 확장자 개입 여지 자체가 없음을 핀.
        parse_document(&bytes).unwrap_or_else(|e| panic!("{rel}: 내용 기반 파싱 실패: {e}"));
    }
}

/// roundtrip 게이트 CLI 핀: 확장자 위장 파일은 실패가 아닌 FORMAT_SKIP + exit 0.
#[test]
fn issue_1914_roundtrip_gates_classify_masqueraded_files_as_format_skip() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tmp = std::env::temp_dir().join("issue_1914_gate_probe");
    fs::create_dir_all(&tmp).expect("temp dir");

    // HWP5(OLE) 실체 → .hwpx 확장자 (10k 서베이 49건 클래스)
    let masq_hwpx = tmp.join("ole_as.hwpx");
    fs::copy(
        repo_root.join("samples/hwpers_test4_complex_table.hwp"),
        &masq_hwpx,
    )
    .expect("copy masq hwpx");
    // HWP3 실체 → .hwp 확장자 (hwp5 게이트 범위 밖 — #1892 도구 축)
    let hwp3_as_hwp = repo_root.join("samples/issue1892_hwp3_drawing_group_roundtrip.hwp");

    let exe = env!("CARGO_BIN_EXE_rhwp");
    let out_dir = tmp.join("out");

    let run = |args: &[&str]| -> (String, bool) {
        let output = Command::new(exe).args(args).output().expect("run rhwp");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        (text, output.status.success())
    };

    let (text, ok) = run(&[
        "hwpx-roundtrip",
        masq_hwpx.to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
    ]);
    assert!(
        text.contains("FORMAT_SKIP") && text.contains("HWP5"),
        "hwpx-roundtrip 위장 HWP5: FORMAT_SKIP+실체 안내여야 함 (종전 PARSE_FAIL EOCD): {text}"
    );
    assert!(ok, "FORMAT_SKIP 은 하드 실패가 아님 (exit 0): {text}");

    let (text, ok) = run(&[
        "hwp5-roundtrip",
        hwp3_as_hwp.to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
    ]);
    assert!(
        text.contains("FORMAT_SKIP") && text.contains("HWP3"),
        "hwp5-roundtrip HWP3 실체: FORMAT_SKIP+실체 안내여야 함 (종전 오도성 IR_DIFF, #1892): {text}"
    );
    assert!(ok, "FORMAT_SKIP 은 하드 실패가 아님 (exit 0): {text}");
}
