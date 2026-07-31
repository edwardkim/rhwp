//! [#3395] `edit replace-text --occurrence N` — 문서 순서 k번째(0 기준) 매치만 치환.
//! 실물 서식의 체크박스(□ 다수 중 해당 항목만 ☑)를 위한 계약. 코어는
//! `replace_all_native` 와 같은 경로(`replace_matches_native`)를 재사용한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 조사 "의" 가 다수(276회) 나오는 표본 — k번째 지목 검증에 적합.
const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-occ-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
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

#[test]
fn occurrence_replaces_exactly_one_match() {
    let p = sample();
    if !p.exists() {
        eprintln!("표본 없음 — 건너뜀");
        return;
    }
    // 전제: 전체 매치 수를 실측한다.
    let sv = run(&["search", p.to_str().unwrap(), "의", "--json"]);
    let s: serde_json::Value = serde_json::from_slice(&sv.stdout).expect("search");
    let total = s["totalMatchCount"].as_u64().expect("total");
    assert!(total >= 3, "표본 전제 실패: {total}");

    let out = temp_path("one");
    let args = [
        "edit",
        "replace-text",
        p.to_str().unwrap(),
        "--find",
        "의",
        "--replace",
        "의◎",
        "--occurrence",
        "1",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_eq!(v["replacedCount"].as_u64(), Some(1), "{v}");
    assert_eq!(v["occurrence"].as_u64(), Some(1), "{v}");

    // 재독 대조: ◎ 는 정확히 1개, "의" 매치는 total-1 (하나만 소비).
    let r1 = run(&["search", out.to_str().unwrap(), "의◎", "--json"]);
    let c1: serde_json::Value = serde_json::from_slice(&r1.stdout).expect("reread1");
    assert_eq!(c1["totalMatchCount"].as_u64(), Some(1), "{c1}");
    let r2 = run(&["search", out.to_str().unwrap(), "의", "--json"]);
    let c2: serde_json::Value = serde_json::from_slice(&r2.stdout).expect("reread2");
    assert_eq!(
        c2["totalMatchCount"].as_u64(),
        Some(total),
        "의◎ 의 '의' 포함해 총수 불변이어야 합니다: {c2}"
    );

    let _ = std::fs::remove_file(&out);
}

#[test]
fn occurrence_out_of_range_reports_zero_and_writes_nothing() {
    let p = sample();
    if !p.exists() {
        eprintln!("표본 없음 — 건너뜀");
        return;
    }
    let out = temp_path("oor");
    let args = [
        "edit",
        "replace-text",
        p.to_str().unwrap(),
        "--find",
        "의",
        "--replace",
        "X",
        "--occurrence",
        "999999",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_eq!(
        v["replacedCount"].as_u64(),
        Some(0),
        "범위 밖은 계수 0: {v}"
    );
    assert!(!out.exists(), "치환 0건이면 출력 파일을 만들지 않는다");
}

#[test]
fn occurrence_invalid_value_is_usage_error() {
    let p = sample();
    if !p.exists() {
        eprintln!("표본 없음 — 건너뜀");
        return;
    }
    let args = [
        "edit",
        "replace-text",
        p.to_str().unwrap(),
        "--find",
        "의",
        "--replace",
        "X",
        "--occurrence",
        "abc",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn capabilities_declares_set_checkbox_tool() {
    let mcp = run(&["capabilities", "--mcp"]);
    let m: serde_json::Value = serde_json::from_slice(&mcp.stdout).expect("mcp");
    let tool = m["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "hwp_set_checkbox")
        .expect("hwp_set_checkbox 선언");
    // 배선 검증: occurrence 자리표시자가 required 와 1:1 (플레이북 DoD).
    let args: Vec<&str> = tool["cli"]["args"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a.as_str())
        .collect();
    assert!(args.contains(&"{occurrence}"), "{args:?}");
    assert!(args.contains(&"--occurrence"), "{args:?}");
}
