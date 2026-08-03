//! [#3830] `batch extract-data` 계약 회귀 테스트.
//!
//! 핵심 계약 두 가지:
//! 1. 배치 레코드는 단건 `extract-data --json` 봉투와 **같은 스키마**다 — 새 추출 로직이
//!    아니라 같은 `DocumentCore::extract_data` 를 재사용한다는 증거다.
//! 2. `--limit` 은 **배치 전체가 아니라 문서마다** 적용된다. 전역 상한으로 잘못 구현하면
//!    앞선 문서가 한도를 다 써버려 뒤 문서가 조용히 0건으로 보이고, 소비자는 "그 문서에
//!    값이 없다"와 "한도를 이미 다 썼다"를 구별할 수 없다. 같은 문서를 stdin 에 두 번
//!    넣고 두 레코드 모두 독립적으로 절단됐는지 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// 실제 정부 문서 — 날짜·금액·수량이 모두 실재해 `--limit` 절단을 검증할 만큼 건수가
/// 많다(단건 `extract_data_contract.rs` 와 같은 오라클 픽스처).
const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

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

fn run_with_stdin(args: &[&str], stdin_body: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    write_stdin_ignoring_early_exit(&mut child, stdin_body);
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

fn field_names(v: &serde_json::Value) -> Vec<String> {
    let mut names: Vec<String> = v
        .as_object()
        .unwrap_or_else(|| panic!("JSON 객체가 아닙니다: {v}"))
        .keys()
        .cloned()
        .collect();
    names.sort();
    names
}

#[test]
fn batch_extract_data_record_is_isomorphic_to_single_command_envelope() {
    let p = sample(SAMPLE);
    let s = p.to_str().unwrap();

    let single = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["extract-data", s, "--json"])
        .output()
        .expect("rhwp 실행 실패");
    assert_eq!(single.status.code(), Some(0), "단건 실행 실패");
    let single_env: serde_json::Value =
        serde_json::from_slice(&single.stdout).expect("단건 봉투 JSON");

    let args = ["batch", "extract-data", "--json"];
    let batch = run_with_stdin(&args, &format!("{s}\n"));
    assert_eq!(batch.status.code(), Some(0), "{}", describe(&args, &batch));

    let records = ndjson(&args, &batch);
    assert_eq!(records.len(), 1, "{}", describe(&args, &batch));
    let v = &records[0];

    assert_eq!(
        field_names(v),
        field_names(&single_env),
        "배치 레코드는 단건 extract-data 봉투와 같은 필드여야 합니다\n배치: {v}\n단건: {single_env}"
    );
    for key in [
        "schemaVersion",
        "kind",
        "itemCount",
        "totalItemCount",
        "truncated",
        "counts",
        "items",
    ] {
        assert_eq!(v[key], single_env[key], "{key} 불일치: {v} / {single_env}");
    }
    assert!(
        v["itemCount"].as_u64().unwrap() > 0,
        "실물 문서인데 0건입니다: {v}"
    );
}

/// [#3830] 핵심 계약: `--limit` 은 문서마다 독립 적용된다 — 배치 전체 상한이 아니다.
/// 같은 문서를 두 번 넣고 둘 다 같은 개수로 절단되는지, 총량(totalItemCount)이 절단
/// 전 그 문서 자체의 총량인지를 고정한다.
#[test]
fn batch_extract_data_limit_applies_per_document_not_globally() {
    let p = sample(SAMPLE);
    let s = p.to_str().unwrap();

    // 오라클: 절단 없는 단건 실행의 총 건수.
    let full = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["extract-data", s, "--json"])
        .output()
        .expect("rhwp 실행 실패");
    let full_env: serde_json::Value = serde_json::from_slice(&full.stdout).expect("단건 봉투");
    let total = full_env["totalItemCount"].as_u64().expect("totalItemCount");
    assert!(
        total > 3,
        "이 테스트는 --limit 3 이 실제로 자르는 픽스처가 필요합니다 (total={total})"
    );

    let args = ["batch", "extract-data", "--json", "--limit", "3"];
    // 같은 문서를 두 줄 — 전역 상한이었다면 둘째 줄은 0건(또는 절반)이 됐을 것이다.
    let output = run_with_stdin(&args, &format!("{s}\n{s}\n"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 2, "{}", describe(&args, &output));
    for (i, v) in records.iter().enumerate() {
        assert_eq!(v["itemCount"], 3, "레코드 {i}: {v}");
        assert_eq!(
            v["totalItemCount"].as_u64(),
            Some(total),
            "레코드 {i}: totalItemCount 는 그 문서 자체의 총량이어야 합니다: {v}"
        );
        assert_eq!(v["truncated"], true, "레코드 {i}: {v}");
        assert_eq!(v["items"].as_array().unwrap().len(), 3, "레코드 {i}: {v}");
    }
}

#[test]
fn batch_extract_data_kind_filters_like_single_command() {
    let p = sample(SAMPLE);
    let s = p.to_str().unwrap();
    let args = ["batch", "extract-data", "--json", "--kind", "date"];
    let output = run_with_stdin(&args, &format!("{s}\n"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let records = ndjson(&args, &output);
    let v = &records[0];
    assert_eq!(v["kind"], "date", "{v}");
    assert!(
        v["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|it| it["kind"] == "date"),
        "date 이외의 항목이 섞였습니다: {v}"
    );
    // 요청하지 않은 종류의 키는 counts 에 없어야 한다 — 단건과 같은 규약(§6-10).
    let counts = v["counts"].as_object().expect("counts");
    assert!(counts.contains_key("date"), "{v}");
    assert!(!counts.contains_key("amount"), "{v}");
    assert!(!counts.contains_key("number"), "{v}");
}

#[test]
fn batch_extract_data_preserves_order_and_reports_partial_failure() {
    let p = sample(SAMPLE);
    let s = p.to_str().unwrap();
    let args = ["batch", "extract-data", "--json"];
    let output = run_with_stdin(
        &args,
        &format!("{s}\n없는파일-batch-extract-data.hwp\n{s}\n"),
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 3, "{}", describe(&args, &output));
    assert!(records[0].get("error").is_none(), "{:?}", records[0]);
    assert!(records[1].get("error").is_some(), "{:?}", records[1]);
    assert_eq!(records[1]["exitClass"], "runtime", "{:?}", records[1]);
    assert_eq!(records[1]["schemaVersion"], "1.0", "{:?}", records[1]);
    assert!(records[2].get("error").is_none(), "{:?}", records[2]);

    // 실패 경로는 이 프로세스가 유일하게 쓴 stdout 이 그 오류 레코드 한 줄뿐이어야
    // 한다 — 진단 텍스트가 stdout 에 섞이지 않는다(stderr 만 진단용).
    let stdout_text = String::from_utf8_lossy(&output.stdout);
    let error_lines: Vec<&str> = stdout_text
        .lines()
        .filter(|l| l.contains("없는파일-batch-extract-data.hwp"))
        .collect();
    assert_eq!(error_lines.len(), 1, "{stdout_text}");
    serde_json::from_str::<serde_json::Value>(error_lines[0]).expect("오류 레코드도 순수 JSON");
}

#[test]
fn batch_kind_and_limit_flags_rejected_for_other_subcommands() {
    // --kind·--limit 는 extract-data 축 전용이다(--query 가 search 전용인 것과 같은 규약).
    for extra in [vec!["--kind", "date"], vec!["--limit", "3"]] {
        let mut args = vec!["batch", "info", "--json"];
        args.extend(extra.clone());
        let output = run_with_stdin(&args, "");
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            describe(&args, &output)
        );
    }
}

#[test]
fn batch_extract_data_invalid_kind_is_usage_error() {
    let args = ["batch", "extract-data", "--json", "--kind", "bogus"];
    let output = run_with_stdin(&args, "");
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn capabilities_batch_declares_extract_data_axis() {
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
    assert!(
        subs.contains(&"extract-data"),
        "capabilities 의 batch 축에 extract-data 가 없습니다: {subs:?}"
    );
    let flags: Vec<&str> = v["batch"]["flags"]
        .as_array()
        .expect("batch.flags")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    for expected in ["--kind", "--limit"] {
        assert!(
            flags.contains(&expected),
            "batch 플래그에 {expected} 누락: {flags:?}"
        );
    }
    assert!(
        v["batch"]["mcp"]["available"]
            .as_array()
            .expect("mcp.available")
            .iter()
            .any(|s| s
                .as_str()
                .is_some_and(|s| s.contains("hwp_batch_extract_data"))),
        "capabilities 의 batch.mcp.available 에 hwp_batch_extract_data 언급이 없습니다: {v}"
    );
}

#[test]
fn mcp_batch_extract_data_tool_is_invocable_from_its_declaration() {
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp"])
        .output()
        .expect("rhwp 실행 실패");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("MCP JSON");
    let tools = v["tools"].as_array().expect("tools");

    let tool = tools
        .iter()
        .find(|t| t["name"] == "hwp_batch_extract_data")
        .expect("hwp_batch_extract_data 도구가 있어야 합니다");

    // 드리프트 가드: inputSchema 는 type·properties·required 를 갖춰야 한다.
    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    let properties = tool["inputSchema"]["properties"]
        .as_object()
        .expect("properties");
    assert!(
        tool["inputSchema"]["required"].is_array(),
        "required 배열이 있어야 합니다: {tool}"
    );

    // --json 은 MCP 를 통한 호출에서 필수다(다른 hwp_batch_* 와 같은 규약) — 인자
    // 템플릿에 항상 박혀 있어야 한다.
    let args_str = tool["cli"]["args"].to_string();
    assert!(
        args_str.contains("--json"),
        "cli.args 에 --json 이 항상 포함돼야 합니다: {args_str}"
    );

    // paths 는 stdin 으로 전달되므로 args 템플릿에는 {paths} 자리표시자가 없어야 하고,
    // MCP_STDIN_TOOLS 목록에 이 도구 이름이 등재돼 있어야 한다(런타임에서 실제 stdin
    // 배선 여부는 mcp_serve.rs 의 run_cli_tool 이 그 목록을 참조해 검증한다).
    assert!(
        !args_str.contains("{paths}"),
        "paths 는 stdin 전용이라 cli.args 자리표시자에 나오면 안 됩니다: {args_str}"
    );

    // 선언된 속성은 paths 를 제외하고 전부 CLI 플래그로 배선돼야 한다 — 선언만 있고
    // 배선이 없으면 클라이언트가 보낸 값이 조용히 무시된다.
    let optional_args_str = tool["cli"]["optionalArgs"].to_string();
    for (name, placeholder) in [
        ("kind", "{kind}"),
        ("limit", "{limit}"),
        ("threads", "{threads}"),
    ] {
        assert!(properties.contains_key(name), "{tool}");
        assert!(
            args_str.contains(placeholder) || optional_args_str.contains(placeholder),
            "{name} 속성이 선언돼 있지만 CLI 로 배선되지 않았습니다: {tool}"
        );
    }
    assert!(properties.contains_key("paths"), "{tool}");
}
