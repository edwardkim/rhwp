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

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const SAMPLE: &str = "samples/hwp3-sample.hwp";
/// 서버가 실제로 말하는 개정판. src/mcp_serve.rs 의 PROTOCOL_VERSION 과 짝이다.
const SERVER_PROTOCOL_VERSION: &str = "2025-06-18";
/// 무응답 결함을 재현할 때 테스트가 hang 하지 않게 하는 상한.
const RESPONSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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

/// 선언된 입력 속성 중 **어느 CLI 경로로도 전달되지 않는** 것 — 즉 스키마에만
/// 존재하는 유령 인자를 stdin 전송 축만 남기고 전부 거부한다.
///
/// 이 목록은 argv 가 아닌 다른 축으로 전달되는 속성만 담는다. 늘리려면 그 축이
/// 실제로 존재함을 근거로 적어야 한다 — allowlist 가 커지면 가드가 무의미해진다.
const NON_ARGV_PROPERTIES: &[(&str, &str)] = &[
    (
        "paths",
        "자식 CLI stdin 으로 한 줄에 하나씩 흘려 넣는다(batch 계열).",
    ),
    (
        "password",
        "민감값이라 argv 금지 — cli.passwordStdin 계약으로 stdin 전달.",
    ),
];

#[test]
fn every_declared_input_property_is_wired_to_the_cli() {
    // 드리프트 가드 2: 이름뿐 아니라 **인자 배선**까지 본다.
    //
    // `inputSchema` 에 선언만 하고 `cli.args` 자리표시자에도 `cli.optionalArgs.when`
    // 에도 넣지 않으면, 서버는 그 인자를 조용히 버린 채 성공을 보고한다. 에이전트는
    // 스키마를 읽고 인자를 보냈으므로 반영됐다고 믿는다 — `dryRun: true` 를 보냈는데
    // 파일이 써지고 응답에는 `"dryRun": false` 가 오는 형태였다(#3712 이전 devel).
    // 컴파일 에러도 런타임 오류도 없이 계약만 거짓말한다.
    let cap = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp"])
        .output()
        .expect("capabilities 실행 실패");
    let manifest: serde_json::Value =
        serde_json::from_slice(&cap.stdout).expect("capabilities --mcp JSON");
    let tools = manifest["tools"].as_array().expect("tools 배열");
    assert!(
        !tools.is_empty(),
        "도구가 0건이면 이 가드는 공허하게 통과한다"
    );

    let mut orphans: Vec<String> = Vec::new();
    for t in tools {
        let name = t["name"].as_str().unwrap_or("<이름없음>");
        let Some(props) = t["inputSchema"]["properties"].as_object() else {
            continue;
        };
        // argv 템플릿(필수)에 쓰인 `{키}` 전부.
        let mut wired: Vec<String> = t["cli"]["args"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .filter(|s| s.starts_with('{') && s.ends_with('}') && s.len() > 2)
                    .map(|s| s[1..s.len() - 1].to_string())
                    .collect()
            })
            .unwrap_or_default();
        // 선택 인자는 `when` 키 자체가 배선 지점이다(값 없는 presence 플래그 포함).
        if let Some(optional) = t["cli"]["optionalArgs"].as_array() {
            for o in optional {
                if let Some(key) = o["when"].as_str() {
                    wired.push(key.to_string());
                }
            }
        }
        for key in props.keys() {
            if wired.iter().any(|w| w == key) {
                continue;
            }
            if NON_ARGV_PROPERTIES.iter().any(|(k, _)| k == key) {
                continue;
            }
            orphans.push(format!(
                "  - {name}.{key} — inputSchema 에만 있고 cli.args/optionalArgs 어디에도 없음"
            ));
        }
    }

    assert!(
        orphans.is_empty(),
        "선언만 되고 배선되지 않은 MCP 입력 인자 {}건:\n{}\n\n\
         스키마에 쓴 인자는 반드시 자식 CLI 에 닿아야 한다. 닿지 않으면 서버는 그 인자를\n\
         조용히 버리고 성공을 보고하며, 에이전트는 반영됐다고 믿는다(dryRun 이 그 형태였다).\n\
         고치는 법: `tool_with_optional_args` 로 `{{ \"when\": \"<키>\", \"args\": [...] }}` 를\n\
         더하라. argv 가 아닌 축(stdin 등)으로 전달한다면 NON_ARGV_PROPERTIES 에 근거와\n\
         함께 등재하라.",
        orphans.len(),
        orphans.join("\n"),
    );
}

/// 값 없는 presence 플래그는 "있으면 켜짐" 이다. `false` 를 존재로 세면 **끄라고 보낸
/// 요청이 켜는 요청이 된다** — 되돌릴 수 없는 쓰기에서 특히 위험하다.
#[test]
fn boolean_false_does_not_inject_a_presence_flag() {
    let p = sample(SAMPLE);
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = std::env::temp_dir().join("rhwp_mcp_dryrun_false.hwp");
    let _ = std::fs::remove_file(&out);

    let mut s = Server::started();
    let r = s.call_tool(
        "hwp_replace_text",
        serde_json::json!({
            "path": p.to_string_lossy(),
            "find": "가",
            "replace": "나",
            "output": out.to_string_lossy(),
            "dryRun": false,
        }),
    );
    let text = r["result"]["content"][0]["text"].as_str().unwrap_or("");
    assert!(
        !text.contains("\"dryRun\":true") && !text.contains("\"dryRun\": true"),
        "dryRun:false 를 보냈는데 --dry-run 이 주입됐다 — presence 플래그가 값을 무시했다: {text}"
    );
    let _ = std::fs::remove_file(&out);
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

// ── 프로토콜 적합성 회귀 ────────────────────────────────────────────────────

/// 원시 프레임들을 순서대로 보내고 응답 줄을 **타임아웃과 함께** `want` 개까지 모은다.
///
/// `Server::request` 를 못 쓰는 이유: 그쪽은 `read_line` 에서 무한정 막힌다. 여기서
/// 검증하려는 결함이 바로 "응답을 아예 안 쓴다"이므로, 그 harness 로는 테스트가
/// 실패하지 않고 **hang** 해버린다. 별도 스레드로 읽고 `recv_timeout` 으로 끊어야
/// 무응답이 실패로 보고된다.
fn raw_frames(frames: &[&str], want: usize) -> Vec<serde_json::Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp mcp-serve 실행 실패");
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else {
                break;
            };
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    for f in frames {
        writeln!(stdin, "{f}").expect("프레임 쓰기 실패");
    }
    stdin.flush().expect("flush");

    let mut out = Vec::new();
    while out.len() < want {
        match rx.recv_timeout(RESPONSE_TIMEOUT) {
            Ok(line) if line.trim().is_empty() => continue,
            Ok(line) => {
                out.push(serde_json::from_str(line.trim()).unwrap_or_else(|e| {
                    panic!("stdout 이 순수 JSON-RPC 가 아닙니다 ({e}): {line}")
                }))
            }
            // 무응답 — hang 대신 여기서 끊고 개수로 호출자가 판정한다.
            Err(_) => break,
        }
    }

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
    out
}

/// paths 없는 stdin 도구는 서버의 JSON-RPC stdin을 자식에게 넘기지 않고 즉시 거부해야
/// 하며, 뒤따르는 요청도 정상 처리해야 한다.
#[test]
fn batch_without_paths_fails_fast_and_protocol_stays_alive() {
    let out = raw_frames(
        &[
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"hwp_batch","arguments":{"subcommand":"info"}}}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#,
        ],
        3,
    );
    assert_eq!(
        out.len(),
        3,
        "batch 오류 뒤에도 세 응답이 와야 합니다: {out:?}"
    );
    assert_eq!(out[0]["id"], 1, "{out:?}");
    assert_eq!(out[1]["id"], 2, "{out:?}");
    assert_eq!(out[1]["result"]["isError"], true, "{out:?}");
    assert_eq!(out[2]["id"], 3, "{out:?}");
    assert!(out[2]["result"].is_object(), "{out:?}");
}

/// paths의 형식 오류는 자식 실행 전에 명확히 거부하고, 서버는 계속 살아 있어야 한다.
#[test]
fn batch_paths_wrong_shapes_rejected_before_spawn() {
    let mut s = Server::started();
    for (args, why) in [
        (
            serde_json::json!({"subcommand": "info", "paths": "a.hwp"}),
            "문자열 paths 는 배열이 아니다",
        ),
        (
            serde_json::json!({"subcommand": "info", "paths": [1, 2, 3]}),
            "비문자열 항목은 걸러내지 않고 거부한다",
        ),
        (
            serde_json::json!({"subcommand": "info", "paths": []}),
            "빈 배열은 선거부한다",
        ),
    ] {
        let r = s.request(
            "tools/call",
            serde_json::json!({"name": "hwp_batch", "arguments": args}),
        );
        assert_eq!(r["result"]["isError"], true, "{why}: {r}");
        let msg = r["result"]["content"][0]["text"].as_str().unwrap_or("");
        assert!(msg.contains("paths"), "{why}: {r}");
    }
    let r = s.request("ping", serde_json::json!({}));
    assert!(r["result"].is_object(), "{r}");
}

/// 올바른 paths 배열은 기존대로 batch stdin 파이프로 흘러야 한다.
#[test]
fn batch_with_paths_still_streams() {
    let mut s = Server::started();
    let envelope = s.call_tool(
        "hwp_batch",
        serde_json::json!({"subcommand": "info", "paths": [sample(SAMPLE).to_string_lossy()]}),
    );
    assert_eq!(envelope["schemaVersion"], "1.0", "{envelope}");
    assert!(
        envelope["pageCount"].as_u64().unwrap_or(0) > 0,
        "batch info 레코드에 pageCount 가 있어야 합니다: {envelope}"
    );
}

/// [MCP 2025-06-18 lifecycle §Version Negotiation]
///   "If the server supports the requested protocol version, it MUST respond with
///    the same version. Otherwise, the server MUST respond with another protocol
///    version it supports."
///
/// 결함일 때 서버는 요청값을 **무조건 그대로** 되비췄다 — "9999-99-99" 도 "banana" 도
/// 합의된 것처럼 보였다. 그래 놓고 몸통은 2025-06-18 전용 표면(structuredContent)을
/// 내보내므로, 엄격한 클라이언트는 끊어야 할 신호를 못 받은 채 못 읽는 응답을 받는다.
#[test]
fn initialize_negotiates_instead_of_echoing_client_version() {
    for requested in ["9999-99-99", "2024-11-05", "2025-03-26", "banana", ""] {
        let mut s = Server::start();
        let r = s.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": requested,
                "capabilities": {},
                "clientInfo": {"name": "contract-test", "version": "0"}
            }),
        );
        let negotiated = r["result"]["protocolVersion"]
            .as_str()
            .unwrap_or_else(|| panic!("initialize 응답에 protocolVersion 이 없습니다: {r}"));
        assert_ne!(
            negotiated, requested,
            "지원하지 않는 개정판을 그대로 되비추면 안 됩니다(요청={requested}): {r}"
        );
        assert_eq!(
            negotiated, SERVER_PROTOCOL_VERSION,
            "지원하지 않는 개정판에는 서버가 지원하는 개정판을 제시해야 합니다: {r}"
        );
    }
}

/// 지원하는 개정판은 **같은 값**으로 돌려줘야 한다(MUST) — 협상이 아니라 확인이다.
/// 위 테스트만 있으면 "항상 서버 버전을 박아라"로 잘못 고쳐도 통과하므로 짝이 필요하다.
#[test]
fn initialize_keeps_supported_version_and_defaults_when_absent() {
    let mut s = Server::start();
    let r = s.request(
        "initialize",
        serde_json::json!({
            "protocolVersion": SERVER_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {"name": "contract-test", "version": "0"}
        }),
    );
    assert_eq!(
        r["result"]["protocolVersion"], SERVER_PROTOCOL_VERSION,
        "지원하는 개정판은 같은 값으로 되돌려줘야 합니다: {r}"
    );

    // 버전이 없거나 문자열이 아닌 경우 — 둘 다 "요청한 개정판이 목록에 없다"와 같은
    // 갈래이므로 서버 기준판을 제시한다.
    for params in [
        serde_json::json!({"capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}),
        serde_json::json!({"protocolVersion": 20250618, "capabilities": {}}),
    ] {
        let mut s = Server::start();
        let r = s.request("initialize", params);
        assert_eq!(
            r["result"]["protocolVersion"], SERVER_PROTOCOL_VERSION,
            "protocolVersion 이 없거나 문자열이 아니면 서버 기준판: {r}"
        );
    }
}

/// [JSON-RPC 2.0 §5] 파싱은 됐지만 Request 객체가 아닌 프레임은 -32600 으로 답해야
/// 하고, 프레임에서 id 를 알아낼 수 없으므로 id 는 null 이다.
///
/// 결함일 때 이 경로는 **한 바이트도 쓰지 않았다** — 그래서 read_line 이 아니라
/// 타임아웃 읽기로 확인한다. 무응답이면 hang 이 아니라 실패로 잡혀야 한다.
#[test]
fn non_object_frames_get_invalid_request_instead_of_silence() {
    for frame in [
        r#"[{"jsonrpc":"2.0","id":1,"method":"ping"}]"#, // JSON-RPC 배치
        r#""ping""#,                                     // 문자열
        "42",                                            // 숫자
        "true",                                          // 불리언
        "null",                                          // null
    ] {
        let out = raw_frames(&[frame], 1);
        assert_eq!(
            out.len(),
            1,
            "비객체 프레임에 응답이 없습니다 — 클라이언트가 영원히 대기합니다: {frame}"
        );
        let r = &out[0];
        assert_eq!(r["jsonrpc"], "2.0", "{r}");
        assert_eq!(
            r["error"]["code"], -32600,
            "비객체 프레임은 -32600 Invalid Request 여야 합니다({frame}): {r}"
        );
        assert_eq!(
            r["id"],
            serde_json::Value::Null,
            "id 를 알아낼 수 없으면 null 이어야 합니다({frame}): {r}"
        );
    }
}

/// MCP 2025-06-18 changelog 는 "Remove support for JSON-RPC batching" 이다.
/// 배열은 그냥 형식 오류가 아니라 **이 개정판에서 사라진 기능**이므로, 사유에
/// 개정판을 밝혀야 호스트가 요청을 한 줄씩 푸는 쪽으로 고칠 수 있다.
#[test]
fn jsonrpc_batch_is_rejected_with_batching_removed_note() {
    let out = raw_frames(
        &[r#"[{"jsonrpc":"2.0","id":1,"method":"ping"},{"jsonrpc":"2.0","id":2,"method":"ping"}]"#],
        1,
    );
    assert_eq!(out.len(), 1, "배치 프레임에 응답이 없습니다");
    let r = &out[0];
    assert_eq!(r["error"]["code"], -32600, "{r}");
    let msg = r["error"]["message"].as_str().unwrap_or_default();
    assert!(
        msg.contains("배치") && msg.contains("2025-06-18"),
        "배치 거절 사유는 어느 개정판에서 사라졌는지 밝혀야 합니다: {r}"
    );
}

/// [JSON-RPC 2.0 §4] method 는 문자열이어야 한다 — 없거나 다른 타입이면 요청 구조가
/// 틀린 것(-32600)이지 메서드가 없는 것(-32601)이 아니다. 결함일 때는 unwrap_or("")
/// 로 빈 이름을 만들어 -32601 과 "지원하지 않는 메서드: " 라는 빈 문구를 냈다.
#[test]
fn request_without_string_method_is_invalid_request_not_method_not_found() {
    for (frame, want_id) in [
        (r#"{"jsonrpc":"2.0","id":7}"#, 7),
        (r#"{"jsonrpc":"2.0","id":8,"method":123}"#, 8),
        (r#"{"jsonrpc":"2.0","id":9,"method":null}"#, 9),
    ] {
        let out = raw_frames(&[frame], 1);
        assert_eq!(out.len(), 1, "응답이 없습니다: {frame}");
        let r = &out[0];
        assert_eq!(
            r["error"]["code"], -32600,
            "method 가 문자열이 아니면 -32600({frame}): {r}"
        );
        assert_eq!(r["id"], want_id, "id 는 그대로 되돌려줍니다({frame}): {r}");
    }
}

/// 결함일 때 배치 프레임은 0 바이트를 냈고 뒤이은 ping 만 응답을 받았다 — 즉 스트림은
/// 살아 있는데 응답 하나가 통째로 증발했다. 고친 뒤에는 프레임 2건에 응답 2건이,
/// 그것도 보낸 순서대로 나와야 한다(단일 루프이므로 순서는 계약이다).
#[test]
fn invalid_frame_does_not_swallow_the_next_response() {
    let out = raw_frames(
        &[
            r#"[{"jsonrpc":"2.0","id":1,"method":"ping"}]"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#,
        ],
        2,
    );
    assert_eq!(
        out.len(),
        2,
        "프레임 2건에 응답 2건이 나와야 합니다: {out:?}"
    );
    let bad = &out[0];
    let good = &out[1];
    assert_eq!(bad["error"]["code"], -32600, "{bad}");
    assert_eq!(bad["id"], serde_json::Value::Null, "{bad}");
    assert!(good["result"].is_object(), "{good}");
    assert_eq!(good["id"], 2, "{good}");
}

// ── [#3627] resources 표면 ─────────────────────────────────────────────────

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
    for x in resources {
        assert!(x["name"].is_string(), "{x}");
        assert!(x["mimeType"].is_string(), "{x}");
    }
}

#[test]
fn initialize_declares_resources_capability() {
    let mut s = Server::start();
    let r = s.request(
        "initialize",
        serde_json::json!({
            "protocolVersion": SERVER_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {"name": "contract-test", "version": "0"}
        }),
    );
    assert!(
        r["result"]["capabilities"]["resources"].is_object(),
        "resources capability 선언이 없습니다: {r}"
    );
}

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
    let r = s.request("resources/read", serde_json::json!({}));
    assert_eq!(r["error"]["code"], -32602, "{r}");
}

// ── R80 1단계: --stats 옵트인 관측성 ────────────────────────────────────
//
// mydocs/tech/agent_architecture/observability_contract.md 의 §3(금지 목록)을
// 계약 테스트로 고정한다 — INV-07("무동작 플래그 금지")에 따라 플래그와
// 계약 테스트가 같은 PR 로 들어간다.

/// `--stats` 없는 `mcp-serve` 는 §1 그대로 — 계측 0, 추가 stderr 출력 0.
#[test]
fn mcp_serve_without_stats_flag_emits_nothing_extra_on_stderr() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp mcp-serve 실행 실패");
    {
        let mut stdin = child.stdin.take().expect("stdin");
        let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-06-18","capabilities":{{}},"clientInfo":{{"name":"contract-test","version":"0"}}}}}}"#
        )
        .expect("초기화 쓰기 실패");
        stdin.flush().expect("flush");
        let mut line = String::new();
        stdout.read_line(&mut line).expect("초기화 응답 읽기 실패");
        // stdin 을 닫아 서버가 EOF 로 정상 종료하게 한다.
    }
    let status = child.wait().expect("서버 종료 대기 실패");
    assert!(status.success(), "서버가 비정상 종료했습니다: {status:?}");
    let mut stderr_out = String::new();
    child
        .stderr
        .take()
        .expect("stderr")
        .read_to_string(&mut stderr_out)
        .expect("stderr 읽기 실패");
    assert!(
        stderr_out.is_empty(),
        "--stats 없이도 stderr 에 출력이 났습니다: {stderr_out:?}"
    );
}

/// `--stats` 는 도구명별 호출 수·오류 수만 stderr 로 낸다 — 문서 내용·경로·
/// 인자 값은 계약(§3)에 따라 절대 새어 나가면 안 된다.
#[test]
fn mcp_serve_stats_summarizes_call_counts_without_leaking_document_data() {
    let p = sample(SAMPLE);
    // 존재하지 않는 경로 — 오류 호출을 만들면서, 통계에 새면 안 될 "민감한
    // 경로 성분"의 대역으로도 쓴다(경로는 계약 §3 상 준식별자로 금지 항목).
    let bogus_path = "/no/such/문서-극비-누설되면-안됨-72d9f3.hwp";
    let unknown_tool = format!("미등록-도구-{}", "x".repeat(32 * 1024));

    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .arg("--stats")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp mcp-serve --stats 실행 실패");
    {
        let mut stdin = child.stdin.take().expect("stdin");
        let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));
        let mut line = String::new();

        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-06-18","capabilities":{{}},"clientInfo":{{"name":"contract-test","version":"0"}}}}}}"#
        )
        .expect("초기화 쓰기 실패");
        stdin.flush().expect("flush");
        stdout.read_line(&mut line).expect("초기화 응답 읽기 실패");

        // 성공 호출 1건 — 실제 문서 경로를 실어 보낸다(통계에 새면 안 됨).
        let ok_call = serde_json::json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": {"name": "hwp_info", "arguments": {"path": p.to_str().unwrap()}}
        });
        writeln!(stdin, "{ok_call}").expect("hwp_info 성공 호출 쓰기 실패");
        stdin.flush().expect("flush");
        line.clear();
        stdout
            .read_line(&mut line)
            .expect("hwp_info 성공 응답 읽기 실패");

        // 오류 호출 1건(같은 도구, 존재하지 않는 경로) — isError:true 경로를
        // 태워 오류 수 집계와 "경로는 절대 안 샌다"를 같은 호출로 검증한다.
        let err_call = serde_json::json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": {"name": "hwp_info", "arguments": {"path": bogus_path}}
        });
        writeln!(stdin, "{err_call}").expect("hwp_info 오류 호출 쓰기 실패");
        stdin.flush().expect("flush");
        line.clear();
        let n = stdout
            .read_line(&mut line)
            .expect("hwp_info 오류 응답 읽기 실패");
        assert!(n > 0, "서버가 오류 호출 후 응답 없이 종료했습니다");
        let resp: serde_json::Value =
            serde_json::from_str(line.trim()).expect("응답 JSON 파싱 실패");
        assert_eq!(
            resp["result"]["isError"], true,
            "존재하지 않는 경로는 isError:true 여야 합니다: {resp}"
        );

        // 호출자가 넣은 임의 도구명은 stats HashMap의 키로 남기면 안 된다. 고유한
        // 이름을 반복 전송하면 옵트인 통계만으로 서버 메모리를 소진할 수 있기 때문이다.
        let unknown_call = serde_json::json!({
            "jsonrpc": "2.0", "id": 4, "method": "tools/call",
            "params": {"name": unknown_tool, "arguments": {}}
        });
        writeln!(stdin, "{unknown_call}").expect("미등록 도구 호출 쓰기 실패");
        stdin.flush().expect("flush");
        line.clear();
        let n = stdout
            .read_line(&mut line)
            .expect("미등록 도구 응답 읽기 실패");
        assert!(n > 0, "서버가 미등록 도구 호출 후 응답 없이 종료했습니다");
        let resp: serde_json::Value =
            serde_json::from_str(line.trim()).expect("미등록 도구 응답 JSON 파싱 실패");
        assert_eq!(
            resp["result"]["isError"], true,
            "미등록 도구는 오류여야 합니다: {resp}"
        );

        // stdin 을 닫아 서버가 EOF 로 정상 종료하며 통계를 stderr 로 낸다.
    }
    let status = child.wait().expect("서버 종료 대기 실패");
    assert!(status.success(), "서버가 비정상 종료했습니다: {status:?}");
    let mut stderr_out = String::new();
    child
        .stderr
        .take()
        .expect("stderr")
        .read_to_string(&mut stderr_out)
        .expect("stderr 읽기 실패");

    assert!(
        stderr_out.contains("mcp-serve --stats"),
        "--stats 요약 머리글이 없습니다: {stderr_out:?}"
    );
    assert!(
        stderr_out.contains("hwp_info"),
        "hwp_info 호출이 도구명으로 집계되지 않았습니다: {stderr_out:?}"
    );
    assert!(
        stderr_out.contains("2회 호출") && stderr_out.contains("오류 1건"),
        "호출 수 2·오류 수 1 이 집계되지 않았습니다: {stderr_out:?}"
    );
    assert!(
        stderr_out.contains("(알 수 없는 도구): 1회 호출, 오류 1건"),
        "미등록 도구가 고정 버킷으로 집계되지 않았습니다: {stderr_out:?}"
    );

    // §3 금지 목록 — 문서 경로는 성공·오류 어느 쪽이든 stderr 에 어떤
    // 형태로도 나타나면 안 된다.
    assert!(
        !stderr_out.contains(p.to_str().unwrap()),
        "성공 호출의 문서 경로가 --stats 출력에 샜습니다: {stderr_out:?}"
    );
    assert!(
        !stderr_out.contains(bogus_path) && !stderr_out.contains("극비-누설되면-안됨"),
        "오류 호출의 문서 경로가 --stats 출력에 샜습니다: {stderr_out:?}"
    );
    assert!(
        !stderr_out.contains(&unknown_tool),
        "호출자가 보낸 미등록 도구명이 --stats 출력에 샜습니다: {stderr_out:?}"
    );
}
