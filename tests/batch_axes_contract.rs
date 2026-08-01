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

// ── [#3626] convert 축 ─────────────────────────────────────────────────────

/// convert 축은 파일을 쓴다 — 테스트마다 격리된 임시 폴더를 쓴다.
fn convert_tmp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-batch-convert-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("임시 폴더 생성 실패");
    dir
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

/// 핵심 계약: 배치 레코드 = 단건 `convert --json` 봉투.
///
/// 샘플이 무손실로 왕복하는지 여부를 테스트가 미리 알 필요가 없도록 **단건을 오라클로
/// 두고** 대조한다 — 필드 이름 집합·판정·종료 코드가 모두 같아야 한다.
#[test]
fn batch_convert_record_is_isomorphic_to_single_command_envelope() {
    let p = sample(SAMPLE);
    let s = p.to_str().unwrap();
    let single_dir = convert_tmp_dir("single");
    let batch_dir = convert_tmp_dir("batch");
    let single_out = single_dir.join("hwp3-sample.hwp");

    let single_args = [
        "convert",
        s,
        single_out.to_str().unwrap(),
        "--verify",
        "--verify-pages",
        "--json",
    ];
    let single = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(single_args)
        .output()
        .expect("rhwp 실행 실패");

    let batch_args = [
        "batch",
        "convert",
        "--out-dir",
        batch_dir.to_str().unwrap(),
        "--verify",
        "--verify-pages",
        "--json",
    ];
    let batch = run_with_stdin(&batch_args, &format!("{s}\n"));

    // 판정을 1 로 접지 않는다: 단건이 0/3/4 중 무엇으로 끝나든 배치도 같아야 한다.
    assert_eq!(
        batch.status.code(),
        single.status.code(),
        "단건과 배치의 종료 코드가 달라졌습니다\n{}",
        describe(&batch_args, &batch)
    );

    let single_env: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&single.stdout).trim())
            .expect("단건 봉투 JSON");
    let records = ndjson(&batch_args, &batch);
    assert_eq!(records.len(), 1, "{}", describe(&batch_args, &batch));
    let v = &records[0];

    assert_eq!(
        field_names(v),
        field_names(&single_env),
        "배치 레코드는 단건 convert 봉투와 같은 필드여야 합니다\n배치: {v}\n단건: {single_env}"
    );
    for key in ["schemaVersion", "format", "wasDistribution"] {
        assert_eq!(v[key], single_env[key], "{key} 불일치: {v} / {single_env}");
    }
    // 판정은 데이터 — 같은 문서라면 단건과 배치의 판정이 같아야 한다.
    assert_eq!(v["verify"], single_env["verify"], "{v}");
    assert_eq!(v["verifyPages"], single_env["verifyPages"], "{v}");
    assert!(v["bytes"].as_u64().unwrap_or(0) > 0, "{v}");

    // 산출물은 --out-dir 아래 <입력이름>.hwp 로 실제로 만들어져야 한다.
    let produced = batch_dir.join("hwp3-sample.hwp");
    assert!(
        produced.is_file(),
        "산출 파일이 없습니다: {}",
        produced.display()
    );

    let _ = std::fs::remove_dir_all(&single_dir);
    let _ = std::fs::remove_dir_all(&batch_dir);
}

/// 기존 규약(순서 보존 + 부분 실패 exit 1)이 **쓰기 축에서도** 성립해야 한다.
/// 검증 판정(3/4)이 있더라도 하드 실패가 있으면 1 이 이긴다 — 소비자가 재실행
/// 대상과 검토 대상을 갈라야 하기 때문이다.
#[test]
fn batch_convert_preserves_order_and_hard_failure_wins() {
    let a = sample(SAMPLE);
    let b = sample(SAMPLE_TABLE);
    let out = convert_tmp_dir("partial");
    let args = [
        "batch",
        "convert",
        "--out-dir",
        out.to_str().unwrap(),
        "--verify",
        "--json",
    ];
    let output = run_with_stdin(
        &args,
        &format!(
            "{}\n없는파일-batch-convert.hwp\n{}\n",
            a.to_str().unwrap(),
            b.to_str().unwrap()
        ),
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
    assert!(records[2].get("error").is_none(), "{:?}", records[2]);
    let _ = std::fs::remove_dir_all(&out);
}

/// `--out-dir` 는 입력 파일 이름만 남긴다. 서로 다른 폴더의 같은 이름은 한 경로로
/// 겹치므로, **절반만 변환된 산출 폴더를 남기지 않도록 쓰기 전에** 거부해야 한다.
#[test]
fn batch_convert_output_collision_is_refused_before_any_write() {
    let root = convert_tmp_dir("collide");
    let dir_a = root.join("a");
    let dir_b = root.join("b");
    std::fs::create_dir_all(&dir_a).expect("a 폴더");
    std::fs::create_dir_all(&dir_b).expect("b 폴더");
    let src = sample(SAMPLE);
    let a = dir_a.join("dup.hwp");
    let b = dir_b.join("dup.hwp");
    std::fs::copy(&src, &a).expect("복사 a");
    std::fs::copy(&src, &b).expect("복사 b");
    let out = root.join("out");

    let args = [
        "batch",
        "convert",
        "--out-dir",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run_with_stdin(
        &args,
        &format!("{}\n{}\n", a.to_str().unwrap(), b.to_str().unwrap()),
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "이름이 겹치는 입력은 사용법 오류여야 합니다\n{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "거부된 실행은 레코드를 내지 않아야 합니다\n{}",
        describe(&args, &output)
    );
    let written = std::fs::read_dir(&out).map(|d| d.count()).unwrap_or(0);
    assert_eq!(
        written,
        0,
        "부분 산출물이 남았습니다: {}\n{}",
        out.display(),
        describe(&args, &output)
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// 목적지를 추측하지 않는다 — stdin 경로 목록만으로 파일을 흩뿌리면 안 된다.
/// 그리고 convert 전용 플래그는 다른 축에서 거부된다(`--query`·`--mode` 와 같은 규약).
#[test]
fn batch_convert_flags_are_axis_scoped() {
    assert_eq!(
        run_with_stdin(&["batch", "convert", "--json"], "")
            .status
            .code(),
        Some(2),
        "--out-dir 없는 convert 는 사용법 오류여야 합니다"
    );
    for extra in [
        vec!["--out-dir", "x"],
        vec!["--verify"],
        vec!["--verify-pages"],
    ] {
        let mut args = vec!["batch", "info", "--json"];
        args.extend(extra);
        let output = run_with_stdin(&args, "");
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            describe(&args, &output)
        );
    }
}

/// 드리프트 가드: 축·플래그·집계 규칙이 자기서술에 함께 있어야 소비자가 3/4 를
/// "부분 실패"로 오해하지 않는다.
#[test]
fn capabilities_batch_declares_convert_axis_and_exit_aggregation() {
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
        subs.contains(&"convert"),
        "batch 축에 convert 누락: {subs:?}"
    );
    let flags: Vec<&str> = v["batch"]["flags"]
        .as_array()
        .expect("batch.flags")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    for expected in ["--out-dir", "--verify", "--verify-pages"] {
        assert!(
            flags.contains(&expected),
            "batch 플래그에 {expected} 누락: {flags:?}"
        );
    }
    assert!(
        v["batch"]["exitAggregation"].is_string(),
        "종료 코드 집계 규칙이 자기서술에 있어야 합니다: {v}"
    );
}
