//! #8160: sample16 전용 회귀. 이전 등록: tests/cli_exit_codes_diagnostic_commands.rs
//! [#2707 후속] `info`/`dump-note-shape`/`dump-endnote-lines`/`dump-pages`/
//! `dump-records`/`build-from-ingest` 도 export 계열과 동일한 종료 코드 계약을 따라야 한다.
//!
//! #2707 은 export-* / convert / export-hwpx 만 고쳤고, 같은 클래스(치명 실패에도
//! 종료 코드 0)가 이 진단·조립 명령들에 남아 있다고 명시적으로 §6.1 에 기록했다
//! (`mydocs/report/task_m100_2707_report.md`). 이 테스트는 그 잔여를 봉인한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// HWP5(CFB) 샘플 — `dump-records` 는 HWP3 CFB 아닌 입력을 지원하지 않는다.
const HWP5_PASSWORD_SAMPLE: &str = "samples/hwp3-sample16-hwp5-2024-password-123456.hwp";
const HWP5_PASSWORD: &str = "123456";

fn hwp5_password_sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(HWP5_PASSWORD_SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_code(args: &[&str], expected: i32) -> Output {
    let output = run(args);
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        describe(args, &output)
    );
    output
}

#[test]
fn issue_8160_dump_records_preserves_hwp5_password_exit_and_output_contract() {
    let encrypted = hwp5_password_sample_path();
    let encrypted = encrypted.to_str().expect("valid utf8 path");

    let missing = assert_code(&["dump-records", encrypted], 2);
    assert!(missing.stdout.is_empty(), "{}", describe(&[], &missing));
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("비밀번호가 필요한 암호 문서"),
        "{}",
        describe(&[], &missing)
    );

    let wrong = assert_code(
        &[
            "dump-records",
            encrypted,
            "--password",
            "wrong-fixture-password",
        ],
        1,
    );
    assert!(wrong.stdout.is_empty(), "{}", describe(&[], &wrong));
    assert!(
        String::from_utf8_lossy(&wrong.stderr).contains("비밀번호 불일치 또는 복호화 실패"),
        "{}",
        describe(&[], &wrong)
    );

    let opened = assert_code(&["dump-records", encrypted, "--password", HWP5_PASSWORD], 0);
    assert!(!opened.stdout.is_empty(), "{}", describe(&[], &opened));
    assert!(opened.stderr.is_empty(), "{}", describe(&[], &opened));
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
