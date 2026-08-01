//! [#3721] 계획 v2 — `--dry-run`: 실행 전에 계획을 검사한다 (#3719 §6-3).
//!
//! 계약: 선검증만 돌고 **디스크는 절대 건드리지 않는다**. 유효하면 preview 저널 + exit 0,
//! 위반이 있으면 기존 경로와 똑같이 invalid[] + exit 2. 계획서가 `dryRun:true` 를 담을 수
//! 있으므로 MCP `hwp_run_plan` 도 인자 추가 없이 같은 계약을 얻는다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-plandry-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp")
}

/// 유효한 계획 + `--dry-run` → preview 저널, exit 0, **산출 파일 없음**.
#[test]
fn dry_run_previews_without_touching_disk() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("ok", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "미리보기사"} },
            { "action": "replace_text", "find": "마케팅", "replace": "기획" },
        ],
    });
    let plan_path = temp_path("ok", "json");
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();

    let output = run(&["run", plan_path.to_str().unwrap(), "--dry-run", "--json"]);
    assert_eq!(output.status.code(), Some(0), "유효한 계획은 exit 0");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");

    assert_eq!(v["dryRun"], true, "dryRun 표시 필수: {v}");
    assert!(
        v["invalid"].as_array().is_none_or(|a| a.is_empty()),
        "유효한 계획이므로 invalid 는 비어 있다: {v}"
    );
    let preview = v["preview"].as_array().expect("preview 배열");
    assert_eq!(preview.len(), 2, "step 수만큼 미리보기: {v}");
    assert_eq!(preview[0]["action"], "fill_fields", "{v}");
    assert_eq!(preview[1]["action"], "replace_text", "{v}");
    // 무엇이 얼마나 바뀔지 — 실행하지 않고도 답이 나와야 한다.
    assert!(
        preview[1]["matches"].as_u64().is_some_and(|n| n >= 1),
        "치환 예정 건수를 미리 보고한다: {v}"
    );
    // 계약의 핵심: 디스크 무변경.
    assert!(!out.exists(), "dry-run 은 산출 파일을 만들지 않는다");

    let _ = std::fs::remove_file(&plan_path);
}

/// 위반이 있는 계획 + `--dry-run` → 기존 선검증 경로와 동일 (invalid[] + exit 2).
#[test]
fn dry_run_reports_violations_same_as_real_run() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("bad", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"없는필드XYZ": "값"} },
            { "action": "replace_text", "find": "이런문자열은없다9999", "replace": "X" },
        ],
    });
    let plan_path = temp_path("bad", "json");
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();

    let output = run(&["run", plan_path.to_str().unwrap(), "--dry-run", "--json"]);
    assert_eq!(output.status.code(), Some(2), "위반은 exit 2");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    let invalid = v["invalid"].as_array().expect("invalid 배열");
    assert_eq!(invalid.len(), 2, "두 위반을 한 번에 보고: {v}");
    assert!(!out.exists(), "위반 시에도 산출 없음");

    let _ = std::fs::remove_file(&plan_path);
}

/// 계획서가 `dryRun:true` 를 직접 담아도 같은 계약 — MCP 경로가 인자 추가 없이 얻는 길.
#[test]
fn plan_carried_dry_run_flag_is_equivalent() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("carried", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "dryRun": true,
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [ { "action": "fill_fields", "data": {"회사명": "계획내플래그"} } ],
    });
    let plan_path = temp_path("carried", "json");
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();

    // --dry-run 플래그 없이, 계획서의 dryRun 만으로.
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_eq!(v["dryRun"], true, "{v}");
    assert!(!out.exists(), "계획서가 실은 dryRun 도 디스크 무변경");

    let _ = std::fs::remove_file(&plan_path);
}

/// 실행 모드는 종전 그대로 — dry-run 추가가 기존 계약을 흔들지 않는다.
#[test]
fn real_run_still_writes_and_reports_dry_run_false() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("real", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [ { "action": "fill_fields", "data": {"회사명": "실제실행"} } ],
    });
    let plan_path = temp_path("real", "json");
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();

    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert!(v["steps"].is_array(), "실행 저널은 steps 를 낸다: {v}");
    assert_ne!(v["dryRun"], true, "실행 모드는 dryRun 이 참이 아니다: {v}");
    assert!(out.exists(), "실행 모드는 산출을 남긴다");

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&plan_path);
}

/// MCP `hwp_run_plan` 도 계획서의 dryRun 으로 같은 계약을 얻는다 (도구 인자 추가 없음).
#[test]
fn mcp_run_plan_honors_plan_dry_run() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("mcp", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "dryRun": true,
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [ { "action": "fill_fields", "data": {"회사명": "MCP미리보기"} } ],
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": "hwp_run_plan", "arguments": { "plan": plan } }
        })
    )
    .unwrap();
    stdin.flush().unwrap();

    let mut line = String::new();
    assert!(stdout.read_line(&mut line).unwrap() > 0);
    let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    let text = v["result"]["content"][0]["text"].as_str().expect("본문");
    let body: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(body["dryRun"], true, "{body}");
    assert!(!out.exists(), "MCP 경로도 디스크 무변경");

    let _ = child.kill();
    let _ = child.wait();
}
