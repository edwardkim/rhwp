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
    write_stdin_ignoring_early_exit(&mut child, stdin_body);
    child.wait_with_output().expect("rhwp 종료 대기 실패")
}

/// stdin 에 본문을 쓰되, 자식이 stdin 을 읽기 전에 종료한 경우의 BrokenPipe 는
/// 무시한다. 인자 검증 거부 계열 테스트는 프로세스가 입력을 소비하기 전에
/// 종료하는 것이 정상 경로라, 쓰기 완료 여부는 검증 대상(종료 코드·출력)이
/// 아니다 (#3763 — batch_axes_contract.rs 와 같은 처리).
fn write_stdin_ignoring_early_exit(child: &mut std::process::Child, stdin_body: &str) {
    use std::io::ErrorKind;
    if let Err(err) = child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin_body.as_bytes())
    {
        assert_eq!(
            err.kind(),
            ErrorKind::BrokenPipe,
            "stdin 쓰기 실패: {err:?}"
        );
    }
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
fn capabilities_search_finds_table_commands() {
    // [#3828 B1] 처음 오는 에이전트가 정확한 명령 이름을 모를 때 "표" 로 관련 명령을
    // 훑을 수 있어야 한다. 부분 문자열 매칭이라 export-tables·table-to-csv·
    // csv-to-table 모두 걸린다.
    let args = ["capabilities", "--search", "표", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["search"], "표", "{v}");
    let commands = v["commands"].as_array().expect("commands 배열");
    assert!(!commands.is_empty(), "{v}");
    for name in ["export-tables", "table-to-csv", "csv-to-table"] {
        assert!(
            commands.iter().any(|c| c["name"] == name),
            "{name} 이 '표' 검색 결과에 없음: {v}"
        );
    }
    // 매치하지 않는 명령은 결과에 없어야 한다 (test-shape 요약 "도형 왕복 테스트" 에는
    // '표'가 없다 — '테스트'와 혼동하기 쉬워 일부러 고른 반례).
    assert!(
        !commands.iter().any(|c| c["name"] == "test-shape"),
        "무관한 명령이 섞임: {v}"
    );
}

#[test]
fn capabilities_search_no_match_is_empty_not_error() {
    // 매치 0건은 에러가 아니라 빈 commands 배열, exit 0.
    let args = ["capabilities", "--search", "없는단어999", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    let commands = v["commands"].as_array().expect("commands 배열");
    assert!(commands.is_empty(), "{v}");
}

#[test]
fn capabilities_search_multi_keyword_is_and() {
    // 여러 키워드는 AND — "표"만으로는 여러 건이지만 "표 병합" 은 병합을 다루는
    // 명령(export-tables: 병합 rowSpan/colSpan 보존)으로 더 좁혀져야 한다.
    let args = ["capabilities", "--search", "표", "--json"];
    let output = run(&args);
    let v = parse_stdout_json(&args, &output);
    let broad = v["commands"].as_array().expect("commands 배열").len();

    let args2 = ["capabilities", "--search", "표 병합", "--json"];
    let output2 = run(&args2);
    assert_eq!(
        output2.status.code(),
        Some(0),
        "{}",
        describe(&args2, &output2)
    );
    let v2 = parse_stdout_json(&args2, &output2);
    let narrow = v2["commands"].as_array().expect("commands 배열");
    assert!(
        narrow.len() < broad,
        "AND 조건이면 키워드 추가로 결과가 줄어들어야 함: 표={broad}건, 표+병합={}건",
        narrow.len()
    );
    assert!(!narrow.is_empty(), "{v2}");
    assert!(narrow.iter().any(|c| c["name"] == "export-tables"), "{v2}");
}

#[test]
fn capabilities_search_human_mode_is_not_json() {
    // --json 없이도 사람이 읽는 출력을 지원한다(다른 명령과 일관).
    let args = ["capabilities", "--search", "표"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
        "human 모드인데 순수 JSON 이 나옴: {stdout}"
    );
    assert!(stdout.contains("export-tables"), "{stdout}");
}

#[test]
fn capabilities_base_output_unchanged_by_search_flag_addition() {
    // 드리프트 가드: 인자 없는 기본 `capabilities` 의 출력은 --search 도입으로도
    // 절대 바뀌지 않는다.
    let args = ["capabilities"];
    let output = run(&args);
    let v = parse_stdout_json(&args, &output);
    assert!(v.get("search").is_none(), "{v}");
    let commands = v["commands"].as_array().expect("commands 배열");
    let cap_entry = commands
        .iter()
        .find(|c| c["name"] == "capabilities")
        .expect("capabilities 자기 항목");
    let flags = cap_entry["flags"].as_array().expect("flags");
    assert!(
        flags.iter().any(|f| f == "--search"),
        "commands[capabilities].flags 에 --search 미등재: {cap_entry}"
    );
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

    // [#3480] set-cell의 넘침 경고는 에이전트가 제출 불가 산출물을 선별하는 신호다.
    // capabilities에는 있으나 MCP 도구 정의에 누락되면 자동 등록 클라이언트가 이를
    // 알 수 없으므로 두 계약에 함께 있어야 한다. #3383의 실제 산출 형식도 같은 이유다.
    let set_cell = tools
        .iter()
        .find(|t| t["name"] == "hwp_set_cell")
        .unwrap_or_else(|| panic!("hwp_set_cell 도구 누락: {v}"));
    let output_fields = set_cell["outputFields"]
        .as_array()
        .unwrap_or_else(|| panic!("hwp_set_cell.outputFields 누락: {set_cell}"));
    for expected in ["overflow", "outputFormat"] {
        assert!(
            output_fields.iter().any(|field| field == expected),
            "hwp_set_cell 출력 계약에 {expected} 누락: {set_cell}"
        );
    }
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
        // [#3697] dump-pages 는 CLI 진단 계약만 우선 노출한다 — #3608 1-C 표는 이
        // 항목에 MCP 도구를 짝짓지 않았다(1-D 의 진단 도구 원칙). 에이전트 수요가
        // 실증되면 별도 이슈로 승격해 이 제외를 지운다.
        .filter(|n| *n != "dump-pages")
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

/// [#3712] exit code 사전이 실제 계약을 따라가는지 — 자기서술 드리프트 가드.
///
/// exit 3 은 처음 `convert/export-hwpx --verify` 하나였다가 `edit 3종 --verify`(#3702)·
/// `run` 계획 단언(#3703)으로 넓어졌다. 사전이 옛 서술에 머물면 에이전트는 자기서술만
/// 읽고 "편집에는 3이 안 나온다"고 판단한다 — 선언이 계약을 배신하는 지점이다.
#[test]
fn exit_code_dictionary_covers_every_verify_surface() {
    let args = ["capabilities"];
    let output = run(&args);
    let v = parse_stdout_json(&args, &output);
    for code in ["0", "1", "2", "3", "4"] {
        let entry = v["exitCodes"][code]
            .as_str()
            .unwrap_or_else(|| panic!("exitCodes.{code} 설명 필요: {v}"));
        assert!(
            !entry.trim().is_empty(),
            "exitCodes.{code} 가 빈 문자열: {v}"
        );
    }
    // exit 3 을 낼 수 있는 표면이 늘면 사전도 함께 늘어야 한다.
    let three = v["exitCodes"]["3"].as_str().unwrap();
    for surface in ["convert", "edit", "run"] {
        assert!(
            three.contains(surface),
            "exit 3 사전이 '{surface}' 표면을 빠뜨렸다: {three}"
        );
    }
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

/// `--help` 가 광고하는 명령 토큰들.
///
/// help 의 명령 줄 패턴은 "정확히 2칸 들여쓰기 + 소문자/하이픈 토큰"이다(옵션·설명 줄은
/// 그보다 깊게 들여쓴다). 양방향 가드가 **같은 파서**를 봐야 한쪽만 통과하는 착시가 없다.
fn help_command_tokens() -> Vec<String> {
    let help = run(&["--help"]);
    let help_text = String::from_utf8_lossy(&help.stdout);
    let mut tokens: Vec<String> = Vec::new();
    for line in help_text.lines() {
        let Some(rest) = line.strip_prefix("  ") else {
            continue;
        };
        if rest.starts_with(' ') || rest.starts_with('-') {
            continue; // 옵션·설명 줄
        }
        let token = rest.split_whitespace().next().unwrap_or("");
        if token.is_empty()
            || !token
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
        {
            continue;
        }
        if !tokens.iter().any(|t| t == token) {
            tokens.push(token.to_string());
        }
    }
    tokens
}

/// `--help` 에 일부러 싣지 않는 명령과 그 사유.
///
/// 여기 넣어도 되는 것은 "사용자가 부를 일이 없는 내부 프로브"뿐이다. 사유 없는
/// 허용목록은 가치가 없으므로 각 항목이 이유 문자열을 동반한다.
const HELP_HIDDEN: &[(&str, &str)] = &[
    (
        "core-pages",
        "코어 페이지 수만 찍는 회귀 조사용 프로브 — 산출물도 --json 계약도 없다",
    ),
    (
        "dump-extents",
        "레이아웃 트리 extent 원시 덤프 — 렌더러 디버깅 전용이라 사용자 어휘가 아니다",
    ),
    (
        "measure-width",
        "파일이 아니라 문자열을 받는 글꼴 폭 계산기 — 문서 처리 명령이 아니다",
    ),
];

#[test]
fn help_covers_every_capabilities_command() {
    // 드리프트 가드 ③(신규): capabilities 가 광고하는 명령은 사람이 보는 `--help` 에도
    // 있어야 한다. 종전 가드는 help→capabilities 한 방향뿐이라, 매뉴얼 절까지 갖춘
    // 사용자용 명령이 help 에서 통째로 빠져도 아무도 못 잡았다(extract-pages 가 실제로
    // 그랬다 — --json 계약까지 가진 명령이 help 에 없었다).
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let help = help_command_tokens();
    assert!(
        help.len() > 10,
        "help 파서가 명령을 거의 못 찾았습니다 — 파서가 조용히 0건을 내면 이 가드가 \
         공허하게 통과합니다: {help:?}"
    );

    let missing: Vec<&str> = cap["commands"]
        .as_array()
        .expect("commands 배열")
        .iter()
        .filter_map(|c| c["name"].as_str())
        .filter(|n| !help.iter().any(|h| h.as_str() == *n))
        .filter(|n| !HELP_HIDDEN.iter().any(|(hidden, _)| *hidden == *n))
        .collect();
    assert!(
        missing.is_empty(),
        "capabilities 에는 있는데 --help 에 없는 명령: {missing:?}\n\
         사용자용이면 print_help 에 추가하고, 내부 프로브면 HELP_HIDDEN 에 사유와 함께 넣으세요."
    );

    // 허용목록이 낡는 것도 같은 부류의 드리프트다 — help 에 실린 명령이 목록에 남아
    // 있으면 "감췄다"는 설명 자체가 거짓이 되므로 지우게 만든다.
    let stale: Vec<&str> = HELP_HIDDEN
        .iter()
        .map(|(hidden, _)| *hidden)
        .filter(|hidden| help.iter().any(|h| h.as_str() == *hidden))
        .collect();
    assert!(
        stale.is_empty(),
        "이미 --help 에 실린 명령이 HELP_HIDDEN 에 남아 있습니다: {stale:?}"
    );

    for (hidden, why) in HELP_HIDDEN {
        assert!(
            !why.trim().is_empty(),
            "{hidden} 의 은닉 사유가 비었습니다."
        );
    }
}

#[test]
fn capabilities_declared_flags_are_real_cli_flags() {
    // 드리프트 가드 ④(신규): `commands[].flags` 에 선언한 플래그는 실제로 존재해야 한다.
    // 매니페스트는 에이전트가 도구 정의를 자동 생성하는 원천이라(cli_json_pipeline_guide),
    // 여기 빠진 플래그는 그 에이전트가 영영 못 쓰는 기능이 된다. 여기서는 축 단위
    // 선언(batch.flags)과 명령 항목 선언(commands[batch].flags)의 어긋남을 잡는다 —
    // 같은 문서 안에서 서로 다른 말을 하고 있으면 어느 쪽도 믿을 수 없다.
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let axis: Vec<&str> = cap["batch"]["flags"]
        .as_array()
        .expect("batch.flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    let entry: Vec<&str> = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "batch")
        .expect("batch 항목")["flags"]
        .as_array()
        .expect("commands[batch].flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    let missing: Vec<&str> = axis
        .iter()
        .copied()
        .filter(|f| !entry.contains(f))
        .collect();
    assert!(
        missing.is_empty(),
        "batch.flags 에는 있는데 commands[batch].flags 에 없는 플래그: {missing:?}\n\
         (같은 매니페스트가 서로 다른 말을 하면 소비자는 어느 쪽도 믿을 수 없다)"
    );

    // edit 의 --occurrence 는 같은 항목 summary 가 이름을 대고 MCP 도구가 고정 배선한다.
    let edit_flags: Vec<&str> = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "edit")
        .expect("edit 항목")["flags"]
        .as_array()
        .expect("commands[edit].flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    let mcp = parse_stdout_json(&["capabilities", "--mcp"], &run(&["capabilities", "--mcp"]));
    let wired = mcp["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_set_checkbox")
        .expect("hwp_set_checkbox")["cli"]["args"]
        .to_string();
    if wired.contains("--occurrence") {
        assert!(
            edit_flags.contains(&"--occurrence"),
            "MCP 도구가 고정 배선하는 --occurrence 가 edit flags 에 없습니다: {edit_flags:?}"
        );
    }
}

#[test]
fn capabilities_formats_write_lists_every_produced_format() {
    // 드리프트 가드 ⑤(신규): 실제로 만들어 내는 형식은 formats.write 에 있어야 한다.
    // 매니페스트만 읽는 에이전트가 "HWP5 로는 못 쓴다"고 오판하면 변환 축을 통째로 못 쓴다.
    // 선언을 믿지 않고 **실제로 만들어 본 뒤** 봉투가 보고한 형식과 대조한다.
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let write: Vec<&str> = cap["formats"]["write"]
        .as_array()
        .expect("formats.write")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let out = std::env::temp_dir().join(format!("rhwp-capdrift-{}.hwp", std::process::id()));
    let args = [
        "convert",
        src.to_str().unwrap(),
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
    let v = parse_stdout_json(&args, &output);
    let produced = v["format"].as_str().expect("format");
    assert!(
        write.contains(&produced),
        "convert 가 실제로 낸 형식 {produced} 이 formats.write 에 없습니다: {write:?}"
    );
    let _ = std::fs::remove_file(&out);
}
