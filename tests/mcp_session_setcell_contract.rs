//! [#3603] `hwp_doc_set_cell` — 핸들의 표 격자에 값을 기록한다(누적, save 가 기록 지점).
//! 좌표 해석은 CLI 와 공유하는 `resolve_table_cell`(추출 helper) — 병합 앵커 안내와
//! overflow 보고가 무상태 hwp_set_cell 과 동형임을 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const SAMPLE: &str = "samples/table-001.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-setcell-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Server {
    fn started() -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .arg("mcp-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("mcp-serve 실행 실패");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut s = Server {
            child,
            stdin,
            stdout,
            next_id: 1,
        };
        s.request("initialize", serde_json::json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}));
        s
    }
    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;
        writeln!(
            self.stdin,
            "{}",
            serde_json::json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
        )
        .unwrap();
        self.stdin.flush().unwrap();
        let mut line = String::new();
        loop {
            line.clear();
            assert!(
                self.stdout.read_line(&mut line).unwrap() > 0,
                "서버 조기 종료"
            );
            if line.trim().is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
            if v.get("id").and_then(|i| i.as_i64()) == Some(id) {
                return v;
            }
        }
    }
    fn call(&mut self, name: &str, args: serde_json::Value) -> (bool, serde_json::Value) {
        let r = self.request(
            "tools/call",
            serde_json::json!({"name":name,"arguments":args}),
        );
        let res = &r["result"];
        let err = res["isError"].as_bool().unwrap_or(false);
        let text = res["content"][0]["text"].as_str().unwrap_or("").to_string();
        (
            err,
            serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text)),
        )
    }
    fn open(&mut self, p: &Path) -> String {
        let (e, v) = self.call("hwp_open", serde_json::json!({"path": p.to_str().unwrap()}));
        assert!(!e, "{v}");
        v["docId"].as_str().unwrap().to_string()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn set_cell_accumulates_and_survives_save() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let d = s.open(&src);

    // 첫 표의 실존 앵커 좌표를 동적으로 고른다 — 무상태 export-tables 로
    // (#3612 미머지 상태와 독립).
    let et = run_cli(&["export-tables", src.to_str().unwrap(), "--json"]);
    let t: serde_json::Value = serde_json::from_slice(&et.stdout).expect("export-tables");
    let cell = &t["tables"][0]["cells"][0];
    let (row, col) = (cell["row"].as_u64().unwrap(), cell["col"].as_u64().unwrap());

    let (e, v) = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({
            "docId": d, "table": 0, "row": row, "col": col, "text": "세션기록 42"
        }),
    );
    assert!(!e, "{v}");
    assert_eq!(v["newText"], "세션기록 42", "{v}");
    assert!(v["overflow"].is_array(), "{v}");

    let out = temp_path("save");
    let (e, sv) = s.call(
        "hwp_doc_save",
        serde_json::json!({"docId": d, "output": out.to_str().unwrap()}),
    );
    assert!(!e, "{sv}");

    // 서버 밖 재독 대조 — export-tables 격자에서 값이 보여야 한다.
    let rr = run_cli(&["export-tables", out.to_str().unwrap(), "--json"]);
    let rv: serde_json::Value = serde_json::from_slice(&rr.stdout).expect("export-tables");
    let found = rv["tables"][0]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["text"].as_str().unwrap_or("").contains("세션기록 42"));
    assert!(found, "저장본 격자에 값이 있어야 합니다: {rv}");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn covered_cell_reports_anchor_via_is_error() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let d = s.open(&src);
    // 병합 스팬이 있는 셀을 찾아 그 덮인 좌표를 찌른다. 없으면 건너뜀.
    let et = run_cli(&["export-tables", src.to_str().unwrap(), "--json"]);
    let t: serde_json::Value = serde_json::from_slice(&et.stdout).expect("export-tables");
    let cells = t["tables"][0]["cells"].as_array().unwrap().clone();
    let Some(m) = cells
        .iter()
        .find(|c| c["colSpan"].as_u64().unwrap_or(1) >= 2)
    else {
        eprintln!("병합 셀 없음 — 건너뜀");
        return;
    };
    let (row, col) = (m["row"].as_u64().unwrap(), m["col"].as_u64().unwrap() + 1);
    let (e, v) = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({
            "docId": d, "table": 0, "row": row, "col": col, "text": "x"
        }),
    );
    assert!(e, "덮인 칸은 isError 여야 합니다: {v}");
    let msg = v.as_str().unwrap_or("");
    assert!(msg.contains("앵커"), "앵커 안내가 있어야 합니다: {msg}");
}

#[test]
fn set_cell_rejects_closed_handle_and_is_listed() {
    let mut s = Server::started();
    let (e, v) = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({
            "docId": "doc-999", "table": 0, "row": 0, "col": 0, "text": "x"
        }),
    );
    assert!(e, "{v}");
    let r = s.request("tools/list", serde_json::json!({}));
    let names: Vec<String> = r["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    assert!(names.contains(&"hwp_doc_set_cell".to_string()), "{names:?}");
}
