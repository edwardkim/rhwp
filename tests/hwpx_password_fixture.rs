//! 관계형·고정형: 독립 Print의 수용 범위에서 원본·공개 API·내용 소유를 보호한다.
//! 주석 갱신: 2026-11-22T02:35:34+09:00; 직전 파일 변경 커밋: 2026-11-05T08:04:33+09:00 d75809517f9185277248dd6be995e89792563222.
//! 측정: 2026-11-22 KST, Print profile, 96DPI, PDFium, 보정 전2px raw.
//! 실제 암호 HWPX와 평문 대응본 전24쪽: Native 최저98.29%, fresh WASM 최저98.22%; 양쪽에서 암호/평문24쪽 PNG가 각각 동일.
//! 근거: mydocs/pr/assets/issues7754_7765/source32/README.md와 문서별 Native/WASM 전쪽 TSV.
//! 암호 문서의 쪽수는 독립 Print의24쪽이며 실제 API·CLI·MCP 경로에서 확인한다.
//!
//!
//! HWPX는 manifest의 `encryption-data`마다 raw-deflate → AES-256-CBC를 적용한다.
//! 합성 ZIP만으로는 PBKDF2 PRF·manifest checksum·lazy BinData 재열기 회귀를 막을 수
//! 없으므로, 실제 암호 HWPX와 같은 문서의 평문 HWPX를 함께 검증한다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use rhwp::parser::{hwpx, parse_document, ParseError};
use rhwp::{parse_document_with_password, wasm_api::HwpDocument};

const ENCRYPTED_FIXTURE: &str = "samples/HWP5-password-123456.hwpx";
const PLAIN_FIXTURE: &str = "samples/HWP5-nopassword-123456.hwpx";
const WRONG_PASSWORD_MESSAGE: &str = "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다";
const FIXTURE_PASSWORD: &[u8] = &[49, 50, 51, 52, 53, 54];

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn fixture_bytes(relative: &str) -> Vec<u8> {
    std::fs::read(fixture_path(relative)).expect("HWPX password fixture를 읽어야 함")
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn run_with_password_stdin(args: &[&str], password: &[u8]) -> Output {
    let mut child = Command::new(rhwp_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    let mut stdin = child.stdin.take().expect("stdin pipe");
    stdin.write_all(password).expect("비밀번호 쓰기");
    stdin.write_all(b"\n").expect("개행 쓰기");
    drop(stdin);
    child.wait_with_output().expect("rhwp 종료 대기 실패")
}

#[test]
fn protected_document_layout_diagnostics_do_not_write_coordinates_to_stderr() {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("output/tests/protected-layout-diagnostics")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("current time")
                .as_nanos()
        ));
    let mut child = Command::new(rhwp_bin())
        .arg("--password-stdin")
        .arg("export-svg")
        .arg(fixture_path(ENCRYPTED_FIXTURE))
        .arg("-o")
        .arg(&output_dir)
        .arg("--json")
        .env("RHWP_DIAG_SNAPALL", "1")
        .env("RHWP_DIAG_TAC", "1")
        .env("RHWP_DEBUG_PARA_TAC", "1")
        .env("RHWP_LAYOUT_DEBUG", "1")
        .env("RHWP_DIAG_ROWH", "1")
        .env("RHWP_DIAG_ROWH_LINES", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start protected document export");
    let mut stdin = child.stdin.take().expect("password stdin");
    stdin
        .write_all(FIXTURE_PASSWORD)
        .expect("write fixture password");
    stdin.write_all(b"\n").expect("terminate fixture password");
    drop(stdin);
    let output = child
        .wait_with_output()
        .expect("finish protected document export");
    assert!(output.status.success(), "protected SVG export failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for marker in [
        "DIAG_TRUST",
        "DIAG_ROWH",
        "DIAG_SPANH",
        "DIAG_SNAPALL",
        "DIAG_SNAP ",
        "TAC_LINE ",
        "LAYOUT_INLINE_TABLE_PARA:",
        "LAYOUT_LS[",
        "LAYOUT_INLINE_TBL[",
        "LAYOUT_OVERFLOW_DRAW:",
    ] {
        assert!(
            !stderr.contains(marker),
            "protected layout diagnostic channel: {marker}"
        );
    }
    assert!(
        std::fs::read_dir(&output_dir)
            .expect("SVG output directory")
            .any(|entry| entry
                .expect("SVG output entry")
                .path()
                .extension()
                .is_some_and(|ext| ext == "svg")),
        "functional SVG export must still produce document pages"
    );
    std::fs::remove_dir_all(output_dir).expect("remove generated SVG output");
}

fn document_shape(document: &rhwp::model::document::Document) -> (usize, usize, Vec<usize>) {
    (
        document.sections.len(),
        document
            .sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum(),
        document
            .bin_data_content
            .iter()
            .map(|content| content.data.load().len())
            .collect(),
    )
}

#[test]
fn actual_odf_hwpx_fixture_requires_the_password_and_matches_plain_counterpart() {
    let encrypted = fixture_bytes(ENCRYPTED_FIXTURE);

    assert!(matches!(
        parse_document(&encrypted),
        Err(ParseError::EncryptedDocument)
    ));
    assert!(matches!(
        hwpx::parse_hwpx(&encrypted),
        Err(hwpx::HwpxError::Encrypted(_))
    ));

    let wrong = parse_document_with_password(&encrypted, b"wrong-fixture-password")
        .expect_err("잘못된 비밀번호는 암호 HWPX를 열면 안 됨");
    assert!(
        wrong.to_string().contains(WRONG_PASSWORD_MESSAGE),
        "wrong password error: {wrong}"
    );

    let decrypted = parse_document_with_password(&encrypted, FIXTURE_PASSWORD)
        .expect("실제 ODF AES-256-CBC HWPX fixture를 열어야 함");
    let plain = parse_document(&fixture_bytes(PLAIN_FIXTURE))
        .expect("같은 문서의 평문 HWPX fixture를 열어야 함");
    assert_eq!(decrypted.header.version.major, 5);
    // 암호를 풀어 연 문서는 원본이 암호 문서다 — HWP3·HWP5 암호 fixture 와 같은 계약(#9563).
    // 종전 기대값(false)은 #3516 에서 들어왔으나 근거가 남아 있지 않다.
    assert!(decrypted.header.encrypted);
    assert_eq!(
        document_shape(&decrypted),
        (1, 365, vec![85078, 93678, 738])
    );
    assert_eq!(document_shape(&decrypted), document_shape(&plain));
    let page_border_fill_id = plain.sections[0]
        .section_def
        .page_border_fill
        .border_fill_id;
    let page_background = plain.doc_info.border_fills[(page_border_fill_id - 1) as usize]
        .fill
        .image
        .as_ref()
        .expect("HWPX 원본의 쪽 배경 imgBrush를 보존해야 함");
    assert_eq!(
        (page_background.brightness, page_background.contrast),
        (-15, 50),
        "header.xml의 <hc:img bright=\"50\" contrast=\"-15\">는 HWP5 공통 ImageFill 저장 순서로 정규화해야 함"
    );
    assert!(
        decrypted.sections[0]
            .paragraphs
            .iter()
            .any(|paragraph| paragraph.text.contains("ᄒᆞᆫ글\u{2007}97 안내문")),
        "복호화 뒤 section 본문을 유지해야 함"
    );

    // 비암호 HWPX에 비밀번호를 전달해도 복호화 ZIP 재작성 없이 종전 결과를 유지한다.
    let plain_with_password =
        parse_document_with_password(&fixture_bytes(PLAIN_FIXTURE), b"ignored")
            .expect("평문 HWPX는 전달된 비밀번호를 무시하고 열어야 함");
    assert_eq!(document_shape(&plain_with_password), document_shape(&plain));

    let document = HwpDocument::from_bytes_with_password(&encrypted, FIXTURE_PASSWORD)
        .expect("공개 HwpDocument API도 암호 HWPX fixture를 열어야 함");
    let plain_document =
        HwpDocument::from_bytes(&fixture_bytes(PLAIN_FIXTURE)).expect("같은 원본의 평문 대응본");
    assert_eq!(document.page_count(), 24, "독립 Print의 전체 쪽수");
    assert_eq!(document.page_count(), plain_document.page_count());
    for page in 0..document.page_count() {
        let protected: serde_json::Value = serde_json::from_str(
            &document
                .get_page_text_layout_native(page)
                .expect("암호 문서의 실제 쪽 내용"),
        )
        .unwrap();
        let plain: serde_json::Value = serde_json::from_str(
            &plain_document
                .get_page_text_layout_native(page)
                .expect("평문 대응본의 실제 쪽 내용"),
        )
        .unwrap();
        assert_eq!(
            protected["runs"],
            plain["runs"],
            "{}쪽 내용·소유·순서를 보존한다",
            page + 1
        );
    }
}

#[test]
fn cli_password_exit_contract_uses_the_actual_odf_hwpx_fixture() {
    let fixture = fixture_path(ENCRYPTED_FIXTURE);
    let fixture = fixture.to_str().expect("utf-8 fixture path");

    let missing = run(&["info", fixture]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("비밀번호가 필요한 암호 문서"),
        "missing password stderr: {}",
        String::from_utf8_lossy(&missing.stderr)
    );

    let wrong = run_with_password_stdin(
        &["info", fixture, "--password-stdin"],
        b"wrong-fixture-password",
    );
    assert_eq!(wrong.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&wrong.stderr).contains(WRONG_PASSWORD_MESSAGE),
        "wrong password stderr: {}",
        String::from_utf8_lossy(&wrong.stderr)
    );

    let opened = run_with_password_stdin(&["info", fixture, "--password-stdin"], FIXTURE_PASSWORD);
    assert_eq!(opened.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&opened.stdout);
    assert!(stdout.contains("페이지 수: 24"), "CLI stdout: {stdout}");
}

#[test]
fn cli_ir_diff_accepts_password_stdin_for_actual_odf_hwpx_fixture() {
    let encrypted = fixture_path(ENCRYPTED_FIXTURE);
    let plain = fixture_path(PLAIN_FIXTURE);
    let encrypted = encrypted.to_str().expect("utf-8 fixture path");
    let plain = plain.to_str().expect("utf-8 fixture path");

    let output = run_with_password_stdin(
        &["ir-diff", encrypted, plain, "--json", "--password-stdin"],
        FIXTURE_PASSWORD,
    );
    assert!(
        matches!(output.status.code(), Some(0) | Some(3)),
        "암호 HWPX 비교는 암호 미전달 오류가 아니라 동일/차이 판정을 내야 함: status={:?}, stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("EncryptedDocument"),
        "ir-diff는 --password-stdin을 일반 열기 명령과 같이 적용해야 함: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ir-diff --json은 판정 봉투를 출력해야 함");
    assert!(
        envelope.get("identical").is_some() && envelope.get("diffCount").is_some(),
        "ir-diff JSON 계약: {envelope}"
    );
}
