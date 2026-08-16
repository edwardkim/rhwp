//! [#4990] `edit insert-text` 출력·안전 계약 회귀 테스트.
//!
//! 검증 원칙은 형제 편집 명령(#3381 set-cell, #3719 insert-image)과 같다:
//! ① `--dry-run` 은 파일을 만들지 않는다 ② 실패 경로의 stdout 은 0바이트
//! ③ 반영 여부는 **산출물 재파싱**으로 확인한다 ④ 범위를 넘으면 조용히 자르지
//! 않고 exit 2 로 실제 길이를 안내한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/field-01.hwp";
const MARKER: &str = "⟦INSERTED⟧";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn sample_arg() -> String {
    sample().to_string_lossy().to_string()
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-instxt-{tag}-{}-{}.{ext}",
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

fn parse_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

fn first_para_text(path: &Path) -> String {
    let bytes = std::fs::read(path).expect("읽기");
    let doc = HwpDocument::from_bytes(&bytes).expect("파싱");
    doc.document().sections[0].paragraphs[0].text.clone()
}

fn insert_text_tool_definition() -> serde_json::Value {
    let args = ["capabilities", "--mcp"];
    let output = run(&args);
    let v = parse_json(&args, &output);
    v["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_insert_text")
        .expect("hwp_insert_text 도구")
        .clone()
}

#[test]
fn insert_text_writes_and_reports_address() {
    let src = sample_arg();
    let out = temp_path("out", "hwp");
    let args = [
        "edit",
        "insert-text",
        src.as_str(),
        "--section",
        "0",
        "--para",
        "0",
        "--offset",
        "0",
        "--text",
        MARKER,
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
    let v = parse_json(&args, &output);
    assert_eq!(v["section"], 0);
    assert_eq!(v["paragraph"], 0);
    assert_eq!(v["offset"], 0);
    assert_eq!(v["text"], MARKER);
    assert_eq!(v["insertedChars"], MARKER.chars().count());
    assert_eq!(v["dryRun"], false);
    assert_eq!(v["output"], out.to_str().unwrap());
    assert!(out.is_file(), "산출물이 있어야 한다");
    let text = first_para_text(&out);
    assert!(
        text.starts_with(MARKER),
        "재파싱한 첫 문단이 삽입 문자열로 시작해야 한다: {text:?}"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_does_not_write_a_file() {
    let src = sample_arg();
    let out = temp_path("dry", "hwp");
    let args = [
        "edit",
        "insert-text",
        src.as_str(),
        "--text",
        MARKER,
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
    let v = parse_json(&args, &output);
    assert_eq!(v["dryRun"], true);
    assert!(v.get("output").is_none(), "{v}");
    assert!(!out.exists(), "dry-run 은 파일을 만들면 안 된다");
}

#[test]
fn empty_text_is_usage_error_with_empty_stdout() {
    let src = sample_arg();
    let args = ["edit", "insert-text", src.as_str(), "--text", "", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

#[test]
fn offset_past_paragraph_is_usage_error() {
    let src = sample_arg();
    let args = [
        "edit",
        "insert-text",
        src.as_str(),
        "--offset",
        "999999",
        "--text",
        MARKER,
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(
        err.contains("문단 길이"),
        "실제 길이를 안내해야 한다: {err}"
    );
}

#[test]
fn unknown_flag_is_usage_error_with_empty_stdout() {
    let src = sample_arg();
    let args = [
        "edit",
        "insert-text",
        src.as_str(),
        "--text",
        MARKER,
        "--존재하지않는옵션",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

#[test]
fn mcp_tool_is_declared() {
    let tool = insert_text_tool_definition();
    assert_eq!(tool["cli"]["command"], "edit");
    let args = tool["cli"]["args"]
        .as_array()
        .expect("cli.args")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();
    assert!(args.contains(&"insert-text"), "{args:?}");
    assert!(args.contains(&"--text"), "{args:?}");
    let required = tool["inputSchema"]["required"]
        .as_array()
        .expect("required");
    assert!(required.iter().any(|v| v == "path"));
    assert!(required.iter().any(|v| v == "text"));
}

#[test]
fn capabilities_lists_insert_text_subcommand() {
    let args = ["capabilities"];
    let output = run(&args);
    let v = parse_json(&args, &output);
    let edit = v["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "edit")
        .expect("edit");
    let names: Vec<&str> = edit["subcommands"]
        .as_array()
        .expect("subcommands")
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    assert!(names.contains(&"insert-text"), "{names:?}");
}
