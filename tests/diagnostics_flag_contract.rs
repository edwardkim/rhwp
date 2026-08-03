//! [#3884 G1·G2·G3] 자기서술 밖 진단 명령(bench·dump·diag)의 플래그·실패 계약.
//!
//! 이 셋은 capabilities 에 `flags`·`json` 을 선언하지 않는 명령이라 드리프트 가드의
//! 시야 밖에 있었고, 그 사각에서 두 가지가 자랐다:
//!
//! - **미지 플래그 침묵 무시** — `--json` 을 붙여도 사람용 텍스트가 exit 0 으로
//!   나온다(dump·diag). 에이전트는 JSON 을 기대하고 파싱하다 깨진다.
//! - **실패 경로 stdout 오염** — bench 는 `--json` 을 "파일 이름"으로 접어 실패시키고,
//!   그 와중에 배너+반쪽 표를 stdout 으로 흘렸다(exit 1 + 518 B).
//!
//! 여기서 고정하는 계약은 하나다: **모르는 옵션은 조용히 무시하지 않는다** —
//! exit 2 + stdout 0바이트 + stderr 안내. 그리고 전건 실패면 stdout 은 빈다.
//!
//! G4(inspect·edit 하위 명령의 자기서술 등재)와 "진단 명령을 자기서술에 넣을지"의
//! 부류 판단은 이 파일 범위 밖이다 — #3884 본문의 열린 질문으로 남는다.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 아무 유효 문서 — 플래그 거부는 문서를 열기 전에 일어나야 한다.
const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn describe(args: &[&str], out: &Output) -> String {
    format!(
        "args={args:?}\nexit={:?}\nstdout={}\nstderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// 거부 계약의 공통 골격: exit 2, stdout 0바이트, stderr 에 문제의 플래그 명시.
fn assert_rejects(args: &[&str], flag: &str) {
    let out = run(args);
    assert_eq!(
        out.status.code(),
        Some(2),
        "미지 플래그는 사용법 오류다: {}",
        describe(args, &out)
    );
    assert!(
        out.stdout.is_empty(),
        "거부하면서 stdout 을 오염시키면 안 된다: {}",
        describe(args, &out)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(flag),
        "무엇이 거부됐는지 stderr 가 말해야 한다({flag}): {}",
        describe(args, &out)
    );
}

// ── bench ────────────────────────────────────────────────────────────────

#[test]
fn bench_rejects_json_flag_with_silent_stdout() {
    // [G1 의 대표 증상] --json 이 "실패한 파일"이 되어 exit 1 + 반쪽 표 518 B 가
    // stdout 으로 새던 자리다. 이제 문서를 열기 전에 사용법 오류로 끊는다.
    let s = sample();
    assert_rejects(&["bench", s.to_str().unwrap(), "--json"], "--json");
}

#[test]
fn bench_rejects_unknown_flag_with_silent_stdout() {
    let s = sample();
    assert_rejects(
        &["bench", s.to_str().unwrap(), "--bogus-flag"],
        "--bogus-flag",
    );
}

#[test]
fn bench_total_failure_keeps_stdout_empty() {
    // [G1] 측정 성공 0건이면 stdout 은 빈다 — 빈 표에 배너를 얹어 내보내면 파이프
    // 소비자가 "측정 결과"로 읽는다. 실패의 전말은 stderr 와 exit 1 이 말한다.
    let args = ["bench", "no-such-file-3884.hwp"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&args, &out));
    assert!(
        out.stdout.is_empty(),
        "전건 실패인데 stdout 이 비지 않았다: {}",
        describe(&args, &out)
    );
    assert!(
        !out.stderr.is_empty(),
        "실패 사유는 stderr 로 나가야 한다: {}",
        describe(&args, &out)
    );
}

#[test]
fn bench_success_still_prints_the_table() {
    // 회귀 가드: 거부·침묵 규율을 넣으면서 성공 경로의 사람용 표까지 없애면 안 된다.
    let s = sample();
    let args = ["bench", s.to_str().unwrap(), "-n", "1"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("=== bench:"),
        "성공하면 배너+표가 그대로 나와야 한다: {}",
        describe(&args, &out)
    );
}

// ── dump ─────────────────────────────────────────────────────────────────

#[test]
fn dump_rejects_unknown_flag_with_silent_stdout() {
    // [G2 실측] 종전: exit 0 + 18,643 B — 오타 난 옵션이 조용히 무시된 채 성공했다.
    let s = sample();
    assert_rejects(
        &["dump", s.to_str().unwrap(), "--bogus-flag"],
        "--bogus-flag",
    );
}

#[test]
fn dump_rejects_json_flag_until_it_has_a_json_contract() {
    // dump 는 --json 계약이 없다. 침묵 무시(사람용 텍스트 + exit 0)보다 정직한 거부가
    // 낫다 — JSON 봉투를 실제로 갖추는 일은 #3884 의 부류 판단(자기서술 등재) 뒤의 몫.
    let s = sample();
    assert_rejects(&["dump", s.to_str().unwrap(), "--json"], "--json");
}

#[test]
fn dump_rejects_flag_in_file_position() {
    // 첫 인자 자리의 플래그가 "파일을 읽을 수 없습니다 - --json"(exit 1)로 새지 않는다.
    assert_rejects(&["dump", "--json"], "--json");
}

#[test]
fn dump_still_accepts_declared_filters() {
    // 회귀 가드: 선언된 --section/--para 는 계속 받는다.
    let s = sample();
    let args = ["dump", s.to_str().unwrap(), "--section", "0"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    assert!(!out.stdout.is_empty(), "{}", describe(&args, &out));
}

// ── diag ─────────────────────────────────────────────────────────────────

#[test]
fn diag_rejects_unknown_flag_with_silent_stdout() {
    // [G2 실측] 종전: diag 는 args[1..] 를 아예 보지 않아 무엇을 붙여도 exit 0 이었다.
    let s = sample();
    assert_rejects(&["diag", s.to_str().unwrap(), "--json"], "--json");
}

#[test]
fn diag_rejects_flag_in_file_position() {
    assert_rejects(&["diag", "--verbose"], "--verbose");
}

// ── run 예외의 자기서술 (G3) ─────────────────────────────────────────────

#[test]
fn run_failure_envelope_exception_is_self_described() {
    // run 은 실패도 봉투로 보고하는 의도된 예외다(judgment-as-data). 예외가 실물에만
    // 있고 자기서술에 없으면, "실패 = stdout 0바이트"를 믿는 소비자가 run 에서 깨진다.
    //
    // 호출은 bare `capabilities` — 이 명령은 플래그 없이 JSON 이 기본 출력이고,
    // `--json` 은 `--search` 전용이다(직접 확인: 병합 전 0889974a0 도 `--json` 을
    // "알 수 없는 옵션"으로 거부했다 — --search 병합의 회귀가 아니라 원래 계약).
    let args = ["capabilities"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("capabilities JSON");
    let failure = v["jsonContract"]["failure"]
        .as_str()
        .expect("jsonContract.failure");
    assert!(
        failure.contains("run"),
        "run 의 stdout 예외가 자기서술에 없다: {failure}"
    );
    assert!(
        failure.contains("invalid"),
        "계획 무효(exit 2 + invalid[]) 예외가 자기서술에 없다: {failure}"
    );
}
