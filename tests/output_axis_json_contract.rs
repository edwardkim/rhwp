//! [#3596] 산출물 축(export-pdf·export-markdown·export-hwpx)의 `--json` 기계 계약.
//!
//! 조회 축은 #3237~#3287 로 stdout 순수 JSON 계약이 완비됐지만, **산출물을 만드는
//! 축**은 사람용 진행 메시지뿐이라 MCP 도구로 노출할 수 없었다. 본 계약은
//! `export-svg --json`(#3287) 매니페스트 선례를 산출 3종으로 확장한다.
//! 종료 코드(#2707: 0/1/2, export-hwpx 는 3/4 포함)는 무변경 — JSON 은 보고만 더한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-outjson-{tag}-{}-{}.{ext}",
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
fn export_pdf_json_manifest() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("pdf", "pdf");
    let args = [
        "export-pdf",
        p.to_str().unwrap(),
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
    assert_eq!(v["format"], "pdf", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert_eq!(v["backend"], "svg", "기본 backend 는 svg: {v}");
    assert_eq!(
        v["output"].as_str(),
        out.to_str(),
        "매니페스트의 output 경로: {v}"
    );
    let bytes = v["bytes"].as_u64().expect("bytes");
    assert!(bytes > 0, "{v}");
    assert!(v["pageCount"].as_u64().unwrap_or(0) >= 1, "{v}");
    assert!(v["renderedCount"].as_u64().unwrap_or(0) >= 1, "{v}");

    // 매니페스트가 가리키는 파일이 실제로 그 크기로 존재해야 한다.
    let meta = std::fs::metadata(&out).expect("PDF 산출물이 존재해야 합니다");
    assert_eq!(
        meta.len(),
        bytes,
        "보고된 bytes 와 실제 파일 크기가 다릅니다"
    );

    let _ = std::fs::remove_file(&out);
}

#[test]
fn export_markdown_json_manifest() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out_dir = temp_path("mddir", "d");
    let args = [
        "export-markdown",
        p.to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
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
    assert_eq!(v["format"], "markdown", "{v}");
    assert!(v["outputDir"].is_string(), "{v}");
    let rendered = v["renderedCount"].as_u64().expect("renderedCount");
    assert!(rendered >= 1, "{v}");
    let pages = v["pages"].as_array().expect("pages 배열");
    assert_eq!(pages.len() as u64, rendered, "{v}");
    for pg in pages {
        assert!(pg["page"].is_u64(), "{pg}");
        let path = pg["path"].as_str().expect("page.path");
        assert!(
            Path::new(path).exists(),
            "매니페스트가 가리키는 MD 파일이 없습니다: {path}"
        );
    }

    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn export_hwpx_json_envelope_with_verify() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("hwpx", "hwpx");
    let args = [
        "export-hwpx",
        p.to_str().unwrap(),
        out.to_str().unwrap(),
        "--verify",
        "--json",
    ];
    let output = run(&args);
    let v = parse_json(&args, &output);

    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["format"], "hwpx", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert!(v["bytes"].as_u64().unwrap_or(0) > 0, "{v}");
    assert!(out.exists(), "변환 산출물은 판정과 무관하게 저장된다");

    // 종료 코드와 verify 봉투가 서로 모순되면 안 된다 (#2707 exit 3 = 차이 검출).
    let identical = v["verify"]["identical"]
        .as_bool()
        .unwrap_or_else(|| panic!("verify.identical 이 있어야 합니다: {v}"));
    let expected_exit = if identical { 0 } else { 3 };
    assert_eq!(
        output.status.code(),
        Some(expected_exit),
        "verify.identical={identical} 인데 종료 코드가 다릅니다\n{}",
        describe(&args, &output)
    );
    if !identical {
        assert!(v["verify"]["diffCount"].as_u64().unwrap_or(0) >= 1, "{v}");
    }

    let _ = std::fs::remove_file(&out);
}

#[test]
fn export_hwpx_json_without_verify_has_no_verdict() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("hwpx-nv", "hwpx");
    let args = [
        "export-hwpx",
        p.to_str().unwrap(),
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
    assert!(
        v["verify"].is_null(),
        "--verify 를 안 줬으면 판정 필드가 없어야 합니다: {v}"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn json_runtime_failure_keeps_stdout_empty() {
    // 실패 경로 stdout 순수성 — 소비자가 부분 산출물을 성공으로 오인하면 안 된다.
    for args in [
        vec!["export-pdf", "없는파일-outjson.hwp", "--json"],
        vec!["export-markdown", "없는파일-outjson.hwp", "--json"],
        vec!["export-hwpx", "없는파일-outjson.hwp", "out.hwpx", "--json"],
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
fn convert_json_envelope_with_verify() {
    // [#3605] 종전에는 allow_json 게이트가 구현 없는 --json 수용을 exit 2 로 막았다.
    // 구현이 생겼으므로 가드의 목적을 전환한다: 봉투·exit 정합을 고정한다.
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("conv", "hwp");
    let args = [
        "convert",
        p.to_str().unwrap(),
        out.to_str().unwrap(),
        "--verify",
        "--json",
    ];
    let output = run(&args);
    let v = parse_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["format"], "hwp5", "{v}");
    assert!(v["bytes"].as_u64().unwrap_or(0) > 0, "{v}");
    assert!(out.exists(), "산출물은 판정과 무관하게 저장된다");
    let identical = v["verify"]["identical"]
        .as_bool()
        .unwrap_or_else(|| panic!("verify.identical 필요: {v}"));
    let expected = if identical { 0 } else { 3 };
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        describe(&args, &output)
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn capabilities_reports_output_axis_json() {
    // 드리프트 가드: 자기서술과 MCP 선언이 새 계약을 함께 실어야 한다.
    let output = run(&["capabilities"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("capabilities JSON");
    for name in ["export-pdf", "export-markdown", "export-hwpx"] {
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
    for t in ["hwp_export_pdf", "hwp_export_markdown", "hwp_convert_hwpx"] {
        assert!(tools.contains(&t), "{t} 가 MCP 선언에 없습니다: {tools:?}");
    }
}
