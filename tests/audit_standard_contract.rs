//! [#4558] 감사 표준 계약 — 보고 기계 대사·리콜 폐쇄집합·적합성 등급(10년 축).
//!
//! 고정하는 것:
//! ① **보고 수치 기계 대사** — `audit-report --deep --keyring --anchor-log
//!    --policy` 의 전 절(scope·reproduction·lineage·attribution·anchoring·
//!    gate)이 픽스처에서 독립 계산한 수치와 불일치 0, ② **보고 서명 왕복** —
//!    `--sign-key` 보고서가 `verify-signature` 에서 valid, 1바이트 변조 =
//!    invalid("감사 보고서를 감사할 수 있다"), ③ **리콜 폐쇄집합** — 3링크
//!    체인의 뿌리를 오염 지목 → 후손 전건(경로 포함)과 자기 자신이 affected,
//!    무관 캡슐은 unaffected, 중간 노드 지목 시 상류(뿌리)는 미영향, sha256
//!    직접 지목도 동작, --ledger 로 영향 청구 좌표 보고, ④ **적합성 사다리** —
//!    L1~L5 전 등급 conformant(전건 서명·앵커·게이트·원장 픽스처), 미서명
//!    캡슐 하나가 섞이면 L3 nonconformant·exit 3(checks 가 미달 항목 명세),
//!    L3 을 --keyring 없이 부르면 판정이 아니라 사용법 오류 exit 2,
//!    ⑤ 방어 — 빈 폴더 conformance 는 exit 2(판정 대상 아님).
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
    let dir = std::env::temp_dir().join(format!("rhwp_y10_{tag}_{}", std::process::id()));
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

fn snippet() -> String {
    let o = run(&["export-text", SAMPLE, "-p", "0", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let env: serde_json::Value = serde_json::from_slice(&o.stdout).expect("봉투");
    let text = env["pages"][0]["text"].as_str().expect("쪽 텍스트");
    text.chars()
        .filter(|c| !c.is_whitespace())
        .take(2)
        .collect()
}

/// 감사 대상 픽스처 — 서명·앵커된 3링크 체인 A→B→C (+선택 미서명 D).
///
/// 체인 유효의 열쇠: 다음 링크의 입력은 앞 링크의 **실산출**이어야 한다
/// (계보 불변식 — run 으로 실물화한 파일을 물린다).
struct Fixture {
    dir: std::path::PathBuf,
    capsules: std::path::PathBuf,
    keyring: String,
    key: String,
    anchor: String,
    policy: String,
}

fn fixture(tag: &str, with_unsigned: bool) -> Fixture {
    let dir = make_dir(tag);
    let capsules = dir.join("capsules");
    std::fs::create_dir_all(&capsules).expect("캡슐 폴더");
    let key = dir.join("auditor.key.json");
    let o = run(&[
        "keygen",
        "--key-id",
        "org-2026",
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
            "keys": [{ "keyId": "org-2026", "publicKey": public, "revoked": null }],
        })
        .to_string(),
    )
    .expect("keyring");
    let find = snippet();
    let anchor = dir.join("anchor.ndjson");

    // 3링크 체인 — 각 링크: run(실산출) → replay --capsule --sign-key → anchor add.
    let mut input = SAMPLE.to_string();
    let mut parent: Option<String> = None;
    for name in ["a", "b", "c"] {
        let out_doc = dir.join(format!("{name}.out.hwp"));
        let cap = capsules.join(format!("{name}.capsule.json"));
        let plan = serde_json::json!({
            "planVersion": "1.0",
            "input": input,
            "output": out_doc.to_string_lossy(),
            "steps": [{ "action": "replace_text", "find": find, "replace": find }],
        })
        .to_string();
        let o = run(&["run", "--plan-json", &plan, "--json"]);
        assert_eq!(
            o.status.code(),
            Some(0),
            "실산출 실패: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let mut args = vec![
            "replay".to_string(),
            "--plan-json".to_string(),
            plan,
            "--capsule".to_string(),
            cap.to_string_lossy().into_owned(),
            "--sign-key".to_string(),
            key.to_string_lossy().into_owned(),
            "--json".to_string(),
        ];
        if let Some(p) = &parent {
            args.push("--parent".to_string());
            args.push(p.clone());
        }
        let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
        let o = run(&args_ref);
        assert_eq!(
            o.status.code(),
            Some(0),
            "캡슐 발급 실패({name}): {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let o = run(&[
            "anchor",
            "add",
            cap.to_str().unwrap(),
            "--log",
            anchor.to_str().unwrap(),
            "--json",
        ]);
        assert_eq!(o.status.code(), Some(0), "앵커 등재 실패({name})");
        input = out_doc.to_string_lossy().into_owned();
        parent = Some(cap.to_string_lossy().into_owned());
    }
    if with_unsigned {
        // 미서명·미앵커 무관 캡슐 D — 리콜의 미영향군이자 L3 미달 요인.
        let cap = capsules.join("d.capsule.json");
        let plan = serde_json::json!({
            "planVersion": "1.0",
            "input": SAMPLE,
            "output": dir.join("d.out.hwp").to_string_lossy(),
            "steps": [{ "action": "replace_text", "find": find, "replace": find }],
        })
        .to_string();
        let o = run(&[
            "replay",
            "--plan-json",
            &plan,
            "--capsule",
            cap.to_str().unwrap(),
            "--json",
        ]);
        assert_eq!(o.status.code(), Some(0));
    }
    let policy = dir.join("policy.json");
    std::fs::write(
        &policy,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "admissionPolicy", "default": "deny",
            "rules": [
                { "id": "R1-계보", "require": { "lineageValid": { "eq": true } } },
                { "id": "R2-서명", "require": { "signerVerdict": { "eq": "valid" } } },
                { "id": "R3-앵커", "require": { "anchoredOk": { "eq": true } } },
            ],
        })
        .to_string(),
    )
    .expect("정책");
    Fixture {
        capsules,
        keyring: keyring.to_string_lossy().into_owned(),
        key: key.to_string_lossy().into_owned(),
        anchor: anchor.to_string_lossy().into_owned(),
        policy: policy.to_string_lossy().into_owned(),
        dir,
    }
}

/// ①·② 보고 수치 기계 대사 + 보고 서명 왕복.
#[test]
fn 감사_보고_기계_대사_계약() {
    let f = fixture("report", false);
    let report = f.dir.join("report.json");
    let o = run(&[
        "audit-report",
        f.capsules.to_str().unwrap(),
        "-o",
        report.to_str().unwrap(),
        "--deep",
        "--keyring",
        &f.keyring,
        "--anchor-log",
        &f.anchor,
        "--policy",
        &f.policy,
        "--sign-key",
        &f.key,
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "audit-report 실패: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    // 기계 대사 — 픽스처에서 독립적으로 아는 수치와 불일치 0.
    assert_eq!(env["capsules"], serde_json::json!(3));
    assert_eq!(env["reproduction"]["attempted"], serde_json::json!(3));
    assert_eq!(env["reproduction"]["reproduced"], serde_json::json!(3));
    assert_eq!(env["reproduction"]["rate"], serde_json::json!(1.0));
    assert_eq!(env["reproduction"]["failures"], serde_json::json!([]));
    // 체인 A→B→C: 머리(자식 없는 노드)는 C 하나, 뿌리는 A 하나.
    assert_eq!(env["lineage"]["graphs"], serde_json::json!(1));
    assert_eq!(env["lineage"]["heads"], serde_json::json!(1));
    assert_eq!(env["lineage"]["valid"], serde_json::json!(1));
    assert_eq!(env["lineage"]["broken"], serde_json::json!([]));
    assert_eq!(env["attribution"]["signed"], serde_json::json!(3));
    assert_eq!(env["attribution"]["unsigned"], serde_json::json!(0));
    assert_eq!(env["attribution"]["validSignatures"], serde_json::json!(3));
    assert_eq!(env["attribution"]["revokedKeyUses"], serde_json::json!(0));
    assert_eq!(env["anchoring"]["anchored"], serde_json::json!(3));
    assert_eq!(env["anchoring"]["unanchored"], serde_json::json!(0));
    assert_eq!(env["gate"]["passed"], serde_json::json!(3));
    assert_eq!(env["gate"]["denied"], serde_json::json!(0));
    let policy_sha = sha256_hex(&std::fs::read(&f.policy).expect("정책"));
    assert_eq!(env["gate"]["policySha256"], serde_json::json!(policy_sha));
    assert_eq!(env["signed"], serde_json::json!(true));
    // 보고서 파일 자체 검증 — kind·auditor·서명 왕복.
    let report_v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).expect("보고서")).expect("파싱");
    assert_eq!(report_v["kind"], serde_json::json!("agentLaborAuditReport"));
    assert_eq!(report_v["auditor"]["keyId"], serde_json::json!("org-2026"));
    let o = run(&[
        "verify-signature",
        report.to_str().unwrap(),
        "--keyring",
        &f.keyring,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0), "보고 서명 valid");
    // 보고서 변조 = 서명 invalid — "감사 보고서를 감사할 수 있다".
    let mut text = std::fs::read_to_string(&report).expect("보고서");
    text.push(' ');
    std::fs::write(&report, text).expect("변조");
    let o = run(&[
        "verify-signature",
        report.to_str().unwrap(),
        "--keyring",
        &f.keyring,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "변조 보고서는 invalid");

    let _ = std::fs::remove_dir_all(&f.dir);
}

/// ③ 리콜 폐쇄집합 — 뿌리/중간 오염·sha 지목·원장 연결.
#[test]
fn 리콜_폐쇄집합_계약() {
    let f = fixture("recall", true);
    let a = f.capsules.join("a.capsule.json");
    let b = f.capsules.join("b.capsule.json");

    // 뿌리 A 오염 — A·B·C 전건 affected(경로 포함), D 만 미영향.
    let o = run(&[
        "recall-scope",
        "--contaminated",
        a.to_str().unwrap(),
        "--among",
        f.capsules.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let env = env_of(&o);
    let affected = env["affected"].as_array().expect("affected");
    let names: Vec<&str> = affected
        .iter()
        .map(|e| e["capsule"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["a.capsule.json", "b.capsule.json", "c.capsule.json"],
        "후손 폐쇄집합 전건"
    );
    assert_eq!(env["unaffected"], serde_json::json!(1), "D 는 미영향");
    // C 의 경로는 오염 뿌리부터 자신까지.
    assert_eq!(
        affected[2]["path"],
        serde_json::json!(["a.capsule.json", "b.capsule.json", "c.capsule.json"])
    );

    // 중간 B 오염 — 상류 A 는 미영향(B·C 만 affected).
    let o = run(&[
        "recall-scope",
        "--contaminated",
        b.to_str().unwrap(),
        "--among",
        f.capsules.to_str().unwrap(),
        "--json",
    ]);
    let env = env_of(&o);
    let names: Vec<&str> = env["affected"]
        .as_array()
        .expect("affected")
        .iter()
        .map(|e| e["capsule"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["b.capsule.json", "c.capsule.json"]);
    assert_eq!(env["unaffected"], serde_json::json!(2), "A·D 미영향");

    // sha256 직접 지목 — 파일 없이 해시 정체성으로 같은 판정.
    let a_sha = sha256_hex(&std::fs::read(&a).expect("A"));
    let o = run(&[
        "recall-scope",
        "--contaminated",
        &a_sha,
        "--among",
        f.capsules.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(env_of(&o)["affected"].as_array().map(Vec::len), Some(3));

    // 원장 연결 — C 를 청구한 원장이 있으면 영향 청구 좌표가 나온다.
    let c = f.capsules.join("c.capsule.json");
    let wo = f.dir.join("wo.json");
    std::fs::write(
        &wo,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "workorder", "workorderId": "wo-1",
            "acceptancePolicy": { "schemaVersion": "1.0", "kind": "admissionPolicy",
                                   "default": "deny", "rules": [] },
        })
        .to_string(),
    )
    .expect("명세서");
    let gate_env = f.dir.join("gate.json");
    std::fs::write(
        &gate_env,
        serde_json::json!({ "schemaVersion": "1.0", "verdict": "allow" }).to_string(),
    )
    .expect("게이트 봉투");
    let claim = f.dir.join("claim.json");
    let ledger = f.dir.join("ledger.ndjson");
    let o = run(&[
        "settle",
        "propose",
        "--workorder",
        wo.to_str().unwrap(),
        "--capsule",
        c.to_str().unwrap(),
        "--gate-envelope",
        gate_env.to_str().unwrap(),
        "-o",
        claim.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let o = run(&[
        "settle",
        "record",
        claim.to_str().unwrap(),
        "--ledger",
        ledger.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let o = run(&[
        "recall-scope",
        "--contaminated",
        a.to_str().unwrap(),
        "--among",
        f.capsules.to_str().unwrap(),
        "--ledger",
        ledger.to_str().unwrap(),
        "--json",
    ]);
    let env = env_of(&o);
    let claims = env["claims"].as_array().expect("claims");
    assert_eq!(claims.len(), 1, "영향 캡슐 C 의 청구가 잡힌다");
    assert_eq!(claims[0]["seq"], serde_json::json!(0));

    let _ = std::fs::remove_dir_all(&f.dir);
}

/// ④·⑤ 적합성 사다리 — 전 등급 통과·미서명 혼입 미달·재료 선검사·빈 폴더.
#[test]
fn 적합성_사다리_계약() {
    let f = fixture("conf", false);
    // L5 재료 — C 청구 원장.
    let c = f.capsules.join("c.capsule.json");
    let wo = f.dir.join("wo.json");
    std::fs::write(
        &wo,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "workorder", "workorderId": "wo-1",
            "acceptancePolicy": { "schemaVersion": "1.0", "kind": "admissionPolicy",
                                   "default": "deny", "rules": [] },
        })
        .to_string(),
    )
    .expect("명세서");
    let gate_env = f.dir.join("gate.json");
    std::fs::write(
        &gate_env,
        serde_json::json!({ "schemaVersion": "1.0", "verdict": "allow" }).to_string(),
    )
    .expect("게이트 봉투");
    let claim = f.dir.join("claim.json");
    let ledger = f.dir.join("ledger.ndjson");
    assert_eq!(
        run(&[
            "settle",
            "propose",
            "--workorder",
            wo.to_str().unwrap(),
            "--capsule",
            c.to_str().unwrap(),
            "--gate-envelope",
            gate_env.to_str().unwrap(),
            "-o",
            claim.to_str().unwrap(),
            "--json",
        ])
        .status
        .code(),
        Some(0)
    );
    assert_eq!(
        run(&[
            "settle",
            "record",
            claim.to_str().unwrap(),
            "--ledger",
            ledger.to_str().unwrap(),
            "--json",
        ])
        .status
        .code(),
        Some(0)
    );

    // 전 등급 conformant — L5 는 검사 항목에 "공개 = 판정 밖" 정직 명시 포함.
    for (level, extra) in [
        ("L1", vec![]),
        ("L2", vec!["--deep"]),
        (
            "L3",
            vec!["--keyring", &f.keyring, "--anchor-log", &f.anchor],
        ),
        (
            "L4",
            vec![
                "--keyring",
                &f.keyring,
                "--anchor-log",
                &f.anchor,
                "--policy",
                &f.policy,
            ],
        ),
        (
            "L5",
            vec![
                "--keyring",
                &f.keyring,
                "--anchor-log",
                &f.anchor,
                "--policy",
                &f.policy,
                "--ledger",
                ledger.to_str().unwrap(),
            ],
        ),
    ] {
        let mut args = vec![
            "conformance",
            f.capsules.to_str().unwrap(),
            "--level",
            level,
        ];
        args.extend(extra);
        args.push("--json");
        let o = run(&args);
        let env = env_of(&o);
        assert_eq!(o.status.code(), Some(0), "{level} conformant 이어야: {env}");
        assert_eq!(env["verdict"], serde_json::json!("conformant"), "{level}");
    }

    // 재료 선검사 — L3 을 keyring 없이 부르면 판정이 아니라 사용법 오류.
    let o = run(&[
        "conformance",
        f.capsules.to_str().unwrap(),
        "--level",
        "L3",
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2), "재료 없는 등급 요구는 exit 2");

    // 미서명 캡슐 혼입 — L3 nonconformant·exit 3, checks 가 미달 항목을 명세.
    let f2 = fixture("conf2", true);
    let o = run(&[
        "conformance",
        f2.capsules.to_str().unwrap(),
        "--level",
        "L3",
        "--keyring",
        &f2.keyring,
        "--anchor-log",
        &f2.anchor,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3));
    let env = env_of(&o);
    assert_eq!(env["verdict"], serde_json::json!("nonconformant"));
    let l3 = env["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["id"] == "L3-귀속")
        .expect("L3-귀속 항목");
    assert_eq!(l3["ok"], serde_json::json!(false));
    assert!(l3["detail"].as_str().unwrap_or("").contains("1/4"));

    // 빈 폴더 — 판정 대상이 아니다.
    let empty = f.dir.join("empty");
    std::fs::create_dir_all(&empty).expect("빈 폴더");
    let o = run(&[
        "conformance",
        empty.to_str().unwrap(),
        "--level",
        "L1",
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2), "빈 폴더는 exit 2");

    let _ = std::fs::remove_dir_all(&f.dir);
    let _ = std::fs::remove_dir_all(&f2.dir);
}
