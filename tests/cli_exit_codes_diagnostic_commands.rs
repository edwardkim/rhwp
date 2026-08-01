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

/// `hwp5-*` 프로브 8종의 종료 코드 계약.
///
/// 진입점이 `pub fn run(args: &[String])` 즉 반환형이 유닛이라 `main` 의 `exit_with` 를
/// 통과하지 못했다 — 인자를 빠뜨리든 파일이 없든 **무조건 exit 0** 이었다. `&&` 나
/// `set -e` 로 엮은 스크립트, CI 게이트가 실패를 성공으로 읽는다.
///
/// 세 갈래를 함께 본다: 명시적 `--help` 만 성공(0), 인자 누락·파싱 실패는 사용법
/// 오류(2), 인자가 갖춰진 뒤의 실행 실패는 런타임 오류(1).
#[test]
fn hwp5_probes_report_contract_exit_codes() {
    const PROBES: [&str; 8] = [
        "hwp5-borderfill-diagonal-probe",
        "hwp5-cell-header-probe",
        "hwp5-contract-analyze",
        "hwp5-contract-probe",
        "hwp5-ctrl-data-trace",
        "hwp5-first-para-control-probe",
        "hwp5-mel-personnel-probe",
        "hwp5-table-probe",
    ];
    for probe in PROBES {
        // ① 인자 없음 → 사용법 오류. 종전에는 0 이었다.
        let no_args = assert_code(&[probe], 2);
        assert!(
            String::from_utf8_lossy(&no_args.stdout).trim().is_empty(),
            "사용법 안내는 stderr 로 나가야 합니다({probe}): {}",
            describe(&[probe], &no_args)
        );

        // ② 명시적 --help 는 성공이다 — 도움말을 물어본 호출까지 실패로 만들면 안 된다.
        assert_code(&[probe, "--help"], 0);
    }
}

/// 인자가 갖춰진 뒤의 읽기·파싱 실패는 런타임 오류(1)여야 한다.
///
/// 프로브마다 필요한 인자 개수가 달라(2개짜리·3개짜리) 한 벌로 묶을 수 없다.
/// 인자 개수를 채운 대표 2종만 확인한다 — 반환형 변경이 실제로 실패를 전달하는지가
/// 요점이고, 그건 한 갈래만 통과해도 증명된다.
#[test]
fn hwp5_probes_report_runtime_failure_after_arity_is_satisfied() {
    let out_dir = std::env::temp_dir().join(format!("rhwp-probe-{}", std::process::id()));
    let out = out_dir.to_string_lossy().to_string();
    for probe in ["hwp5-table-probe", "hwp5-cell-header-probe"] {
        let args = [
            probe,
            "없는A-probe.hwp",
            "없는B-probe.hwp",
            "--out-dir",
            &out,
        ];
        let output = assert_code(&args, 1);
        assert!(
            String::from_utf8_lossy(&output.stdout).trim().is_empty(),
            "실패 시 stdout 은 비어야 합니다({probe}): {}",
            describe(&args, &output)
        );
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}
