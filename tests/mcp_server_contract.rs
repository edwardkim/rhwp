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
