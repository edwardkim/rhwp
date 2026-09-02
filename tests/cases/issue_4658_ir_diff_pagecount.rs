//! [#4658] ir-diff --json 은 IR 필드가 같아도 pageCount 가 다르면 identical:true 를 내지 않는다.
//!
//! `samples/2026_oss_rst.hwp` 와 `samples/hwpx/2026_oss_rst.hwpx` 는 IR 비교 필드가
//! 같고 쪽수만 갈린다(출처 프로파일 — HWP5-origin 마커). ir-diff 는 조판 전용
//! 데이터를 두 번째 IR 로 올리지 않지만, JSON 봉투는 `info --json` 과 같은
//! pageCount 를 양쪽 싣고 쪽수가 다르면 identical:false + categories.pageCount 로
//! 게이트한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const HWP: &str = "samples/2026_oss_rst.hwp";
const HWPX: &str = "samples/hwpx/2026_oss_rst.hwpx";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
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

fn parse_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

fn info_page_count(path: &Path) -> u64 {
    let p = path.to_str().expect("utf-8 path");
    let args = ["info", p, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    parse_json(&args, &output)["pageCount"]
        .as_u64()
        .unwrap_or_else(|| panic!("info.pageCount 필요: {}", describe(&args, &output)))
}

#[test]
fn ir_diff_json_always_reports_page_counts() {
    let a = sample(HWP);
    if !a.exists() {
        eprintln!("샘플 없음 — 건너뜀: {HWP}");
        return;
    }
    let a_str = a.to_str().unwrap();
    let args = ["ir-diff", a_str, a_str, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    let pages = info_page_count(&a);
    assert_eq!(v["identical"], true, "{v}");
    assert_eq!(v["diffCount"], 0, "{v}");
    assert_eq!(v["pageCountA"], pages, "{v}");
    assert_eq!(v["pageCountB"], pages, "{v}");
    assert_eq!(v["pageCountA"], v["pageCountB"], "{v}");
}

#[test]
fn oss_rst_ir_diff_is_not_identical_when_page_counts_differ() {
    let hwp = sample(HWP);
    let hwpx = sample(HWPX);
    if !hwp.exists() || !hwpx.exists() {
        eprintln!("샘플 없음 — 건너뜀: {HWP} / {HWPX}");
        return;
    }
    let a = hwp.to_str().unwrap();
    let b = hwpx.to_str().unwrap();
    let args = ["ir-diff", a, b, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    let pages_a = info_page_count(&hwp);
    let pages_b = info_page_count(&hwpx);
    assert_ne!(
        pages_a, pages_b,
        "전제: {HWP} 와 {HWPX} 의 info.pageCount 가 달라야 한다 ({pages_a} vs {pages_b})"
    );
    assert_eq!(v["pageCountA"], pages_a, "{v}");
    assert_eq!(v["pageCountB"], pages_b, "{v}");
    assert_eq!(v["identical"], false, "{v}");
    assert!(
        v["diffCount"].as_u64().unwrap() >= 1,
        "pageCount 차이가 diffCount 에 잡혀야 한다: {v}"
    );
    let cats = v["categories"].as_object().expect("categories 객체");
    assert!(
        cats.keys().any(|k| k == "pageCount"),
        "named diff pageCount 가 categories 에 있어야 한다: {v}"
    );
}
