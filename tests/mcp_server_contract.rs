//! [#3140] `mcp-serve` — rhwp 를 실제 MCP 서버로 노출하는 stdio JSON-RPC 계약.
//!
//! `capabilities --mcp`(#3263)는 도구 **선언**만 냈다 — 실행하려면 외부 호스트가
//! 매니페스트를 해석해 CLI 를 fork 해야 했다. 본 명령은 그 마지막 층을 채운다:
//! MCP stdio 전송(줄 단위 JSON-RPC 2.0)로 initialize → tools/list → tools/call 을
//! 직접 받고, 선언과 실행이 한 프로세스에서 만난다.
//!
//! 세션(#3140 의 "상태 유지" 공백): `hwp_open` 이 문서를 파싱해 핸들을 돌려주고,
//! `hwp_doc_text` 가 재파싱 없이 핸들에서 읽으며, `hwp_close` 가 해제한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// 살아있는 mcp-serve 프로세스와 그 stdio 파이프.
struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Server {
    fn start() -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .arg("mcp-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("rhwp mcp-serve 실행 실패");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        Server {
            child,
            stdin,
            stdout,
            next_id: 1,
        }
    }

    /// 요청 1건을 보내고 같은 id 의 응답 1줄을 기다린다.
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
                .unwrap_or_else(|e| panic!("stdout 이 순수 JSON-RPC 가 아닙니다 ({e}): {line}"));
            // 서버발 알림은 건너뛰고 내 id 의 응답만 취한다.
            if v.get("id").and_then(|i| i.as_i64()) == Some(id) {
                return v;
            }
        }
    }

    fn notify(&mut self, method: &str) {
        let msg = serde_json::json!({"jsonrpc": "2.0", "method": method});
        writeln!(self.stdin, "{msg}").expect("알림 쓰기 실패");
        self.stdin.flush().expect("flush");
    }

    /// initialize 핸드셰이크까지 마친 서버를 돌려준다.
    fn started() -> Server {
        let mut s = Server::start();
        let r = s.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "contract-test", "version": "0"}
            }),
        );
        assert!(
            r["result"]["serverInfo"]["name"].is_string(),
            "initialize 응답에 serverInfo 가 없습니다: {r}"
        );
        assert!(
            r["result"]["capabilities"]["tools"].is_object(),
            "tools capability 선언이 없습니다: {r}"
        );
        s.notify("notifications/initialized");
        s
    }

    /// tools/call 을 보내고 content[0].text 를 JSON 으로 파싱해 돌려준다.
    fn call_tool(&mut self, name: &str, args: serde_json::Value) -> serde_json::Value {
        let r = self.request(
            "tools/call",
            serde_json::json!({"name": name, "arguments": args}),
        );
        let result = &r["result"];
        assert_eq!(
            result["isError"], false,
            "{name} 호출이 isError 를 보고했습니다: {r}"
        );
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("{name} 응답에 content[0].text 가 없습니다: {r}"));
        serde_json::from_str(text)
            .unwrap_or_else(|e| panic!("{name} 의 text 가 JSON 이 아닙니다 ({e}): {text}"))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn initialize_handshake_and_ping() {
    let mut s = Server::started();
    let r = s.request("ping", serde_json::json!({}));
    assert!(
        r["result"].is_object(),
        "ping 은 빈 result 를 돌려준다: {r}"
    );
}

#[test]
fn tools_list_matches_capabilities_manifest() {
    // 드리프트 가드: 서버가 노출하는 도구는 capabilities --mcp 선언과 같은 목록이어야
    // 한다(단일 출처). 세션 도구 3종(open/doc_text/close)은 서버 전용으로 추가된다.
    let cap = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp"])
        .output()
        .expect("capabilities 실행 실패");
    let manifest: serde_json::Value =
        serde_json::from_slice(&cap.stdout).expect("capabilities --mcp JSON");
    let declared: Vec<String> = manifest["tools"]
        .as_array()
        .expect("tools 배열")
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    let mut s = Server::started();
    let r = s.request("tools/list", serde_json::json!({}));
    let served: Vec<String> = r["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("tools/list 응답에 tools 배열이 없습니다: {r}"))
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    for name in &declared {
        assert!(
            served.contains(name),
            "capabilities 선언 도구 {name} 이 서버 tools/list 에 없습니다: {served:?}"
        );
    }
    for extra in ["hwp_open", "hwp_doc_text", "hwp_close"] {
        assert!(
            served.contains(&extra.to_string()),
            "세션 도구 {extra} 가 없습니다: {served:?}"
        );
    }
    // MCP 필수 필드.
    for t in r["result"]["tools"].as_array().unwrap() {
        assert!(t["description"].is_string(), "{t}");
        assert!(t["inputSchema"].is_object(), "{t}");
    }
}

#[test]
fn tools_call_stateless_info_works() {
    let p = sample(SAMPLE);
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let v = s.call_tool("hwp_info", serde_json::json!({"path": p.to_str().unwrap()}));
    assert!(
        v["pageCount"].as_u64().unwrap_or(0) >= 1,
        "hwp_info 가 페이지 수를 돌려줘야 합니다: {v}"
    );
}

#[test]
fn session_open_read_close_without_reparse() {
    let p = sample(SAMPLE);
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();

    let opened = s.call_tool("hwp_open", serde_json::json!({"path": p.to_str().unwrap()}));
    let doc_id = opened["docId"]
        .as_str()
        .unwrap_or_else(|| panic!("hwp_open 이 docId 를 돌려줘야 합니다: {opened}"))
        .to_string();
    assert!(opened["pageCount"].as_u64().unwrap_or(0) >= 1, "{opened}");

    // 같은 핸들로 두 번 읽는다 — 프로세스가 살아있으므로 재파싱이 없어야 한다.
    let t1 = s.call_tool("hwp_doc_text", serde_json::json!({"docId": doc_id}));
    let t2 = s.call_tool(
        "hwp_doc_text",
        serde_json::json!({"docId": doc_id, "page": 0}),
    );
    assert!(t1["pages"].is_array(), "{t1}");
    assert!(
        t2["pages"].as_array().map(|a| a.len()) == Some(1),
        "page 지정 시 1페이지만: {t2}"
    );

    let closed = s.call_tool("hwp_close", serde_json::json!({"docId": doc_id}));
    assert_eq!(closed["closed"], true, "{closed}");

    // 닫힌 핸들 사용은 isError 여야 한다.
    let r = s.request(
        "tools/call",
        serde_json::json!({"name": "hwp_doc_text", "arguments": {"docId": doc_id}}),
    );
    assert_eq!(
        r["result"]["isError"], true,
        "닫힌 핸들 재사용은 isError=true: {r}"
    );
}

#[test]
fn unknown_method_returns_jsonrpc_error() {
    let mut s = Server::started();
    let r = s.request("no/such-method", serde_json::json!({}));
    assert_eq!(
        r["error"]["code"], -32601,
        "알 수 없는 메서드는 -32601: {r}"
    );
}

#[test]
fn unknown_tool_returns_is_error() {
    let mut s = Server::started();
    let r = s.request(
        "tools/call",
        serde_json::json!({"name": "no_such_tool", "arguments": {}}),
    );
    assert_eq!(r["result"]["isError"], true, "{r}");
}

// ── [#3627] resources 표면 ─────────────────────────────────────────────────

/// `resources/list` 는 자기서술 매니페스트 1종 + canonical 문서 3종을 낸다.
#[test]
fn resources_list_declares_manifest_and_docs() {
    let mut s = Server::started();
    let r = s.request("resources/list", serde_json::json!({}));
    let resources = r["result"]["resources"]
        .as_array()
        .unwrap_or_else(|| panic!("resources/list 응답에 resources 배열이 없습니다: {r}"));
    let uris: Vec<&str> = resources
        .iter()
        .map(|x| x["uri"].as_str().unwrap_or_default())
        .collect();
    for expected in [
        "rhwp://capabilities/mcp",
        "rhwp://docs/llms.txt",
        "rhwp://docs/agent_knowledge_map.md",
        "rhwp://docs/agent_troubleshooting_guide.md",
    ] {
        assert!(uris.contains(&expected), "{expected} 가 없습니다: {uris:?}");
    }
    // MCP 필수 필드 — 빠지면 호스트가 리소스 목록을 그리지 못한다.
    for x in resources {
        assert!(x["name"].is_string(), "{x}");
        assert!(x["mimeType"].is_string(), "{x}");
    }
}

/// initialize 가 resources capability 를 선언해야 한다 — 선언이 없으면 호스트는
/// `resources/list` 를 아예 부르지 않는다. 스펙상 빈 객체가 "subscribe·listChanged
/// 둘 다 미지원" 의 정식 선언이다(생략이 아니라).
#[test]
fn initialize_declares_resources_capability() {
    let mut s = Server::start();
    let r = s.request(
        "initialize",
        serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "contract-test", "version": "0"}
        }),
    );
    assert!(
        r["result"]["capabilities"]["resources"].is_object(),
        "resources capability 선언이 없습니다: {r}"
    );
}

/// 복제 금지 가드 — 리소스 본문은 저장소의 canonical 문서와 **바이트 동일**해야 한다.
/// `include_str!` 로 원본을 그대로 가리키므로 복제본이 생기면 여기서 갈린다.
#[test]
fn resources_read_serves_canonical_docs() {
    let mut s = Server::started();
    let r = s.request(
        "resources/read",
        serde_json::json!({"uri": "rhwp://docs/agent_troubleshooting_guide.md"}),
    );
    let c = &r["result"]["contents"][0];
    assert_eq!(
        c["uri"], "rhwp://docs/agent_troubleshooting_guide.md",
        "contents[].uri 는 요청 URI 와 같아야 합니다: {r}"
    );
    assert_eq!(c["mimeType"], "text/markdown", "{r}");
    let text = c["text"]
        .as_str()
        .unwrap_or_else(|| panic!("contents[].text 가 없습니다: {r}"));
    let on_disk = std::fs::read_to_string(sample("mydocs/manual/agent_troubleshooting_guide.md"))
        .expect("실패 사전 문서 읽기 실패");
    assert_eq!(
        text, on_disk,
        "리소스 본문이 저장소 canonical 문서와 다릅니다 — 복제본이 생겼습니다"
    );
}

/// 프로필 정합 — 자기서술 매니페스트가 tools/list 에 없는 도구를 광고하면 에이전트가
/// "알 수 없는 도구" 를 밟는다. 두 표면은 같은 단일 출처를 써야 한다.
#[test]
fn resources_read_capabilities_matches_tools_list() {
    let mut s = Server::started();
    let served: Vec<String> = s.request("tools/list", serde_json::json!({}))["result"]["tools"]
        .as_array()
        .expect("tools 배열")
        .iter()
        .map(|t| t["name"].as_str().unwrap_or_default().to_string())
        .collect();

    let r = s.request(
        "resources/read",
        serde_json::json!({"uri": "rhwp://capabilities/mcp"}),
    );
    let c = &r["result"]["contents"][0];
    assert_eq!(c["mimeType"], "application/json", "{r}");
    let manifest: serde_json::Value = serde_json::from_str(
        c["text"]
            .as_str()
            .unwrap_or_else(|| panic!("contents[].text 가 없습니다: {r}")),
    )
    .expect("매니페스트가 JSON 이 아닙니다");
    for t in manifest["tools"].as_array().expect("tools 배열") {
        let name = t["name"].as_str().unwrap_or_default().to_string();
        assert!(
            served.contains(&name),
            "매니페스트가 광고한 {name} 이 tools/list 에 없습니다: {served:?}"
        );
    }
}

/// 없는 리소스는 스펙이 정한 -32002 와 `data.uri` 로 답한다.
#[test]
fn resources_read_unknown_uri_returns_resource_not_found() {
    let mut s = Server::started();
    let r = s.request(
        "resources/read",
        serde_json::json!({"uri": "rhwp://docs/no_such_doc.md"}),
    );
    assert_eq!(
        r["error"]["code"], -32002,
        "알 수 없는 리소스는 -32002: {r}"
    );
    assert_eq!(
        r["error"]["data"]["uri"], "rhwp://docs/no_such_doc.md",
        "error.data.uri 로 문제의 URI 를 돌려줘야 합니다: {r}"
    );

    // uri 자체가 없으면 요청 구조 오류다 — 리소스 부재와 구분한다.
    let r = s.request("resources/read", serde_json::json!({}));
    assert_eq!(r["error"]["code"], -32602, "{r}");
}
