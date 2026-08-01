//! [#3694] did-you-mean — 이름 환각 교정 단서 (#3630 P1 구현).
//! 후보는 capabilities 명령 목록 단일 출처, 임계 초과 시 무제안(오제안 0 원칙).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn unknown_command_hints_closest_and_keeps_exit_2() {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("exprot-svg") // 오타
        .output()
        .expect("rhwp");
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("힌트: 가장 가까운 명령은 'export-svg' 입니다"),
        "{err}"
    );
}

#[test]
fn gibberish_command_gets_no_hint() {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("코끼리코끼리")
        .output()
        .expect("rhwp");
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("힌트:"), "임계 초과는 무제안: {err}");
}

#[test]
fn mcp_unknown_tool_reports_did_you_mean_json() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mcp-serve");
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"hwp_serch","arguments":{{}}}}}}"#
    )
    .unwrap();
    stdin.flush().unwrap();
    let mut line = String::new();
    assert!(stdout.read_line(&mut line).unwrap() > 0);
    let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(v["result"]["isError"], true, "{v}");
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    let body: serde_json::Value =
        serde_json::from_str(text).unwrap_or_else(|e| panic!("구조화 JSON 이어야 함({e}): {text}"));
    // 하위호환: error 필드가 기존 원문을 담는다.
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .contains("알 수 없는 도구"),
        "{body}"
    );
    assert_eq!(body["didYouMean"][0], "hwp_search", "{body}");
    let _ = child.kill();
    let _ = child.wait();
}
