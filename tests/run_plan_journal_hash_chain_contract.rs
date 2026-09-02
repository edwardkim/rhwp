//! [#4378 R23] `run` 저널 지문 체인 — 연속 실행의 `outputSha256` = `inputSha256`.
//!
//! R22 는 계획서 `preconditions.inputSha256` 로 낡은 기준의 실행을 **거절**한다
//! (CAS, 사전 차단). R23 은 그와 다른 축이다 — 도구는 매 성공 실행마다 저널에
//! `inputSha256`/`outputSha256` 지문만 낸다. 그 지문을 이어 붙여 "누가 어떤
//! 상태에서 무엇을 바꿨나"를 재구성하는 것은 호출자(무상태 CLI 철학) 몫이므로,
//! 이 계약이 고정하는 것은 재구성이 **가능함**이다:
//!
//! 1. 연속 실행: 저널 1의 `outputSha256` = 저널 2의 `inputSha256`.
//! 2. 불연속(다른 도구가 중간에 문서를 건드림): 두 값이 달라 저널 비교만으로
//!    드러난다 — 단, `preconditions` 없이 호출했으므로 실행 자체는 막히지
//!    않는다(R23 은 탐지이지 R22 의 차단이 아니다).

#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

use sha2::{Digest, Sha256};

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

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn tmp(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "journal_hash_chain_{name}_{}_{nonce}.hwp",
        std::process::id()
    ))
}

/// 대상 문서 1쪽 본문에서 실제로 존재하는 2글자 조각 — replace_text 선검증
/// (매치 존재)을 통과시키기 위한 실측 기반 find 값.
fn existing_snippet(path: &std::path::Path) -> String {
    let o = run(&["export-text", path.to_str().unwrap(), "-p", "0", "--json"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    let text = env["pages"][0]["text"].as_str().expect("쪽 텍스트");
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(chars.len() >= 2, "샘플 1쪽에 본문이 있어야 한다");
    chars[..2].iter().collect()
}

fn run_plan_in_place(path: &std::path::Path, find: &str, replace: &str) -> serde_json::Value {
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": path.to_string_lossy(),
        "output": path.to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": find, "replace": replace }],
    })
    .to_string();
    let o = run(&["run", "--plan-json", &plan, "--json"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "run 실행 실패: {}",
        String::from_utf8_lossy(&o.stdout)
    );
    serde_json::from_slice(&o.stdout).expect("run 저널 봉투")
}

fn is_sha256_hex(v: &serde_json::Value) -> bool {
    v.as_str()
        .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// 저널이 `inputSha256`/`outputSha256` 을 64자리 16진으로 싣는다 — 필드 존재 자체의
/// 최소 계약. 필드가 없던 시절에는 이 단언이 곧 red 였다.
#[test]
fn journal_carries_input_and_output_sha256() {
    let doc = tmp("fields_present");
    std::fs::copy(SAMPLE, &doc).expect("샘플 복사");
    let find = existing_snippet(&doc);
    let journal = run_plan_in_place(&doc, &find, &find);
    assert!(
        is_sha256_hex(&journal["inputSha256"]),
        "inputSha256 이 64자리 16진이 아니다: {journal}"
    );
    assert!(
        is_sha256_hex(&journal["outputSha256"]),
        "outputSha256 이 64자리 16진이 아니다: {journal}"
    );
    let _ = std::fs::remove_file(&doc);
}

/// 저널의 `inputSha256` 이 실제로 실행에 쓰인 입력 바이트의 해시와 같다 — R22 와
/// 같은 해시 함수(파일 바이트 sha256)를 재사용했음을 실증한다.
#[test]
fn journal_input_sha256_matches_actual_input_bytes() {
    let doc = tmp("input_matches");
    std::fs::copy(SAMPLE, &doc).expect("샘플 복사");
    let original_bytes = std::fs::read(&doc).expect("원본 읽기");
    let expected = sha256_hex(&original_bytes);
    let find = existing_snippet(&doc);
    let journal = run_plan_in_place(&doc, &find, &find);
    assert_eq!(journal["inputSha256"], expected, "{journal}");
    let _ = std::fs::remove_file(&doc);
}

/// 저널의 `outputSha256` 이 실제로 디스크에 쓰인 산출 바이트의 해시와 같다.
#[test]
fn journal_output_sha256_matches_actual_written_bytes() {
    let doc = tmp("output_matches");
    std::fs::copy(SAMPLE, &doc).expect("샘플 복사");
    let find = existing_snippet(&doc);
    let journal = run_plan_in_place(&doc, &find, &find);
    let written = std::fs::read(&doc).expect("산출 읽기");
    let expected = sha256_hex(&written);
    assert_eq!(journal["outputSha256"], expected, "{journal}");
    let _ = std::fs::remove_file(&doc);
}

/// DoD ① — 연속 실행: 앞 저널의 outputSha256 = 뒤 저널의 inputSha256.
#[test]
fn consecutive_runs_chain_output_to_input() {
    let doc = tmp("chain_continuous");
    std::fs::copy(SAMPLE, &doc).expect("샘플 복사");

    let find1 = existing_snippet(&doc);
    let journal1 = run_plan_in_place(&doc, &find1, &find1);

    let find2 = existing_snippet(&doc);
    let journal2 = run_plan_in_place(&doc, &find2, &find2);

    // Null == Null 로 통과하는 무의미한 일치를 막는다 — 두 값이 실제 지문이어야
    // "연속"이라는 단언에 뜻이 있다.
    assert!(is_sha256_hex(&journal1["outputSha256"]), "{journal1}");
    assert!(is_sha256_hex(&journal2["inputSha256"]), "{journal2}");
    assert_eq!(
        journal1["outputSha256"], journal2["inputSha256"],
        "연속 실행이면 앞 outputSha256 = 뒤 inputSha256 이어야 한다\n저널1: {journal1}\n저널2: {journal2}"
    );
    let _ = std::fs::remove_file(&doc);
}

/// DoD ② — 불연속: 다른 도구(`edit replace-text`, `run` 이 아닌 별도 진입점)가
/// 중간에 문서를 건드리면 저널 비교만으로 그 사실이 드러난다. `preconditions`
/// 를 주지 않았으므로 실행 자체는 막히지 않는다(R23 = 탐지, R22 = 차단).
#[test]
fn interleaved_edit_by_another_tool_breaks_the_chain_detectably() {
    let doc = tmp("chain_broken");
    std::fs::copy(SAMPLE, &doc).expect("샘플 복사");

    let find1 = existing_snippet(&doc);
    let journal1 = run_plan_in_place(&doc, &find1, &find1);

    // "다른 도구" — run 계획서가 아니라 단발 edit 진입점으로 같은 문서를 고친다.
    let find_for_other_tool = existing_snippet(&doc);
    let o = run(&[
        "edit",
        "replace-text",
        doc.to_str().unwrap(),
        "--find",
        &find_for_other_tool,
        "--replace",
        "다른도구",
        "-o",
        doc.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "끼어든 편집 실패: {}",
        String::from_utf8_lossy(&o.stdout)
    );

    // preconditions 없이 계획 2를 실행 — R22 의 차단이 아니라 R23 의 탐지를 본다.
    let find2 = existing_snippet(&doc);
    let journal2 = run_plan_in_place(&doc, &find2, &find2);

    assert_ne!(
        journal1["outputSha256"], journal2["inputSha256"],
        "끼어든 편집이 있었는데도 저널이 연속으로 보이면 사슬이 탐지력을 잃은 것이다\n저널1: {journal1}\n저널2: {journal2}"
    );
    let _ = std::fs::remove_file(&doc);
}
