//! [#4378 R22×R24] CAS 계약 — 계획서 `preconditions.inputSha256` 와
//! `edit replace-text --expect-sha256`.
//!
//! 고정하는 것: ① 해시 불일치면 **실행 0·저장 0**(run: exit 2 +
//! invalid[].code=preconditionFailed / edit: exit 3 + preconditionFailed 봉투),
//! ② 일치하면 종전과 동일하게 완주, ③ 형식 오류(64자리 16진 아님)는 exit 2,
//! ④ planSchema 가 1.1 과 Preconditions 정의를 자기서술.
//!
//! 근거: #3905 M1 — 두 에이전트의 exit 0 이 편집 하나를 무신호로 지우는 경합
//! 실측. CAS 는 그 유실의 차단기다.

#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

use sha2::{Digest, Sha256};

const SAMPLE: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";
const ZERO64: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn sample_sha256() -> String {
    let bytes = std::fs::read(SAMPLE).expect("샘플 읽기");
    let out = Sha256::digest(&bytes);
    out.iter().map(|b| format!("{b:02x}")).collect()
}

/// 샘플 1쪽 본문에서 실제로 존재하는 2글자 조각을 찾는다 — 계획의 replace_text
/// 선검증(매치 존재)을 통과시키기 위한 실측 기반 find 값.
fn existing_snippet() -> String {
    let o = run(&["export-text", SAMPLE, "-p", "0", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    let text = env["pages"][0]["text"].as_str().expect("쪽 텍스트");
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(chars.len() >= 2, "샘플 1쪽에 본문이 있어야 한다");
    chars[..2].iter().collect()
}

fn plan_json(sha: Option<&str>, output: &std::path::Path, find: &str) -> String {
    let mut plan = serde_json::json!({
        "planVersion": "1.0",
        "input": SAMPLE,
        "output": output.to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": find, "replace": find }],
    });
    if let Some(sha) = sha {
        plan["preconditions"] = serde_json::json!({ "inputSha256": sha });
    }
    plan.to_string()
}

fn in_place_plan_json(input: &std::path::Path, sha: &str, find: &str, replace: &str) -> String {
    serde_json::json!({
        "planVersion": "1.0",
        "input": input.to_string_lossy(),
        "output": input.to_string_lossy(),
        "preconditions": { "inputSha256": sha },
        "steps": [{ "action": "replace_text", "find": find, "replace": replace }],
    })
    .to_string()
}

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("cas_contract_{name}.hwp"))
}

// ── run 계획서 축 (R22) ──────────────────────────────────────────────────

#[test]
fn plan_with_wrong_sha_is_rejected_without_writing() {
    let out = tmp("plan_wrong");
    let _ = std::fs::remove_file(&out);
    let find = existing_snippet();
    let o = run(&[
        "run",
        "--plan-json",
        &plan_json(Some(ZERO64), &out, &find),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2), "계획 무효 = exit 2");
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    let inv = &env["invalid"][0];
    assert_eq!(inv["code"], "preconditionFailed", "{env}");
    assert_eq!(inv["expected"], ZERO64);
    assert_eq!(inv["actual"].as_str().map(str::len), Some(64));
    assert!(!out.exists(), "거절 시 산출물이 있어서는 안 된다");
}

#[test]
fn plan_with_correct_sha_completes() {
    let out = tmp("plan_ok");
    let _ = std::fs::remove_file(&out);
    let sha = sample_sha256();
    let find = existing_snippet();
    let o = run(&[
        "run",
        "--plan-json",
        &plan_json(Some(&sha), &out, &find),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "일치 시 종전과 동일 완주: {}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert!(out.exists(), "정상 완주는 산출물을 쓴다");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn plan_with_malformed_sha_is_usage_error() {
    let out = tmp("plan_bad");
    let find = existing_snippet();
    let o = run(&[
        "run",
        "--plan-json",
        &plan_json(Some("xyz"), &out, &find),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2));
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    assert!(
        env["error"].as_str().unwrap_or_default().contains("64자리"),
        "{env}"
    );
}

#[test]
fn plan_with_non_string_precondition_is_usage_error() {
    let out = tmp("plan_non_string");
    let find = existing_snippet();
    for value in [
        serde_json::Value::Null,
        serde_json::json!(42),
        serde_json::json!(true),
        serde_json::json!({ "sha": ZERO64 }),
    ] {
        let mut plan: serde_json::Value =
            serde_json::from_str(&plan_json(None, &out, &find)).expect("계획 JSON");
        plan["preconditions"] = serde_json::json!({ "inputSha256": value });
        let o = run(&["run", "--plan-json", &plan.to_string(), "--json"]);
        assert_eq!(o.status.code(), Some(2), "잘못된 타입은 거절: {plan}");
        let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
        assert!(
            env["error"].as_str().unwrap_or_default().contains("문자열"),
            "{env}"
        );
        assert!(!out.exists(), "잘못된 전제조건은 산출물을 쓰지 않는다");
    }
}

#[test]
fn plan_with_non_object_preconditions_is_usage_error() {
    let out = tmp("plan_non_object");
    let find = existing_snippet();
    let mut plan: serde_json::Value =
        serde_json::from_str(&plan_json(None, &out, &find)).expect("계획 JSON");
    plan["preconditions"] = serde_json::json!([]);
    let o = run(&["run", "--plan-json", &plan.to_string(), "--json"]);
    assert_eq!(o.status.code(), Some(2));
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    assert!(
        env["error"].as_str().unwrap_or_default().contains("객체"),
        "{env}"
    );
    assert!(!out.exists());
}

#[test]
fn present_preconditions_require_exactly_input_sha256() {
    let out = tmp("plan_missing_key");
    let find = existing_snippet();
    for preconditions in [
        serde_json::json!({}),
        serde_json::json!({ "inputSha265": ZERO64 }),
        serde_json::json!({ "inputSha256": ZERO64, "unexpected": true }),
    ] {
        let mut plan: serde_json::Value =
            serde_json::from_str(&plan_json(None, &out, &find)).expect("계획 JSON");
        plan["preconditions"] = preconditions;
        let o = run(&["run", "--plan-json", &plan.to_string(), "--json"]);
        assert_eq!(o.status.code(), Some(2), "누락·미지 키는 거절: {plan}");
        assert!(!out.exists(), "잘못된 전제조건은 산출물을 쓰지 않는다");
    }
}

#[test]
#[cfg(debug_assertions)]
fn concurrent_in_place_plans_with_one_expected_hash_cannot_both_commit() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let input =
        std::env::temp_dir().join(format!("rhwp_cas_race_{}_{nonce}.hwp", std::process::id()));
    std::fs::copy(SAMPLE, &input).expect("경합용 입력 복사");
    let original = std::fs::read(&input).expect("경합용 입력 읽기");
    let expected = Sha256::digest(&original)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let find = existing_snippet();
    let plan_a = in_place_plan_json(&input, &expected, &find, "가");
    let plan_b = in_place_plan_json(&input, &expected, &find, "나");
    let barrier =
        std::env::temp_dir().join(format!("rhwp_cas_barrier_{}_{nonce}", std::process::id()));
    std::fs::create_dir(&barrier).expect("CAS barrier 폴더");

    let mut first = Command::new(rhwp_bin())
        .args(["run", "--plan-json", &plan_a, "--json"])
        .env("RHWP_INTERNAL_TEST_CAS_BARRIER", &barrier)
        .spawn()
        .expect("첫 CAS 실행");
    let mut second = Command::new(rhwp_bin())
        .args(["run", "--plan-json", &plan_b, "--json"])
        .env("RHWP_INTERNAL_TEST_CAS_BARRIER", &barrier)
        .spawn()
        .expect("둘째 CAS 실행");
    let first_status = first.wait().expect("첫 CAS 종료").code();
    let second_status = second.wait().expect("둘째 CAS 종료").code();
    let mut codes = [first_status, second_status];
    codes.sort();
    assert_eq!(codes, [Some(0), Some(2)], "정확히 한 실행만 commit");
    let checked = std::fs::read_dir(&barrier)
        .expect("CAS barrier 읽기")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("checked-"))
        .count();
    assert_eq!(
        checked, 1,
        "잠금이 없으면 두 프로세스가 최초 해시 검사를 모두 통과하는 mutation proof"
    );
    assert_ne!(
        Sha256::digest(std::fs::read(&input).expect("최종 입력")),
        Sha256::digest(original),
        "성공한 한 편집은 실제 파일을 바꿔야 한다"
    );
    let _ = std::fs::remove_file(input);
    let _ = std::fs::remove_dir_all(barrier);
}

// ── edit 단발 축 (R24, 기함: replace-text) ───────────────────────────────

#[test]
fn edit_with_wrong_sha_exits_3_without_writing() {
    let out = tmp("edit_wrong");
    let _ = std::fs::remove_file(&out);
    let find = existing_snippet();
    let o = run(&[
        "edit",
        "replace-text",
        SAMPLE,
        "--find",
        &find,
        "--replace",
        &find,
        "-o",
        out.to_str().unwrap(),
        "--expect-sha256",
        ZERO64,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "검증 단언 실패 = exit 3");
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    assert_eq!(env["preconditionFailed"]["kind"], "inputSha256", "{env}");
    assert_eq!(env["preconditionFailed"]["expected"], ZERO64);
    assert!(!out.exists(), "불일치 시 저장 금지");
}

#[test]
fn edit_with_correct_sha_completes() {
    let out = tmp("edit_ok");
    let _ = std::fs::remove_file(&out);
    let sha = sample_sha256();
    let find = existing_snippet();
    let o = run(&[
        "edit",
        "replace-text",
        SAMPLE,
        "--find",
        &find,
        "--replace",
        &find,
        "-o",
        out.to_str().unwrap(),
        "--expect-sha256",
        &sha,
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert!(out.exists());
    let _ = std::fs::remove_file(&out);
}

// ── 자기서술 축 ─────────────────────────────────────────────────────────

#[test]
fn plan_schema_describes_preconditions_and_bumps_version() {
    let o = run(&["export-plan-schema", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    assert_eq!(env["planSchemaVersion"], "1.1", "minor 범프: {env}");
    let schema = &env["schema"];
    assert!(
        schema["$defs"]["Preconditions"]["properties"]["inputSha256"].is_object(),
        "$defs.Preconditions.inputSha256 부재"
    );
    assert!(
        schema["properties"]["preconditions"].is_object()
            || schema["$defs"]["Plan"]["properties"]["preconditions"].is_object(),
        "루트에 preconditions 속성이 서술돼야 한다"
    );
}
