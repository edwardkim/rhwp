//! [#5019] `edit insert-column-break` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::wasm_api::HwpDocument;

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/field-01.hwp")
        .to_string_lossy()
        .into_owned()
}
fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-inscb-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin()).args(args).output().expect("rhwp")
}
fn para_count(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    HwpDocument::from_bytes(&bytes).unwrap().document().sections[0]
        .paragraphs
        .len()
}

/// 문단 **중간** 커서에서는 문단이 갈린다.
///
/// [#5019] 종전 이 시험은 `--offset 0` 으로 분할을 확인했다. 그 자리는 분할하는 자리가
/// 아니다 — HWPX `columnBreak` 와 HWP5 break 비트는 쪽 나눔(#7218)과 같은 break-before
/// 축이라, 문단 시작의 단 나눔은 그 문단 자신의 속성이다. 분할 계약은 실제로 가를
/// 자리에서 잠근다. offset 0 계약은 `issue_5019_column_break_at_paragraph_start.rs` 다.
#[test]
fn insert_column_break_splits_paragraph() {
    let src = sample();
    let before = para_count(Path::new(&src));
    let out = temp("out");
    // samples/field-01.hwp 문단 3 = "전략 기획서" → 오프셋 2 에서 "전략" / " 기획서".
    let args = [
        "edit",
        "insert-column-break",
        src.as_str(),
        "--para",
        "3",
        "--offset",
        "2",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(para_count(&out) > before);
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).expect("봉투 JSON");
    assert_eq!(envelope["paragraphDelta"], 1, "{envelope}");
    assert_eq!(envelope["columnBreakParagraph"], 4, "{envelope}");
    let _ = std::fs::remove_file(&out);
}

/// 문단 시작은 문단을 가르지 않고, 봉투가 그 사실을 알린다.
#[test]
fn insert_column_break_at_paragraph_start_reports_no_paragraph_shift() {
    let src = sample();
    let before = para_count(Path::new(&src));
    let out = temp("start");
    let args = [
        "edit",
        "insert-column-break",
        src.as_str(),
        "--para",
        "3",
        "--offset",
        "0",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(
        para_count(&out),
        before,
        "문단 시작의 단 나눔은 문단 수를 바꾸지 않는다",
    );
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).expect("봉투 JSON");
    assert_eq!(envelope["paragraphDelta"], 0, "{envelope}");
    assert_eq!(envelope["columnBreakParagraph"], 3, "{envelope}");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let out = temp("dry");
    let args = [
        "edit",
        "insert-column-break",
        src.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0));
    assert!(!out.exists());
}

#[test]
fn mcp_declared() {
    let output = run(&["capabilities", "--mcp"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "hwp_insert_column_break"));
}
