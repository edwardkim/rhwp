//! [#3480] 채운 값이 칸에 들어가는지 검사해 보고하는 계약.
//!
//! 에이전트·스크립트는 **렌더 결과를 보지 않는다.** JSON 응답만 보고 다음 단계로 간다.
//! 값이 칸을 넘쳐 표 경계를 벗어나고 행 높이가 어긋나도 `filledCount` 만 보면 성공이라,
//! 사람이라면 절대 제출하지 않을 문서가 완성본으로 넘어간다.
//!
//! 이 검사는 **rhwp 만 할 수 있다** — 조판 엔진이 있어야 "이 글자열이 이 셀 폭에
//! 맞는가"를 답할 수 있기 때문이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 실물 대학 서식. 표0 의 `성명`(row2,col3) 은 폭이 좁은 값 칸이다.
const SAMPLE_FORM: &str = "samples/복학원서.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_FORM)
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-fit-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
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

/// `set-cell` 을 `--dry-run --json` 으로 돌려 봉투를 얻는다(파일을 만들지 않는다).
fn set_cell_dry(text: &str) -> Option<serde_json::Value> {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return None;
    }
    let out = temp_path("dry");
    let args = [
        "edit",
        "set-cell",
        src.to_str().unwrap(),
        "--table",
        "0",
        "--row",
        "2",
        "--col",
        "3",
        "--text",
        text,
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("set-cell --json 이 순수 JSON 이어야 합니다");
    Some(v)
}

#[test]
fn overflowing_value_is_reported() {
    // 본론: 좁은 칸을 넘치는 값이면 그 사실을 알려야 한다.
    // 알리지 않으면 소비자가 깨진 산출물을 완성본으로 판단한다.
    let Some(v) = set_cell_dry("홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상")
    else {
        return;
    };

    let overflow = v["overflow"]
        .as_array()
        .unwrap_or_else(|| panic!("overflow 배열이 있어야 합니다: {v}"));
    assert_eq!(overflow.len(), 1, "넘치는 값 하나를 보고해야 합니다: {v}");

    let o = &overflow[0];
    let cell_w = o["cellWidthPx"]
        .as_f64()
        .unwrap_or_else(|| panic!("cellWidthPx 누락: {o}"));
    let text_w = o["textWidthPx"]
        .as_f64()
        .unwrap_or_else(|| panic!("textWidthPx 누락: {o}"));
    assert!(cell_w > 0.0, "{o}");
    assert!(
        text_w > cell_w,
        "넘친다고 보고했으면 글자 폭이 칸 폭보다 커야 합니다: {o}"
    );
    assert!(o["target"].is_string(), "어느 칸인지 알려야 합니다: {o}");
}

#[test]
fn fitting_value_reports_no_overflow() {
    // 무회귀 가드: 맞는 값은 조용해야 한다. 과잉 경고는 신호를 죽인다.
    let Some(v) = set_cell_dry("홍가상") else {
        return;
    };
    let overflow = v["overflow"]
        .as_array()
        .unwrap_or_else(|| panic!("overflow 배열이 있어야 합니다: {v}"));
    assert!(
        overflow.is_empty(),
        "칸에 들어가는 값인데 넘친다고 보고했습니다: {v}"
    );
}

#[test]
fn dry_run_reports_overflow_before_writing_any_file() {
    // 파일을 만들기 전에 알아야 값을 고칠 수 있다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("prewrite");
    let args = [
        "edit",
        "set-cell",
        src.to_str().unwrap(),
        "--table",
        "0",
        "--row",
        "2",
        "--col",
        "3",
        "--text",
        "홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상",
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("set-cell JSON");
    assert!(
        !v["overflow"].as_array().expect("overflow").is_empty(),
        "{v}"
    );
    assert!(
        !out.exists(),
        "--dry-run 은 파일을 만들면 안 됩니다: {}",
        out.display()
    );
}

#[test]
fn overflow_does_not_block_the_edit() {
    // 여러 줄이 정상인 칸도 있으므로 채우기를 막지 않는다 — 신호만 준다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("nonblock");
    let args = [
        "edit",
        "set-cell",
        src.to_str().unwrap(),
        "--table",
        "0",
        "--row",
        "2",
        "--col",
        "3",
        "--text",
        "홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상홍가상",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "넘쳐도 성공으로 끝나야 합니다(신호만 준다)\n{}",
        describe(&args, &output)
    );
    assert!(out.exists(), "출력 파일은 생성되어야 합니다");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("set-cell JSON");
    assert!(
        !v["overflow"].as_array().expect("overflow").is_empty(),
        "{v}"
    );
    let _ = std::fs::remove_file(&out);
}
