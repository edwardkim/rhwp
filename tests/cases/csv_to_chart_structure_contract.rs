//! [#5652] `csv-to-chart --structure` · `edit set-chart-data` 구조 편집 CLI 계약.
//!
//! 구조 편집(행·열 증감, 계열명·라벨 변경)은 **옵트인**이다 — 플래그 없는 CSV 는 B1 그대로
//! 치수 불일치를 거부한다(`tests/chart_csv_contract.rs` 가 그 계약을 쥔다). 여기서는 플래그가
//! 있을 때의 왕복, 종류별 가드의 CLI 전달(exit 2 + `invalid[]`), `edit set-chart-data --dry-run`
//! 이 코어 검증을 거치는 것, capabilities/`--mcp`/`--help` 3면 자기서술을 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BAR: &str = "samples/chart/세로막대형/묶은세로막대형.hwpx";
const PIE: &str = "samples/chart/원형/원형대원형.hwpx";

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

fn tmp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-chart-structure-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn json_of(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout JSON 파싱 실패: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn path_str(p: &Path) -> String {
    p.to_str().expect("경로").to_string()
}

fn reasons(env: &serde_json::Value) -> Vec<String> {
    env["invalid"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| v["reason"].as_str().unwrap_or("").to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn ops(env: &serde_json::Value) -> Vec<String> {
    env["changed"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v["op"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// 차트 1 을 CSV 로 뽑아 (문서 경로, CSV 경로, CSV 본문) 을 돌려준다.
fn seed(doc: &str, dir: &Path) -> (String, PathBuf, String) {
    let file = path_str(&sample(doc));
    let csv_path = dir.join("seed.csv");
    let out = run(&[
        "chart-to-csv",
        &file,
        "--chart",
        "1",
        "-o",
        &path_str(&csv_path),
    ]);
    assert_eq!(out.status.code(), Some(0), "기준선 CSV: {out:?}");
    let body = std::fs::read_to_string(&csv_path).expect("CSV 읽기");
    (file, csv_path, body)
}

/// CSV 레코드(CRLF) 목록.
fn records(csv: &str) -> Vec<String> {
    csv.split("\r\n")
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

fn join(records: &[String]) -> String {
    let mut s = records.join("\r\n");
    s.push_str("\r\n");
    s
}

fn csv_to_chart(file: &str, csv: &Path, dest: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "csv-to-chart",
        file,
        "--csv",
        csv.to_str().unwrap(),
        "--chart",
        "1",
        "-o",
        dest.to_str().unwrap(),
        "--json",
    ];
    args.extend_from_slice(extra);
    run(&args)
}

/// 플래그 없이 행을 더한 CSV 는 여전히 exit 2 — 메시지가 `structure` 를 안내한다.
#[test]
fn extra_row_without_structure_flag_is_refused_with_a_hint() {
    let dir = tmp_dir();
    let (file, csv_path, body) = seed(BAR, &dir);
    let mut rows = records(&body);
    rows.push("추가항목,45,44,43".to_string());
    std::fs::write(&csv_path, join(&rows)).unwrap();
    let dest = dir.join("no_flag.hwpx");
    let out = csv_to_chart(&file, &csv_path, &dest, &[]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let env = json_of(&out);
    assert!(
        reasons(&env).contains(&"valueCountMismatch".to_string()),
        "{env}"
    );
    let msg = env["invalid"][0]["message"].as_str().unwrap_or_default();
    assert!(msg.contains("structure"), "{msg}");
    assert!(!dest.exists(), "거부했는데 파일을 썼다");
}

/// `--structure` 로 행을 더하면 ①② 에 쓰이고, 다시 뽑은 CSV 에 그 행이 있다.
#[test]
fn structure_flag_appends_a_row_and_round_trips_through_chart_to_csv() {
    let dir = tmp_dir();
    let (file, csv_path, body) = seed(BAR, &dir);
    let mut rows = records(&body);
    assert_eq!(rows.len(), 5, "머리 1 + 데이터 4: {rows:?}");
    rows.push("추가항목,45,44,43".to_string());
    std::fs::write(&csv_path, join(&rows)).unwrap();
    let dest = dir.join("row_add.hwpx");
    let out = csv_to_chart(&file, &csv_path, &dest, &["--structure"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let env = json_of(&out);
    assert_eq!(env["wrote"].as_array().expect("wrote").len(), 2, "{env}");
    assert!(ops(&env).iter().any(|op| op == "appendPoints"), "{env}");
    assert!(dest.exists());

    let back = dir.join("back.csv");
    let out = run(&[
        "chart-to-csv",
        &path_str(&dest),
        "--chart",
        "1",
        "-o",
        &path_str(&back),
    ]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let rows = records(&std::fs::read_to_string(&back).unwrap());
    assert_eq!(rows.len(), 6);
    assert_eq!(rows[5], "추가항목,45,44,43");
}

/// 열(계열) 추가·삭제와 계열명 변경도 CSV 한 장으로 된다.
#[test]
fn structure_flag_adds_and_removes_series_and_renames() {
    let dir = tmp_dir();
    let (file, csv_path, body) = seed(BAR, &dir);
    let base = records(&body);

    // 열 추가
    let rows: Vec<String> = base
        .iter()
        .enumerate()
        .map(|(i, r)| {
            if i == 0 {
                format!("{r},추가계열")
            } else {
                format!("{r},6")
            }
        })
        .collect();
    std::fs::write(&csv_path, join(&rows)).unwrap();
    let dest = dir.join("col_add.hwpx");
    let out = csv_to_chart(&file, &csv_path, &dest, &["--structure"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(ops(&json_of(&out)).iter().any(|op| op == "appendSeries"));
    let back = run(&["chart-to-csv", &path_str(&dest), "--chart", "1"]);
    let back = String::from_utf8_lossy(&back.stdout).to_string();
    assert!(records(&back)[0].ends_with(",추가계열"), "{back}");
    assert!(records(&back)[1].ends_with(",6"), "{back}");

    // 열 삭제(마지막) + 계열명 변경
    let rows: Vec<String> = base
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mut fields: Vec<&str> = r.split(',').collect();
            fields.pop();
            if i == 0 {
                fields[1] = "이름바뀐계열";
            }
            fields.join(",")
        })
        .collect();
    std::fs::write(&csv_path, join(&rows)).unwrap();
    let dest = dir.join("col_del.hwpx");
    let out = csv_to_chart(&file, &csv_path, &dest, &["--structure"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let env = json_of(&out);
    let ops = ops(&env);
    assert!(ops.iter().any(|op| op == "truncateSeries"), "{env}");
    assert!(ops.iter().any(|op| op == "renameSeries"), "{env}");
    let back = run(&["chart-to-csv", &path_str(&dest), "--chart", "1"]);
    let back = String::from_utf8_lossy(&back.stdout).to_string();
    assert_eq!(records(&back)[0], ",이름바뀐계열,계열 2", "{back}");
}

/// 원형에 계열을 더하는 CSV 는 `--structure` 여도 가드가 막는다 — exit 2 + `pieSeriesCountFixed`.
#[test]
fn pie_series_add_via_csv_exits_two_with_pie_guard() {
    let dir = tmp_dir();
    let (file, csv_path, body) = seed(PIE, &dir);
    let rows: Vec<String> = records(&body)
        .iter()
        .enumerate()
        .map(|(i, r)| {
            if i == 0 {
                format!("{r},추가계열")
            } else {
                format!("{r},1")
            }
        })
        .collect();
    std::fs::write(&csv_path, join(&rows)).unwrap();
    let dest = dir.join("pie.hwpx");
    let out = csv_to_chart(&file, &csv_path, &dest, &["--structure"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let env = json_of(&out);
    assert!(
        reasons(&env).contains(&"pieSeriesCountFixed".to_string()),
        "{env}"
    );
    assert!(env["wrote"].as_array().expect("wrote").is_empty());
    assert!(!dest.exists(), "거부했는데 파일을 썼다");
}

/// `edit set-chart-data --dry-run` 은 코어 검증을 거친다 — 가드 거부는 exit 2, 통과는 op 보고.
#[test]
fn set_chart_data_dry_run_goes_through_core_validation() {
    let dir = tmp_dir();
    let pie = path_str(&sample(PIE));
    let data = serde_json::json!({
        "structure": true,
        "series": [
            {"name": "계열 1", "values": ["1", "2", "3", "4"]},
            {"name": "추가계열", "values": ["1", "2", "3", "4"]}
        ],
    })
    .to_string();
    let dest = dir.join("dry_pie.hwpx");
    let out = run(&[
        "edit",
        "set-chart-data",
        &pie,
        "--chart",
        "1",
        "--data",
        &data,
        "-o",
        &path_str(&dest),
        "--dry-run",
        "--json",
    ]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "dry-run 이 가드를 건너뛰었다: {out:?}"
    );
    let env = json_of(&out);
    assert!(
        reasons(&env).contains(&"pieSeriesCountFixed".to_string()),
        "{env}"
    );
    assert!(!dest.exists());

    // 통과하는 구조 편집의 dry-run — 파일은 없고 op 는 보인다.
    let bar = path_str(&sample(BAR));
    let read = run(&["chart-to-csv", &bar, "--chart", "1", "--json"]);
    assert_eq!(read.status.code(), Some(0), "{read:?}");
    let data = serde_json::json!({
        "structure": true,
        "labels": ["항목 1", "항목 2", "항목 3", "항목 4", "추가항목"],
        "series": [
            {"name": "계열 1", "values": ["4.3", "2.5", "3.5", "4.5", "45"]},
            {"name": "계열 2", "values": ["2.4", "4.4", "1.8", "2.8", "44"]},
            {"name": "계열 3", "values": ["2", "2", "3", "5", "43"]}
        ],
    })
    .to_string();
    let dest = dir.join("dry_bar.hwpx");
    let out = run(&[
        "edit",
        "set-chart-data",
        &bar,
        "--chart",
        "1",
        "--data",
        &data,
        "-o",
        &path_str(&dest),
        "--dry-run",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let env = json_of(&out);
    assert_eq!(env["dryRun"], true, "{env}");
    assert!(env["changedCount"].as_u64().unwrap_or(0) > 0, "{env}");
    assert!(ops(&env).iter().any(|op| op == "appendPoints"), "{env}");
    assert!(!dest.exists(), "dry-run 인데 파일을 썼다");
}

/// capabilities · `--mcp` · `--help` 3면이 `structure` 를 함께 선언한다.
#[test]
fn capabilities_mcp_and_help_declare_structure() {
    let cap = json_of(&run(&["capabilities"]));
    let cmd = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "csv-to-chart")
        .expect("csv-to-chart");
    let flags: Vec<&str> = cmd["flags"]
        .as_array()
        .expect("flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(
        flags.contains(&"--structure"),
        "capabilities flags: {flags:?}"
    );

    let mcp = json_of(&run(&["capabilities", "--mcp"]));
    let tools = mcp["tools"].as_array().expect("tools");
    let csv_tool = tools
        .iter()
        .find(|t| t["name"] == "hwp_csv_to_chart")
        .expect("hwp_csv_to_chart");
    assert_eq!(
        csv_tool["inputSchema"]["properties"]["structure"]["type"], "boolean",
        "{csv_tool}"
    );
    let set_tool = tools
        .iter()
        .find(|t| t["name"] == "hwp_set_chart_data")
        .expect("hwp_set_chart_data");
    let data_desc = set_tool["inputSchema"]["properties"]["data"]["description"]
        .as_str()
        .unwrap_or_default();
    assert!(data_desc.contains("structure"), "{set_tool}");
    let fields: Vec<&str> = set_tool["outputFields"]
        .as_array()
        .expect("outputFields")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(fields.contains(&"invalid"), "{fields:?}");

    let help = run(&["--help"]);
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(
        help_text.contains("--structure"),
        "--help 에 --structure 가 없다"
    );
}
