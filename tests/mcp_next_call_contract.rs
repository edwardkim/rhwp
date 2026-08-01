//! [#3699] nextCall — isError 에 기계가 따라할 교정 호출 동봉 (#3630 P4).
//! error 필드가 기존 원문을 담아 하위호환, nextCall.name 은 실존 도구만.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn serve_call(req: &str) -> serde_json::Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mcp-serve");
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    writeln!(stdin, "{req}").unwrap();
    stdin.flush().unwrap();
    let mut line = String::new();
    assert!(stdout.read_line(&mut line).unwrap() > 0);
    let _ = child.kill();
    let _ = child.wait();
    serde_json::from_str(line.trim()).unwrap()
}

fn error_body(v: &serde_json::Value) -> serde_json::Value {
    assert_eq!(v["result"]["isError"], true, "{v}");
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    serde_json::from_str(text).unwrap_or_else(|e| panic!("구조화 JSON 이어야 함({e}): {text}"))
}

#[test]
fn closed_handle_carries_next_call_to_open() {
    let v = serve_call(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hwp_doc_fill_fields","arguments":{"docId":"doc-999","data":{"a":"b"}}}}"#,
    );
    let body = error_body(&v);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("열려 있지 않은 핸들"),
        "하위호환 원문: {body}"
    );
    assert_eq!(body["nextCall"]["name"], "hwp_open", "{body}");
    assert!(body["nextCall"]["why"].is_string(), "{body}");
}

#[test]
fn unknown_tool_next_call_is_registered_tool() {
    let v = serve_call(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hwp_serch","arguments":{}}}"#,
    );
    let body = error_body(&v);
    assert_eq!(body["nextCall"]["name"], "hwp_search", "{body}");
    // 실존 도구 대조: capabilities --mcp 선언에 있어야 한다.
    let caps = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp"])
        .output()
        .unwrap();
    let m: serde_json::Value = serde_json::from_slice(&caps.stdout).unwrap();
    assert!(
        m["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == body["nextCall"]["name"]),
        "{body}"
    );
}

#[test]
fn close_on_unknown_handle_also_guides_open() {
    let v = serve_call(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hwp_close","arguments":{"docId":"doc-999"}}}"#,
    );
    let body = error_body(&v);
    assert_eq!(body["nextCall"]["name"], "hwp_open", "{body}");
}
