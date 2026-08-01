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

#[test]
fn set_cell_rejects_numeric_indices_before_narrowing() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let d = s.open(&src);
    let (err, value) = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({
            "docId": d, "table": 0, "row": 65_536u64, "col": 0, "text": "wrap 금지"
        }),
    );
    assert!(err, "u16 초과 row는 isError 여야 합니다: {value}");
    assert!(
        value.as_str().unwrap_or_default().contains("65535"),
        "범위 안내가 있어야 합니다: {value}"
    );
}

/// [#3603] 세션 도구는 CLI 와 **같은 문장으로** 제어문자를 거부한다.
///
/// 종전에는 세션 경로만 통과시켜, 한 셀 문단 안에 raw 개행이 들어간 채 IR 에 누적됐다
/// (저장본을 다시 읽어야 겨우 드러난다). 거부 문장은 테스트가 따로 적어 두지 않고 CLI 를
/// 실제로 돌려 얻는다 — 문자열을 두 벌 관리하면 그 자체가 다음 어긋남의 씨앗이다.
#[test]
fn set_cell_rejects_control_chars_with_cli_message() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let cli_out = temp_path("ctrlchar-cli");
    let cli = run_cli(&[
        "edit",
        "set-cell",
        src.to_str().unwrap(),
        "--table",
        "0",
        "--row",
        "0",
        "--col",
        "0",
        "--text",
        "가\n나",
        "-o",
        cli_out.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        cli.status.code(),
        Some(2),
        "CLI 는 EXIT_USAGE(2) 로 끊습니다"
    );
    assert!(!cli_out.exists(), "거부된 편집은 파일을 만들지 않습니다");
    let cli_message = String::from_utf8_lossy(&cli.stderr).trim().to_string();
    assert!(!cli_message.is_empty(), "CLI 안내문이 비어 있습니다");

    let mut s = Server::started();
    let d = s.open(&src);
    let (err, before) = s.call("hwp_doc_tables", serde_json::json!({"docId": d}));
    assert!(!err, "{before}");

    for bad in ["가\n나", "가\t나", "가\r나", "줄1\r\n줄2"] {
        let (err, v) = s.call(
            "hwp_doc_set_cell",
            serde_json::json!({"docId": d, "table": 0, "row": 0, "col": 0, "text": bad}),
        );
        assert!(err, "{bad:?} 는 isError 여야 합니다: {v}");
        assert_eq!(
            v.as_str().unwrap_or_default(),
            cli_message,
            "{bad:?}: CLI 와 같은 문장으로 거부해야 합니다"
        );
    }

    // 거부는 핸들을 건드리기 전에 끝난다 — 격자가 한 글자도 달라지면 안 된다.
    let (err, after) = s.call("hwp_doc_tables", serde_json::json!({"docId": d}));
    assert!(!err, "{after}");
    assert_eq!(before, after, "거부된 편집이 핸들 IR 을 바꿨습니다");
}

/// 봉투 키가 무상태 `edit set-cell --json` 과 같다.
///
/// overflow 에서 `text` 가 빠지면 여러 칸을 연달아 채운 에이전트가 '어느 값이 넘쳤는지'
/// 를 되짚을 수 없고, `keepStyle` 이 빠지면 검정 정규화가 걸렸는지 봉투만으로 못 읽는다.
#[test]
fn set_cell_envelope_carries_keep_style_and_overflow_text() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let d = s.open(&src);

    let (err, v) = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({
            "docId": d, "table": 0, "row": 0, "col": 0, "text": "봉투대조", "keepStyle": true
        }),
    );
    assert!(!err, "{v}");
    assert_eq!(
        v["keepStyle"],
        serde_json::json!(true),
        "요청한 keepStyle 이 봉투에 그대로 실려야 합니다: {v}"
    );

    // 기본값(미지정)도 봉투에 실린다 — 무엇이 적용됐는지 늘 알 수 있어야 한다.
    let (err, d2) = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({"docId": d, "table": 0, "row": 0, "col": 0, "text": "기본"}),
    );
    assert!(!err, "{d2}");
    assert_eq!(v["keepStyle"].is_boolean(), true, "{v}");
    assert_eq!(d2["keepStyle"], serde_json::json!(false), "{d2}");

    // 넘치는 값을 넣어 overflow 를 강제하고 CLI 와 키 집합을 대조한다.
    let long = "가나다라마바사아자차카타파하".repeat(4);
    let cli = run_cli(&[
        "edit",
        "set-cell",
        src.to_str().unwrap(),
        "--table",
        "0",
        "--row",
        "0",
        "--col",
        "0",
        "--text",
        &long,
        "--dry-run",
        "--json",
    ]);
    assert_eq!(
        cli.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let cv: serde_json::Value = serde_json::from_slice(&cli.stdout).expect("edit set-cell --json");
    let cli_overflow = cv["overflow"].as_array().expect("overflow").clone();
    if cli_overflow.is_empty() {
        eprintln!("이 샘플에서는 넘침이 나지 않음 — 키 대조 건너뜀");
        return;
    }

    let (err, ov) = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({"docId": d, "table": 0, "row": 0, "col": 0, "text": long}),
    );
    assert!(!err, "{ov}");
    let sess_overflow = ov["overflow"].as_array().expect("overflow");
    assert_eq!(sess_overflow.len(), cli_overflow.len(), "{ov}");
    let cli_keys: Vec<&String> = cli_overflow[0].as_object().expect("obj").keys().collect();
    let sess_keys: Vec<&String> = sess_overflow[0].as_object().expect("obj").keys().collect();
    assert_eq!(
        sess_keys, cli_keys,
        "overflow 항목 키 집합이 CLI 와 같아야 합니다: {ov}"
    );
    assert_eq!(sess_overflow[0]["text"], serde_json::json!(long), "{ov}");
}
