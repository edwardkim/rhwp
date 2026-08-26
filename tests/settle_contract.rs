//! [#4553] 정산 증빙 계약 — workorder·claim 3해시 고정·원장 이중 청구(9년 축).
//!
//! 고정하는 것:
//! ① 3해시 왕복 — `settle propose` 가 명세서·캡슐·게이트 봉투의 파일 바이트
//!    sha256 셋을 청구에 고정하고, `verify` 가 전 축 green(workorderOk·
//!    capsuleOk·gateOk·gateVerdict allow·signerOk·duplicate false)·exit 0,
//! ② 원장 왕복 — `record` 가 5년 로그 **동형** 체인(ndjson·prevEntryHash·
//!    seq 연번)에 기입하고, ③ **P3 이중 청구** — 같은 캡슐 2회 record 는
//!    두 번째가 exit 3(existingSeq 보고), 기입 후 verify --ledger 도
//!    duplicate true·exit 3, ④ **P1/P4 변조 검출** — 캡슐/명세서 바꿔치기가
//!    각각 capsuleOk/workorderOk false 로 환원, ⑤ 검수 미통과 — 해시가 맞아도
//!    gateVerdict 가 allow 가 아니면 rejected, ⑥ 방어 — acceptancePolicy 없는
//!    명세서는 propose 가 거부(exit 2, 분쟁을 산문으로 되돌리지 않는다),
//!    깨진 원장 기입 거부(exit 3), 서명 후 청구 변조는 signerOk false.
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

fn make_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp_settle_{tag}_{}", std::process::id()));
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

/// 명세서·게이트 봉투·서명 캡슐 픽스처 일습을 만든다.
struct Fixture {
    dir: std::path::PathBuf,
    workorder: String,
    capsule: String,
    gate_env: String,
    keyring: String,
    key: String,
}

fn fixture(tag: &str) -> Fixture {
    let dir = make_dir(tag);
    // 발주 명세서 — 검수 기준(6년 정책 인라인)과 문자열 금액.
    let workorder = dir.join("wo.json");
    std::fs::write(
        &workorder,
        serde_json::json!({
            "schemaVersion": "1.0",
            "kind": "workorder",
            "workorderId": "wo-2026-0142",
            "title": "표 문서 검증 납품",
            "acceptancePolicy": {
                "schemaVersion": "1.0", "kind": "admissionPolicy", "default": "deny",
                "rules": [{ "key": "reproduced", "op": "eq", "value": true }],
            },
            "unitPrice": { "amount": "50000", "currency": "KRW", "per": "capsule" },
        })
        .to_string(),
    )
    .expect("명세서 저장");
    // 게이트 판정 봉투 — settle 은 이 파일을 해시로 고정하고 verdict 만 재확인한다.
    let gate_env = dir.join("gate.envelope.json");
    std::fs::write(
        &gate_env,
        serde_json::json!({ "schemaVersion": "1.0", "verdict": "allow", "violations": [] })
            .to_string(),
    )
    .expect("게이트 봉투 저장");
    // 청구자 키와 keyring.
    let key = dir.join("vendor.key.json");
    let o = run(&[
        "keygen",
        "--key-id",
        "vendor-7",
        "--out",
        key.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let public = env_of(&o)["publicKey"]
        .as_str()
        .expect("publicKey")
        .to_string();
    let keyring = dir.join("keyring.json");
    std::fs::write(
        &keyring,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "keyring",
            "keys": [{ "keyId": "vendor-7", "publicKey": public, "revoked": null }],
        })
        .to_string(),
    )
    .expect("keyring 저장");
    // 납품 캡슐 — 실제 replay 산출 (스텝은 문서 실존 문자열의 무해 치환).
    let o = run(&["export-text", SAMPLE, "-p", "0", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let text = env_of(&o)["pages"][0]["text"]
        .as_str()
        .expect("쪽 텍스트")
        .to_string();
    let snippet: String = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(2)
        .collect();
    let capsule = dir.join("work.capsule.json");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": SAMPLE,
        "output": dir.join("work.out.hwp").to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": snippet, "replace": snippet }],
    })
    .to_string();
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
        "캡슐 발급 실패: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    Fixture {
        workorder: workorder.to_string_lossy().into_owned(),
        capsule: capsule.to_string_lossy().into_owned(),
        gate_env: gate_env.to_string_lossy().into_owned(),
        keyring: keyring.to_string_lossy().into_owned(),
        key: key.to_string_lossy().into_owned(),
        dir,
    }
}

/// ①~③ 3해시 왕복·원장 기입·이중 청구.
#[test]
fn 정산_왕복_계약() {
    let f = fixture("roundtrip");
    let claim = f.dir.join("claim.json");
    let ledger = f.dir.join("ledger.ndjson");

    // ① 청구 발급(서명 포함) — 봉투의 3해시가 파일 바이트 해시와 일치.
    let o = run(&[
        "settle",
        "propose",
        "--workorder",
        &f.workorder,
        "--capsule",
        &f.capsule,
        "--gate-envelope",
        &f.gate_env,
        "-o",
        claim.to_str().unwrap(),
        "--sign-key",
        &f.key,
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "propose 실패: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    assert_eq!(env["signed"], serde_json::json!(true));
    for (field, path) in [
        ("workorderSha256", &f.workorder),
        ("capsuleSha256", &f.capsule),
        ("gateEnvelopeSha256", &f.gate_env),
    ] {
        let expected = sha256_hex(&std::fs::read(path).expect("픽스처"));
        assert_eq!(env[field], serde_json::json!(expected), "{field} 고정");
    }
    assert!(
        std::path::Path::new(&format!("{}.sig.json", claim.display())).exists(),
        "청구 사이드카"
    );

    // 전 축 검증 — 빈 원장 포함 모두 green.
    let o = run(&[
        "settle",
        "verify",
        claim.to_str().unwrap(),
        "--workorder",
        &f.workorder,
        "--capsule",
        &f.capsule,
        "--gate-envelope",
        &f.gate_env,
        "--keyring",
        &f.keyring,
        "--ledger",
        ledger.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "verify 실패: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    for k in ["workorderOk", "capsuleOk", "gateOk", "signerOk", "ledgerOk"] {
        assert_eq!(env[k], serde_json::json!(true), "{k}");
    }
    assert_eq!(env["gateVerdict"], serde_json::json!("allow"));
    assert_eq!(
        env["workorderSignerOk"],
        serde_json::Value::Null,
        "미서명 명세서는 null 보고"
    );
    assert_eq!(env["duplicate"], serde_json::json!(false));
    assert_eq!(env["verdict"], serde_json::json!("ok"));

    // ② 원장 기입 — 동형 체인 규약(seq 0·prevEntryHash null·kind).
    let o = run(&[
        "settle",
        "record",
        claim.to_str().unwrap(),
        "--ledger",
        ledger.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let env = env_of(&o);
    assert_eq!(env["seq"], serde_json::json!(0));
    assert_eq!(env["duplicate"], serde_json::json!(false));
    let line0: serde_json::Value = serde_json::from_str(
        std::fs::read_to_string(&ledger)
            .expect("원장")
            .lines()
            .next()
            .expect("첫 줄"),
    )
    .expect("원장 줄 파싱");
    assert_eq!(line0["kind"], serde_json::json!("settlementLedger"));
    assert_eq!(line0["prevEntryHash"], serde_json::Value::Null);
    assert_eq!(line0["verdict"], serde_json::json!("accepted"));

    // ③ 이중 청구 — 같은 캡슐 2회 record = 두 번째 exit 3 + existingSeq.
    let o = run(&[
        "settle",
        "record",
        claim.to_str().unwrap(),
        "--ledger",
        ledger.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "이중 청구는 거부");
    let env = env_of(&o);
    assert_eq!(env["duplicate"], serde_json::json!(true));
    assert_eq!(env["existingSeq"], serde_json::json!(0));
    // 기입 후 verify --ledger 도 duplicate 를 본다 — 같은 원장, 같은 판정.
    let o = run(&[
        "settle",
        "verify",
        claim.to_str().unwrap(),
        "--workorder",
        &f.workorder,
        "--capsule",
        &f.capsule,
        "--gate-envelope",
        &f.gate_env,
        "--ledger",
        ledger.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3));
    let env = env_of(&o);
    assert_eq!(env["duplicate"], serde_json::json!(true));
    assert_eq!(env["verdict"], serde_json::json!("rejected"));

    let _ = std::fs::remove_dir_all(&f.dir);
}

/// ④~⑥ P1/P4 변조·검수 미통과·방어 3종.
#[test]
fn 정산_방어_계약() {
    let f = fixture("defense");
    let claim = f.dir.join("claim.json");
    let o = run(&[
        "settle",
        "propose",
        "--workorder",
        &f.workorder,
        "--capsule",
        &f.capsule,
        "--gate-envelope",
        &f.gate_env,
        "-o",
        claim.to_str().unwrap(),
        "--sign-key",
        &f.key,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));

    let verify = |wo: &str, cap: &str, gate: &str| -> (Option<i32>, serde_json::Value) {
        let o = run(&[
            "settle",
            "verify",
            claim.to_str().unwrap(),
            "--workorder",
            wo,
            "--capsule",
            cap,
            "--gate-envelope",
            gate,
            "--keyring",
            &f.keyring,
            "--json",
        ]);
        (o.status.code(), env_of(&o))
    };

    // ④ P1 산출물 바꿔치기 — 캡슐 1바이트 추가 = capsuleOk false·exit 3.
    let tampered_cap = f.dir.join("tampered.capsule.json");
    let mut bytes = std::fs::read(&f.capsule).expect("캡슐");
    bytes.push(b' ');
    std::fs::write(&tampered_cap, &bytes).expect("변조 캡슐");
    let (code, env) = verify(&f.workorder, tampered_cap.to_str().unwrap(), &f.gate_env);
    assert_eq!(code, Some(3));
    assert_eq!(env["capsuleOk"], serde_json::json!(false));
    assert_eq!(
        env["workorderOk"],
        serde_json::json!(true),
        "다른 축은 무사"
    );

    // P4 명세서 사후 변경 — workorderOk false.
    let tampered_wo = f.dir.join("tampered.wo.json");
    let text = std::fs::read_to_string(&f.workorder)
        .expect("명세서")
        .replace("50000", "99000");
    std::fs::write(&tampered_wo, text).expect("변조 명세서");
    let (code, env) = verify(tampered_wo.to_str().unwrap(), &f.capsule, &f.gate_env);
    assert_eq!(code, Some(3));
    assert_eq!(env["workorderOk"], serde_json::json!(false));

    // ⑤ 검수 미통과 — deny 봉투에 고정된 청구는 해시가 맞아도 rejected.
    let deny_gate = f.dir.join("gate.deny.json");
    std::fs::write(
        &deny_gate,
        serde_json::json!({ "schemaVersion": "1.0", "verdict": "deny", "violations": ["R1"] })
            .to_string(),
    )
    .expect("deny 봉투");
    let deny_claim = f.dir.join("claim.deny.json");
    let o = run(&[
        "settle",
        "propose",
        "--workorder",
        &f.workorder,
        "--capsule",
        &f.capsule,
        "--gate-envelope",
        deny_gate.to_str().unwrap(),
        "-o",
        deny_claim.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "발급 자체는 된다 — 판정은 verify 의 몫"
    );
    let o = run(&[
        "settle",
        "verify",
        deny_claim.to_str().unwrap(),
        "--workorder",
        &f.workorder,
        "--capsule",
        &f.capsule,
        "--gate-envelope",
        deny_gate.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3));
    let env = env_of(&o);
    assert_eq!(env["gateOk"], serde_json::json!(true), "해시는 맞다");
    assert_eq!(env["gateVerdict"], serde_json::json!("deny"));
    assert_eq!(env["verdict"], serde_json::json!("rejected"));

    // ⑥-1 검수 기준 없는 명세서 — propose 가 거부(exit 2).
    let bare_wo = f.dir.join("bare.wo.json");
    std::fs::write(
        &bare_wo,
        serde_json::json!({ "schemaVersion": "1.0", "kind": "workorder", "workorderId": "x" })
            .to_string(),
    )
    .expect("맨 명세서");
    let o = run(&[
        "settle",
        "propose",
        "--workorder",
        bare_wo.to_str().unwrap(),
        "--capsule",
        &f.capsule,
        "--gate-envelope",
        &f.gate_env,
        "-o",
        f.dir.join("x.json").to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(2),
        "acceptancePolicy 없는 명세서 거부"
    );

    // ⑥-2 깨진 원장 기입 거부 — 중간 줄 변조 후 record exit 3.
    let ledger = f.dir.join("ledger.ndjson");
    let o = run(&[
        "settle",
        "record",
        claim.to_str().unwrap(),
        "--ledger",
        ledger.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    // 마지막 줄 내용 변조는 체인이 못 잡는다(후속 줄이 봉인, 최종 봉인은
    // 체크포인트 공표 — 5년 동형의 한계 그대로). 여기서는 구조 파손(kind)을 쓴다.
    let text = std::fs::read_to_string(&ledger)
        .expect("원장")
        .replace("settlementLedger", "settlementLedgerX");
    std::fs::write(&ledger, text).expect("원장 변조");
    let other_claim = f.dir.join("claim2.json");
    let o = run(&[
        "settle",
        "propose",
        "--workorder",
        &f.workorder,
        "--capsule",
        &f.capsule,
        "--gate-envelope",
        deny_gate.to_str().unwrap(),
        "-o",
        other_claim.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let o = run(&[
        "settle",
        "record",
        other_claim.to_str().unwrap(),
        "--ledger",
        ledger.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "깨진 원장에는 기입하지 않는다");

    // ⑥-3 서명 후 청구 변조 — signerOk false (파일 바이트 서명).
    let mut claim_text = std::fs::read_to_string(&claim).expect("청구");
    claim_text.push(' ');
    let moved = f.dir.join("claim.moved.json");
    std::fs::write(&moved, &claim_text).expect("변조 청구");
    std::fs::copy(
        format!("{}.sig.json", claim.display()),
        format!("{}.sig.json", moved.display()),
    )
    .expect("사이드카 복사");
    let o = run(&[
        "settle",
        "verify",
        moved.to_str().unwrap(),
        "--workorder",
        &f.workorder,
        "--capsule",
        &f.capsule,
        "--gate-envelope",
        &f.gate_env,
        "--keyring",
        &f.keyring,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3));
    assert_eq!(env_of(&o)["signerOk"], serde_json::json!(false));

    let _ = std::fs::remove_dir_all(&f.dir);
}
