//! [#4549] 연합 번들 계약 — 오프라인 교환의 5단 판정.
//!
//! 고정하는 것: ① export→verify 왕복 — 폐쇄집합(2링크)+서명+머클 증명이
//! 수신자 trust-domain 기준으로 전건 green, ② 컨테이너 변조(zip 내 캡슐 1
//! 바이트)는 매니페스트 해시가 폭로, ③ 부모 누락은 폐쇄집합 완전성이 폭로,
//! ④ **F2 방어** — 서명 판정은 동봉 keyring 이 아니라 trust-domain 의
//! keyring: 낯선 도메인 기준으로는 unknownKey 로 깨진다, ⑤ 사용법 규약.

#![cfg(not(target_arch = "wasm32"))]

use std::io::{Read as _, Write as _};
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
    let dir = std::env::temp_dir().join(format!("rhwp_bundle_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("작업 폴더");
    dir
}

/// 서명·등재된 2링크 체인 + 체크포인트 + 도메인 파일 일습을 만든다.
/// 반환: (머리 캡슐, trust-domain 경로, 앵커 로그, 체크포인트).
fn federation_fixture(dir: &std::path::Path) -> (String, String, String, String) {
    let find = existing_snippet();
    let key = dir.join("k.json");
    let o = run(&[
        "keygen",
        "--key-id",
        "fed.test/agent#1",
        "--out",
        key.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let kd: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&key).unwrap()).unwrap();

    // 체인: run 실산출 → A(뿌리) → B(parent A) — 계보 불변식 성립 구조.
    let o1 = dir.join("o1.hwp");
    let plan_a = serde_json::json!({
        "planVersion": "1.0", "input": SAMPLE,
        "output": o1.to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": find, "replace": find }],
    })
    .to_string();
    let plan_a_path = dir.join("plan_a.json");
    std::fs::write(&plan_a_path, &plan_a).unwrap();
    assert_eq!(
        run(&["run", plan_a_path.to_str().unwrap(), "--json"])
            .status
            .code(),
        Some(0)
    );
    let cap_a = dir.join("a.capsule.json");
    assert_eq!(
        run(&[
            "replay",
            "--plan-json",
            &plan_a,
            "--capsule",
            cap_a.to_str().unwrap(),
            "--sign-key",
            key.to_str().unwrap(),
            "--json"
        ])
        .status
        .code(),
        Some(0)
    );
    let plan_b = serde_json::json!({
        "planVersion": "1.0", "input": o1.to_string_lossy(),
        "output": dir.join("o2.hwp").to_string_lossy(),
        "steps": [{ "action": "replace_text", "find": find, "replace": find }],
    })
    .to_string();
    let cap_b = dir.join("b.capsule.json");
    assert_eq!(
        run(&[
            "replay",
            "--plan-json",
            &plan_b,
            "--capsule",
            cap_b.to_str().unwrap(),
            "--parent",
            cap_a.to_str().unwrap(),
            "--sign-key",
            key.to_str().unwrap(),
            "--json"
        ])
        .status
        .code(),
        Some(0)
    );

    // 앵커 로그 + 체크포인트.
    let log = dir.join("anchor.ndjson");
    for cap in [&cap_a, &cap_b] {
        assert_eq!(
            run(&[
                "anchor",
                "add",
                cap.to_str().unwrap(),
                "--log",
                log.to_str().unwrap(),
                "--json"
            ])
            .status
            .code(),
            Some(0)
        );
    }
    let cp = dir.join("cp.json");
    let o = run(&[
        "anchor",
        "checkpoint",
        "--log",
        log.to_str().unwrap(),
        "-o",
        cp.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let root = env_of(&o)["merkleRoot"].as_str().unwrap().to_string();

    // 수신자 보유 trust-domain — 발신 키·체크포인트를 자기 경로로 받았다는 가정.
    let td = dir.join("trust-domain.json");
    std::fs::write(
        &td,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "trustDomain", "domain": "fed.test",
            "keyring": { "schemaVersion": "1.0", "kind": "keyring",
                "keys": [{ "keyId": "fed.test/agent#1", "publicKey": kd["publicKey"], "revoked": null }] },
            "checkpoints": [{ "upToSeq": 1, "merkleRoot": root }],
        })
        .to_string(),
    )
    .unwrap();
    (
        cap_b.to_string_lossy().into_owned(),
        td.to_string_lossy().into_owned(),
        log.to_string_lossy().into_owned(),
        cp.to_string_lossy().into_owned(),
    )
}

/// zip 안의 한 항목을 바꿔치기해 다시 쓴다.
fn rewrite_bundle_entry(bundle: &std::path::Path, target: &str, mutate: impl Fn(&mut Vec<u8>)) {
    let file = std::fs::File::open(bundle).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    for i in 0..archive.len() {
        let mut e = archive.by_index(i).unwrap();
        let name = e.name().to_string();
        let mut bytes = Vec::new();
        e.read_to_end(&mut bytes).unwrap();
        if name == target {
            mutate(&mut bytes);
        }
        entries.push((name, bytes));
    }
    let out = std::fs::File::create(bundle).unwrap();
    let mut zw = zip::ZipWriter::new(out);
    for (name, bytes) in entries {
        zw.start_file::<_, ()>(name, zip::write::FileOptions::default())
            .unwrap();
        zw.write_all(&bytes).unwrap();
    }
    zw.finish().unwrap();
}

#[test]
fn export_verify_roundtrip_and_three_attacks() {
    let dir = make_dir("fed");
    let (head, td, log, cp) = federation_fixture(&dir);
    let bundle = dir.join("work.lineage-bundle");

    // ── export: 폐쇄집합 2 + 서명 2 + 증명 2.
    let o = run(&[
        "bundle",
        "export",
        &head,
        "-o",
        bundle.to_str().unwrap(),
        "--anchor-log",
        &log,
        "--checkpoint",
        &cp,
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let env = env_of(&o);
    assert_eq!(env["capsules"], 2);
    assert_eq!(env["signatures"], 2);
    assert_eq!(env["proofs"], 2);

    // ── verify: 5단 전건 green.
    let o = run(&[
        "bundle",
        "verify",
        bundle.to_str().unwrap(),
        "--trust-domain",
        &td,
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    let env = env_of(&o);
    assert_eq!(env["verdict"], "ok", "{env}");
    assert_eq!(env["containerOk"], true);
    assert_eq!(env["closureOk"], true);
    assert_eq!(env["lineageValid"], true);
    assert_eq!(env["signed"]["valid"], 2);
    assert_eq!(env["anchored"]["ok"], 2);
    assert_eq!(env["anchored"]["checkpointTrusted"], true);

    // ── 공격 ①: 운송 중 변조 — zip 안 캡슐에 후행 공백.
    let tampered = dir.join("tampered.lineage-bundle");
    std::fs::copy(&bundle, &tampered).unwrap();
    rewrite_bundle_entry(&tampered, "capsules/b.capsule.json", |b| b.push(b' '));
    let o = run(&[
        "bundle",
        "verify",
        tampered.to_str().unwrap(),
        "--trust-domain",
        &td,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "운송 변조 = 검증 단언 실패");
    assert_eq!(env_of(&o)["containerOk"], false);

    // ── 공격 ②: 조상 은닉 — 부모 캡슐 항목을 빈 이름 없는… (매니페스트는 그대로,
    //    부모 파일만 다른 내용으로 바꿔치기하면 컨테이너가 먼저 잡으므로, 폐쇄집합
    //    검사를 겨냥해 매니페스트에서 부모 항목·파일을 함께 제거한 번들을 만든다.)
    let dropped = dir.join("dropped.lineage-bundle");
    {
        let file = std::fs::File::open(&bundle).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let out = std::fs::File::create(&dropped).unwrap();
        let mut zw = zip::ZipWriter::new(out);
        for i in 0..archive.len() {
            let mut e = archive.by_index(i).unwrap();
            let name = e.name().to_string();
            if name == "capsules/a.capsule.json" || name == "signatures/a.capsule.json.sig.json" {
                continue; // 조상 은닉
            }
            let mut bytes = Vec::new();
            e.read_to_end(&mut bytes).unwrap();
            if name == "manifest.json" {
                let mut m: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                let files = m["files"].as_array().cloned().unwrap();
                m["files"] = serde_json::json!(files
                    .into_iter()
                    .filter(|f| !f["path"].as_str().unwrap_or("").contains("a.capsule.json"))
                    .collect::<Vec<_>>());
                bytes = serde_json::to_vec_pretty(&m).unwrap();
            }
            zw.start_file::<_, ()>(name, zip::write::FileOptions::default())
                .unwrap();
            zw.write_all(&bytes).unwrap();
        }
        zw.finish().unwrap();
    }
    let o = run(&[
        "bundle",
        "verify",
        dropped.to_str().unwrap(),
        "--trust-domain",
        &td,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "조상 은닉 = 폐쇄집합 위반");
    assert_eq!(env_of(&o)["closureOk"], false);

    // ── 공격 ③ (F2): 낯선 도메인 — 다른 키링의 수신자에겐 unknownKey.
    let stranger = dir.join("stranger-domain.json");
    let o = run(&[
        "keygen",
        "--key-id",
        "stranger/x#1",
        "--out",
        dir.join("s.json").to_str().unwrap(),
        "--json",
    ]);
    let pk = env_of(&o)["publicKey"].clone();
    std::fs::write(
        &stranger,
        serde_json::json!({
            "schemaVersion": "1.0", "kind": "trustDomain", "domain": "stranger",
            "keyring": { "schemaVersion": "1.0", "kind": "keyring",
                "keys": [{ "keyId": "stranger/x#1", "publicKey": pk, "revoked": null }] },
            "checkpoints": [],
        })
        .to_string(),
    )
    .unwrap();
    let o = run(&[
        "bundle",
        "verify",
        bundle.to_str().unwrap(),
        "--trust-domain",
        stranger.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3), "동봉 keyring 을 믿지 않는다(F2)");
    let env = env_of(&o);
    assert_eq!(env["signed"]["invalid"], 2, "{env}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn usage_conventions() {
    let o = run(&["bundle"]);
    assert_eq!(o.status.code(), Some(2));
    let o = run(&["bundle", "verify", "nope.lineage-bundle"]);
    assert_eq!(
        o.status.code(),
        Some(2),
        "--trust-domain 없는 verify 는 사용법 오류"
    );
    let o = run(&["bundle", "export", "--json"]);
    assert_eq!(o.status.code(), Some(2));
}
