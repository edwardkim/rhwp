//! [#5347] `layout-anomaly` 기계 계약 — `--batch` / `--types` / json·exit.
//!
//! 기본 종료 코드는 이상 신호가 있어도 0 이다. `--strict` 만 overflow·overlap
//! 확정 신호를 3 으로 올린다. `--batch` 는 `render-diff --batch` 와 같이
//! 재귀 `.hwp`/`.hwpx` 를 정렬 순으로 보고하고, 파일별 실패는 `error` 레코드(DATA)다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CLEAN: &str = "samples/hwp3-sample.hwp";
const OVERFLOW: &str = "samples/table_giant_cell_overfill.hwpx";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
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

fn parse_stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

fn ndjson(args: &[&str], output: &Output) -> Vec<serde_json::Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l).unwrap_or_else(|e| {
                panic!(
                    "NDJSON 한 줄이 JSON 이 아닙니다 ({e}): {l}\n{}",
                    describe(args, output)
                )
            })
        })
        .collect()
}

fn assert_silent_failure(args: &[&str], want: i32) {
    let output = run(args);
    assert_eq!(
        output.status.code(),
        Some(want),
        "{}",
        describe(args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 0바이트여야 합니다.\n{}",
        describe(args, &output)
    );
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("rhwp-layout-anomaly-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("임시 폴더 생성");
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn copy_as(&self, rel: &str, name: &str) {
        let src = sample(rel);
        if let Some(parent) = Path::new(name).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(self.0.join(parent)).expect("하위 폴더");
            }
        }
        std::fs::copy(&src, self.0.join(name)).expect("샘플 복사");
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn capabilities() -> serde_json::Value {
    parse_stdout_json(&["capabilities"], &run(&["capabilities"]))
}

fn command_entry() -> serde_json::Value {
    capabilities()["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "layout-anomaly")
        .expect("layout-anomaly 항목")
        .clone()
}

// ── M02-3: 기본 exit 0, --strict 만 비영 ───────────────────────────────

#[test]
fn layout_anomaly_default_exit_zero_even_with_signal() {
    let src = sample(OVERFLOW);
    let args = ["layout-anomaly", src.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "기본은 이상 신호가 있어도 0 이다.\n{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["mode"], "single", "{v}");
    assert_eq!(v["hasSignal"], true, "{v}");
    assert!(v["overflowCount"].as_u64().unwrap() > 0, "{v}");
    assert!(v["types"].is_null(), "{v}");
}

#[test]
fn layout_anomaly_strict_signal_exits_three() {
    let src = sample(OVERFLOW);
    let args = [
        "layout-anomaly",
        src.to_str().unwrap(),
        "--json",
        "--strict",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "--strict + 확정 신호는 3 이다.\n{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["hasSignal"], true, "{v}");
    assert_eq!(v["strict"], true, "{v}");
}

#[test]
fn layout_anomaly_strict_clean_stays_zero() {
    let src = sample(CLEAN);
    let args = [
        "layout-anomaly",
        src.to_str().unwrap(),
        "--json",
        "--strict",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["hasSignal"], false, "{v}");
}

/// [#6348] `-p` 는 `pages` 배열뿐 아니라 카운트·`hasSignal`·`--strict` 종료코드까지
/// 그 쪽으로 좁힌다.
///
/// 종전에는 배열만 걸러 `pages: []` 인데 `overflowCount` 는 문서 전체 값이 실렸고,
/// 신호가 하나도 없는 쪽을 지정해도 `--strict` 가 3 을 냈다. 그 값으로는
/// "이 쪽이 깨끗한가"를 판정할 수 없다.
///
/// 쪽 번호는 필터 없는 실행에서 뽑는다. 조판이 바뀌어 신호가 다른 쪽으로 옮겨가도
/// 이 계약 자체는 계속 검사된다.
#[test]
fn layout_anomaly_page_filter_scopes_counts_and_strict_exit() {
    let src = sample(OVERFLOW);
    let path = src.to_str().unwrap();

    let all_args = ["layout-anomaly", path, "--json"];
    let all_out = run(&all_args);
    let all = parse_stdout_json(&all_args, &all_out);
    let pages = all["pages"].as_array().unwrap();
    let page_count = all["pageCount"].as_u64().unwrap();

    // 확정 신호(overflow)가 있는 쪽 하나와, 신호가 아예 없는 쪽 하나.
    let dirty = pages
        .iter()
        .find(|p| p["overflow"].as_array().is_some_and(|o| !o.is_empty()))
        .map(|p| p["page"].as_u64().unwrap())
        .unwrap_or_else(|| panic!("스캐폴딩 전제: overflow 있는 쪽이 있어야 한다\n{all}"));
    let flagged: Vec<u64> = pages.iter().map(|p| p["page"].as_u64().unwrap()).collect();
    let clean = (0..page_count)
        .find(|p| !flagged.contains(p))
        .unwrap_or_else(|| panic!("스캐폴딩 전제: 신호 없는 쪽이 있어야 한다\n{all}"));

    let dirty_s = dirty.to_string();
    let args = ["layout-anomaly", path, "-p", &dirty_s, "--json"];
    let v = parse_stdout_json(&args, &run(&args));
    assert_eq!(v["pages"].as_array().unwrap().len(), 1, "{v}");
    assert_eq!(v["overflowCount"], 1, "그 쪽의 overflow 만 세야 한다\n{v}");
    assert_eq!(v["hasSignal"], true, "{v}");
    // pageCount 는 필터와 무관한 문서 메타데이터다.
    assert_eq!(v["pageCount"], page_count, "{v}");
    assert_eq!(v["pageFilter"], dirty, "{v}");

    let clean_s = clean.to_string();
    let args = ["layout-anomaly", path, "-p", &clean_s, "--json"];
    let v = parse_stdout_json(&args, &run(&args));
    assert!(v["pages"].as_array().unwrap().is_empty(), "{v}");
    for key in [
        "overflowCount",
        "offCanvasCount",
        "overlapCount",
        "textOverlapCount",
        "emptyPageCount",
    ] {
        assert_eq!(
            v[key], 0,
            "빈 pages 와 0 이 아닌 {key} 가 함께 나오면 안 된다\n{v}"
        );
    }
    assert_eq!(v["hasSignal"], false, "{v}");

    // 종료코드도 같은 집합에서 나온다.
    let args = ["layout-anomaly", path, "-p", &clean_s, "--strict", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "신호 없는 쪽만 봤으면 --strict 도 0 이다.\n{}",
        describe(&args, &output)
    );
    let args = ["layout-anomaly", path, "-p", &dirty_s, "--strict", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "그 쪽에 확정 신호가 있으면 3 이다.\n{}",
        describe(&args, &output)
    );

    // 필터가 없으면 종전 그대로 문서 전체.
    assert!(all["overflowCount"].as_u64().unwrap() >= 1, "{all}");
    assert_eq!(all["hasSignal"], true, "{all}");
    assert!(all["pageFilter"].is_null(), "{all}");
}

#[test]
fn layout_anomaly_usage_errors_are_exit_two_with_silent_stdout() {
    let src = sample(CLEAN);
    let path = src.to_str().unwrap();
    assert_silent_failure(&["layout-anomaly", path, "--nope", "--json"], 2);
    assert_silent_failure(&["layout-anomaly", "--json"], 2);
    assert_silent_failure(&["layout-anomaly", path, "--types", "NotAType"], 2);
    assert_silent_failure(&["layout-anomaly", "--batch"], 2);
}

// ── M02-1: --batch ──────────────────────────────────────────────────────

#[test]
fn layout_anomaly_batch_json_streams_ndjson_and_keeps_failed_loads() {
    let dir = TempDir::new("mixed");
    dir.copy_as(OVERFLOW, "ok.hwpx");
    std::fs::write(dir.path().join("깨진문서.hwp"), b"not a document").expect("깨진 파일");

    let args = [
        "layout-anomaly",
        "--batch",
        dir.path().to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "로드 실패가 있으면 1 이 --strict 의 3보다 우선한다.\n{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(
        records.len(),
        2,
        "입력 2건은 모두 레코드를 남겨야 합니다: {records:?}"
    );
    let mut sources: Vec<&str> = records
        .iter()
        .map(|r| r["source"].as_str().expect("source"))
        .collect();
    sources.sort_unstable();
    assert_eq!(sources, ["ok.hwpx", "깨진문서.hwp"], "{records:?}");

    let failed: Vec<&serde_json::Value> = records
        .iter()
        .filter(|r| r.get("error").is_some())
        .collect();
    assert_eq!(failed.len(), 1, "{records:?}");
    assert_eq!(failed[0]["mode"], "batch", "{}", failed[0]);
    assert_eq!(failed[0]["hasSignal"], false, "{}", failed[0]);
}

#[test]
fn layout_anomaly_batch_report_is_order_stable_and_recursive() {
    let dir = TempDir::new("order");
    dir.copy_as(CLEAN, "z-last.hwp");
    dir.copy_as(CLEAN, "a-first.hwp");
    dir.copy_as(OVERFLOW, "mid/nested.hwpx");

    let args = [
        "layout-anomaly",
        "--batch",
        dir.path().to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let records = ndjson(&args, &output);
    let sources: Vec<&str> = records
        .iter()
        .map(|r| r["source"].as_str().expect("source"))
        .collect();
    assert_eq!(
        sources,
        ["a-first.hwp", "mid/nested.hwpx", "z-last.hwp"],
        "{records:?}"
    );
}

#[test]
fn layout_anomaly_batch_strict_without_load_fail_exits_three() {
    let dir = TempDir::new("strict");
    dir.copy_as(OVERFLOW, "over.hwpx");
    let args = [
        "layout-anomaly",
        "--batch",
        dir.path().to_str().unwrap(),
        "--json",
        "--strict",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );
    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["hasSignal"], true, "{}", records[0]);
    assert!(records[0].get("error").is_none(), "{}", records[0]);
}

#[test]
fn layout_anomaly_batch_json_stdout_carries_no_human_summary() {
    let dir = TempDir::new("summary");
    dir.copy_as(CLEAN, "clean.hwp");
    let args = [
        "layout-anomaly",
        "--batch",
        dir.path().to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("layout-anomaly 요약"),
        "요약은 stderr 로 가야 합니다.\n{}",
        describe(&args, &output)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("layout-anomaly 요약"),
        "요약이 통째로 사라지면 사람이 배치 결과를 못 읽습니다.\n{}",
        describe(&args, &output)
    );
}

// ── M02-2: --types ──────────────────────────────────────────────────────

#[test]
fn layout_anomaly_types_filter_changes_overflow_set() {
    let src = sample(OVERFLOW);
    let all = [
        "layout-anomaly",
        src.to_str().unwrap(),
        "--json",
        "--types",
        "Table",
    ];
    let output = run(&all);
    assert_eq!(output.status.code(), Some(0), "{}", describe(&all, &output));
    let v = parse_stdout_json(&all, &output);
    assert_eq!(v["types"], serde_json::json!(["Table"]), "{v}");
    let table_count = v["overflowCount"].as_u64().unwrap();

    let none = [
        "layout-anomaly",
        src.to_str().unwrap(),
        "--json",
        "--types",
        "Image",
    ];
    let output = run(&none);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&none, &output)
    );
    let v = parse_stdout_json(&none, &output);
    assert!(
        v["overflowCount"].as_u64().unwrap() < table_count,
        "Image 필터는 표 overflow 를 빼야 한다: {v}"
    );
}

// ── 자기서술 드리프트 가드 ──────────────────────────────────────────────

#[test]
fn capabilities_declares_layout_anomaly_batch_and_types() {
    let entry = command_entry();
    assert_eq!(entry["json"], true, "{entry}");
    let flags: Vec<&str> = entry["flags"]
        .as_array()
        .expect("flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    for want in [
        "--json",
        "--batch",
        "--types",
        "-p",
        "--strict",
        "--overflow-tolerance",
        "--overlap-tolerance",
    ] {
        assert!(flags.contains(&want), "{want} 선언 누락: {entry}");
    }

    let src = sample(CLEAN);
    let args = ["layout-anomaly", src.to_str().unwrap(), "--json"];
    let v = parse_stdout_json(&args, &run(&args));
    for field in entry["recordFields"].as_array().expect("recordFields") {
        let name = field.as_str().expect("recordFields 항목");
        assert!(
            v.get(name).is_some(),
            "선언한 봉투 필드 {name} 이 실제 출력에 없습니다: {v}"
        );
    }
}

#[test]
fn layout_anomaly_declared_flags_are_actually_accepted() {
    let src = sample(CLEAN);
    let path = src.to_str().unwrap();
    let dir = TempDir::new("flags");
    dir.copy_as(CLEAN, "a.hwp");
    let folder = dir.path().to_str().unwrap().to_string();
    let cases: Vec<Vec<&str>> = vec![
        vec!["layout-anomaly", path, "--json"],
        vec!["layout-anomaly", path, "--strict", "--json"],
        vec!["layout-anomaly", path, "-p", "0", "--json"],
        vec![
            "layout-anomaly",
            path,
            "--overflow-tolerance",
            "2",
            "--json",
        ],
        vec!["layout-anomaly", path, "--overlap-tolerance", "3", "--json"],
        vec!["layout-anomaly", path, "--types", "Table,Image", "--json"],
        vec!["layout-anomaly", "--batch", &folder, "--json"],
    ];
    for args in cases {
        let output = run(&args);
        assert_ne!(
            output.status.code(),
            Some(2),
            "선언한 플래그를 사용법 오류로 거절하면 안 됩니다.\n{}",
            describe(&args, &output)
        );
    }
}

#[test]
fn help_mentions_batch_and_exit_contract() {
    let output = run(&["layout-anomaly", "--help"]);
    let joined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        joined.contains("layout-anomaly --batch"),
        "help 에 --batch 안내가 없습니다:\n{joined}"
    );
    assert!(
        joined.contains("--types"),
        "help 에 --types 안내가 없습니다:\n{joined}"
    );
    assert!(
        joined.contains("exit 3") || joined.contains("종료 코드는 0"),
        "help 가 exit 계약을 밝히지 않습니다:\n{joined}"
    );
}
