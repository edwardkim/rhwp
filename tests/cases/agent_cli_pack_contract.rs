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
        "grep",
        "hidden-text",
        "unicode-scan",
        "explore",
        "table-inspect",
        "explain",
        "notes",
        "field-diff",
        "bookmarks",
        "charts",
        "digest",
        "page-hashes",
        "empty-fields",
        "merged-tables",
        "encrypted",
        "armor",
        "stego-scan",
        "sweep",
        "outline-nav",
        "field-locate",
        "captions",
        "headers-footers",
        "batch-info",
        "search-count",
        "doc-info",
        "page-info",
        "section-def",
        "field-get",
        "page-pos",
        "para-page",
        "chart-data",
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

#[test]
fn grep_finds_linux_with_page_and_cell() {
    let path = sample("samples/hwp3-sample.hwp");
    let out = run(&[
        "grep",
        "--json",
        "--limit",
        "5",
        path.to_str().unwrap(),
        "--q",
        "Linux",
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "grep");
    assert!(v["untrustedContent"].as_bool().unwrap(), "{v}");
    assert!(v["matchCount"].as_u64().unwrap() >= 1, "{v}");
    let hit = &v["matches"][0];
    assert_eq!(hit["section"], 0, "{hit}");
    assert_eq!(hit["paragraph"], 0, "{hit}");
    assert_eq!(hit["page"], 0, "{hit}");
    assert!(hit["context"].as_str().unwrap().contains("Linux"), "{hit}");
    assert!(
        v["matches"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m.get("cell").is_some()),
        "expected a table-cell hit in {v}"
    );
}

#[test]
fn grep_unknown_flag_is_usage() {
    let path = sample("samples/form-01.hwp");
    let out = run(&["grep", path.to_str().unwrap(), "--q", "x", "--nope"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn hidden_text_clean_on_real_samples() {
    for rel in ["samples/form-01.hwp", "samples/hwp3-sample.hwp"] {
        let path = sample(rel);
        let out = run(&["hidden-text", "--json", path.to_str().unwrap()]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{rel} stderr {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v = stdout_json(&out);
        assert_envelope(&v, "hidden-text");
        assert_eq!(v["clean"], true, "{rel} {v}");
        assert_eq!(v["hiddenCharCount"], 0, "{rel} {v}");
        assert!(v["hiddenText"].as_array().unwrap().is_empty(), "{rel} {v}");
    }
}

#[test]
fn unicode_scan_on_hwp3_sample() {
    let path = sample("samples/hwp3-sample.hwp");
    let out = run(&["unicode-scan", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "unicode-scan");
    assert_eq!(v["clean"], true, "{v}");
    assert_eq!(v["findingCount"], 0, "{v}");
    assert!(v["scannedChars"].as_u64().unwrap() > 0, "{v}");
}

#[test]
fn explore_routes_form_sample_to_fill() {
    let path = sample("samples/form-01.hwp");
    let out = run(&["explore", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "explore");
    assert_eq!(v["untrustedContent"], false, "{v}");
    assert!(v["fieldCount"].as_u64().unwrap() >= 1, "{v}");
    let names: Vec<&str> = v["menu"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["affordance"].as_str())
        .collect();
    assert!(names.contains(&"form-fill"), "menu={names:?}");
    assert!(names.contains(&"triage-overview"), "menu={names:?}");
}

#[test]
fn table_inspect_recipe_sample_header_row() {
    let path = sample("samples/hwp_table_test.hwp");
    let out = run(&[
        "table-inspect",
        "--json",
        "--table",
        "0",
        path.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "table-inspect");
    assert_eq!(v["tableCount"], 10, "{v}");
    assert_eq!(v["emittedCount"], 1, "{v}");
    let t0 = &v["tables"][0];
    assert_eq!(t0["rows"], 4, "{t0}");
    assert_eq!(t0["cols"], 3, "{t0}");
    assert_eq!(t0["csvReady"], true, "{t0}");
    let texts: Vec<&str> = t0["cells"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["row"] == 0)
        .filter_map(|c| c["text"].as_str())
        .collect();
    assert_eq!(texts, ["제목", "담당자", "세부 내용"], "{t0}");
}

#[test]
fn table_inspect_hwp3_sample_count() {
    let path = sample("samples/hwp3-sample.hwp");
    let out = run(&["table-inspect", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "table-inspect");
    assert_eq!(v["tableCount"], 6, "{v}");
    assert_eq!(v["emittedCount"], 6, "{v}");
}

#[test]
fn explain_describes_form_and_hwp3() {
    let form = sample("samples/form-01.hwp");
    let out = run(&["explain", "--json", form.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "explain");
    assert_eq!(v["format"], "hwp5", "{v}");
    assert!(v["fieldCount"].as_u64().unwrap() >= 1, "{v}");
    assert_eq!(v["tableCount"], 0, "{v}");
    assert!(v["summary"].as_str().unwrap().contains("누름틀"), "{v}");

    let hwp3 = sample("samples/hwp3-sample.hwp");
    let out = run(&["explain", "--json", hwp3.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_envelope(&v, "explain");
    assert_eq!(v["format"], "hwp3", "{v}");
    assert_eq!(v["tableCount"], 6, "{v}");
    assert_eq!(v["fieldCount"], 0, "{v}");
}

#[test]
fn notes_counts_on_real_samples() {
    for rel in ["samples/form-01.hwp", "samples/hwp3-sample.hwp"] {
        let path = sample(rel);
        let out = run(&["notes", "--json", path.to_str().unwrap()]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{rel} stderr {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v = stdout_json(&out);
        assert_envelope(&v, "notes");
        assert!(v["footnoteCount"].is_number(), "{rel} {v}");
        assert!(v["endnoteCount"].is_number(), "{rel} {v}");
        assert_eq!(
            v["noteCount"].as_u64().unwrap(),
            v["footnoteCount"].as_u64().unwrap() + v["endnoteCount"].as_u64().unwrap(),
            "{rel} {v}"
        );
    }
}

#[test]
fn field_diff_form_vs_hwp3_and_self() {
    let form = sample("samples/form-01.hwp");
    let hwp3 = sample("samples/hwp3-sample.hwp");
    let out = run(&[
        "field-diff",
        "--json",
        form.to_str().unwrap(),
        hwp3.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(3),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "field-diff");
    assert_eq!(v["equal"], false, "{v}");
    assert!(v["countA"].as_u64().unwrap() >= 1, "{v}");
    assert_eq!(v["countB"], 0, "{v}");
    let only_a: Vec<&str> = v["onlyInA"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| n.as_str())
        .collect();
    assert!(
        only_a.iter().any(|n| n.contains("myMsg")),
        "expected myMsg field in {only_a:?}"
    );

    let out = run(&[
        "field-diff",
        "--json",
        form.to_str().unwrap(),
        form.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_envelope(&v, "field-diff");
    assert_eq!(v["equal"], true, "{v}");
    assert!(v["onlyInA"].as_array().unwrap().is_empty(), "{v}");
    assert!(v["onlyInB"].as_array().unwrap().is_empty(), "{v}");
}

#[test]
fn digest_pages_hwp3_sample() {
    let path = sample(SAMPLE);
    let out = run(&[
        "digest",
        "--json",
        "--max-chars",
        "80",
        path.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "digest");
    assert!(v["untrustedContent"].as_bool().unwrap(), "{v}");
    assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");
    let page0 = &v["pages"][0];
    assert!(
        page0["excerpt"].as_str().unwrap().chars().count() <= 80,
        "{page0}"
    );
    assert!(
        page0["firstLine"].as_str().unwrap().contains("Linux")
            || page0["excerpt"].as_str().unwrap().contains("Linux"),
        "{page0}"
    );
}

#[test]
fn page_hashes_stable_on_hwp3() {
    let path = sample(SAMPLE);
    let a = run(&["page-hashes", "--json", path.to_str().unwrap()]);
    let b = run(&["page-hashes", "--json", path.to_str().unwrap()]);
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(b.status.code(), Some(0));
    let va = stdout_json(&a);
    let vb = stdout_json(&b);
    assert_envelope(&va, "page-hashes");
    assert_eq!(va["pages"], vb["pages"], "{va} vs {vb}");
    assert_eq!(
        va["pages"].as_array().unwrap().len() as u64,
        va["pageCount"].as_u64().unwrap()
    );
}

#[test]
fn bookmarks_and_charts_are_counts() {
    for rel in ["samples/form-01.hwp", "samples/hwp3-sample.hwp"] {
        let path = sample(rel);
        let out = run(&["bookmarks", "--json", path.to_str().unwrap()]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{rel} stderr {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v = stdout_json(&out);
        assert_envelope(&v, "bookmarks");
        assert!(v["bookmarkCount"].is_number(), "{rel} {v}");

        let out = run(&["charts", "--json", path.to_str().unwrap()]);
        assert_eq!(out.status.code(), Some(0), "{rel}");
        let v = stdout_json(&out);
        assert_envelope(&v, "charts");
        assert_eq!(
            v["chartCount"],
            v["charts"].as_array().unwrap().len() as u64,
            "{rel} {v}"
        );
    }
}

#[test]
fn empty_fields_on_form_sample() {
    let path = sample("samples/form-01.hwp");
    let out = run(&["empty-fields", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "empty-fields");
    assert!(v["fieldCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["emptyCount"].as_u64().unwrap() >= 1, "{v}");
    let names: Vec<&str> = v["empty"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| n["name"].as_str())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("myMsg")),
        "expected empty myMsg, got {names:?}"
    );
}

#[test]
fn merged_tables_on_table_sample() {
    let path = sample("samples/hwp_table_test.hwp");
    let out = run(&["merged-tables", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "merged-tables");
    assert_eq!(v["tableCount"], 10, "{v}");
    assert!(v["mergedCount"].is_number(), "{v}");
}

#[test]
fn encrypted_plain_samples_exit_ok() {
    for rel in ["samples/form-01.hwp", "samples/hwp3-sample.hwp"] {
        let path = sample(rel);
        let out = run(&["encrypted", "--json", path.to_str().unwrap()]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{rel} stderr {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v = stdout_json(&out);
        assert_envelope(&v, "encrypted");
        assert_eq!(v["encrypted"], false, "{rel} {v}");
    }
}

#[test]
fn armor_fences_hwp3_body() {
    let path = sample(SAMPLE);
    let out = run(&[
        "armor",
        "--json",
        "--max-chars",
        "400",
        path.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "armor");
    let nonce = v["nonce"].as_str().unwrap();
    assert_eq!(nonce.len(), 32, "{v}");
    let text = v["armoredText"].as_str().unwrap();
    assert!(text.contains(&format!("⟦UNTRUSTED:{nonce}⟧")), "{text}");
    assert!(text.contains(&format!("⟦/UNTRUSTED:{nonce}⟧")), "{text}");
    assert!(text.contains("Linux"), "{text}");
    assert!(v["untrustedContent"].as_bool().unwrap(), "{v}");
}

#[test]
fn stego_scan_clean_on_real_samples() {
    let form = sample("samples/form-01.hwp");
    let out = run(&["stego-scan", "--json", form.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "form-01 stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "stego-scan");
    assert_eq!(v["clean"], true, "{v}");
    assert_eq!(v["findingCount"], 0, "{v}");
    assert!(v["scannedChars"].is_number(), "{v}");

    let hwp3 = sample(SAMPLE);
    let out = run(&["stego-scan", "--json", hwp3.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "hwp3 stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "stego-scan");
    assert_eq!(v["clean"], true, "{v}");
    assert_eq!(v["findingCount"], 0, "{v}");
    assert!(v["scannedChars"].as_u64().unwrap() > 0, "{v}");
}

#[test]
fn sweep_on_form_and_hwp3() {
    for rel in ["samples/form-01.hwp", "samples/hwp3-sample.hwp"] {
        let path = sample(rel);
        let out = run(&["sweep", "--json", path.to_str().unwrap()]);
        assert!(
            matches!(out.status.code(), Some(0) | Some(3)),
            "{rel} code {:?} stderr {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
        let v = stdout_json(&out);
        assert_envelope(&v, "sweep");
        assert!(v["threat"]["clean"].is_boolean(), "{rel} {v}");
        assert!(v["injection"]["clean"].is_boolean(), "{rel} {v}");
        assert!(v["hiddenText"]["clean"].is_boolean(), "{rel} {v}");
        assert!(v["unicode"]["clean"].is_boolean(), "{rel} {v}");
        assert!(v["stego"]["clean"].is_boolean(), "{rel} {v}");
        let expected = v["threat"]["clean"].as_bool().unwrap()
            && v["injection"]["clean"].as_bool().unwrap()
            && v["hiddenText"]["clean"].as_bool().unwrap()
            && v["unicode"]["clean"].as_bool().unwrap()
            && v["stego"]["clean"].as_bool().unwrap();
        assert_eq!(v["clean"], expected, "{rel} {v}");
        assert_eq!(
            out.status.code(),
            Some(if expected { 0 } else { 3 }),
            "{rel} {v}"
        );
    }
}

#[test]
fn digest_unknown_flag_is_usage() {
    let path = sample(SAMPLE);
    let out = run(&["digest", path.to_str().unwrap(), "--nope"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn field_locate_form_sample() {
    let path = sample("samples/form-01.hwp");
    let out = run(&["field-locate", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "field-locate");
    assert!(v["fieldCount"].as_u64().unwrap() >= 1, "{v}");
    let names: Vec<&str> = v["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("myMsg")),
        "expected myMsg in {names:?}"
    );
    assert!(v["fields"][0]["section"].is_number(), "{v}");
    assert!(v["fields"][0]["paragraph"].is_number(), "{v}");
}

#[test]
fn table_csv_all_hwp3() {
    let path = sample(SAMPLE);
    let out = run(&["table-csv", "--json", "--all", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "table-csv");
    assert_eq!(v["all"], true, "{v}");
    assert_eq!(v["tableCount"], 6, "{v}");
    assert_eq!(v["tables"].as_array().unwrap().len(), 6, "{v}");
    assert!(v["tables"][0]["csv"].as_str().unwrap().contains(','), "{v}");
}

#[test]
fn batch_info_two_samples() {
    let a = sample("samples/form-01.hwp");
    let b = sample(SAMPLE);
    let out = run(&[
        "batch-info",
        "--json",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "batch-info");
    assert_eq!(v["fileCount"], 2, "{v}");
    assert_eq!(v["okCount"], 2, "{v}");
    assert_eq!(v["failCount"], 0, "{v}");
    assert_eq!(v["files"][0]["format"], "hwp5", "{v}");
    assert_eq!(v["files"][1]["format"], "hwp3", "{v}");
}

#[test]
fn outline_nav_and_headers_on_samples() {
    for rel in ["samples/form-01.hwp", "samples/hwp3-sample.hwp"] {
        let path = sample(rel);
        let out = run(&["outline-nav", "--json", path.to_str().unwrap()]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{rel} outline stderr {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v = stdout_json(&out);
        assert_envelope(&v, "outline-nav");
        assert!(v["outlineCount"].is_number(), "{rel} {v}");

        let out = run(&["headers-footers", "--json", path.to_str().unwrap()]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{rel} hf stderr {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v = stdout_json(&out);
        assert_envelope(&v, "headers-footers");
        assert!(v["itemCount"].is_number(), "{rel} {v}");
    }
}

#[test]
fn search_count_linux_hwp3() {
    let path = sample(SAMPLE);
    let out = run(&[
        "search-count",
        "--json",
        path.to_str().unwrap(),
        "--q",
        "Linux",
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "search-count");
    assert!(v["matchCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(
        v.get("matches").is_none(),
        "search-count must not list matches: {v}"
    );
}

#[test]
fn page_pos_and_para_page_roundtrip() {
    let path = sample(SAMPLE);
    let out = run(&["page-pos", "--json", "--page", "0", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "page-pos");
    assert_eq!(v["position"]["ok"], true, "{v}");
    let sec = v["position"]["sec"].as_u64().unwrap().to_string();
    let para = v["position"]["para"].as_u64().unwrap().to_string();

    let out = run(&[
        "para-page",
        "--json",
        "--section",
        &sec,
        "--para",
        &para,
        path.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "para-page");
    assert_eq!(v["page"], 0, "{v}");
}

#[test]
fn page_info_and_section_def_hwp3() {
    let path = sample(SAMPLE);
    let out = run(&["page-info", "--json", "--page", "0", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "page-info");
    assert!(v["info"].is_object() || v["info"].is_string(), "{v}");

    let out = run(&[
        "section-def",
        "--json",
        "--section",
        "0",
        path.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "section-def");
    assert!(v["def"].is_object(), "{v}");
}

#[test]
fn doc_info_on_form() {
    let path = sample("samples/form-01.hwp");
    let out = run(&["doc-info", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "doc-info");
    assert!(v["info"].is_object() || v["info"].is_string(), "{v}");
}

#[test]
fn field_get_form_and_missing() {
    let path = sample("samples/form-01.hwp");
    let locate = run(&["field-locate", "--json", path.to_str().unwrap()]);
    assert_eq!(locate.status.code(), Some(0));
    let lv = stdout_json(&locate);
    let name = lv["fields"][0]["name"].as_str().unwrap();

    let out = run(&[
        "field-get",
        "--json",
        "--name",
        name,
        path.to_str().unwrap(),
    ]);
    assert!(
        matches!(out.status.code(), Some(0) | Some(3)),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "field-get");
    assert_eq!(v["name"], name, "{v}");

    let out = run(&[
        "field-get",
        "--json",
        "--name",
        "__no_such_field__",
        path.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(3));
    let v = stdout_json(&out);
    assert_eq!(v["found"], false, "{v}");
}

#[test]
fn chart_data_missing_on_form() {
    let path = sample("samples/form-01.hwp");
    let out = run(&[
        "chart-data",
        "--json",
        "--chart",
        "0",
        path.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.is_empty() || stdout_json(&out).get("schemaVersion").is_none());
}

#[test]
fn captions_on_table_sample() {
    let path = sample("samples/hwp_table_test.hwp");
    let out = run(&["captions", "--json", path.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_envelope(&v, "captions");
    assert_eq!(v["tableCount"], 10, "{v}");
    assert!(v["captionCount"].is_number(), "{v}");
}
