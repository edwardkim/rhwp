//! [#4393] 에이전트 노동 감사 계약 — 작업 캡슐(`replay --capsule`)과 `audit`.
//!
//! 고정하는 것: ① 캡슐 왕복 — 발급한 캡슐은 감사에서 재현된다, ② 변조 검출 —
//! 영수증 해시를 바꾼 캡슐은 failed[] 에 기대/실제와 함께 잡히고 exit 3,
//! ③ 빈 폴더는 exit 2 + stdout 0바이트(실패 stdout 순수성), ④ 재현율 회계
//! (total·reproduced·reproducedRate)가 봉투 데이터다.

#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

const SAMPLE: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn existing_snippet() -> String {
    let o = run(&["export-text", SAMPLE, "-p", "0", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    let text = env["pages"][0]["text"].as_str().expect("쪽 텍스트");
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    chars[..2].iter().collect()
}

fn make_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp_audit_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("감사 폴더");
    dir
}

fn issue_capsule(dir: &std::path::Path, name: &str, find: &str) {
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": SAMPLE,
        "output": dir.join("claimed.hwp").to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": find, "replace": find }],
    })
    .to_string();
    let capsule = dir.join(name);
    let o = run(&[
        "replay",
        "--plan-json",
        &plan,
        "--capsule",
        capsule.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "캡슐 발급 실패\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(capsule.exists(), "캡슐 파일이 만들어져야 한다");
}

fn audit(dir: &std::path::Path) -> (Option<i32>, serde_json::Value) {
    let o = run(&["audit", dir.to_str().unwrap(), "--json"]);
    let env = serde_json::from_slice(&o.stdout).unwrap_or(serde_json::json!({}));
    (o.status.code(), env)
}

#[test]
fn issued_capsules_reproduce_and_tampered_one_is_caught() {
    let dir = make_dir("roundtrip");
    let find = existing_snippet();
    issue_capsule(&dir, "a.capsule.json", &find);
    issue_capsule(&dir, "b.capsule.json", &find);

    // 전건 재현 — exit 0, 재현율 1.0.
    let (code, env) = audit(&dir);
    assert_eq!(code, Some(0), "{env}");
    assert_eq!(env["total"], 2);
    assert_eq!(env["reproduced"], 2);
    assert_eq!(env["reproducedRate"], 1.0);
    assert_eq!(env["failed"].as_array().map(Vec::len), Some(0));

    // b 를 변조 — 영수증의 outputSha256 을 0으로.
    let b = dir.join("b.capsule.json");
    let mut capsule: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&b).unwrap()).unwrap();
    capsule["receipt"]["outputSha256"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    std::fs::write(&b, serde_json::to_string_pretty(&capsule).unwrap()).unwrap();

    let (code, env) = audit(&dir);
    assert_eq!(code, Some(3), "변조 1건 = 검증 단언 실패: {env}");
    assert_eq!(env["total"], 2);
    assert_eq!(env["reproduced"], 1);
    let failed = env["failed"].as_array().expect("failed 배열");
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["capsule"], "b.capsule.json");
    assert_eq!(failed[0]["actual"].as_str().map(str::len), Some(64));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tampered_input_receipt_is_caught_before_output_credit() {
    let dir = make_dir("input_tamper");
    issue_capsule(&dir, "input.capsule.json", &existing_snippet());
    let path = dir.join("input.capsule.json");
    let mut capsule: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    capsule["receipt"]["inputSha256"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    std::fs::write(&path, serde_json::to_string_pretty(&capsule).unwrap()).unwrap();

    let (code, env) = audit(&dir);
    assert_eq!(code, Some(3), "입력 영수증 변조는 감사 실패: {env}");
    assert_eq!(env["reproduced"], 0);
    let failure = &env["failed"][0];
    assert_eq!(failure["kind"], "inputSha256");
    assert_eq!(failure["actual"].as_str().map(str::len), Some(64));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn output_neutral_plan_and_receipt_step_tampering_are_caught() {
    let dir = make_dir("plan_tamper");
    let find = existing_snippet();
    issue_capsule(&dir, "plan.capsule.json", &find);
    issue_capsule(&dir, "plan-text.capsule.json", &find);
    issue_capsule(&dir, "steps.capsule.json", &find);

    // replay는 output을 scratch 경로로 바꾸므로 이 plan 필드만 바꾸면 산출은 같다.
    // 그래도 raw planText와 불일치하므로 감사 성공으로 집계하면 안 된다.
    let plan_path = dir.join("plan.capsule.json");
    let mut plan_capsule: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&plan_path).unwrap()).unwrap();
    plan_capsule["plan"]["output"] = serde_json::json!("output-neutral-tamper.hwp");
    std::fs::write(
        &plan_path,
        serde_json::to_string_pretty(&plan_capsule).unwrap(),
    )
    .unwrap();

    // parsed plan과 planText를 함께 바꿔도 원문 receipt.planSha256과 불일치한다.
    let plan_text_path = dir.join("plan-text.capsule.json");
    let mut plan_text_capsule: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&plan_text_path).unwrap()).unwrap();
    let mut changed_plan: serde_json::Value =
        serde_json::from_str(plan_text_capsule["planText"].as_str().unwrap()).unwrap();
    changed_plan["output"] = serde_json::json!("another-output-neutral-tamper.hwp");
    plan_text_capsule["plan"] = changed_plan.clone();
    plan_text_capsule["planText"] =
        serde_json::json!(serde_json::to_string(&changed_plan).unwrap());
    std::fs::write(
        &plan_text_path,
        serde_json::to_string_pretty(&plan_text_capsule).unwrap(),
    )
    .unwrap();

    let steps_path = dir.join("steps.capsule.json");
    let mut steps_capsule: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&steps_path).unwrap()).unwrap();
    steps_capsule["receipt"]["steps"] = serde_json::json!(999);
    std::fs::write(
        &steps_path,
        serde_json::to_string_pretty(&steps_capsule).unwrap(),
    )
    .unwrap();

    let (code, env) = audit(&dir);
    assert_eq!(code, Some(3), "계획·step 영수증 변조는 감사 실패: {env}");
    assert_eq!(env["reproduced"], 0);
    let failures = env["failed"].as_array().expect("실패 목록");
    assert_eq!(failures.len(), 3);
    let failure_for = |name: &str| {
        failures
            .iter()
            .find(|failure| failure["capsule"] == name)
            .unwrap_or_else(|| panic!("{name} 실패 없음: {env}"))
    };
    assert!(failure_for("plan.capsule.json")["error"]
        .as_str()
        .unwrap_or_default()
        .contains("plan 과 planText"));
    assert!(failure_for("plan-text.capsule.json")["error"]
        .as_str()
        .unwrap_or_default()
        .contains("planText 와 receipt.planSha256"));
    assert!(failure_for("steps.capsule.json")["error"]
        .as_str()
        .unwrap_or_default()
        .contains("receipt.steps 와 planText.steps"));
    assert!(failure_for("steps.capsule.json")["error"]
        .as_str()
        .unwrap_or_default()
        .contains("plan.steps 길이와 receipt.steps"));
    let _ = std::fs::remove_dir_all(&dir);
}

// APFS rejects invalid UTF-8 path components with EILSEQ, so macOS cannot
// construct the fixture required to exercise this Unix-byte contract.
#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn non_utf8_capsule_name_is_included_and_reported() {
    use std::os::unix::ffi::OsStringExt;

    let dir = make_dir("non_utf8");
    let name = std::ffi::OsString::from_vec(b"\xff-bad.capsule.json".to_vec());
    std::fs::write(dir.join(name), b"{}").unwrap();

    let (code, env) = audit(&dir);
    assert_eq!(code, Some(3), "비 UTF-8 이름 capsule도 감사 대상: {env}");
    assert_eq!(env["total"], 1);
    assert_eq!(env["reproduced"], 0);
    let reported = env["failed"][0]["capsule"].as_str().unwrap_or_default();
    assert!(!reported.is_empty());
    assert!(reported.ends_with("-bad.capsule.json"), "{reported:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn empty_dir_is_usage_error_with_silent_stdout() {
    let dir = make_dir("empty");
    let o = run(&["audit", dir.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(
        o.stdout.is_empty(),
        "실패 경로 stdout 은 0바이트: {}",
        String::from_utf8_lossy(&o.stdout)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
