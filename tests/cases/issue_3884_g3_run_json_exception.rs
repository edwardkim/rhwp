//! [#3884 G3] `run` 실패 봉투 예외가 capabilities 에 자기서술되고, 계획 안 문서
//! 부재 실측과 맞는다.
//!
//! `run_plan_engine` 은 MCP `hwp_run_plan` 과 저널을 공유하므로 실패를 stdout
//! 0바이트로 바꾸지 않는다. 소비자는 `jsonContract.failure` 의 예외를 읽고
//! 봉투를 파싱한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::process::{Command, Output};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn describe(args: &[&str], out: &Output) -> String {
    format!(
        "args={args:?}\nexit={:?}\nstdout={}\nstderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn write_plan(tag: &str, plan: &serde_json::Value) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "rhwp-3884-g3-{tag}-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::write(&path, serde_json::to_vec_pretty(plan).expect("plan json")).expect("plan write");
    path
}

fn capabilities_failure() -> String {
    let args = ["capabilities"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("capabilities JSON");
    v["jsonContract"]["failure"]
        .as_str()
        .expect("jsonContract.failure")
        .to_string()
}

#[test]
fn capabilities_failure_declares_run_missing_document_exception() {
    let failure = capabilities_failure();
    assert!(
        failure.contains("run"),
        "run 의 stdout 예외가 자기서술에 없다: {failure}"
    );
    assert!(
        failure.contains("계획 안 문서 부재"),
        "계획 안 문서 부재가 jsonContract.failure 에 없다: {failure}"
    );
    assert!(
        failure.contains("error"),
        "입력 오류 봉투 키 error 가 자기서술에 없다: {failure}"
    );
    assert!(
        failure.contains("invalid"),
        "계획 무효 invalid[] 예외가 자기서술에 없다: {failure}"
    );
}

#[test]
fn run_json_missing_document_emits_error_envelope() {
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": "no-such-file-3884-g3.hwp",
        "output": "out-3884-g3.hwp",
        "steps": [{ "action": "replace_text", "find": "a", "replace": "b" }],
    });
    let plan_path = write_plan("missing-doc", &plan);
    let args = ["run", plan_path.to_str().expect("utf-8"), "--json"];
    let out = run(&args);
    let _ = std::fs::remove_file(&plan_path);

    assert_eq!(
        out.status.code(),
        Some(1),
        "계획 안 문서 부재는 입력 오류(exit 1)다: {}",
        describe(&args, &out)
    );
    assert!(
        !out.stdout.is_empty(),
        "run 실패도 봉투를 stdout 으로 낸다: {}",
        describe(&args, &out)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout JSON 아님 ({e}): {}", describe(&args, &out)));
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    let err = v["error"].as_str().unwrap_or("");
    assert!(
        err.contains("입력을 읽을 수 없습니다"),
        "문서 부재 사유가 error 에 없다: {v}"
    );
    assert!(
        err.contains("no-such-file-3884-g3.hwp"),
        "없는 경로가 error 에 없다: {v}"
    );

    let failure = capabilities_failure();
    assert!(
        failure.contains("exit 1") && failure.contains("error"),
        "자기서술(입력 오류 exit 1 + error)이 실측과 어긋난다: {failure}"
    );
}
