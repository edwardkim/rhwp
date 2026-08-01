//! #3604 보호 문서 MCP 계약.
//!
//! 실제 HWP5/HWP3/HWPX fixture로 세션 암호 입력과 무상태 자식 stdin 전달을
//! 검증한다. 암호 값이 도구 응답에 에코되지 않는지도 함께 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const PASSWORD: &str = "123456";
const WRONG_PASSWORD: &str = "mcp-password-must-not-echo";
const FIXTURES: &[(&str, u64)] = &[
    ("samples/hwp3-sample16-hwp5-2024-password-123456.hwp", 64),
    ("samples/HWP3-password-123456.hwp", 24),
    ("samples/HWP5-password-123456.hwpx", 23),
];

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Server {
    fn started() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .arg("mcp-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("mcp-serve 실행 실패");
        let stdin = child.stdin.take().expect("mcp stdin");
        let stdout = BufReader::new(child.stdout.take().expect("mcp stdout"));
        let mut server = Self {
            child,
            stdin,
            stdout,
            next_id: 1,
        };
        let initialized = server.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "mcp-password-contract", "version": "0"}
            }),
        );
        assert_eq!(initialized["result"]["serverInfo"]["name"], "rhwp");
        server.notify("notifications/initialized");
        server
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;
        writeln!(
            self.stdin,
            "{}",
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
        )
        .expect("MCP 요청 쓰기 실패");
        self.stdin.flush().expect("MCP 요청 flush 실패");

        let mut line = String::new();
        loop {
            line.clear();
            assert!(
                self.stdout
                    .read_line(&mut line)
                    .expect("MCP 응답 읽기 실패")
                    > 0,
                "MCP 서버가 응답 없이 종료했습니다 ({method})"
            );
            let response: serde_json::Value =
                serde_json::from_str(line.trim()).expect("MCP stdout은 JSON-RPC여야 합니다");
            if response["id"].as_i64() == Some(id) {
                return response;
            }
        }
    }

    fn notify(&mut self, method: &str) {
        writeln!(
            self.stdin,
            "{}",
            serde_json::json!({"jsonrpc": "2.0", "method": method})
        )
        .expect("MCP 알림 쓰기 실패");
        self.stdin.flush().expect("MCP 알림 flush 실패");
    }

    fn tool_response(&mut self, name: &str, arguments: serde_json::Value) -> serde_json::Value {
        self.request(
            "tools/call",
            serde_json::json!({"name": name, "arguments": arguments}),
        )
    }

    fn call_json_tool(&mut self, name: &str, arguments: serde_json::Value) -> serde_json::Value {
        let response = self.tool_response(name, arguments);
        assert_eq!(response["result"]["isError"], false, "{name}: {response}");
        serde_json::from_str(
            response["result"]["content"][0]["text"]
                .as_str()
                .unwrap_or_else(|| panic!("{name} content text 누락: {response}")),
        )
        .unwrap_or_else(|error| panic!("{name} JSON 봉투 파싱 실패: {error}"))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn response_text(response: &serde_json::Value) -> &str {
    response["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
}

#[test]
fn session_open_declares_a_write_only_password() {
    let mut server = Server::started();
    let listed = server.request("tools/list", serde_json::json!({}));
    let open = listed["result"]["tools"]
        .as_array()
        .expect("tools 목록")
        .iter()
        .find(|tool| tool["name"] == "hwp_open")
        .expect("hwp_open 선언");
    assert_eq!(
        open["inputSchema"]["properties"]["password"]["type"],
        "string"
    );
    assert_eq!(
        open["inputSchema"]["properties"]["password"]["writeOnly"],
        true
    );
    assert!(
        !open["inputSchema"]["required"]
            .as_array()
            .expect("required")
            .iter()
            .any(|value| value == "password"),
        "password는 선택 입력이어야 합니다: {open}"
    );
}

#[test]
fn protected_documents_open_in_a_password_session_without_echoing_password() {
    for (fixture, expected_pages) in FIXTURES {
        let path = fixture_path(fixture);
        if !path.exists() {
            eprintln!("fixture 없음 — 건너뜀: {}", path.display());
            continue;
        }
        let path = path.to_str().expect("UTF-8 fixture path");
        let mut server = Server::started();

        let missing = server.tool_response("hwp_open", serde_json::json!({"path": path}));
        assert_eq!(missing["result"]["isError"], true, "{fixture}: {missing}");

        let wrong = server.tool_response(
            "hwp_open",
            serde_json::json!({"path": path, "password": WRONG_PASSWORD}),
        );
        assert_eq!(wrong["result"]["isError"], true, "{fixture}: {wrong}");
        assert!(
            !response_text(&wrong).contains(WRONG_PASSWORD),
            "오류 응답에 비밀번호를 에코하면 안 됩니다: {wrong}"
        );

        let opened = server.call_json_tool(
            "hwp_open",
            serde_json::json!({"path": path, "password": PASSWORD}),
        );
        assert_eq!(
            opened["pageCount"].as_u64(),
            Some(*expected_pages),
            "{fixture}: {opened}"
        );
        let doc_id = opened["docId"].as_str().expect("docId").to_string();
        let text = server.call_json_tool(
            "hwp_doc_text",
            serde_json::json!({"docId": doc_id, "page": 0}),
        );
        assert_eq!(
            text["pages"].as_array().map(Vec::len),
            Some(1),
            "{fixture}: {text}"
        );
        let closed = server.call_json_tool("hwp_close", serde_json::json!({"docId": doc_id}));
        assert_eq!(closed["closed"], true, "{fixture}: {closed}");
    }
}

#[test]
fn stateless_document_tool_uses_password_stdin_without_cli_argument_echo() {
    let fixture = fixture_path(FIXTURES[0].0);
    if !fixture.exists() {
        eprintln!("fixture 없음 — 건너뜀: {}", fixture.display());
        return;
    }
    let capabilities = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp"])
        .output()
        .expect("capabilities 실행 실패");
    assert!(capabilities.status.success(), "capabilities 실패");
    let manifest: serde_json::Value =
        serde_json::from_slice(&capabilities.stdout).expect("capabilities JSON");
    let info_definition = manifest["tools"]
        .as_array()
        .expect("capabilities tools")
        .iter()
        .find(|tool| tool["name"] == "hwp_info")
        .expect("hwp_info 선언");
    assert_eq!(
        info_definition["inputSchema"]["properties"]["password"]["writeOnly"],
        true
    );
    assert_eq!(
        info_definition["cli"]["passwordStdin"]["argument"],
        "password"
    );
    assert_eq!(
        info_definition["cli"]["passwordStdin"]["flag"],
        "--password-stdin"
    );
    assert!(
        !info_definition["cli"]["args"]
            .as_array()
            .expect("hwp_info cli args")
            .iter()
            .any(|arg| arg == "{password}"),
        "비밀번호는 cli.args 자리에 있으면 안 됩니다: {info_definition}"
    );

    let path = fixture.to_str().expect("UTF-8 fixture path");
    let mut server = Server::started();

    let info = server.call_json_tool(
        "hwp_info",
        serde_json::json!({"path": path, "password": PASSWORD}),
    );
    assert_eq!(info["pageCount"].as_u64(), Some(64), "{info}");

    let wrong = server.tool_response(
        "hwp_info",
        serde_json::json!({"path": path, "password": WRONG_PASSWORD}),
    );
    assert_eq!(wrong["result"]["isError"], true, "{wrong}");
    assert!(
        !response_text(&wrong).contains(WRONG_PASSWORD),
        "자식 CLI 오류를 MCP로 전달할 때 비밀번호를 에코하면 안 됩니다: {wrong}"
    );
}

#[test]
fn batch_tools_reject_password_to_keep_stdin_unambiguous() {
    let mut server = Server::started();
    let response = server.tool_response(
        "hwp_batch",
        serde_json::json!({
            "subcommand": "info",
            "paths": ["samples/hwp3-sample.hwp"],
            "password": WRONG_PASSWORD,
        }),
    );
    assert_eq!(response["result"]["isError"], true, "{response}");
    assert!(
        !response_text(&response).contains(WRONG_PASSWORD),
        "batch 거부 오류에 비밀번호를 에코하면 안 됩니다: {response}"
    );
}
