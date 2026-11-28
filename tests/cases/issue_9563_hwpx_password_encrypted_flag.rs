//! #9563: 암호로 연 HWPX 의 `header.encrypted` 가 false 로 남던 문제.
//!
//! HWPX 파서는 모델 FileHeader 를 만들 때 `encrypted: false` 로 고정했고, 암호를 풀어 연 경우에도
//! 바꾸지 않았다. 그래서 같은 조건의 HWP5 암호 문서(`true`)와 달리 문서 정보·`info`·`explain`·`explore`
//! 등에서 암호 없는 문서로 보였다. 수정 뒤에도 저장은 이 값에 따라 달라지지 않아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::document_core::DocumentCore;

/// 비밀번호 "123456" 로 걸린 HWPX(ODF AES-256-CBC).
const SAMPLE_ENCRYPTED_HWPX: &str = "samples/HWP5-password-123456.hwpx";
const PASSWORD: &[u8] = b"123456";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn encrypted_hwpx() -> Vec<u8> {
    std::fs::read(sample(SAMPLE_ENCRYPTED_HWPX)).expect("암호 HWPX 샘플")
}

fn document_info(core: &DocumentCore) -> serde_json::Value {
    serde_json::from_str(&core.get_document_info()).expect("문서 정보 JSON")
}

/// [#3289] nextest 가 런타임에 주입하는 CARGO_BIN_EXE_rhwp 를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

#[test]
fn issue_9563_password_opened_hwpx_reports_encrypted() {
    let core =
        DocumentCore::from_bytes_with_password(&encrypted_hwpx(), PASSWORD).expect("암호로 열기");
    assert!(
        core.document().header.encrypted,
        "암호로 연 HWPX 는 encrypted 여야 한다"
    );
    assert_eq!(document_info(&core)["encrypted"], true);
}

#[test]
fn issue_9563_plain_hwpx_reports_not_encrypted() {
    // 같은 문서를 암호 없이 HWPX 로 저장한 평문 HWPX. 비밀번호를 넘겨 열어도 평문이면 false 다.
    let core =
        DocumentCore::from_bytes_with_password(&encrypted_hwpx(), PASSWORD).expect("암호로 열기");
    let plain = core.export_hwpx_native().expect("평문 HWPX 저장");
    let reopened = DocumentCore::from_bytes(&plain).expect("평문 HWPX 열기");
    assert!(!reopened.document().header.encrypted);
    assert_eq!(document_info(&reopened)["encrypted"], false);
    let with_password =
        DocumentCore::from_bytes_with_password(&plain, PASSWORD).expect("평문 HWPX 에 비밀번호");
    assert!(
        !with_password.document().header.encrypted,
        "평문 HWPX 에 비밀번호를 넘겨도 encrypted 가 아니다"
    );
}

#[test]
fn issue_9563_encrypted_flag_does_not_change_saved_bytes() {
    // 저장 경로는 이 값으로 암호를 걸지 않는다: 같은 문서의 플래그만 바꿔 저장한 HWP·HWPX 바이트가 같다.
    let mut flagged =
        DocumentCore::from_bytes_with_password(&encrypted_hwpx(), PASSWORD).expect("암호로 열기");
    let mut cleared =
        DocumentCore::from_bytes_with_password(&encrypted_hwpx(), PASSWORD).expect("암호로 열기");
    assert!(flagged.document_mut().header.encrypted);
    cleared.document_mut().header.encrypted = false;
    assert_eq!(
        flagged.export_hwp_with_adapter().expect("HWP 저장"),
        cleared.export_hwp_with_adapter().expect("HWP 저장"),
        "HWP 저장본이 encrypted 플래그에 따라 달라지면 안 된다"
    );
    assert_eq!(
        flagged.export_hwpx_native().expect("HWPX 저장"),
        cleared.export_hwpx_native().expect("HWPX 저장"),
        "HWPX 저장본이 encrypted 플래그에 따라 달라지면 안 된다"
    );
    // 비밀번호 없이 저장한 HWP 는 평문으로 다시 열린다(저장 → 다시 열기 → 재저장 → 다시 열기)
    let hwp = flagged.export_hwp_with_adapter().expect("HWP 저장");
    let reopened = DocumentCore::from_bytes(&hwp).expect("평문 HWP 다시 열기");
    assert!(!reopened.document().header.encrypted);
    let again = DocumentCore::from_bytes(&reopened.export_hwp_native().expect("HWP 재저장"))
        .expect("다시 열기");
    assert!(!again.document().header.encrypted);
    // 비밀번호를 주어 저장하면 암호 문서가 되고, 그 비밀번호로 열면 encrypted 다
    let protected = flagged
        .export_hwp_with_adapter_with_password(PASSWORD)
        .expect("암호 HWP 저장");
    assert!(
        DocumentCore::from_bytes(&protected).is_err(),
        "비밀번호 없이 열리지 않음"
    );
    let reopened =
        DocumentCore::from_bytes_with_password(&protected, PASSWORD).expect("암호 HWP 열기");
    assert!(reopened.document().header.encrypted);
}

#[test]
fn issue_9563_cli_explain_reports_encrypted_for_password_hwpx() {
    let p = sample(SAMPLE_ENCRYPTED_HWPX);
    let args = [
        "explain",
        p.to_str().unwrap(),
        "--password",
        "123456",
        "--json",
    ];
    let output = Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["encrypted"], true, "{v}");
}
