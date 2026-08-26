//! [#4551] 선택적 공개 계약 — 가림 캡슐·부분 개봉·바이트 완전 복원(8년 축).
//!
//! 고정하는 것:
//! ① 가림 왕복 — `disclose redact` 가 만든 가림 캡슐에 **비밀 원문이 없다**
//!    (누설 검사: find/replace 문자열·planText 원문이 파일 바이트에 부재),
//!    구조 골격(planVersion·action)은 공개 유지, ② 부분 개봉 — 개봉 파일에서
//!    필드 몇 개만 추린 부분 개봉이 exit 0·verifiedFields 정확·나머지는
//!    unopened 로 계수, ③ **개봉 위조 검출** — 값 한 글자를 바꾼 개봉은
//!    mismatched·exit 3 (salt 커밋이 사전 대입과 위조를 함께 막는다),
//!    ④ **바이트 완전 복원** — 전체 개봉으로 복원한 캡슐이 원본과 sha256
//!    동일하고, 그래서 **원본의 Ed25519 사이드카가 복원본에서 그대로 valid**
//!    (가림·복원이 4년 서명 축과 어긋나지 않는 급소), ⑤ 방어 — 부분 개봉으로
//!    restore 시도는 커버리지 부족 exit 3, 개봉 kind 오류는 exit 2, 캡슐이
//!    아닌 입력의 redact 는 exit 2.
//!
//! 실행 전제: 없음(임시 폴더 픽스처 자급). 판정은 전부 봉투 데이터다.

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
    let dir = std::env::temp_dir().join(format!("rhwp_disclose_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("작업 폴더");
    dir
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// 서명 키를 발급하고 (키 경로, publicKey b64) 를 돌려준다.
fn keygen(dir: &std::path::Path, key_id: &str) -> (String, String) {
    let key_path = dir.join("signer.key.json");
    let o = run(&[
        "keygen",
        "--key-id",
        key_id,
        "--out",
        key_path.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let env = env_of(&o);
    (
        key_path.to_string_lossy().into_owned(),
        env["publicKey"].as_str().expect("publicKey").to_string(),
    )
}

fn write_keyring(dir: &std::path::Path, key_id: &str, public: &str) -> String {
    let path = dir.join("keyring.json");
    std::fs::write(
        &path,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "keyring",
            "keys": [{ "keyId": key_id, "publicKey": public, "revoked": null }],
        })
        .to_string(),
    )
    .expect("keyring 저장");
    path.to_string_lossy().into_owned()
}

/// 비밀 편집 문자열이 담긴 서명 캡슐을 발급한다.
fn signed_capsule(dir: &std::path::Path, key: &str, secret: &str) -> String {
    let cap = dir.join("secret.capsule.json");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": SAMPLE,
        "output": dir.join("secret.out.hwp").to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": secret, "replace": secret }],
    })
    .to_string();
    let o = run(&[
        "replay",
        "--plan-json",
        &plan,
        "--capsule",
        cap.to_str().unwrap(),
        "--sign-key",
        key,
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "서명 캡슐 발급 실패: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    cap.to_string_lossy().into_owned()
}

/// ①~④ 가림 왕복 — 누설 검사·부분 개봉·위조 검출·바이트 복원+원본 서명 valid.
#[test]
fn 가림_왕복_계약() {
    let dir = make_dir("roundtrip");
    let secret = existing_snippet();
    let (key, public) = keygen(&dir, "agent-2026");
    let keyring = write_keyring(&dir, "agent-2026", &public);
    let capsule = signed_capsule(&dir, &key, &secret);
    let original_bytes = std::fs::read(&capsule).expect("원본 캡슐");
    let original_sha = sha256_hex(&original_bytes);

    // ① 가림 발급 — 커밋 수는 문자열 잎 수(구조 키 제외)와 같아야 한다.
    let redacted = dir.join("secret.redacted.json");
    let opening = dir.join("secret.opening.json");
    let o = run(&[
        "disclose",
        "redact",
        &capsule,
        "-o",
        redacted.to_str().unwrap(),
        "--opening-out",
        opening.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "redact 실패: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    assert_eq!(
        env["originalCapsuleSha256"],
        serde_json::json!(original_sha)
    );
    let committed = env["committedFields"].as_u64().expect("committedFields") as usize;
    assert!(
        committed >= 3,
        "input·output·find·replace 최소 커밋: {committed}"
    );

    // 누설 검사 — 가림 파일 바이트에 비밀 원문이 없어야 한다(이 축의 존재 이유).
    let redacted_text = std::fs::read_to_string(&redacted).expect("가림 캡슐");
    assert!(
        !redacted_text.contains(&secret),
        "가림 캡슐에 비밀 편집 문자열이 샜다"
    );
    let redacted_env: serde_json::Value = serde_json::from_str(&redacted_text).expect("가림 파싱");
    assert_eq!(redacted_env["planRedacted"], serde_json::json!(true));
    // 구조 골격은 공개 유지 — planVersion·action 은 평문이어야 한다.
    assert_eq!(
        redacted_env["plan"]["planVersion"],
        serde_json::json!("1.0")
    );
    assert_eq!(
        redacted_env["plan"]["steps"][0]["action"],
        serde_json::json!("replace_text")
    );
    // 반면 find 는 커밋 객체로 바뀌어야 한다.
    assert!(redacted_env["plan"]["steps"][0]["find"]["committed"].is_string());

    // ② 부분 개봉 — 개봉 파일에서 find 포인터 하나만 추린다.
    let full: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&opening).expect("개봉")).expect("개봉 파싱");
    let find_ptr = "/steps/0/find";
    let partial = dir.join("partial.opening.json");
    std::fs::write(
        &partial,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "capsuleOpening",
            "originalCapsuleSha256": full["originalCapsuleSha256"],
            "openings": { find_ptr: full["openings"][find_ptr] },
        })
        .to_string(),
    )
    .expect("부분 개봉 저장");
    let o = run(&[
        "disclose",
        "verify",
        redacted.to_str().unwrap(),
        "--opening",
        partial.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0), "부분 개봉은 성공 판정");
    let env = env_of(&o);
    assert_eq!(env["verdict"], serde_json::json!("ok"));
    assert_eq!(env["verifiedFields"], serde_json::json!([find_ptr]));
    assert_eq!(env["mismatched"], serde_json::json!([]));
    assert_eq!(env["unopened"], serde_json::json!(committed - 1));

    // ③ 개봉 위조 — 값을 바꾸면 커밋이 어긋나 exit 3.
    let mut forged = full.clone();
    forged["openings"][find_ptr]["value"] = serde_json::json!(format!("{secret}조작"));
    let forged_path = dir.join("forged.opening.json");
    std::fs::write(&forged_path, forged.to_string()).expect("위조 개봉 저장");
    let o = run(&[
        "disclose",
        "verify",
        redacted.to_str().unwrap(),
        "--opening",
        forged_path.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "위조 개봉은 exit 3");
    let env = env_of(&o);
    assert_eq!(env["verdict"], serde_json::json!("mismatch"));
    assert!(
        env["mismatched"]
            .as_array()
            .expect("mismatched")
            .iter()
            .any(|p| p == find_ptr),
        "위조된 포인터가 mismatched 에 있어야 한다"
    );

    // ④ 전체 복원 — 바이트 동일 + 원본 사이드카가 복원본에서 valid.
    let restored = dir.join("restored.capsule.json");
    let o = run(&[
        "disclose",
        "restore",
        redacted.to_str().unwrap(),
        "--opening",
        opening.to_str().unwrap(),
        "-o",
        restored.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "restore 실패: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    assert_eq!(env["byteIdentical"], serde_json::json!(true));
    let restored_bytes = std::fs::read(&restored).expect("복원 캡슐");
    assert_eq!(
        sha256_hex(&restored_bytes),
        original_sha,
        "복원은 바이트 단위 원본 재현이어야 한다"
    );
    let o = run(&[
        "verify-signature",
        restored.to_str().unwrap(),
        "--keyring",
        &keyring,
        "--sig",
        &format!("{capsule}.sig.json"),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0), "원본 서명이 복원본에서 valid");
    assert_eq!(env_of(&o)["verdict"], serde_json::json!("valid"));

    let _ = std::fs::remove_dir_all(&dir);
}

/// ⑤ 방어 — 부분 개봉 restore 거부·kind 오류·비캡슐 redact 거부.
#[test]
fn 가림_방어_계약() {
    let dir = make_dir("defense");
    let secret = existing_snippet();
    let (key, _public) = keygen(&dir, "agent-def");
    let capsule = signed_capsule(&dir, &key, &secret);

    let redacted = dir.join("d.redacted.json");
    let opening = dir.join("d.opening.json");
    let o = run(&[
        "disclose",
        "redact",
        &capsule,
        "-o",
        redacted.to_str().unwrap(),
        "--opening-out",
        opening.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));

    // 부분 개봉으로는 복원 불가 — 커버리지 부족 exit 3 이고 산출물이 없어야 한다.
    let full: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&opening).expect("개봉")).expect("파싱");
    let ptr = "/steps/0/find";
    let partial = dir.join("d.partial.json");
    std::fs::write(
        &partial,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "capsuleOpening",
            "originalCapsuleSha256": full["originalCapsuleSha256"],
            "planText": full["planText"],
            "openings": { ptr: full["openings"][ptr] },
        })
        .to_string(),
    )
    .expect("부분 저장");
    let restored = dir.join("d.restored.json");
    let o = run(&[
        "disclose",
        "restore",
        redacted.to_str().unwrap(),
        "--opening",
        partial.to_str().unwrap(),
        "-o",
        restored.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "부분 개봉 복원은 거부");
    assert!(!restored.exists(), "거부 시 복원 파일을 만들지 않는다");

    // 개봉 kind 오류 — capsuleOpening 이 아니면 exit 2.
    let bad_kind = dir.join("d.badkind.json");
    std::fs::write(
        &bad_kind,
        serde_json::json!({ "schemaVersion": "1.0", "kind": "keyring", "openings": {} })
            .to_string(),
    )
    .expect("저장");
    let o = run(&[
        "disclose",
        "verify",
        redacted.to_str().unwrap(),
        "--opening",
        bad_kind.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2), "개봉 kind 오류는 사용법 오류");

    // 캡슐이 아닌 입력의 redact — kind 검사 exit 2.
    let not_capsule = dir.join("d.notcap.json");
    std::fs::write(
        &not_capsule,
        serde_json::json!({ "schemaVersion": "1.0", "kind": "keyring", "keys": [] }).to_string(),
    )
    .expect("저장");
    let o = run(&[
        "disclose",
        "redact",
        not_capsule.to_str().unwrap(),
        "-o",
        redacted.to_str().unwrap(),
        "--opening-out",
        opening.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2), "비캡슐 redact 는 사용법 오류");

    let _ = std::fs::remove_dir_all(&dir);
}
