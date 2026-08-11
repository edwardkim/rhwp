//! [#4537] 하네스 계약 — 검증 루프 단일 명령(init·wrap·status).
//!
//! 고정하는 것: ① init 이 작업장 규약(capsules/·키·키링)을 만들고 키
//! 덮어쓰기를 거부한다, ② wrap 이 **실산출을 실제로 만들고**(replay 와의
//! 차이) 영수증·캡슐을 연번으로 쌓으며 **직전 캡슐을 자동 부모 연결**한다
//! — 체인이 스스로 자란다, ③ status 가 체인·서명·(--deep) 재현을 한 봉투로
//! 판정하고 캡슐 사후 변조를 brokenAt 으로 폭로한다(exit 3), ④ 사용법 규약.
//!
//! 판정(status)은 쓰기가 없어 최상위 `harness-status`(category=diagnostic)로
//! 분리돼 있다 — 쓰기 명령(harness)과 한 표면을 쓰면 MCP readOnlyHint 주석이
//! category 와 모순된다(mcp_tool_annotations_contract ②).

#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

const SAMPLE: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn env_of(o: &Output) -> serde_json::Value {
    serde_json::from_slice(&o.stdout).unwrap_or(serde_json::json!({}))
}

fn existing_snippet() -> String {
    let o = run(&["export-text", SAMPLE, "-p", "0", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    let text = env["pages"][0]["text"].as_str().expect("쪽 텍스트");
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    chars[..2].iter().collect()
}

fn make_ws(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp_harness_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn plan_json(input: &str, output: &std::path::Path, find: &str) -> String {
    serde_json::json!({
        "planVersion": "1.0",
        "input": input,
        "output": output.to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": find, "replace": find }],
    })
    .to_string()
}

#[test]
fn init_wrap_chain_status_roundtrip_and_tamper() {
    let ws = make_ws("loop");
    let ws_s = ws.to_string_lossy().into_owned();

    // ── init: 작업장 + 키 + 키링.
    let o = run(&[
        "harness",
        "init",
        &ws_s,
        "--key-id",
        "test.example/loop#1",
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    assert_eq!(env["keyId"], "test.example/loop#1");
    assert!(ws.join("capsules").is_dir());
    let key = ws.join("harness.key.json");
    let keyring = ws.join("keyring.json");
    assert!(key.exists() && keyring.exists());

    // 키 덮어쓰기 거부.
    let o = run(&["harness", "init", &ws_s, "--key-id", "test.example/loop#1"]);
    assert_eq!(o.status.code(), Some(2), "기존 키 덮어쓰기는 사용법 오류");

    // ── wrap #1: 실산출이 진짜로 생긴다 (replay 와의 차이).
    let find = existing_snippet();
    let o1 = ws.join("o1.hwp");
    let o = run(&[
        "harness",
        "wrap",
        "--plan",
        &plan_json(SAMPLE, &o1, &find),
        "--dir",
        &ws_s,
        "--sign-key",
        key.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    assert!(o1.exists(), "wrap 은 계획의 output 을 실제로 만든다");
    assert_eq!(env["parent"], serde_json::Value::Null, "첫 캡슐은 뿌리");
    assert_eq!(env["signed"], true);
    let cap1 = env["capsule"].as_str().expect("캡슐 파일명").to_string();
    assert!(cap1.starts_with("0001_"), "연번 파일명: {cap1}");

    // ── wrap #2: 직전 캡슐 자동 부모 연결 — 체인이 스스로 자란다.
    let o2 = ws.join("o2.hwp");
    let o = run(&[
        "harness",
        "wrap",
        "--plan",
        &plan_json(o1.to_str().unwrap(), &o2, &find),
        "--dir",
        &ws_s,
        "--sign-key",
        key.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    assert_eq!(
        env["parent"],
        serde_json::json!(cap1),
        "자동 부모 = 직전 캡슐"
    );
    let cap2 = env["capsule"].as_str().unwrap().to_string();
    assert!(cap2.starts_with("0002_"));

    // ── status: 체인 + 서명 + deep 재현 전부 green.
    let o = run(&[
        "harness-status",
        &ws_s,
        "--keyring",
        keyring.to_str().unwrap(),
        "--deep",
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    let env = env_of(&o);
    assert_eq!(env["capsules"], 2);
    assert_eq!(env["chainValid"], true);
    assert_eq!(env["signed"]["valid"], 2);
    assert_eq!(env["signed"]["invalid"], 0);
    assert_eq!(env["reproduced"]["ok"], 2, "{env}");
    assert_eq!(env["verdict"], "ok");

    // ── 사후 변조: 첫 캡슐에 후행 공백 1바이트 — JSON 은 그대로 유효하므로
    // 파싱 검출이 아니라 **자식이 기록한 부모 해시**가 폭로해야 한다.
    let cap1_path = ws.join("capsules").join(&cap1);
    let mut bytes = std::fs::read(&cap1_path).unwrap();
    bytes.push(b' ');
    std::fs::write(&cap1_path, &bytes).unwrap();
    let o = run(&["harness-status", &ws_s, "--json"]);
    assert_eq!(o.status.code(), Some(3), "변조 = 검증 단언 실패");
    let env = env_of(&o);
    assert_eq!(env["chainValid"], false);
    assert!(
        env["brokenAt"].as_str().unwrap_or("").contains(&cap2),
        "자식 캡슐이 부모 변조를 폭로한다: {env}"
    );
    let _ = std::fs::remove_dir_all(&ws);
}

#[test]
fn usage_conventions() {
    let o = run(&["harness"]);
    assert_eq!(o.status.code(), Some(2));
    // 판정은 harness 하위가 아니다 — 옛 표면은 사용법 오류로 남는다.
    let o = run(&["harness", "status", "nope"]);
    assert_eq!(o.status.code(), Some(2), "판정은 harness-status 로 나갔다");
    let o = run(&["harness", "wrap", "--dir", "nope", "--json"]);
    assert_eq!(o.status.code(), Some(2), "--plan 없는 wrap 은 사용법 오류");
    let ws = make_ws("usage");
    std::fs::create_dir_all(&ws).unwrap();
    // init 없는 폴더에 wrap → 작업장 아님 (capsules/ 부재).
    let o = run(&[
        "harness",
        "wrap",
        "--plan",
        "{\"planVersion\":\"1.0\",\"input\":\"x\",\"output\":\"y\",\"steps\":[]}",
        "--dir",
        ws.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2));
    let _ = std::fs::remove_dir_all(&ws);
}
