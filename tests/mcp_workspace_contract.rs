//! [#4357 W1] 워크스페이스 계약 — 에이전트 전용 문서 런타임 v1.
//!
//! 고정하는 것: ① `--workspace` 기동 시 결정론 인벤토리(w1.. 정렬 id), ② id 로
//! 열기 → 안정 ID 트리(p0../t0..), ③ 변이 저널(변이 도구 후 SHA-256 전/후 자동
//! 기록), ④ 워크스페이스 없이 기동하면 hwp_ws_* 가 안내와 함께 명시적으로
//! 실패한다(조용한 빈 결과 금지).

#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};

struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl Server {
    fn spawn(extra_args: &[&str]) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .arg("mcp-serve")
            .args(extra_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("mcp-serve 기동");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        let mut server = Server {
            child,
            stdin,
            stdout,
            next_id: 1,
        };
        let init = server.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "workspace-contract", "version": "0" }
            }),
        );
        assert!(init.get("result").is_some(), "initialize 실패: {init}");
        server
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;
        let line = serde_json::json!({
            "jsonrpc": "2.0", "id": id, "method": method, "params": params
        });
        writeln!(self.stdin, "{line}").expect("요청 쓰기");
        loop {
            let mut buf = String::new();
            let n = self.stdout.read_line(&mut buf).expect("응답 읽기");
            assert!(n > 0, "서버가 응답 없이 종료됨 (method={method})");
            let value: serde_json::Value = match serde_json::from_str(buf.trim()) {
                Ok(v) => v,
                Err(_) => continue, // 알림 등 비-JSON 줄은 계약 대상이 아니다.
            };
            if value.get("id").and_then(|v| v.as_u64()) == Some(id) {
                return value;
            }
        }
    }

    /// tools/call 을 보내고 content[0].text 를 JSON 으로 판다.
    fn call(&mut self, name: &str, arguments: serde_json::Value) -> (bool, serde_json::Value) {
        let resp = self.request(
            "tools/call",
            serde_json::json!({ "name": name, "arguments": arguments }),
        );
        let result = &resp["result"];
        let is_error = result["isError"].as_bool().unwrap_or(false);
        let text = result["content"][0]["text"].as_str().unwrap_or_default();
        let body = serde_json::from_str(text).unwrap_or(serde_json::json!({ "raw": text }));
        (is_error, body)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn workspace_roundtrip_list_open_tree_journal() {
    let mut server = Server::spawn(&["--workspace", "samples/basic"]);

    // ① 인벤토리 — 결정론 id.
    let (err, list) = server.call("hwp_ws_list", serde_json::json!({}));
    assert!(!err, "hwp_ws_list 실패: {list}");
    let count = list["count"].as_u64().expect("count");
    assert!(count > 0, "samples/basic 에 문서가 있어야 한다: {list}");
    assert_eq!(list["truncated"], false);
    assert_eq!(list["entries"][0]["id"], "w1", "정렬 1번 id 는 w1: {list}");

    // ② id 로 열기 → 핸들.
    let (err, opened) = server.call("hwp_ws_open", serde_json::json!({ "id": "w1" }));
    assert!(!err, "hwp_ws_open 실패: {opened}");
    let doc_id = opened["docId"].as_str().expect("docId").to_string();

    // ③ 안정 ID 트리.
    let (err, tree) = server.call("hwp_doc_tree", serde_json::json!({ "docId": doc_id }));
    assert!(!err, "hwp_doc_tree 실패: {tree}");
    let page_count = tree["pageCount"].as_u64().expect("pageCount");
    assert!(page_count >= 1);
    assert_eq!(tree["nodes"]["pages"][0], "p0");
    assert!(tree["nodes"]["tables"].is_array());

    // ④ 변이 저널 — save 1회가 자동 기록되고 digest 는 64자리 hex, IR 무변경.
    let out = std::env::temp_dir().join("ws_contract_save.hwp");
    let (err, _saved) = server.call(
        "hwp_doc_save",
        serde_json::json!({ "docId": doc_id, "output": out.to_string_lossy() }),
    );
    assert!(!err, "hwp_doc_save 실패");
    let (err, journal) = server.call("hwp_ws_journal", serde_json::json!({}));
    assert!(!err);
    assert_eq!(journal["count"], 1, "저널 1건: {journal}");
    let entry = &journal["entries"][0];
    assert_eq!(entry["tool"], "hwp_doc_save");
    assert_eq!(entry["docId"], serde_json::json!(doc_id));
    assert_eq!(entry["isError"], false);
    let before = entry["digestBefore"].as_str().expect("digestBefore");
    assert_eq!(before.len(), 64, "sha256 hex 64자리: {before}");
    assert_eq!(
        entry["digestBefore"], entry["digestAfter"],
        "save 는 IR 을 바꾸지 않는다 — changed:false 의 근거"
    );
    assert_eq!(entry["changed"], false);
    let _ = std::fs::remove_file(out);
}

#[test]
fn workspace_tools_fail_loudly_without_workspace() {
    let mut server = Server::spawn(&[]);
    let (err, body) = server.call("hwp_ws_list", serde_json::json!({}));
    assert!(err, "워크스페이스 없이 hwp_ws_list 는 명시적 오류여야 한다");
    assert!(
        body["raw"]
            .as_str()
            .unwrap_or_default()
            .contains("--workspace"),
        "안내에 기동 방법이 있어야 한다: {body}"
    );
    let (err, body) = server.call("hwp_ws_open", serde_json::json!({ "id": "w1" }));
    assert!(err, "hwp_ws_open 도 동일: {body}");
}

#[test]
fn workspace_tools_are_listed() {
    let mut server = Server::spawn(&["--workspace", "samples/basic"]);
    let resp = server.request("tools/list", serde_json::json!({}));
    let names: Vec<&str> = resp["result"]["tools"]
        .as_array()
        .expect("tools 배열")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    for expected in [
        "hwp_ws_list",
        "hwp_ws_open",
        "hwp_doc_tree",
        "hwp_ws_journal",
    ] {
        assert!(names.contains(&expected), "{expected} 미등재: {names:?}");
    }
}
