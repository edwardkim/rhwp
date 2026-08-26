//! [#4509] 캡슐 서명 계약 — 귀속(4년 축) 발급·검증·계보 통합.
//!
//! 고정하는 것:
//! ① keygen — 발급 봉투(keyId·publicKey·keyFile)와 **덮어쓰기 거부**(비밀키
//!    파일 보호), ② 서명 왕복 — `replay --capsule --sign-key` 가 만든 사이드카가
//!    `verify-signature` 에서 valid·exit 0, ③ **결정론 서명** — 같은 키·같은
//!    캡슐 바이트에 두 번 서명하면 서명 문자열이 같다(Ed25519 결정론 — 이
//!    저장소 결정론 문화와의 정합 실측), ④ 변조 폭로 — 캡슐 1바이트 변조 =
//!    invalid·exit 3, ⑤ 미등록 키 = unknownKey, 폐기 키 = revoked (둘 다
//!    exit 3, 판정은 봉투 데이터), ⑥ lineage `--keyring` 통합 — 서명된 2링크
//!    체인 valid + signerOk true, 무효 서명 링크는 brokenAt, **opt-in 무파손**
//!    (--keyring 없으면 signerOk 축 자체가 봉투에 없다), ⑦ --sign-key 는
//!    --capsule 과 함께만(사용법 오류).

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
    let dir = std::env::temp_dir().join(format!("rhwp_sign_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("작업 폴더");
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

/// 키를 발급하고 (키 경로, keyId, publicKey b64) 를 돌려준다.
fn keygen(dir: &std::path::Path, name: &str, key_id: &str) -> (String, String, String) {
    let key_path = dir.join(name);
    let o = run(&[
        "keygen",
        "--key-id",
        key_id,
        "--out",
        key_path.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "keygen 실패: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    assert_eq!(env["keyId"], key_id);
    let public = env["publicKey"].as_str().expect("publicKey").to_string();
    (
        key_path.to_string_lossy().into_owned(),
        key_id.to_string(),
        public,
    )
}

/// keyring.json 파일을 조립한다.
fn write_keyring(dir: &std::path::Path, entries: &[(&str, &str, Option<&str>)]) -> String {
    let keys: Vec<serde_json::Value> = entries
        .iter()
        .map(|(id, public, revoked)| {
            serde_json::json!({
                "keyId": id,
                "publicKey": public,
                "revoked": revoked.map(|r| serde_json::json!({ "at": "2026-08-10", "reason": r })),
            })
        })
        .collect();
    let path = dir.join("keyring.json");
    std::fs::write(
        &path,
        serde_json::json!({ "schemaVersion": "1.0", "kind": "keyring", "keys": keys }).to_string(),
    )
    .expect("keyring 저장");
    path.to_string_lossy().into_owned()
}

/// 서명 캡슐을 발급하고 캡슐 경로를 돌려준다.
fn signed_capsule(
    dir: &std::path::Path,
    name: &str,
    key: &str,
    find: &str,
    parent: Option<&str>,
) -> String {
    let cap = dir.join(name);
    let plan = plan_json(SAMPLE, &dir.join(format!("{name}.out.hwp")), find);
    let mut args = vec![
        "replay".to_string(),
        "--plan-json".to_string(),
        plan,
        "--capsule".to_string(),
        cap.to_string_lossy().into_owned(),
        "--sign-key".to_string(),
        key.to_string(),
        "--json".to_string(),
    ];
    if let Some(p) = parent {
        args.push("--parent".to_string());
        args.push(p.to_string());
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let o = run(&args_ref);
    assert_eq!(
        o.status.code(),
        Some(0),
        "서명 캡슐 발급 실패: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        std::path::Path::new(&format!("{}.sig.json", cap.display())).exists(),
        "사이드카가 생겨야 한다"
    );
    cap.to_string_lossy().into_owned()
}

#[test]
fn keygen_envelope_and_overwrite_refusal() {
    let dir = make_dir("keygen");
    let (key_path, _, public) = keygen(&dir, "a.key.json", "test.example/agent#1");
    assert_eq!(public.len(), 44, "Ed25519 공개키 base64 는 44자: {public}");

    // 같은 경로 재발급 → 덮어쓰기 거부 (비밀키 보호).
    let o = run(&[
        "keygen",
        "--key-id",
        "test.example/agent#1",
        "--out",
        &key_path,
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(2),
        "기존 키 파일 덮어쓰기는 사용법 오류"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sign_verify_roundtrip_and_deterministic_signature() {
    let dir = make_dir("roundtrip");
    let find = existing_snippet();
    let (key, key_id, public) = keygen(&dir, "k.json", "test.example/agent#1");
    let keyring = write_keyring(&dir, &[(&key_id, &public, None)]);
    let cap = signed_capsule(&dir, "a.capsule.json", &key, &find, None);

    // 왕복 — valid, exit 0.
    let o = run(&["verify-signature", &cap, "--keyring", &keyring, "--json"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    let env = env_of(&o);
    assert_eq!(env["verdict"], "valid", "{env}");
    assert_eq!(env["signatureOk"], true);
    assert_eq!(env["keyKnown"], true);
    assert_eq!(env["capsuleShaMatches"], true);
    assert_eq!(env["keyId"], key_id);

    // 결정론 — 같은 키로 같은 캡슐 바이트에 다시 서명하면 서명이 같다.
    let sc_path = format!("{cap}.sig.json");
    let first: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&sc_path).unwrap()).unwrap();
    std::fs::remove_file(&sc_path).unwrap();
    // 캡슐은 결정론이므로 같은 계획 재발급 = 같은 캡슐 바이트 = 같은 서명이어야 한다.
    let cap2 = signed_capsule(&dir, "a.capsule.json", &key, &find, None);
    assert_eq!(cap, cap2);
    let second: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&sc_path).unwrap()).unwrap();
    assert_eq!(
        first["signature"], second["signature"],
        "Ed25519 는 결정론 서명 — 같은 키·같은 바이트면 같은 서명"
    );
    assert_eq!(first["capsuleSha256"], second["capsuleSha256"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tampered_capsule_unknown_key_and_revoked_key() {
    let dir = make_dir("adversary");
    let find = existing_snippet();
    let (key, key_id, public) = keygen(&dir, "k.json", "test.example/agent#1");
    let keyring = write_keyring(&dir, &[(&key_id, &public, None)]);
    let cap = signed_capsule(&dir, "a.capsule.json", &key, &find, None);

    // ① 캡슐 1바이트 변조 → invalid, exit 3.
    let mut bytes = std::fs::read(&cap).unwrap();
    let last = bytes.len() - 1;
    bytes[last] = if bytes[last] == b'}' { b' ' } else { b'}' };
    let tampered = dir.join("tampered.capsule.json");
    std::fs::write(&tampered, &bytes).unwrap();
    let o = run(&[
        "verify-signature",
        tampered.to_str().unwrap(),
        "--sig",
        &format!("{cap}.sig.json"),
        "--keyring",
        &keyring,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "변조 = 검증 단언 실패");
    let env = env_of(&o);
    assert_eq!(env["verdict"], "invalid", "{env}");
    assert_eq!(env["capsuleShaMatches"], false);

    // ② 키 등록부에 없는 키 → unknownKey.
    let empty_ring = write_keyring(&dir, &[("other/agent#9", &public, None)]);
    let o = run(&["verify-signature", &cap, "--keyring", &empty_ring, "--json"]);
    assert_eq!(o.status.code(), Some(3));
    assert_eq!(env_of(&o)["verdict"], "unknownKey");

    // ③ 폐기 키 → revoked (서명 자체는 유효해도 판정은 폐기 우선).
    let revoked_ring = write_keyring(&dir, &[(&key_id, &public, Some("유출 신고"))]);
    let o = run(&[
        "verify-signature",
        &cap,
        "--keyring",
        &revoked_ring,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3));
    let env = env_of(&o);
    assert_eq!(env["verdict"], "revoked", "{env}");
    assert_eq!(
        env["signatureOk"], true,
        "폐기라도 암호학적 사실은 그대로 보고"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn lineage_signer_axis_and_optin_compat() {
    let dir = make_dir("lineage");
    let find = existing_snippet();
    let (key, key_id, public) = keygen(&dir, "k.json", "test.example/agent#1");
    let keyring = write_keyring(&dir, &[(&key_id, &public, None)]);

    // 서명된 2링크 체인: A(뿌리) ← B(parent A). B 의 입력은 A 의 실산출이어야
    // lineageOk 가 성립하므로, A 계획의 산출을 run 으로 실제로 만들어 쓴다.
    let o1 = dir.join("o1.hwp");
    let plan_a = plan_json(SAMPLE, &o1, &find);
    let plan_a_path = dir.join("plan_a.json");
    std::fs::write(&plan_a_path, &plan_a).unwrap();
    let o = run(&["run", plan_a_path.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(0));

    let cap_a = dir.join("a.capsule.json");
    let o = run(&[
        "replay",
        "--plan-json",
        &plan_a,
        "--capsule",
        cap_a.to_str().unwrap(),
        "--sign-key",
        &key,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));

    let plan_b = plan_json(o1.to_str().unwrap(), &dir.join("o2.hwp"), &find);
    let cap_b = dir.join("b.capsule.json");
    let o = run(&[
        "replay",
        "--plan-json",
        &plan_b,
        "--capsule",
        cap_b.to_str().unwrap(),
        "--parent",
        cap_a.to_str().unwrap(),
        "--sign-key",
        &key,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));

    // --keyring: 두 링크 모두 signerOk true, 체인 유효.
    let o = run(&[
        "lineage",
        cap_b.to_str().unwrap(),
        "--keyring",
        &keyring,
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    let env = env_of(&o);
    assert_eq!(env["valid"], true, "{env}");
    assert_eq!(env["links"][0]["signerOk"], true);
    assert_eq!(env["links"][1]["signerOk"], true);
    assert_eq!(env["links"][0]["keyId"], key_id);

    // opt-in 무파손: --keyring 없으면 signerOk 축 자체가 봉투에 없다.
    let o = run(&["lineage", cap_b.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let env = env_of(&o);
    assert_eq!(env["valid"], true);
    assert!(
        env["links"][0].get("signerOk").is_none(),
        "opt-in 규약 위반 — keyring 없이 signerOk 가 실렸다: {env}"
    );

    // 부모 서명을 무효화(사이드카 변조) → 그 링크에서 brokenAt.
    let sc_a = format!("{}.sig.json", cap_a.display());
    let mut sc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&sc_a).unwrap()).unwrap();
    sc["signature"] = serde_json::json!(base64_flip(sc["signature"].as_str().unwrap()));
    std::fs::write(&sc_a, sc.to_string()).unwrap();
    let o = run(&[
        "lineage",
        cap_b.to_str().unwrap(),
        "--keyring",
        &keyring,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "무효 서명 링크 = 깨진 계보");
    let env = env_of(&o);
    assert_eq!(env["valid"], false);
    assert_eq!(env["links"][1]["signerOk"], false, "{env}");
    assert_eq!(
        env["brokenAt"],
        serde_json::json!(cap_a.display().to_string())
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// base64 문자열의 첫 글자를 다른 유효 문자로 바꿔 서명을 무효화한다.
fn base64_flip(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    chars[0] = if chars[0] == 'A' { 'B' } else { 'A' };
    chars.into_iter().collect()
}

#[test]
fn usage_conventions() {
    // --sign-key 는 --capsule 과 함께만.
    let o = run(&[
        "replay",
        "--plan-json",
        "{}",
        "--sign-key",
        "nope.json",
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(2),
        "서명 대상 없는 --sign-key 는 사용법 오류"
    );

    // keygen 필수 인자.
    let o = run(&["keygen", "--json"]);
    assert_eq!(o.status.code(), Some(2));

    // verify-signature 필수 인자.
    let o = run(&["verify-signature", "--json"]);
    assert_eq!(o.status.code(), Some(2));
}
