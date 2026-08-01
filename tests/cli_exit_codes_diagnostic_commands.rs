//! [#2707 후속] `info`/`dump-note-shape`/`dump-endnote-lines`/`dump-pages`/
//! `dump-records`/`build-from-ingest` 도 export 계열과 동일한 종료 코드 계약을 따라야 한다.
//!
//! #2707 은 export-* / convert / export-hwpx 만 고쳤고, 같은 클래스(치명 실패에도
//! 종료 코드 0)가 이 진단·조립 명령들에 남아 있다고 명시적으로 §6.1 에 기록했다
//! (`mydocs/report/task_m100_2707_report.md`). 이 테스트는 그 잔여를 봉인한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";
/// HWP5(CFB) 샘플 — `dump-records` 는 HWP3 CFB 아닌 입력을 지원하지 않는다.
const HWP5_SAMPLE: &str = "samples/2010-01-06.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn hwp5_sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(HWP5_SAMPLE)
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

/// 인자 없이 호출 → 사용법 오류(2).
#[test]
fn missing_arguments_report_usage_error() {
    for cmd in ["info", "dump-note-shape", "dump-pages", "dump-records"] {
        assert_code(&[cmd], 2);
    }
    // dump-endnote-lines 는 인자 4개 미만이면 사용법 오류.
    assert_code(&["dump-endnote-lines", "x.hwp"], 2);
    assert_code(&["build-from-ingest"], 2);
}

/// 존재하지 않는 입력 파일 → 런타임 실패(1). #2707 이전에는 전부 0이었다.
#[test]
fn unreadable_input_reports_runtime_failure() {
    assert_code(&["info", "does-not-exist.hwp"], 1);
    assert_code(&["dump-note-shape", "does-not-exist.hwp"], 1);
    assert_code(&["dump-pages", "does-not-exist.hwp"], 1);
    assert_code(&["dump-records", "does-not-exist.hwp"], 1);
    assert_code(
        &["dump-endnote-lines", "does-not-exist.hwp", "0", "0", "0"],
        1,
    );
    assert_code(
        &["build-from-ingest", "does-not-exist.json", "-o", "out.hwpx"],
        1,
    );
}

/// dump-pages 페이지 범위 초과 → 사용법 오류(2) (형제 명령과 정합, #2551 후속 확인).
#[test]
fn dump_pages_out_of_range_reports_usage_error() {
    let sample = sample_path();
    let sample = sample.to_str().expect("valid utf8 path");
    assert_code(&["dump-pages", sample, "-p", "999999"], 2);
}

/// build-from-ingest 출력 경로 누락 → 사용법 오류(2).
#[test]
fn build_from_ingest_missing_output_reports_usage_error() {
    assert_code(
        &[
            "build-from-ingest",
            "tools/rhwp-ingest/schema/sample_minimal.json",
        ],
        2,
    );
}

/// 성공 경로는 여전히 0이어야 한다 (회귀 방지).
#[test]
fn successful_diagnostic_commands_return_zero() {
    let sample = sample_path();
    let sample = sample.to_str().expect("valid utf8 path");
    for cmd in ["info", "dump-note-shape", "dump-pages"] {
        let output = run(&[cmd, sample]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            describe(&[cmd, sample], &output)
        );
    }

    let hwp5_sample = hwp5_sample_path();
    let hwp5_sample = hwp5_sample.to_str().expect("valid utf8 path");
    let output = run(&["dump-records", hwp5_sample]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&["dump-records", hwp5_sample], &output)
    );
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

/// `render-diff` 는 읽기·파싱 실패를 런타임 오류(1)로 보고해야 한다.
///
/// 종전에는 세 갈래(A 로드·B 로드·자기 라운드트립 읽기)가 모두 2를 냈다. 계약상 2는
/// **인자 개수/형식이 틀린 사용법 오류** 전용이라, 재시도 래퍼가 "입력 파일이 없다"를
/// "내 호출이 틀렸다"로 오독해 인자를 고치려 든다 — 고칠 게 없는데.
#[test]
fn render_diff_separates_runtime_failure_from_usage_error() {
    // 읽기 실패 = 1 (두 파일 형태, 한 파일 형태 모두)
    assert_code(&["render-diff", "없는A.hwp", "없는B.hwp"], 1);
    assert_code(&["render-diff", "없는하나.hwp"], 1);
    // 인자 개수가 틀린 것은 여전히 2 — 이쪽을 1로 바꾸면 안 된다.
    assert_code(&["render-diff"], 2);
}

/// `test-field` 는 패닉(101)이 아니라 계약 코드로 끝나야 한다.
///
/// 종전에는 인자를 생략하면 저장소에 없는 하드코딩 경로를 `.expect()` 로 읽어
/// exit 101 이었다 — 계약(0/1/2/3/4)에 없는 코드라 CI 게이트가 분류할 수 없다.
/// 형제 명령 `test-caption` 은 이미 같은 계약을 지키고 있다.
#[test]
fn test_field_reports_contract_codes_instead_of_panicking() {
    let no_args = assert_code(&["test-field"], 2);
    assert!(
        !String::from_utf8_lossy(&no_args.stderr).contains("panicked"),
        "패닉 흔적이 남아 있습니다: {}",
        describe(&["test-field"], &no_args)
    );
    let missing = assert_code(&["test-field", "없는파일-testfield.hwp"], 1);
    assert!(
        !String::from_utf8_lossy(&missing.stderr).contains("panicked"),
        "패닉 흔적이 남아 있습니다: {}",
        describe(&["test-field", "없는파일-testfield.hwp"], &missing)
    );
}

/// `gen-pua` 의 positional 은 **출력** 경로다 — 기존 파일을 덮어쓰지 않는다.
///
/// capabilities 가 다른 진단 명령과 나란히 노출하는 탓에 `rhwp gen-pua 문서.hwp` 를
/// "이 문서를 조사"로 읽은 호출이 실제로 원본을 말없이 덮어썼다(조사 중 발생). 조사
/// 목적 호출이 데이터를 파괴하면 안 되므로, 사용자가 명시한 경로가 이미 있으면 거부한다.
#[test]
fn gen_pua_refuses_to_overwrite_an_existing_file() {
    let victim = std::env::temp_dir().join(format!(
        "rhwp-genpua-victim-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    let original = b"NOT-A-REAL-HWP-BUT-MUST-SURVIVE".to_vec();
    std::fs::write(&victim, &original).expect("피해자 파일 생성");

    let args = ["gen-pua", victim.to_str().unwrap()];
    let output = assert_code(&args, 2);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("출력"),
        "인자가 출력 경로임을 밝혀야 합니다: {}",
        describe(&args, &output)
    );

    let after = std::fs::read(&victim).expect("피해자 파일 재독");
    assert_eq!(
        after, original,
        "gen-pua 가 기존 파일을 덮어썼습니다 — 조사 목적 호출이 데이터를 파괴합니다"
    );
    let _ = std::fs::remove_file(&victim);
}
