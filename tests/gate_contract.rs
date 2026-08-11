//! [#4545] 게이트 계약 — 반입 정책 기계 판정.
//!
//! 고정하는 것: ① 전 축 규칙(reproduced·lineageValid·signerVerdict·
//! anchoredOk)이 재계산으로 allow, ② 위반은 violations[{rule,key,expected,
//! actual}] 로 명세되고 exit 3, ③ **미지 판정 키·연산자 = 로드 시점 exit 2**
//! (오타가 항상-참이 되는 구멍 차단), ④ **deny 기본** — 빈 규칙은 통과가
//! 아니다, ⑤ 판정 재료 미지정(예: --keyring 없이 signer 규칙)은 통과가 아니라
//! unavailable 위반, ⑥ 정책 서명 보고(policySigned — 4년 축 재사용).

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

fn make_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp_gate_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("작업 폴더");
    dir
}

/// 서명·등재까지 갖춘 캡슐 하나를 준비한다 — (캡슐, 키링, 앵커로그).
fn full_capsule(dir: &std::path::Path) -> (String, String, String) {
    let find = existing_snippet();
    let key = dir.join("k.json");
    let o = run(&[
        "keygen",
        "--key-id",
        "gate.test/agent#1",
        "--out",
        key.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let kd: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&key).unwrap()).unwrap();
    let keyring = dir.join("keyring.json");
    std::fs::write(
        &keyring,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "keyring",
            "keys": [{ "keyId": "gate.test/agent#1", "publicKey": kd["publicKey"], "revoked": null }],
        })
        .to_string(),
    )
    .unwrap();
    let cap = dir.join("work.capsule.json");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": SAMPLE,
        "output": dir.join("out.hwp").to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": find, "replace": find }],
    })
    .to_string();
    let o = run(&[
        "replay",
        "--plan-json",
        &plan,
        "--capsule",
        cap.to_str().unwrap(),
        "--sign-key",
        key.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let log = dir.join("anchor.ndjson");
    let o = run(&[
        "anchor",
        "add",
        cap.to_str().unwrap(),
        "--log",
        log.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    (
        cap.to_string_lossy().into_owned(),
        keyring.to_string_lossy().into_owned(),
        log.to_string_lossy().into_owned(),
    )
}

fn write_policy(dir: &std::path::Path, name: &str, body: serde_json::Value) -> String {
    let path = dir.join(name);
    std::fs::write(&path, body.to_string()).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn full_axis_allow_and_tamper_deny() {
    let dir = make_dir("allow");
    let (cap, keyring, log) = full_capsule(&dir);
    let policy = write_policy(
        &dir,
        "p.json",
        serde_json::json!({
            "kind": "admissionPolicy", "name": "전축", "defaultVerdict": "deny",
            "rules": [
                { "id": "R1-재현", "require": { "reproduced": { "eq": true } } },
                { "id": "R2-계보", "require": { "lineageValid": { "eq": true } } },
                { "id": "R3-서명", "require": { "signerVerdict": { "eq": "valid" },
                                               "signerKeyId": { "in": ["gate.test/agent#1"] } } },
                { "id": "R4-앵커", "require": { "anchoredOk": { "eq": true } } },
                { "id": "R5-깊이", "require": { "lineageDepth": { "lte": 10 } } }
            ],
        }),
    );
    let o = run(&[
        "gate",
        &cap,
        "--policy",
        &policy,
        "--keyring",
        &keyring,
        "--anchor-log",
        &log,
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
    assert_eq!(env["verdict"], "allow", "{env}");
    assert_eq!(env["evaluated"], 6);
    assert_eq!(env["violations"].as_array().map(Vec::len), Some(0));

    // 캡슐 후행 공백 변조 → 서명 invalid + 앵커 미등재 → deny, 위반 명세.
    let mut bytes = std::fs::read(&cap).unwrap();
    bytes.push(b' ');
    std::fs::write(&cap, &bytes).unwrap();
    let o = run(&[
        "gate",
        &cap,
        "--policy",
        &policy,
        "--keyring",
        &keyring,
        "--anchor-log",
        &log,
        "--deep",
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "변조 반입 거부");
    let env = env_of(&o);
    assert_eq!(env["verdict"], "deny");
    let rules: Vec<&str> = env["violations"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["rule"].as_str())
        .collect();
    assert!(rules.contains(&"R3-서명"), "{env}");
    assert!(rules.contains(&"R4-앵커"), "{env}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unknown_key_and_operator_are_load_errors() {
    let dir = make_dir("typo");
    let (cap, _, _) = full_capsule(&dir);
    // 오타 키 — 항상-참 구멍이 아니라 로드 거부.
    let p1 = write_policy(
        &dir,
        "typo.json",
        serde_json::json!({
            "kind": "admissionPolicy", "name": "오타", "rules":
                [{ "id": "R1", "require": { "reproducd": { "eq": true } } }],
        }),
    );
    let o = run(&["gate", &cap, "--policy", &p1, "--json"]);
    assert_eq!(o.status.code(), Some(2), "미지 판정 키는 사용법 오류");
    // 미지 연산자.
    let p2 = write_policy(
        &dir,
        "op.json",
        serde_json::json!({
            "kind": "admissionPolicy", "name": "연산자", "rules":
                [{ "id": "R1", "require": { "reproduced": { "regex": ".*" } } }],
        }),
    );
    let o = run(&["gate", &cap, "--policy", &p2, "--json"]);
    assert_eq!(o.status.code(), Some(2), "미지 연산자는 사용법 오류");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn deny_default_and_unavailable_judgment() {
    let dir = make_dir("deny");
    let (cap, _, _) = full_capsule(&dir);
    // 빈 규칙 + deny 기본 = 거부.
    let p = write_policy(
        &dir,
        "empty.json",
        serde_json::json!({
            "kind": "admissionPolicy", "name": "빈", "defaultVerdict": "deny", "rules": [],
        }),
    );
    let o = run(&["gate", &cap, "--policy", &p, "--json"]);
    assert_eq!(o.status.code(), Some(3), "빈 규칙은 통과가 아니다");

    // 판정 재료 미지정 — --keyring 없이 서명 규칙 → unavailable 위반.
    let p = write_policy(
        &dir,
        "sig.json",
        serde_json::json!({
            "kind": "admissionPolicy", "name": "서명요구", "rules":
                [{ "id": "R1", "require": { "signerVerdict": { "eq": "valid" } } }],
        }),
    );
    let o = run(&["gate", &cap, "--policy", &p, "--json"]);
    assert_eq!(o.status.code(), Some(3), "모르는 것은 통과시키지 않는다");
    let env = env_of(&o);
    assert!(
        env["violations"][0]["actual"]
            .as_str()
            .unwrap_or("")
            .contains("unavailable"),
        "{env}"
    );

    // reproduced 규칙 + --deep 없음 → 역시 unavailable (신고를 읽지 않는다).
    let p = write_policy(
        &dir,
        "repro.json",
        serde_json::json!({
            "kind": "admissionPolicy", "name": "재현요구", "rules":
                [{ "id": "R1", "require": { "reproduced": { "eq": true } } }],
        }),
    );
    let o = run(&["gate", &cap, "--policy", &p, "--json"]);
    assert_eq!(o.status.code(), Some(3), "재현은 재실행 없이 말할 수 없다");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn policy_signature_report() {
    let dir = make_dir("psig");
    let (cap, keyring, _) = full_capsule(&dir);
    let p = write_policy(
        &dir,
        "p.json",
        serde_json::json!({
            "kind": "admissionPolicy", "name": "서명정책", "rules":
                [{ "id": "R1", "require": { "lineageValid": { "eq": true } } }],
        }),
    );
    // 미서명 정책 + --policy-keyring → policySigned:false (보고 필드).
    let o = run(&[
        "gate",
        &cap,
        "--policy",
        &p,
        "--policy-keyring",
        &keyring,
        "--json",
    ]);
    assert_eq!(env_of(&o)["policySigned"], false);
    // --policy-keyring 없으면 null (판정 축 꺼짐).
    let o = run(&["gate", &cap, "--policy", &p, "--json"]);
    assert_eq!(env_of(&o)["policySigned"], serde_json::Value::Null);
    let _ = std::fs::remove_dir_all(&dir);
}
