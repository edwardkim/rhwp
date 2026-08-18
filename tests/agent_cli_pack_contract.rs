//! rhwp-agent 에 추가한 조회 CLI 계약.
//!
//! 기존 `agent_toolkit_contract.rs` 를 수정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";
const SAMPLE_B: &str = "samples/hwpx/form-01.hwpx";

fn agent_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-agent")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-agent").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(agent_bin())
        .args(args)
        .output()
        .expect("rhwp-agent 실행 실패")
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout JSON 아님 ({e})\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_envelope(v: &serde_json::Value, command: &str) {
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["tool"], "rhwp-agent", "{v}");
    assert_eq!(v["command"], command, "{v}");
    assert!(v["untrustedContent"].is_boolean(), "{v}");
    assert!(v["untrustedFields"].is_array(), "{v}");
}

#[test]
fn capabilities_lists_new_commands() {
    let out = run(&["capabilities", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    let names: Vec<&str> = v["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    for need in [
        "info",
        "format",
        "pages",
        "search",
        "contains",
        "fields",
        "tables",
        "hash",
        "plan-lint",
        "envelope-lint",
        "compare-pages",
        "compare-text",
        "hangul-ratio",
        "magic",
        "nextcall",
        "extract-data",
        "field-values",
        "table-csv",
        "form-ready",
        "threat-scan",
        "injection-scan",
        "structure",
    ] {
        assert!(names.contains(&need), "missing {need} in {names:?}");
    }
}

#[test]
fn info_json_envelope() {
    let path = sample(SAMPLE);
    let args = ["info", "--json", path.to_str().unwrap()];
    let out = run(&args);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "info");
    assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["charCount"].is_number(), "{v}");
}

#[test]
fn format_detects_hwp3() {
    let path = sample(SAMPLE);
    let out = run(&["format", "--json", path.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_envelope(&v, "format");
    assert_eq!(v["format"], "hwp3", "{v}");
}

#[test]
fn unknown_flag_is_usage() {
    let path = sample(SAMPLE);
    let out = run(&["info", "--nope", path.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty(), "stdout must be empty on usage error");
}

#[test]
fn missing_file_is_usage() {
    let out = run(&["info", "--json"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn compare_pages_two_samples() {
    let a = sample(SAMPLE);
    let b = sample(SAMPLE_B);
    let out = run(&[
        "compare-pages",
        "--json",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
    ]);
    assert!(matches!(out.status.code(), Some(0) | Some(3)));
    let v = stdout_json(&out);
    assert_envelope(&v, "compare-pages");
    assert!(v["pageCountA"].is_number(), "{v}");
    assert!(v["pageCountB"].is_number(), "{v}");
}

#[test]
fn hash_is_stable() {
    let path = sample(SAMPLE);
    let out = run(&["hash", "--json", path.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_envelope(&v, "hash");
    assert!(v["hash"].as_str().unwrap().len() >= 32, "{v}");
}

#[test]
fn plan_lint_rejects_empty_object() {
    let dir = std::env::temp_dir().join(format!("rhwp_agent_planlint_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let plan = dir.join("bad.json");
    std::fs::write(&plan, "{}").unwrap();
    let out = run(&["plan-lint", "--json", plan.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    let v = stdout_json(&out);
    assert_envelope(&v, "plan-lint");
    assert_eq!(v["ok"], false);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn search_finds_or_reports_zero() {
    let path = sample(SAMPLE);
    let out = run(&["search", "--json", path.to_str().unwrap(), "--q", " "]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "search");
    assert!(v["matchCount"].is_number(), "{v}");
}

#[test]
fn form_ready_on_form_sample() {
    let path = sample("samples/form-01.hwp");
    let out = run(&["form-ready", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "form-ready");
    assert_eq!(v["ready"], true, "{v}");
    assert!(v["fieldCount"].as_u64().unwrap() >= 1, "{v}");
}

#[test]
fn field_values_names_form_sample() {
    let path = sample("samples/form-01.hwp");
    let out = run(&["field-values", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "field-values");
    let names: Vec<&str> = v["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("myMsg") || !n.is_empty()),
        "expected a field name, got {names:?}"
    );
}

#[test]
fn extract_data_on_real_sample() {
    let path = sample("samples/hwp3-sample.hwp");
    let out = run(&[
        "extract-data",
        "--json",
        "--kind",
        "all",
        "--limit",
        "20",
        path.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "extract-data");
    assert!(v["totalItemCount"].is_number(), "{v}");
}

#[test]
fn threat_scan_runs() {
    let path = sample("samples/hwp3-sample.hwp");
    let out = run(&["threat-scan", "--json", path.to_str().unwrap()]);
    assert!(matches!(out.status.code(), Some(0) | Some(3)));
    let v = stdout_json(&out);
    assert_envelope(&v, "threat-scan");
    assert!(v["clean"].is_boolean(), "{v}");
}
