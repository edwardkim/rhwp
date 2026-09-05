//! [#3694] did-you-mean — 이름 환각 교정 단서 (#3630 P1 구현).
//! 후보는 capabilities 명령 목록 단일 출처, 임계 초과 시 무제안(오제안 0 원칙).
#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

#[test]
fn unknown_command_hints_closest_and_keeps_exit_2() {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("exprot-svg") // 오타
        .output()
        .expect("rhwp");
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("힌트: 가장 가까운 명령은 'export-svg' 입니다"),
        "{err}"
    );
}

#[test]
fn gibberish_command_gets_no_hint() {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("코끼리코끼리")
        .output()
        .expect("rhwp");
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("힌트:"), "임계 초과는 무제안: {err}");
}
