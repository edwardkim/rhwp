//! [#4391] 작업 영수증 계약 — `rhwp replay`.
//!
//! 고정하는 것: ① attest 가 (입력·계획·산출) SHA-256 3종을 발급하고 사용자
//! 경로에 아무것도 쓰지 않는다, ② **결정론** — 같은 계획의 두 영수증이 같은
//! outputSha256 을 낸다(제3자 재현의 전제), ③ verify 는 일치 exit 0 /
//! 불일치 exit 3(reproduced:false), ④ 형식 오류는 exit 2.

#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

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

fn existing_snippet() -> String {
    let o = run(&["export-text", SAMPLE, "-p", "0", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    let text = env["pages"][0]["text"].as_str().expect("쪽 텍스트");
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    chars[..2].iter().collect()
}

/// 계획의 output 은 존재하지 않는 경로 — replay 가 이 경로를 건드리면 안 된다.
fn plan(find: &str, claimed_out: &std::path::Path) -> String {
    serde_json::json!({
        "planVersion": "1.0",
        "input": SAMPLE,
        "output": claimed_out.to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": find, "replace": find }],
    })
    .to_string()
}

fn attest(plan_text: &str) -> serde_json::Value {
    let o = run(&["replay", "--plan-json", plan_text, "--json"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "attest 실패: {}",
        String::from_utf8_lossy(&o.stdout)
    );
    serde_json::from_slice(&o.stdout).expect("영수증 봉투")
}

#[test]
fn attest_issues_three_hashes_and_touches_no_user_paths() {
    let claimed = std::env::temp_dir().join("replay_contract_claimed_untouched.hwp");
    let _ = std::fs::remove_file(&claimed);
    let receipt = attest(&plan(&existing_snippet(), &claimed));
    assert_eq!(receipt["mode"], "attest");
    for key in ["inputSha256", "planSha256", "outputSha256"] {
        assert_eq!(
            receipt[key].as_str().map(str::len),
            Some(64),
            "{key} 는 64자리 hex: {receipt}"
        );
    }
    assert_eq!(receipt["reproduced"], serde_json::Value::Null);
    assert!(receipt["steps"].as_u64().unwrap_or(0) >= 1);
    assert!(
        !claimed.exists(),
        "replay 는 계획의 output 경로를 건드리지 않는다 — 임시 산출만 쓴다"
    );
}

#[test]
fn two_attests_are_byte_deterministic() {
    let claimed = std::env::temp_dir().join("replay_contract_det.hwp");
    let find = existing_snippet();
    let a = attest(&plan(&find, &claimed));
    let b = attest(&plan(&find, &claimed));
    assert_eq!(
        a["outputSha256"], b["outputSha256"],
        "같은 계획의 두 재실행은 같은 산출 바이트를 내야 한다 — 제3자 재현의 전제"
    );
    assert_eq!(a["inputSha256"], b["inputSha256"]);
    assert_eq!(a["planSha256"], b["planSha256"]);
}

#[test]
fn verify_matches_own_receipt_and_rejects_wrong_claim() {
    let claimed = std::env::temp_dir().join("replay_contract_verify.hwp");
    let find = existing_snippet();
    let plan_text = plan(&find, &claimed);
    let receipt = attest(&plan_text);
    let out_sha = receipt["outputSha256"].as_str().unwrap().to_string();

    // 일치 — 재현 성공.
    let ok = run(&[
        "replay",
        "--plan-json",
        &plan_text,
        "--expect-output-sha256",
        &out_sha,
        "--json",
    ]);
    assert_eq!(ok.status.code(), Some(0));
    let env: serde_json::Value = serde_json::from_slice(&ok.stdout).unwrap();
    assert_eq!(env["mode"], "verify");
    assert_eq!(env["reproduced"], true);

    // 불일치 — 주장 기각(exit 3), 판정은 봉투 데이터.
    let bad = run(&[
        "replay",
        "--plan-json",
        &plan_text,
        "--expect-output-sha256",
        ZERO64,
        "--json",
    ]);
    assert_eq!(bad.status.code(), Some(3), "재현 불일치 = 검증 단언 실패");
    let env: serde_json::Value = serde_json::from_slice(&bad.stdout).unwrap();
    assert_eq!(env["reproduced"], false);
    assert_eq!(env["expectedOutputSha256"], ZERO64);
}

#[test]
fn malformed_expected_hash_is_usage_error() {
    let claimed = std::env::temp_dir().join("replay_contract_bad.hwp");
    let o = run(&[
        "replay",
        "--plan-json",
        &plan(&existing_snippet(), &claimed),
        "--expect-output-sha256",
        "xyz",
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2));
}
