//! [#4401] 작업 계보 계약 — `replay --parent` 링크 + `rhwp lineage` 체인 검증.
//!
//! 고정하는 것: ① 2링크 왕복 — 실산출(run)→캡슐 A→캡슐 B(parent A) 체인이
//! 유효(depth 2)하고, **계보 불변식**(부모 산출 해시 == 자식 입력 해시)이
//! run↔replay 교차 결정론 위에서 성립한다, ② 부모 파일 사후 변조는 기록 해시
//! 대조(parentOk:false)로 폭로되고 exit 3 + brokenAt 명세, ③ `--deep` 은 링크마다
//! 재실행 재현을 판정한다, ④ 머리 캡슐 없음/무인자/미지 옵션의 실패 규약.

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

fn run_in(dir: &std::path::Path, args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .current_dir(dir)
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

fn make_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp_lineage_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("계보 폴더");
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

fn lineage(head: &std::path::Path, deep: bool) -> (Option<i32>, serde_json::Value) {
    let head = head.to_str().unwrap();
    let mut args = vec!["lineage", head, "--json"];
    if deep {
        args.insert(2, "--deep");
    }
    let o = run(&args);
    let env = serde_json::from_slice(&o.stdout).unwrap_or(serde_json::json!({}));
    (o.status.code(), env)
}

#[test]
fn two_link_chain_is_valid_and_tampered_parent_is_exposed() {
    let dir = make_dir("chain");
    let find = existing_snippet();

    // 실작업 1: run 으로 O1 을 실제 디스크에 산출.
    let o1 = dir.join("o1.hwp");
    let plan_a = plan_json(SAMPLE, &o1, &find);
    let plan_a_path = dir.join("plan_a.json");
    std::fs::write(&plan_a_path, &plan_a).unwrap();
    let o = run(&["run", plan_a_path.to_str().unwrap(), "--json"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert!(o1.exists());

    // 캡슐 A 발급 (부모 없음 — 계보의 뿌리).
    let cap_a = dir.join("a.capsule.json");
    let o = run(&[
        "replay",
        "--plan-json",
        &plan_a,
        "--capsule",
        cap_a.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );

    // 뿌리 하나만으로도 계보는 유효 — depth 1, 판정 축 3종은 전부 null.
    let (code, env) = lineage(&cap_a, false);
    assert_eq!(code, Some(0), "{env}");
    assert_eq!(env["depth"], 1);
    assert_eq!(env["valid"], true);
    assert_eq!(env["links"][0]["parentOk"], serde_json::Value::Null);
    assert_eq!(env["links"][0]["lineageOk"], serde_json::Value::Null);

    // 실작업 2: O1 을 입력으로 하는 캡슐 B — parent 로 A 를 지목.
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
        "--json",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    let b: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&cap_b).unwrap()).unwrap();
    assert_eq!(b["parent"]["capsule"], "a.capsule.json");
    assert_eq!(b["parent"]["sha256"].as_str().map(str::len), Some(64));

    // 체인 유효 — parentOk(파일 무결)와 lineageOk(부모 산출=자식 입력)가 모두 참.
    // lineageOk 는 run 이 쓴 O1 과 replay 의 임시 재실행이 같은 바이트라는
    // run↔replay 교차 결정론의 직접 증거다.
    let (code, env) = lineage(&cap_b, false);
    assert_eq!(code, Some(0), "{env}");
    assert_eq!(env["depth"], 2);
    assert_eq!(env["valid"], true);
    assert_eq!(env["brokenAt"], serde_json::Value::Null);
    assert_eq!(env["links"][1]["parentOk"], true);
    assert_eq!(env["links"][1]["lineageOk"], true, "계보 불변식: {env}");

    // --deep: 링크마다 재실행 재현까지.
    let (code, env) = lineage(&cap_b, true);
    assert_eq!(code, Some(0), "{env}");
    assert_eq!(env["links"][0]["reproduced"], true);
    assert_eq!(env["links"][1]["reproduced"], true);

    // 링크 해시를 지우면 검증을 생략하지 않고 머리 캡슐에서 즉시 실패한다.
    let b_original = std::fs::read_to_string(&cap_b).unwrap();
    let mut b_without_sha = b.clone();
    b_without_sha["parent"]
        .as_object_mut()
        .unwrap()
        .remove("sha256");
    std::fs::write(
        &cap_b,
        serde_json::to_string_pretty(&b_without_sha).unwrap(),
    )
    .unwrap();
    let (code, env) = lineage(&cap_b, false);
    assert_eq!(code, Some(3), "누락 해시는 fail-closed: {env}");
    assert_eq!(env["valid"], false);
    assert_eq!(env["brokenAt"], cap_b.to_str().unwrap());
    assert!(env["links"][0]["error"]
        .as_str()
        .unwrap_or_default()
        .contains("parent.sha256"));
    std::fs::write(&cap_b, &b_original).unwrap();

    // parent 필드 자체가 없으면 합법 root 로 오인하지 않고 즉시 실패한다.
    let mut b_without_parent = b.clone();
    b_without_parent.as_object_mut().unwrap().remove("parent");
    std::fs::write(
        &cap_b,
        serde_json::to_string_pretty(&b_without_parent).unwrap(),
    )
    .unwrap();
    let (code, env) = lineage(&cap_b, false);
    assert_eq!(code, Some(3), "누락 parent 필드는 fail-closed: {env}");
    assert_eq!(env["valid"], false);
    assert_eq!(env["brokenAt"], cap_b.to_str().unwrap());
    assert!(env["links"][0]["error"]
        .as_str()
        .unwrap_or_default()
        .contains("parent 필드"));
    std::fs::write(&cap_b, &b_original).unwrap();

    // 계획 해시나 캡슐 계획이 누락·변조되면 shallow 검사도 fail-closed 한다.
    let mut b_without_plan_sha = b.clone();
    b_without_plan_sha["receipt"]
        .as_object_mut()
        .unwrap()
        .remove("planSha256");
    std::fs::write(
        &cap_b,
        serde_json::to_string_pretty(&b_without_plan_sha).unwrap(),
    )
    .unwrap();
    let (code, env) = lineage(&cap_b, false);
    assert_eq!(code, Some(3), "누락 계획 해시는 fail-closed: {env}");
    assert!(env["links"][0]["error"]
        .as_str()
        .unwrap_or_default()
        .contains("planSha256"));

    let mut b_with_tampered_plan = b.clone();
    b_with_tampered_plan["plan"]["steps"] = serde_json::json!([]);
    std::fs::write(
        &cap_b,
        serde_json::to_string_pretty(&b_with_tampered_plan).unwrap(),
    )
    .unwrap();
    let (code, env) = lineage(&cap_b, false);
    assert_eq!(code, Some(3), "계획 변조는 fail-closed: {env}");
    assert!(env["links"][0]["error"]
        .as_str()
        .unwrap_or_default()
        .contains("plan 과 planText"));

    let mut b_with_tampered_steps = b.clone();
    b_with_tampered_steps["receipt"]["steps"] = serde_json::json!(999);
    std::fs::write(
        &cap_b,
        serde_json::to_string_pretty(&b_with_tampered_steps).unwrap(),
    )
    .unwrap();
    let (code, env) = lineage(&cap_b, false);
    assert_eq!(code, Some(3), "shallow step 변조도 fail-closed: {env}");
    assert!(env["links"][0]["error"]
        .as_str()
        .unwrap_or_default()
        .contains("plan.steps 길이와 receipt.steps"));
    std::fs::write(&cap_b, &b_original).unwrap();

    // 부모 캡슐을 사후 변조 — 자식이 기록한 파일 해시가 폭로한다.
    let mut a: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&cap_a).unwrap()).unwrap();
    a["receipt"]["outputSha256"] = serde_json::json!(ZERO64);
    std::fs::write(&cap_a, serde_json::to_string_pretty(&a).unwrap()).unwrap();

    let (code, env) = lineage(&cap_b, false);
    assert_eq!(code, Some(3), "변조된 계보 = 검증 단언 실패: {env}");
    assert_eq!(env["valid"], false);
    assert_eq!(env["brokenAt"], cap_a.to_str().unwrap());
    assert_eq!(env["links"][1]["parentOk"], false);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bare_capsule_filename_resolves_parent_against_current_directory() {
    let dir = make_dir("bare_filename");
    let input = std::fs::canonicalize(SAMPLE).expect("샘플 절대 경로");
    let plan = plan_json(
        input.to_str().unwrap(),
        &dir.join("bare-output.hwp"),
        &existing_snippet(),
    );
    std::fs::write(dir.join("a.capsule.json"), b"parent bytes").unwrap();

    let o = run_in(
        &dir,
        &[
            "replay",
            "--plan-json",
            &plan,
            "--capsule",
            "b.capsule.json",
            "--parent",
            "a.capsule.json",
            "--json",
        ],
    );
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let capsule: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("b.capsule.json")).unwrap())
            .unwrap();
    assert_eq!(capsule["parent"]["capsule"], "a.capsule.json");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn capsule_path_cannot_overwrite_the_same_parent_file() {
    let dir = make_dir("same_parent");
    let parent = dir.join("parent.capsule.json");
    let original = b"parent sentinel bytes";
    std::fs::write(&parent, original).unwrap();
    let plan = plan_json(SAMPLE, &dir.join("unused-output.hwp"), &existing_snippet());

    let o = run(&[
        "replay",
        "--plan-json",
        &plan,
        "--capsule",
        parent.to_str().unwrap(),
        "--parent",
        parent.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stdout.is_empty(), "거절 경로 stdout은 비어야 한다");
    assert!(String::from_utf8_lossy(&o.stderr).contains("같은 기존 파일"));
    assert_eq!(std::fs::read(&parent).unwrap(), original);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn capsule_symlink_alias_cannot_overwrite_the_parent_file() {
    use std::os::unix::fs::symlink;

    let dir = make_dir("parent_symlink");
    let parent = dir.join("parent.capsule.json");
    let alias = dir.join("alias.capsule.json");
    let original = b"parent sentinel bytes";
    std::fs::write(&parent, original).unwrap();
    symlink(&parent, &alias).unwrap();
    let plan = plan_json(SAMPLE, &dir.join("unused-output.hwp"), &existing_snippet());

    let o = run(&[
        "replay",
        "--plan-json",
        &plan,
        "--capsule",
        alias.to_str().unwrap(),
        "--parent",
        parent.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stdout.is_empty(), "거절 경로 stdout은 비어야 한다");
    assert!(String::from_utf8_lossy(&o.stderr).contains("같은 기존 파일"));
    assert_eq!(std::fs::read(&parent).unwrap(), original);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn invalid_utf8_capsule_is_not_lossily_accepted_as_json() {
    let dir = make_dir("invalid_utf8");
    let cap = dir.join("invalid.capsule.json");
    let plan = plan_json(SAMPLE, &dir.join("unused-output.hwp"), &existing_snippet());
    let o = run(&[
        "replay",
        "--plan-json",
        &plan,
        "--capsule",
        cap.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(o.status.code(), Some(0));

    // U+FFFD로 치환하면 유효한 JSON 문자열이 되므로 기존 lossy parser는 통과했다.
    let mut bytes = std::fs::read(&cap).unwrap();
    let closing = bytes
        .iter()
        .rposition(|byte| *byte == b'}')
        .expect("최상위 JSON 닫힘");
    let mut injected = b",\n  \"invalidUtf8\": \"".to_vec();
    injected.push(0xff);
    injected.extend_from_slice(b"\"\n");
    bytes.splice(closing..closing, injected);
    std::fs::write(&cap, bytes).unwrap();

    let (code, env) = lineage(&cap, false);
    assert_eq!(code, Some(3), "invalid UTF-8은 계보 실패: {env}");
    assert_eq!(env["valid"], false);
    assert_eq!(env["brokenAt"], cap.to_str().unwrap());
    assert!(env["links"][0]["error"]
        .as_str()
        .unwrap_or_default()
        .contains("JSON 파싱 실패"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn failure_conventions() {
    // 무인자 → 사용법 오류.
    let o = run(&["lineage"]);
    assert_eq!(o.status.code(), Some(2));
    // 미지 옵션 → 사용법 오류.
    let o = run(&["lineage", "x.capsule.json", "--nope"]);
    assert_eq!(o.status.code(), Some(2));
    // 머리 캡슐이 없음 → 실행 오류(1) + stdout 0바이트 (실패 stdout 순수성).
    let o = run(&["lineage", "definitely_missing.capsule.json", "--json"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        o.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
}
