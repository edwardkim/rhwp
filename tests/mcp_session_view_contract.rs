//! [#3609] `mcp-serve` 세션 조회·렌더 완결 — hwp_doc_info / hwp_doc_fields /
//! hwp_doc_tables / hwp_doc_render_page.
//!
//! 편집 루프의 조회 3종과 눈검증 렌더가 무상태 전용이라 세션 이점이 루프 중간마다
//! 새어나갔다. 계약의 핵심: ① 봉투는 무상태 경로와 동형(같은 helper 재사용)
//! ② fill 직후 hwp_doc_fields 에 값이 **재파싱 없이** 반영 ③ render 산출물 실존.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// 누름틀 11개 + 표를 가진 HWP5 서식.
const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-mcpview-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
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
            .expect("rhwp mcp-serve 실행 실패");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        let mut s = Server {
            child,
            stdin,
            stdout,
            next_id: 1,
        };
        let r = s.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "session-view-test", "version": "0"}
            }),
        );
        assert!(r["result"]["serverInfo"]["name"].is_string(), "{r}");
        s
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;
        let msg =
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        writeln!(self.stdin, "{msg}").expect("요청 쓰기 실패");
        self.stdin.flush().expect("flush");
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.stdout.read_line(&mut line).expect("응답 읽기 실패");
            assert!(n > 0, "서버가 응답 없이 종료했습니다 (method={method})");
            if line.trim().is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line.trim())
                .unwrap_or_else(|e| panic!("stdout 이 JSON-RPC 가 아닙니다 ({e}): {line}"));
            if v.get("id").and_then(|i| i.as_i64()) == Some(id) {
                return v;
            }
        }
    }

    fn call(&mut self, name: &str, args: serde_json::Value) -> (bool, serde_json::Value) {
        let r = self.request(
            "tools/call",
            serde_json::json!({"name": name, "arguments": args}),
        );
        let result = &r["result"];
        let is_error = result["isError"].as_bool().unwrap_or(false);
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let v = serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text));
        (is_error, v)
    }

    fn open(&mut self, path: &Path) -> String {
        let (err, v) = self.call(
            "hwp_open",
            serde_json::json!({"path": path.to_str().unwrap()}),
        );
        assert!(!err, "hwp_open 실패: {v}");
        v["docId"].as_str().expect("docId").to_string()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn doc_info_matches_stateless_envelope_shape() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, v) = s.call("hwp_doc_info", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{v}");
    // 무상태 info --json 과 같은 어휘 (info_json_value 재사용).
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["format"], "hwp5", "{v}");
    assert!(v["sizeBytes"].as_u64().unwrap_or(0) > 0, "{v}");
    assert!(v["pageCount"].as_u64().unwrap_or(0) >= 1, "{v}");
    assert!(v["fonts"].is_array(), "{v}");
}

#[test]
fn doc_fields_reflects_session_fill_without_reparse() {
    // 본론: fill 직후 같은 핸들의 fields 조회에 값이 보여야 한다 — 재파싱 없이.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    let (err, before) = s.call("hwp_doc_fields", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{before}");
    assert!(before["fieldCount"].as_u64().unwrap_or(0) >= 1, "{before}");

    let (err, f) = s.call(
        "hwp_doc_fill_fields",
        serde_json::json!({"docId": doc_id, "data": {"회사명": "뷰검증 주식회사"}}),
    );
    assert!(!err, "{f}");

    let (err, after) = s.call("hwp_doc_fields", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{after}");
    let val = after["fields"]
        .as_array()
        .expect("fields")
        .iter()
        .find(|x| x["name"] == "회사명")
        .and_then(|x| x["value"].as_str())
        .unwrap_or("");
    assert_eq!(
        val, "뷰검증 주식회사",
        "fill 이 세션 조회에 반영돼야 합니다: {after}"
    );
}

#[test]
fn doc_tables_returns_grid_envelope() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, v) = s.call("hwp_doc_tables", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{v}");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["tableCount"].is_u64(), "{v}");
    assert!(v["tables"].is_array(), "{v}");
}

#[test]
fn doc_render_page_writes_svg_manifest() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let out = temp_path("render", "svg");
    let (err, v) = s.call(
        "hwp_doc_render_page",
        serde_json::json!({"docId": doc_id, "page": 0, "output": out.to_str().unwrap()}),
    );
    assert!(!err, "{v}");
    assert_eq!(v["page"].as_u64(), Some(0), "{v}");
    let bytes = v["bytes"].as_u64().expect("bytes");
    assert!(bytes > 0, "{v}");
    let meta = std::fs::metadata(&out).expect("SVG 산출물이 존재해야 합니다");
    assert_eq!(meta.len(), bytes, "보고 bytes ≠ 실제 크기");
    let head = std::fs::read_to_string(&out).unwrap_or_default();
    assert!(head.contains("<svg"), "SVG 형식이어야 합니다");

    // 페이지 범위 초과는 isError.
    let (err, e) = s.call(
        "hwp_doc_render_page",
        serde_json::json!({"docId": doc_id, "page": 9999, "output": out.to_str().unwrap()}),
    );
    assert!(err, "범위 초과는 isError: {e}");

    // JSON u64가 u32으로 wrap되어 0쪽을 렌더하면 안 된다.
    let (err, e) = s.call(
        "hwp_doc_render_page",
        serde_json::json!({"docId": doc_id, "page": 4_294_967_296u64, "output": out.to_str().unwrap()}),
    );
    assert!(err, "u32 초과 page는 isError 여야 합니다: {e}");

    let _ = std::fs::remove_file(&out);
}

#[test]
fn view_tools_reject_closed_handle_and_are_listed() {
    let mut s = Server::started();
    for (name, args) in [
        ("hwp_doc_info", serde_json::json!({"docId": "doc-999"})),
        ("hwp_doc_fields", serde_json::json!({"docId": "doc-999"})),
        ("hwp_doc_tables", serde_json::json!({"docId": "doc-999"})),
        (
            "hwp_doc_render_page",
            serde_json::json!({"docId": "doc-999", "page": 0, "output": "x.svg"}),
        ),
    ] {
        let (err, v) = s.call(name, args);
        assert!(err, "{name} 는 닫힌 핸들에 isError 여야 합니다: {v}");
    }
    let r = s.request("tools/list", serde_json::json!({}));
    let names: Vec<String> = r["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    for t in [
        "hwp_doc_info",
        "hwp_doc_fields",
        "hwp_doc_tables",
        "hwp_doc_render_page",
    ] {
        assert!(names.contains(&t.to_string()), "{t} 누락: {names:?}");
    }
}
