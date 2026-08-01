//! [#3703] 계획 실행기 `rhwp run` — 선언적 편집 계획의 정적 선검증·원자 실행·저널.
//! 계약: 선검증 실패 = exit 2 + 디스크 무변경, 성공 = 저널 봉투 + 단 한 번 저장,
//! 왕복 재독으로 편집 실적용 확인, MCP `hwp_run_plan` 선언.
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
        "rhwp-runplan-{tag}-{}-{}.{ext}",
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

fn write_plan(tag: &str, plan: &serde_json::Value) -> PathBuf {
    let p = temp_path(tag, "json");
    std::fs::write(&p, serde_json::to_vec_pretty(plan).unwrap()).unwrap();
    p
}

/// 선검증 실패는 실행 0 — exit 2 에 출력 파일이 아예 생기지 않아야 한다.
/// 저널은 어느 step 이 왜 불가한지 데이터로 말한다.
#[test]
fn prevalidation_failure_is_exit_2_with_no_output() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("preval", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "검증사"} },
            { "action": "fill_fields", "data": {"존재하지않는필드XYZ": "값"} },
        ],
    });
    let plan_path = write_plan("preval", &plan);
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(2), "선검증 실패는 exit 2");
    assert!(!out.exists(), "실행 0 증명 — 출력 파일 부재");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    let invalid = v["invalid"].as_array().expect("invalid[]");
    assert_eq!(invalid.len(), 1, "{v}");
    assert_eq!(invalid[0]["step"], 1, "0-기반 step 지목: {v}");
    assert!(
        invalid[0]["reason"].as_str().unwrap_or("").contains("존재하지않는필드XYZ"),
        "왜 불가한지: {v}"
    );
    let _ = std::fs::remove_file(&plan_path);
}

/// 중간 step 이 불가하면 앞 step 이 유효해도 디스크 무변경 (자연 트랜잭션).
#[test]
fn mid_plan_invalid_step_leaves_disk_unchanged() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("atomic", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "선행유효"} },
            { "action": "replace_text", "find": "이런문자열은문서에없다9999", "replace": "X" },
        ],
    });
    let plan_path = write_plan("atomic", &plan);
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(2), "중간 불가 = 전체 불가");
    assert!(!out.exists(), "선행 step 도 디스크에 닿지 않는다");
    let _ = std::fs::remove_file(&plan_path);
}

/// 정상 계획: 저널이 step 별 결과와 verify 자기검증을 담고 exit 0.
#[test]
fn journal_reports_steps_and_verify() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("journal", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "계획실행사"} },
        ],
        "assertions": { "notFoundEmpty": true, "verify": true },
    });
    let plan_path = write_plan("journal", &plan);
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_eq!(output.status.code(), Some(0), "{v}");
    assert!(out.exists(), "단언 통과 시에만 단 한 번 저장");
    let steps = v["steps"].as_array().expect("steps[]");
    assert_eq!(steps.len(), 1, "{v}");
    assert_eq!(steps[0]["action"], "fill_fields", "{v}");
    assert_eq!(steps[0]["filledCount"], 1, "{v}");
    assert_eq!(v["verify"]["identical"], true, "자기검증 동봉: {v}");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&plan_path);
}

/// 왕복 재독 — 산출물을 다시 읽어 계획의 편집이 실제 적용됐음을 확인한다.
#[test]
fn rerun_reread_confirms_edits_applied() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("reread", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "왕복재독사"} },
        ],
        "assertions": { "verify": true },
    });
    let plan_path = write_plan("reread", &plan);
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let fields = run(&["fields", out.to_str().unwrap(), "--json"]);
    assert_eq!(fields.status.code(), Some(0));
    let fv: serde_json::Value = serde_json::from_slice(&fields.stdout).expect("fields");
    let text = fv.to_string();
    assert!(text.contains("왕복재독사"), "산출물 재독에 새 값: {fv}");
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&plan_path);
}

/// capabilities --mcp 가 hwp_run_plan 도구를 선언한다 (에이전트 발견 가능성).
#[test]
fn capabilities_declares_run_plan_tool() {
    let output = run(&["capabilities", "--mcp"]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("caps");
    let names: Vec<&str> = v["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"hwp_run_plan"), "{names:?}");
}

/// mcp-serve hwp_run_plan — 인라인 계획 객체로 같은 엔진을 태우고 저널을 돌려준다.
#[test]
fn mcp_run_plan_returns_journal() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("mcp", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [ { "action": "fill_fields", "data": {"회사명": "MCP계획사"} } ],
        "assertions": { "verify": true },
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
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"hwp_run_plan","arguments":{{"plan":{}}}}}}}"#,
        serde_json::to_string(&plan).unwrap()
    )
    .unwrap();
    stdin.flush().unwrap();
    let mut line = String::new();
    assert!(stdout.read_line(&mut line).unwrap() > 0);
    let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(v["result"]["isError"], false, "{v}");
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    let journal: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(journal["verify"]["identical"], true, "{journal}");
    assert!(out.exists());
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_file(&out);
}
