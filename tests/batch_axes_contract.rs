//! [#3346] `batch` 신규 축(search·export-tables·fields) 계약 회귀 테스트.
//!
//! 핵심 계약: 배치 레코드는 **단건 명령의 봉투와 같은 스키마**다 — 소비자가 단건/배치를
//! 같은 코드로 읽는다. 입력 순서 보존·부분 실패 exit 1 은 기존 batch 규약 그대로다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SAMPLE: &str = "samples/hwp3-sample.hwp";
/// 표를 가진 문서.
const SAMPLE_TABLE: &str = "samples/table-001.hwp";
/// 누름틀을 가진 문서.
const SAMPLE_FIELDS: &str = "samples/field-01.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run_with_stdin(args: &[&str], stdin_body: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
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

fn ndjson(args: &[&str], output: &Output) -> Vec<serde_json::Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l)
                .unwrap_or_else(|e| panic!("NDJSON 아님 ({e}): {l}\n{}", describe(args, output)))
        })
        .collect()
}

#[test]
fn batch_search_records_share_single_command_schema() {
    let p = sample(SAMPLE);
    let s = p.to_str().unwrap();
    let args = ["batch", "search", "--query", "의", "--json"];
    let output = run_with_stdin(&args, &format!("{s}\n{s}\n"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 2, "{}", describe(&args, &output));
    for v in &records {
        // 단건 `search --json` 봉투와 같은 필드들.
        assert_eq!(v["schemaVersion"], "1.0", "{v}");
        assert!(v["source"].is_string(), "{v}");
        assert_eq!(v["query"], "의", "{v}");
        assert!(v["matchCount"].as_u64().is_some(), "{v}");
        assert!(v["matches"].is_array(), "{v}");
        assert!(v.get("error").is_none(), "{v}");
    }
    assert!(
        records[0]["matchCount"].as_u64().unwrap() >= 1,
        "문서에 있는 검색어인데 0건입니다: {:?}",
        records[0]
    );
}

#[test]
fn batch_export_tables_records_share_single_command_schema() {
    let p = sample(SAMPLE_TABLE);
    let s = p.to_str().unwrap();
    let args = ["batch", "export-tables", "--json"];
    let output = run_with_stdin(&args, &format!("{s}\n"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 1, "{}", describe(&args, &output));
    let v = &records[0];
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert!(v["tableCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["tables"].is_array(), "{v}");
    // 병합 보존이 배치 경로에서도 유지되는지 — 단건과 같은 추출기를 쓴다는 증거.
    let has_merge = v["tables"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["cells"].as_array().unwrap().iter())
        .any(|c| {
            c["colSpan"].as_u64().unwrap_or(1) >= 2 || c["rowSpan"].as_u64().unwrap_or(1) >= 2
        });
    assert!(has_merge, "병합 정보가 배치에서도 보존되어야 합니다: {v}");
}

#[test]
fn batch_fields_records_share_single_command_schema() {
    let p = sample(SAMPLE_FIELDS);
    let s = p.to_str().unwrap();
    let args = ["batch", "fields", "--json"];
    let output = run_with_stdin(&args, &format!("{s}\n"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    let v = &records[0];
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["fieldCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["fields"].is_array(), "{v}");
}

#[test]
fn batch_new_axes_preserve_input_order_and_report_partial_failure() {
    // 기존 batch 규약(순서 보존 + 부분 실패 exit 1)이 신규 축에서도 성립해야 한다.
    let p = sample(SAMPLE);
    let s = p.to_str().unwrap();
    let args = ["batch", "search", "--query", "의", "--json"];
    let output = run_with_stdin(&args, &format!("{s}\n없는파일-batch-search.hwp\n{s}\n"));
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 3, "{}", describe(&args, &output));
    // 입력 순서 보존: 두 번째가 실패 레코드여야 한다.
    assert!(records[0].get("error").is_none(), "{:?}", records[0]);
    assert!(records[1].get("error").is_some(), "{:?}", records[1]);
    assert_eq!(records[1]["exitClass"], "runtime", "{:?}", records[1]);
    assert_eq!(records[1]["schemaVersion"], "1.0", "{:?}", records[1]);
    assert!(records[2].get("error").is_none(), "{:?}", records[2]);
}

#[test]
fn batch_search_without_query_is_usage_error() {
    let args = ["batch", "search", "--json"];
    let output = run_with_stdin(&args, "");
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn batch_query_flag_rejected_for_other_subcommands() {
    // --query 는 search 축 전용이다 (--mode 가 export-structure 전용인 것과 같은 규약).
    let args = ["batch", "info", "--json", "--query", "x"];
    let output = run_with_stdin(&args, "");
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn batch_existing_axes_still_work() {
    // 무회귀 가드: 기존 3축이 그대로 동작해야 한다.
    let p = sample(SAMPLE);
    let s = p.to_str().unwrap();
    for sub in ["info", "export-text", "export-structure"] {
        let args = ["batch", sub, "--json"];
        let output = run_with_stdin(&args, &format!("{s}\n"));
        assert_eq!(
            output.status.code(),
            Some(0),
            "기존 축 {sub} 회귀\n{}",
            describe(&args, &output)
        );
        let records = ndjson(&args, &output);
        assert_eq!(records.len(), 1, "{sub}");
        assert_eq!(records[0]["schemaVersion"], "1.0", "{sub}");
    }
}

#[test]
fn capabilities_batch_list_includes_new_axes() {
    // 드리프트 가드: 축을 추가했으면 자기서술도 같이 갱신되어야 한다.
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities"])
        .output()
        .expect("rhwp 실행 실패");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("capabilities JSON");
    let subs: Vec<&str> = v["batch"]["subcommands"]
        .as_array()
        .expect("batch.subcommands")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    for expected in ["search", "export-tables", "fields"] {
        assert!(
            subs.contains(&expected),
            "capabilities 의 batch 축에 {expected} 가 없습니다: {subs:?}"
        );
    }
}

#[test]
fn mcp_batch_tools_are_invocable_from_their_declaration() {
    // [#3346] MCP 도구는 **선언만 보고 호출**할 수 있어야 한다. `--query` 가 필수인
    // search 축을 인자 자리표시자 없이 hwp_batch 의 enum 에만 넣으면, 매니페스트를
    // 따르는 클라이언트가 `batch search --json` 을 만들어 항상 exit 2 를 받는다.
    // 그래서 search 는 전용 도구(hwp_batch_search)로 분리한다.
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp"])
        .output()
        .expect("rhwp 실행 실패");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("MCP JSON");
    let tools = v["tools"].as_array().expect("tools");

    let batch = tools
        .iter()
        .find(|t| t["name"] == "hwp_batch")
        .expect("hwp_batch 도구");
    let subs: Vec<&str> = batch["inputSchema"]["properties"]["subcommand"]["enum"]
        .as_array()
        .expect("subcommand enum")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    assert!(
        !subs.contains(&"search"),
        "search 는 --query 가 필수라 hwp_batch 로는 호출할 수 없습니다: {subs:?}"
    );

    let search = tools
        .iter()
        .find(|t| t["name"] == "hwp_batch_search")
        .expect("hwp_batch_search 도구가 있어야 합니다");
    let required: Vec<&str> = search["inputSchema"]["required"]
        .as_array()
        .expect("required")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    assert!(required.contains(&"query"), "{search}");

    // 인자 템플릿에 {query} 자리표시자가 실제로 있어야 값을 넘길 수 있다.
    let args_str = search["cli"]["args"].to_string();
    assert!(
        args_str.contains("{query}"),
        "cli.args 에 {{query}} 자리표시자가 필요합니다: {args_str}"
    );
}
