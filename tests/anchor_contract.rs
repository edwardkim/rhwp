//! [#4543] 앵커 계약 — 투명성 로그(add·checkpoint·verify)와 lineage 통합.
//!
//! 고정하는 것: ① add 가 줄 해시 체인으로 append-only 를 강제하고 **깨진
//! 로그에는 등재를 거부**한다(exit 3), ② 중간 줄 사후 변조는 다음 줄의 기록
//! 해시가 폭로한다, ③ checkpoint 머클 루트와 verify 의 경로 증명이 잎→루트로
//! 재계산된다, ④ 미등재 캡슐은 logged:false·exit 3, ⑤ lineage `--anchor-log`
//! 는 opt-in 6번째 축(anchoredOk) — 미지정 시 필드 부재(무파손), 미등재는
//! false 이되 체인을 깨지 않는다(등재 강제는 게이트의 직무).

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
    let dir = std::env::temp_dir().join(format!("rhwp_anchor_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("작업 폴더");
    dir
}

/// 캡슐 3개를 발급해 경로 목록을 돌려준다.
fn three_capsules(dir: &std::path::Path) -> Vec<String> {
    let find = existing_snippet();
    (0..3)
        .map(|i| {
            let cap = dir.join(format!("c{i}.capsule.json"));
            let plan = serde_json::json!({
                "planVersion": "1.0",
                "input": SAMPLE,
                "output": dir.join(format!("o{i}.hwp")).to_string_lossy(),
                "steps": [{ "action": "replace_text", "find": find, "replace": format!("앵커{i}") }],
            })
            .to_string();
            let o = run(&["replay", "--plan-json", &plan, "--capsule", cap.to_str().unwrap(), "--json"]);
            assert_eq!(o.status.code(), Some(0));
            cap.to_string_lossy().into_owned()
        })
        .collect()
}

#[test]
fn append_chain_checkpoint_merkle_and_tamper() {
    let dir = make_dir("chain");
    let caps = three_capsules(&dir);
    let log = dir.join("anchor.ndjson");
    let log_s = log.to_string_lossy().into_owned();

    // ── 등재 3건 — seq 연번.
    for (i, cap) in caps.iter().enumerate() {
        let o = run(&["anchor", "add", cap, "--log", &log_s, "--json"]);
        assert_eq!(
            o.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&o.stderr)
        );
        assert_eq!(env_of(&o)["seq"], i);
    }

    // ── 체크포인트 — 머클 루트.
    let cp = dir.join("cp.json");
    let o = run(&[
        "anchor",
        "checkpoint",
        "--log",
        &log_s,
        "-o",
        cp.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let env = env_of(&o);
    assert_eq!(env["upToSeq"], 2);
    assert_eq!(env["merkleRoot"].as_str().map(str::len), Some(64));

    // ── verify — 등재·무결·머클 경로 전부 참.
    let o = run(&[
        "anchor",
        "verify",
        &caps[1],
        "--log",
        &log_s,
        "--checkpoint",
        cp.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    let env = env_of(&o);
    assert_eq!(env["logged"], true);
    assert_eq!(env["seq"], 1);
    assert_eq!(env["logChainOk"], true);
    assert_eq!(env["inCheckpoint"], true, "{env}");
    assert!(env["merklePath"].as_array().is_some_and(|p| !p.is_empty()));

    // ── 미등재 캡슐 — logged:false, exit 3 (판정은 데이터).
    let stranger = dir.join("stranger.capsule.json");
    std::fs::write(&stranger, b"{\"kind\":\"workCapsule\"}").unwrap();
    let o = run(&[
        "anchor",
        "verify",
        stranger.to_str().unwrap(),
        "--log",
        &log_s,
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(3));
    assert_eq!(env_of(&o)["logged"], false);

    // ── 중간 줄 사후 변조 — 다음 줄의 기록 해시가 폭로하고, 깨진 로그에는
    //    등재도 거부된다.
    let text = std::fs::read_to_string(&log).unwrap();
    let tampered = text.replacen("anchorLog", "anchorLoG", 1);
    assert_ne!(text, tampered);
    std::fs::write(&log, tampered).unwrap();
    let o = run(&["anchor", "verify", &caps[0], "--log", &log_s, "--json"]);
    assert_eq!(o.status.code(), Some(3), "변조 로그 = 검증 단언 실패");
    let o = run(&["anchor", "add", &caps[0], "--log", &log_s, "--json"]);
    assert_eq!(o.status.code(), Some(3), "깨진 로그에는 등재 거부");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn lineage_anchored_axis_is_optin_and_reports_data() {
    let dir = make_dir("lineage");
    let caps = three_capsules(&dir);
    let log = dir.join("anchor.ndjson");
    let log_s = log.to_string_lossy().into_owned();
    // 첫 캡슐만 등재.
    let o = run(&["anchor", "add", &caps[0], "--log", &log_s, "--json"]);
    assert_eq!(o.status.code(), Some(0));

    // opt-in 무파손: --anchor-log 없으면 필드 자체가 없다.
    let o = run(&["lineage", &caps[0], "--json"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(env_of(&o)["links"][0].get("anchoredOk").is_none());

    // 등재 캡슐 true / 미등재 캡슐 false — 단, 체인은 깨지 않는다.
    let o = run(&["lineage", &caps[0], "--anchor-log", &log_s, "--json"]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(env_of(&o)["links"][0]["anchoredOk"], true);
    let o = run(&["lineage", &caps[1], "--anchor-log", &log_s, "--json"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "미등재는 데이터이지 파손이 아니다"
    );
    let env = env_of(&o);
    assert_eq!(env["links"][0]["anchoredOk"], false);
    assert_eq!(env["valid"], true);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn usage_conventions() {
    let o = run(&["anchor"]);
    assert_eq!(o.status.code(), Some(2));
    let o = run(&["anchor", "add", "x.capsule.json"]);
    assert_eq!(o.status.code(), Some(2), "--log 없는 add 는 사용법 오류");
    let dir = make_dir("usage");
    let empty = dir.join("empty.ndjson");
    std::fs::write(&empty, b"").unwrap();
    let o = run(&[
        "anchor",
        "checkpoint",
        "--log",
        empty.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2), "빈 로그 체크포인트는 사용법 오류");
    let _ = std::fs::remove_dir_all(&dir);
}
