//! [#3237/#3238] CLI `--json` 출력 계약 + `batch` 서브커맨드 회귀 테스트.
//!
//! 계약: `--json` 모드의 stdout 은 순수 JSON(NDJSON)이고 `schemaVersion` 을 포함한다.
//! 필드 추가는 허용, 기존 필드의 변경·삭제는 본 테스트가 실패로 잡는다.
//! 종료 코드는 [#2707] 계약(0/1/2)을 그대로 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// 파싱까지 성공하는 실제 샘플.
const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

/// stdin 으로 파일 목록을 흘려 넣는 batch 실행 헬퍼.
fn run_with_stdin(args: &[&str], stdin_body: &str) -> Output {
    let mut child = Command::new(rhwp_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin_body.as_bytes())
        .expect("stdin 쓰기 실패");
    child.wait_with_output().expect("rhwp 종료 대기 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn parse_stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

// ── info --json ────────────────────────────────────────────────────────────

#[test]
fn info_json_contract() {
    let sample = sample_path();
    let args = ["info", "--json", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    // 스키마 고정: 아래 필드의 존재·타입이 계약이다 (필드 추가는 허용).
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert_eq!(v["format"], "hwp3", "{v}");
    assert!(v["sizeBytes"].as_u64().is_some(), "{v}");
    assert!(v["sections"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["paraCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["fonts"].is_array(), "{v}");
}

#[test]
fn info_json_missing_file_exit_runtime_and_silent_stdout() {
    let args = ["info", "--json", "없는파일-json.hwp"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    // 실패 시 stdout 에 부분 JSON 을 흘리지 않는다 — 소비자는 stdout 만 파싱한다.
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

#[test]
fn info_json_multiple_files_exit_usage_silent_stdout() {
    let first = sample_path();
    let second = sample_path();
    let args = [
        "info",
        first.to_str().unwrap(),
        second.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "추가 입력을 조용히 무시하면 안 됩니다.\n{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

// ── export-text --json ─────────────────────────────────────────────────────

#[test]
fn export_text_json_contract() {
    let sample = sample_path();
    let args = ["export-text", "--json", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    let pages = v["pages"].as_array().expect("pages 배열");
    assert_eq!(
        pages.len() as u64,
        v["pageCount"].as_u64().unwrap(),
        "pageCount 는 pages 길이와 같아야 합니다: {v}"
    );
    assert!(pages[0]["page"].as_u64().is_some(), "{v}");
    assert!(pages[0]["text"].is_string(), "{v}");
}

#[test]
fn export_text_default_output_unchanged() {
    // 기존 성공 경로 무변경 가드: --json 없이는 종전 그대로 사람용 출력 + 파일 저장.
    let sample = sample_path();
    let out_dir = std::env::temp_dir().join(format!(
        "rhwp-json-guard-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    let args = [
        "export-text",
        sample.to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("문서 로드 완료"),
        "기본 출력이 바뀌면 안 됩니다.\n{}",
        describe(&args, &output)
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

// ── batch export-text --json ───────────────────────────────────────────────

#[test]
fn batch_export_text_json_all_success() {
    let sample = sample_path();
    let sample_str = sample.to_str().unwrap();
    let args = ["batch", "export-text", "--json"];
    let stdin_body = format!("{sample_str}\n{sample_str}\n");
    let output = run_with_stdin(&args, &stdin_body);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 2, "{}", describe(&args, &output));
    for line in lines {
        let v: serde_json::Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("NDJSON 아님 ({e}): {line}"));
        assert_eq!(v["schemaVersion"], "1.0", "{v}");
        assert!(v["source"].is_string(), "{v}");
        assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");
        assert!(v["text"].is_string(), "{v}");
        assert!(v.get("error").is_none(), "{v}");
    }
}

#[test]
fn batch_export_text_json_partial_failure_exit_runtime() {
    let sample = sample_path();
    let args = ["batch", "export-text", "--json"];
    let stdin_body = format!("{}\n없는파일-batch.hwp\n", sample.to_str().unwrap());
    let output = run_with_stdin(&args, &stdin_body);
    // 부분 실패도 실패다 — 성공분은 스트림에 남고 종료 코드가 신호한다.
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 2, "{}", describe(&args, &output));
    let records: Vec<serde_json::Value> = lines
        .iter()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("NDJSON 아님 ({e}): {l}")))
        .collect();
    assert!(
        records.iter().any(|v| v.get("error").is_none()),
        "성공 레코드가 있어야 합니다: {records:?}"
    );
    let failed: Vec<&serde_json::Value> = records
        .iter()
        .filter(|v| v.get("error").is_some())
        .collect();
    assert_eq!(failed.len(), 1, "{records:?}");
    assert_eq!(failed[0]["exitClass"], "runtime", "{records:?}");
    // 실패 레코드도 성공 레코드와 같은 스키마 계약을 따른다.
    assert_eq!(failed[0]["schemaVersion"], "1.0", "{records:?}");
}

// ── capabilities ───────────────────────────────────────────────────────────

#[test]
fn capabilities_json_contract() {
    // [#3263] 도구 자기서술: 에이전트가 첫 호출 1회로 도구 전체를 파악하는 입구.
    let args = ["capabilities"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["tool"], "rhwp", "{v}");
    assert!(v["version"].is_string(), "{v}");
    assert!(v["exitCodes"]["1"].is_string(), "{v}");
    let commands = v["commands"].as_array().expect("commands 배열");
    assert!(commands.len() >= 20, "전 명령 수록: {v}");
    // --json 계약 명령은 machine-readable 표시가 있어야 한다.
    for name in [
        "info",
        "export-text",
        "export-structure",
        "export-svg",
        "export-tables",
        "search",
        "fields",
        "ir-diff",
    ] {
        let cmd = commands
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("{name} 누락: {v}"));
        assert_eq!(cmd["json"], true, "{cmd}");
        assert!(cmd["summary"].is_string(), "{cmd}");
        assert!(cmd["category"].is_string(), "{cmd}");
    }
    let batch_subs = v["batch"]["subcommands"].as_array().expect("batch");
    assert!(batch_subs.iter().any(|s| s == "export-structure"), "{v}");
}

#[test]
fn capabilities_mcp_tool_definitions_contract() {
    // [#3263] `--mcp` 는 MCP 서버가 그대로 등록할 수 있는 도구 정의를 낸다 —
    // 서버 저자가 도구 목록·입력 스키마를 손으로 베껴 쓰지 않게 하는 것이 목적이다.
    let args = ["capabilities", "--mcp"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["protocol"], "mcp", "{v}");
    let tools = v["tools"].as_array().expect("tools 배열");
    assert!(!tools.is_empty(), "{v}");

    for t in tools {
        // MCP 도구 필수 3종: name·description·inputSchema
        let name = t["name"]
            .as_str()
            .unwrap_or_else(|| panic!("name 누락: {t}"));
        assert!(
            name.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            "MCP 도구 이름은 안전 문자만 써야 합니다: {t}"
        );
        assert!(t["description"].is_string(), "{t}");
        let schema = &t["inputSchema"];
        assert_eq!(schema["type"], "object", "{t}");
        assert!(schema["properties"].is_object(), "{t}");
        assert!(schema["required"].is_array(), "{t}");
        // 실행 방법(어떤 CLI 명령으로 내려가는지)이 있어야 서버가 배선할 수 있다.
        assert!(t["cli"]["command"].is_string(), "cli.command 누락: {t}");
    }

    // 파일을 받는 도구는 path 를 필수 입력으로 선언해야 한다.
    let info = tools
        .iter()
        .find(|t| t["cli"]["command"] == "info")
        .unwrap_or_else(|| panic!("info 도구 누락: {v}"));
    let required = info["inputSchema"]["required"].as_array().unwrap();
    assert!(required.iter().any(|r| r == "path"), "{info}");
    assert!(
        info["inputSchema"]["properties"]["path"]["type"] == "string",
        "{info}"
    );
}

#[test]
fn capabilities_mcp_covers_every_json_command() {
    // 드리프트 가드 ③: `--json` 계약을 가진 명령은 MCP 도구로도 노출되어야 한다.
    // 새 계약 명령을 capabilities 에만 넣고 MCP 에서 빠뜨리면 이 테스트가 잡는다.
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let mcp = parse_stdout_json(&["capabilities", "--mcp"], &run(&["capabilities", "--mcp"]));

    let mcp_commands: Vec<&str> = mcp["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["cli"]["command"].as_str())
        .collect();

    let missing: Vec<&str> = cap["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["json"] == true)
        .filter_map(|c| c["name"].as_str())
        // capabilities 자신은 도구가 아니라 도구 목록의 원천이라 제외한다.
        .filter(|n| *n != "capabilities")
        .filter(|n| !mcp_commands.contains(n))
        .collect();
    assert!(
        missing.is_empty(),
        "--json 계약 명령인데 MCP 도구로 안 나오는 것: {missing:?}"
    );
}

#[test]
fn capabilities_version_matches_version_flag() {
    // 드리프트 가드 ①: capabilities.version 은 `--version` 과 같은 원천이어야 한다.
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let ver_out = run(&["--version"]);
    let ver_line = String::from_utf8_lossy(&ver_out.stdout);
    let ver = ver_line.trim().trim_start_matches("rhwp v");
    assert_eq!(cap["version"], ver, "version 불일치: {ver_line}");
}

#[test]
fn capabilities_covers_every_help_command() {
    // 드리프트 가드 ②: `--help` 에 보이는 명령은 capabilities 에도 있어야 한다.
    // 새 명령을 help 에만 추가하면 이 테스트가 잡는다.
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let names: Vec<String> = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .map(|c| c["name"].as_str().expect("name").to_string())
        .collect();

    let help = run(&["--help"]);
    let help_text = String::from_utf8_lossy(&help.stdout);
    let mut missing = Vec::new();
    for line in help_text.lines() {
        // help 의 명령 줄 패턴: 정확히 2칸 들여쓰기 + 소문자/하이픈 토큰.
        if let Some(rest) = line.strip_prefix("  ") {
            if rest.starts_with(' ') || rest.starts_with('-') {
                continue; // 옵션·설명 줄
            }
            let token = rest.split_whitespace().next().unwrap_or("");
            if !token.is_empty()
                && token
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
                && !names.iter().any(|n| n == token)
            {
                missing.push(token.to_string());
            }
        }
    }
    assert!(
        missing.is_empty(),
        "--help 에는 있는데 capabilities 에 없는 명령: {missing:?}"
    );
}

// ── export-structure --json ────────────────────────────────────────────────

#[test]
fn export_structure_json_envelope_contract() {
    // [#3261] 계약 봉투: 한 줄 JSON, schemaVersion·source·mode·nodeCount·structure.
    let sample = sample_path();
    let args = ["export-structure", "--json", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "봉투는 한 줄이어야 합니다.\n{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert!(v["mode"].is_string(), "{v}");
    assert!(v["nodeCount"].as_u64().is_some(), "{v}");
    assert!(v["structure"].is_object(), "{v}");
}

#[test]
fn export_structure_multiple_files_exit_usage_silent_stdout() {
    let first = sample_path();
    let second = sample_path();
    let args = [
        "export-structure",
        first.to_str().unwrap(),
        second.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "마지막 파일로 바꿔 읽으면 안 됩니다.\n{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

#[test]
fn export_structure_default_output_unchanged() {
    // 기본 출력(무봉투 pretty JSON)은 종전과 동일해야 한다 — 봉투 필드가 없음을 고정.
    let sample = sample_path();
    let args = ["export-structure", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert!(
        v.get("schemaVersion").is_none(),
        "기본 출력에 봉투가 생기면 기존 소비자가 깨집니다: {v}"
    );
}

#[test]
fn batch_export_structure_json_contract() {
    let sample = sample_path();
    let sample_str = sample.to_str().unwrap();
    let args = ["batch", "export-structure", "--json", "--mode", "outline"];
    let stdin_body = format!("{sample_str}\n{sample_str}\n");
    let output = run_with_stdin(&args, &stdin_body);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let records: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("NDJSON 아님 ({e}): {l}")))
        .collect();
    assert_eq!(records.len(), 2, "{}", describe(&args, &output));
    for v in &records {
        assert_eq!(v["schemaVersion"], "1.0", "{v}");
        assert_eq!(v["mode"], "outline", "{v}");
        assert!(v["nodeCount"].as_u64().is_some(), "{v}");
        assert!(v["structure"].is_object(), "{v}");
    }
}

#[test]
fn batch_structure_invalid_mode_is_usage_error() {
    let args = ["batch", "export-structure", "--json", "--mode", "elephant"];
    let output = run_with_stdin(&args, "");
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn batch_mode_flag_rejected_for_other_subcommands() {
    // --mode 는 export-structure 전용이다.
    let args = ["batch", "export-text", "--json", "--mode", "outline"];
    let output = run_with_stdin(&args, "");
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn batch_info_json_shares_single_command_schema() {
    // `batch info --json` 레코드는 `info --json` 과 같은 스키마다 — 소비자가
    // 단건/배치를 같은 코드로 읽는 계약.
    let sample = sample_path();
    let sample_str = sample.to_str().unwrap();
    let args = ["batch", "info", "--json"];
    let stdin_body = format!("{sample_str}\n없는파일-batch-info.hwp\n");
    let output = run_with_stdin(&args, &stdin_body);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let records: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("NDJSON 아님 ({e}): {l}")))
        .collect();
    assert_eq!(records.len(), 2, "{}", describe(&args, &output));
    let ok = &records[0];
    assert_eq!(ok["schemaVersion"], "1.0", "{ok}");
    assert_eq!(ok["format"], "hwp3", "{ok}");
    assert!(ok["pageCount"].as_u64().unwrap() >= 1, "{ok}");
    assert!(ok["paraCount"].as_u64().unwrap() >= 1, "{ok}");
    assert!(ok["fonts"].is_array(), "{ok}");
    assert!(records[1].get("error").is_some(), "{records:?}");
    assert_eq!(records[1]["exitClass"], "runtime", "{records:?}");
}

#[test]
fn batch_threads_parallel_keeps_input_order() {
    // --threads 병렬 처리에서도 NDJSON 은 stdin 입력 순서를 유지한다.
    let sample = sample_path();
    let sample_str = sample.to_str().unwrap();
    let args = ["batch", "export-text", "--json", "--threads", "4"];
    let stdin_body = format!("{sample_str}\n없는파일-order.hwp\n{sample_str}\n");
    let output = run_with_stdin(&args, &stdin_body);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let records: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("NDJSON 아님 ({e}): {l}")))
        .collect();
    assert_eq!(records.len(), 3, "{}", describe(&args, &output));
    assert!(records[0].get("error").is_none(), "{records:?}");
    assert!(records[1].get("error").is_some(), "{records:?}");
    assert!(records[2].get("error").is_none(), "{records:?}");
}

#[test]
fn batch_without_json_is_usage_error() {
    let args = ["batch", "export-text"];
    let output = run_with_stdin(&args, "");
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn batch_unknown_subcommand_is_usage_error() {
    let args = ["batch", "export-png", "--json"];
    let output = run_with_stdin(&args, "");
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
