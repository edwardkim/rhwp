//! [#3600] 생성·미리보기 축의 `--json` 기계 계약 — build-from-ingest(유일한 문서
//! 생성 경로)와 thumbnail(문서를 열지 않는 초경량 미리보기).
//!
//! Stage 6(#2659) 잔여 축. 계약 모양은 산출물 축(#3596)과 같다: 동작 무변경,
//! `--json` 모드에서만 stdout 순수 JSON, 실패 경로 stdout 은 비운다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// ingest 스키마 표본 (저장소 동봉).
const INGEST_SAMPLE: &str = "tools/rhwp-ingest/schema/sample_minimal.json";
/// PrvImage 썸네일을 내장한 HWP5 문서.
const THUMB_SAMPLE: &str = "samples/field-01.hwp";

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-genprev-{tag}-{}-{}.{ext}",
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

#[test]
fn build_from_ingest_json_envelope() {
    let src = repo(INGEST_SAMPLE);
    if !src.exists() {
        eprintln!("표본 없음 — 건너뜀");
        return;
    }
    let out = temp_path("ingest", "hwpx");
    let args = [
        "build-from-ingest",
        src.to_str().unwrap(),
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
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["format"], "hwpx", "{v}");
    assert!(v["source"].is_string(), "{v}");
    let bytes = v["bytes"].as_u64().expect("bytes");
    assert!(bytes > 0, "{v}");
    assert!(v["questionCount"].is_u64(), "{v}");
    assert!(v["paragraphCount"].as_u64().unwrap_or(0) >= 1, "{v}");

    // 봉투가 가리키는 산출물이 실제로 그 크기로 존재하고, 파싱 가능한 HWPX 여야 한다.
    let meta = std::fs::metadata(&out).expect("산출물이 존재해야 합니다");
    assert_eq!(meta.len(), bytes, "보고 bytes ≠ 실제 크기");
    let info = run(&["info", out.to_str().unwrap(), "--json"]);
    let iv: serde_json::Value = serde_json::from_slice(&info.stdout).expect("info --json");
    assert_eq!(iv["format"], "hwpx", "생성물 재파싱 실측: {iv}");

    let _ = std::fs::remove_file(&out);
}

#[test]
fn thumbnail_json_file_mode() {
    let src = repo(THUMB_SAMPLE);
    if !src.exists() {
        eprintln!("표본 없음 — 건너뜀");
        return;
    }
    let out = temp_path("thumb", "png");
    let args = [
        "thumbnail",
        src.to_str().unwrap(),
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
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert!(v["format"].is_string(), "png|bmp|gif: {v}");
    assert!(
        v["mime"].as_str().unwrap_or("").starts_with("image/"),
        "{v}"
    );
    let bytes = v["bytes"].as_u64().expect("bytes");
    assert!(bytes > 0, "{v}");
    assert_eq!(v["output"].as_str(), out.to_str(), "{v}");
    let meta = std::fs::metadata(&out).expect("썸네일 파일이 존재해야 합니다");
    assert_eq!(meta.len(), bytes, "보고 bytes ≠ 실제 크기");

    let _ = std::fs::remove_file(&out);
}

#[test]
fn thumbnail_json_data_uri_mode() {
    // 파일을 만들지 않는 모드 — 봉투 안에 dataUri 가 실린다 (VLM 직행용).
    let src = repo(THUMB_SAMPLE);
    if !src.exists() {
        eprintln!("표본 없음 — 건너뜀");
        return;
    }
    let args = ["thumbnail", src.to_str().unwrap(), "--data-uri", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert!(
        v["dataUri"]
            .as_str()
            .unwrap_or("")
            .starts_with("data:image/"),
        "{v}"
    );
    assert!(
        v["output"].is_null(),
        "파일 모드가 아니면 output 은 null: {v}"
    );
}

#[test]
fn genpreview_json_runtime_failure_keeps_stdout_empty() {
    for args in [
        vec![
            "build-from-ingest",
            "없는파일-genprev.json",
            "-o",
            "x.hwpx",
            "--json",
        ],
        vec!["thumbnail", "없는파일-genprev.hwp", "--json"],
    ] {
        let args: Vec<&str> = args;
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            describe(&args, &output)
        );
        assert!(
            output.stdout.is_empty(),
            "실패 경로 stdout 은 비어야 합니다.\n{}",
            describe(&args, &output)
        );
    }
}

#[test]
fn capabilities_reports_genpreview_json() {
    let output = run(&["capabilities"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("capabilities JSON");
    for name in ["build-from-ingest", "thumbnail"] {
        let entry = v["commands"]
            .as_array()
            .expect("commands")
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("{name} 이 capabilities 에 없습니다"));
        assert_eq!(
            entry["json"], true,
            "{name} 은 json:true 여야 합니다: {entry}"
        );
    }

    let mcp = run(&["capabilities", "--mcp"]);
    let m: serde_json::Value = serde_json::from_slice(&mcp.stdout).expect("mcp JSON");
    let tools: Vec<&str> = m["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    for t in ["hwp_build_from_ingest", "hwp_thumbnail"] {
        assert!(tools.contains(&t), "{t} 가 MCP 선언에 없습니다: {tools:?}");
    }
}
