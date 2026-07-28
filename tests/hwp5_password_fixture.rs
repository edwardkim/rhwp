//! 실제 한컴 HWP5 EncryptVersion 4 fixture의 복호화 회귀 계약.
//!
//! `hwp3-sample16-hwp5-2024-password-123456.hwp`는 공개 검증용 fixture이며, 아래
//! 비밀번호 바이트도 재현용 비밀값이 아니다. 합성 CFB test만으로는 실제 한컴
//! 스트림의 키 파생·CFB·압축·BinData 조합 회귀를 막을 수 없으므로 함께 고정한다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use rhwp::parser::{parse_document, ParseError};
use rhwp::{parse_document_with_password, wasm_api::HwpDocument};

const FIXTURE: &str = "samples/hwp3-sample16-hwp5-2024-password-123456.hwp";
const WRONG_PASSWORD_MESSAGE: &str = "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다";
const FIXTURE_PASSWORD: &[u8] = &[49, 50, 51, 52, 53, 54];

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
}

fn fixture_bytes() -> Vec<u8> {
    std::fs::read(fixture_path()).expect("암호 HWP5 fixture를 읽어야 함")
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
fn actual_encryptversion4_fixture_requires_the_password_and_preserves_structure() {
    let bytes = fixture_bytes();

    assert!(matches!(
        parse_document(&bytes),
        Err(ParseError::EncryptedDocument)
    ));

    let wrong = parse_document_with_password(&bytes, b"wrong-fixture-password")
        .expect_err("잘못된 비밀번호는 문서를 열면 안 됨");
    assert!(
        wrong.to_string().contains(WRONG_PASSWORD_MESSAGE),
        "wrong password error: {wrong}"
    );

    let document = parse_document_with_password(&bytes, FIXTURE_PASSWORD)
        .expect("실제 EncryptVersion 4 fixture를 열어야 함");
    assert_eq!(document.header.version.major, 5);
    assert!(document.header.encrypted);
    assert_eq!(document.sections.len(), 1);
    assert_eq!(
        document
            .sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum::<usize>(),
        1058
    );
    assert_eq!(document.bin_data_content.len(), 7);

    let hwp_document = HwpDocument::from_bytes_with_password(&bytes, FIXTURE_PASSWORD)
        .expect("공개 HwpDocument API도 fixture를 열어야 함");
    assert_eq!(hwp_document.page_count(), 64);

    let saved = hwp_document
        .export_hwp_native()
        .expect("암호 문서를 일반 HWP로 저장해야 함");
    let reparsed = parse_document(&saved).expect("저장한 일반 HWP를 비밀번호 없이 다시 열어야 함");
    assert!(!reparsed.header.encrypted);
    assert_eq!(reparsed.sections.len(), 1);
    assert_eq!(
        reparsed
            .sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum::<usize>(),
        1058
    );
}

#[test]
fn cli_password_exit_contract_uses_the_actual_fixture() {
    let fixture = fixture_path();
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
    assert!(stdout.contains("암호화: 예"), "CLI stdout: {stdout}");
    assert!(stdout.contains("페이지 수: 64"), "CLI stdout: {stdout}");
}
