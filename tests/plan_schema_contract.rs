//! [#3719 §6-4·§6-8] `export-plan-schema` 계약과 조건부 step.
//!
//! 스키마는 계획을 **만드는** 쪽이 읽는 문서다. 그래서 여기서 지키는 것은 두 가지다 —
//! (1) 스키마 자체의 건전성(끊어진 참조·설명 없는 정의가 없을 것),
//! (2) **스키마와 실행기가 같은 말을 할 것**. 스키마가 광고하는 action·조건을 실행기가
//! 모르거나, 실행기가 아는 것을 스키마가 빠뜨리면 계획 생성기는 통과할 리 없는 계획을
//! 자신 있게 만들어 낸다. 그 어긋남은 컴파일 에러도 런타임 오류도 내지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-planschema-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

/// 테스트가 중간에 죽어도 임시 산출물을 남기지 않는다.
struct TempFileGuard(PathBuf);

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn schema_body() -> serde_json::Value {
    let out = run(&["export-plan-schema", "--bare"]);
    assert_eq!(out.status.code(), Some(0), "--bare 실행 실패");
    serde_json::from_slice(&out.stdout).expect("스키마 본문 JSON")
}

/// 계획을 파일에 쓰고 `run --json` 으로 돌린 뒤 (종료 코드, 저널) 을 돌려준다.
fn run_plan(tag: &str, plan: &serde_json::Value) -> (i32, serde_json::Value) {
    let plan_path = TempFileGuard::new(temp_path(tag, "json"));
    std::fs::write(plan_path.path(), serde_json::to_vec_pretty(plan).unwrap()).unwrap();
    let output = run(&[
        "run",
        plan_path.path().to_str().expect("계획 경로 UTF-8"),
        "--json",
    ]);
    let journal: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "저널 JSON 파싱 실패({e}). stdout: {} stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), journal)
}

/// 샘플의 누름틀 (이름, 현재값) 목록 — 문서 순서 = 동명 필드의 순번 순서.
///
/// 이름·값을 테스트에 하드코딩하면 샘플이 바뀔 때 **조용히** 낡는다(그 테스트는 계속
/// 통과하지만 아무것도 지키지 않게 된다). 실물에서 읽어 쓴다.
fn sample_fields() -> Vec<(String, String)> {
    let p = sample();
    if !p.exists() {
        return Vec::new();
    }
    let out = run(&["fields", p.to_str().unwrap(), "--json"]);
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return Vec::new();
    };
    v["fields"]
        .as_array()
        .map(|fields| {
            fields
                .iter()
                .filter_map(|f| {
                    let name = f["name"].as_str().filter(|n| !n.is_empty())?;
                    Some((
                        name.to_string(),
                        f["value"].as_str().unwrap_or("").to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 샘플의 실제 누름틀 이름 하나.
fn a_real_field_name() -> Option<String> {
    sample_fields().into_iter().next().map(|(name, _)| name)
}

/// 샘플 본문 텍스트 전체 — `textFound` 조건의 사례를 실물에서 고르기 위한 것.
fn sample_text() -> String {
    let p = sample();
    if !p.exists() {
        return String::new();
    }
    let out = run(&["export-text", p.to_str().unwrap(), "--json"]);
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return String::new();
    };
    v["pages"]
        .as_array()
        .map(|pages| {
            pages
                .iter()
                .filter_map(|p| p["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

/// 계획 하나를 돌려 저널의 step 배열만 돌려준다 (exit 0 을 단언).
fn journal_steps(tag: &str, steps: serde_json::Value) -> serde_json::Value {
    let out = TempFileGuard::new(temp_path(tag, "hwp"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": sample().to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": steps,
    });
    let (code, journal) = run_plan(tag, &plan);
    assert_eq!(code, 0, "계획이 실행되지 않았다: {journal}");
    journal["steps"].clone()
}

// ── 1) 봉투·플래그 계약 ──────────────────────────────────────────────────

#[test]
fn envelope_carries_version_dialect_and_definition_count() {
    let args = ["export-plan-schema"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{args:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("봉투 JSON");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["planSchemaVersion"], "1.0", "{v}");
    assert_eq!(
        v["dialect"], "https://json-schema.org/draft/2020-12/schema",
        "소비자가 파서를 고를 수 있어야 한다: {v}"
    );
    // 스키마가 통째로 비는 회귀를 숫자 하나로 잡는다.
    let count = v["definitionCount"].as_u64().expect("definitionCount");
    let defs = v["schema"]["$defs"].as_object().expect("$defs");
    assert_eq!(
        count as usize,
        defs.len(),
        "definitionCount 가 실제 정의 수와 다르다"
    );
    assert!(count >= 9, "정의가 너무 적다: {count}");
}

#[test]
fn bare_emits_the_schema_body_without_the_envelope() {
    let v = schema_body();
    assert_eq!(v["$ref"], "#/$defs/Plan", "{v}");
    assert!(v["$defs"]["Plan"].is_object());
    assert!(
        v.get("definitionCount").is_none(),
        "--bare 는 봉투 키를 싣지 않는다 — JSON Schema 도구에 그대로 먹이는 용도다"
    );
}

#[test]
fn output_file_and_json_report() {
    let out_file = TempFileGuard::new(temp_path("out", "json"));
    let path = out_file.path().to_str().expect("경로 UTF-8").to_string();
    let out = run(&["export-plan-schema", "-o", &path, "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("저장 보고 봉투");
    assert_eq!(v["output"], path.as_str(), "{v}");
    let bytes = v["bytes"].as_u64().expect("bytes");
    let written = std::fs::read(out_file.path()).expect("저장된 스키마");
    assert_eq!(written.len() as u64, bytes, "보고한 크기와 실제가 다르다");
    let parsed: serde_json::Value = serde_json::from_slice(&written).expect("저장본도 JSON");
    assert_eq!(
        parsed["definitionCount"].as_u64(),
        parsed["schema"]["$defs"]
            .as_object()
            .map(|d| d.len() as u64),
        "저장본이 반쪽만 쓰이면 여기서 걸린다: {parsed}"
    );
}

#[test]
fn unknown_option_is_usage_error_with_silent_stdout() {
    let out = run(&["export-plan-schema", "--nope"]);
    assert_eq!(out.status.code(), Some(2), "알 수 없는 옵션 = 사용법 오류");
    assert!(
        out.stdout.is_empty(),
        "실패 경로의 stdout 은 0바이트여야 한다 — 파이프로 받는 쪽이 반쪽 JSON 을 먹으면 안 된다: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// ── 2) 스키마 건전성 ─────────────────────────────────────────────────────

#[test]
fn every_ref_resolves_and_nothing_is_orphaned() {
    let schema = schema_body();
    let defs = schema["$defs"].as_object().expect("$defs");
    let mut referenced = std::collections::HashSet::new();
    referenced.insert("Plan".to_string());
    let mut missing = Vec::new();
    collect_refs(&schema, &mut |name| {
        referenced.insert(name.to_string());
        if !defs.contains_key(name) {
            missing.push(name.to_string());
        }
    });
    assert!(missing.is_empty(), "정의되지 않은 참조: {missing:?}");
    let orphans: Vec<&String> = defs.keys().filter(|k| !referenced.contains(*k)).collect();
    assert!(orphans.is_empty(), "아무도 참조하지 않는 정의: {orphans:?}");
}

#[test]
fn every_definition_and_property_carries_a_description() {
    // 설명 없는 정의는 모델에게 없는 정의다 — 이름만 보고 무엇을 넣을지 알 수 없다.
    // 유닛 테스트와 겹치지만, 여기서는 **실제 바이너리가 낸 산출물**을 본다.
    let schema = schema_body();
    let mut missing = Vec::new();
    for (name, def) in schema["$defs"].as_object().expect("$defs") {
        check_described(def, name, &mut missing);
    }
    assert!(
        missing.is_empty(),
        "설명 없는 스키마 노드 {}건: {missing:?}",
        missing.len()
    );
}

// ── 3) 스키마 ↔ 실행기 드리프트 가드 ────────────────────────────────────

#[test]
fn schema_actions_are_exactly_what_the_engine_accepts() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let schema = schema_body();
    let variants = schema["$defs"]["Step"]["oneOf"]
        .as_array()
        .expect("Step.oneOf");
    assert!(!variants.is_empty(), "분기가 0건이면 이 가드는 공허하다");

    // 스키마가 광고하는 action 은 전부 실행기가 알아야 한다. 필드가 빠진 껍데기 step 을
    // 보내 "왜 불가한지"만 본다 — 문서 내용을 몰라도 되는 판정이다.
    for variant in variants {
        let name = variant["$ref"]
            .as_str()
            .and_then(|r| r.strip_prefix("#/$defs/"))
            .expect("oneOf 분기는 $defs 참조");
        let action = schema["$defs"][name]["properties"]["action"]["const"]
            .as_str()
            .unwrap_or_else(|| panic!("{name} 에 action const 가 없다"));
        let out = TempFileGuard::new(temp_path(&format!("action-{action}"), "hwp"));
        let plan = serde_json::json!({
            "planVersion": "1.0",
            "input": p.to_str().unwrap(),
            "output": out.path().to_str().unwrap(),
            "steps": [ { "action": action } ],
        });
        let (code, journal) = run_plan(&format!("action-{action}"), &plan);
        assert_eq!(code, 2, "껍데기 step 은 선검증에서 걸린다: {journal}");
        let reason = journal["invalid"][0]["reason"].as_str().unwrap_or("");
        assert!(
            !reason.contains("알 수 없는 action"),
            "스키마가 광고하는 action '{action}' 을 실행기가 모른다: {reason}"
        );
    }

    // 반대 방향 — 스키마에 없는 action 은 실행기도 거절해야 한다. 이쪽이 무너지면
    // 스키마는 "표면 전체"가 아니라 "표면의 일부"가 되고, 소비자는 그 사실을 모른다.
    let out = TempFileGuard::new(temp_path("action-unknown", "hwp"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [ { "action": "delete_page" } ],
    });
    let (code, journal) = run_plan("action-unknown", &plan);
    assert_eq!(code, 2, "{journal}");
    assert!(
        journal["invalid"][0]["reason"]
            .as_str()
            .unwrap_or("")
            .contains("알 수 없는 action"),
        "스키마에 없는 action 을 실행기가 받아들였다: {journal}"
    );
}

#[test]
fn schema_conditions_are_exactly_what_the_engine_accepts() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let schema = schema_body();
    let keys: Vec<String> = schema["$defs"]["StepCondition"]["properties"]
        .as_object()
        .expect("StepCondition.properties")
        .keys()
        .cloned()
        .collect();
    assert!(!keys.is_empty(), "조건이 0종이면 이 가드는 공허하다");
    assert_eq!(
        schema["$defs"]["StepCondition"]["additionalProperties"], false,
        "조건절은 닫혀 있어야 한다 — 실행기가 모르는 키를 거절하기 때문이다"
    );

    for key in &keys {
        // 각 조건이 **거짓**이 되는 사례를 만든다. 통과하면 (1) 실행기가 그 키를 알고
        // (2) 거짓 판정이 skipped 저널로 나온다는 두 사실이 한 번에 증명된다.
        let condition = match key.as_str() {
            "fieldExists" => serde_json::json!({ "fieldExists": "이런누름틀은없다9999" }),
            "fieldEquals" => serde_json::json!({
                "fieldEquals": { "name": "이런누름틀은없다9999", "value": "값" }
            }),
            "textFound" => serde_json::json!({ "textFound": "이런문자열은문서에없다9999" }),
            other => panic!(
                "스키마에 새 조건 '{other}' 이 생겼는데 이 테스트에 거짓 사례가 없습니다 — \
                 조건을 늘렸으면 여기도 늘리세요(그러지 않으면 새 조건은 검증 밖에 있습니다)"
            ),
        };
        let out = TempFileGuard::new(temp_path(&format!("cond-{key}"), "hwp"));
        let plan = serde_json::json!({
            "planVersion": "1.0",
            "input": p.to_str().unwrap(),
            "output": out.path().to_str().unwrap(),
            // 실행기가 조건을 몰랐다면 이 step 은 선검증에서 걸려 exit 2 가 된다.
            "steps": [ { "action": "set_checkbox", "occurrence": 999999, "if": condition } ],
        });
        let (code, journal) = run_plan(&format!("cond-{key}"), &plan);
        assert_eq!(
            code, 0,
            "조건 '{key}' 이 거짓이면 step 을 건너뛰고 선검증도 면제되어야 한다: {journal}"
        );
        let step = &journal["steps"][0];
        assert_eq!(step["skipped"], true, "조건 '{key}': {journal}");
        assert!(
            step["reason"]
                .as_str()
                .is_some_and(|r| r.contains(key.as_str())),
            "건너뛴 사유에 어떤 조건이었는지 남아야 한다 (조건 '{key}'): {journal}"
        );
    }

    // 스키마에 없는 조건 키는 실행기도 거절해야 한다 (문법 오류 = invalid + exit 2).
    let out = TempFileGuard::new(temp_path("cond-unknown", "hwp"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [ { "action": "set_checkbox", "occurrence": 0, "if": { "fieldMissing": "x" } } ],
    });
    let (code, journal) = run_plan("cond-unknown", &plan);
    assert_eq!(code, 2, "{journal}");
    assert!(
        journal["invalid"][0]["reason"]
            .as_str()
            .unwrap_or("")
            .contains("알 수 없는 조건"),
        "스키마에 없는 조건을 실행기가 받아들였다: {journal}"
    );
    assert!(
        !out.path().exists(),
        "조건 문법 오류는 실행 0 — 산출물이 생기면 안 된다"
    );
}

// ── 4) 조건부 step 동작 (§6-8) ──────────────────────────────────────────

#[test]
fn false_condition_skips_the_step_and_exempts_prevalidation() {
    let p = sample();
    let Some(field) = a_real_field_name() else {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    };
    let out = TempFileGuard::new(temp_path("exempt", "hwp"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": { field.as_str(): "실행됨" } },
            // 조건이 없으면 이 step 은 선검증 위반(없는 필드)이라 계획 전체가 exit 2 다.
            // 조건이 거짓이므로 면제되어야 한다 — 여기가 §6-8 의 핵심 계약이다.
            { "action": "fill_fields", "data": { "이런누름틀은없다9999": "값" },
              "if": { "fieldExists": "이런누름틀은없다9999" } },
        ],
        "assertions": { "verify": true },
    });
    let (code, journal) = run_plan("exempt", &plan);
    assert_eq!(code, 0, "거짓 조건 step 은 위반이 아니다: {journal}");
    let steps = journal["steps"].as_array().expect("steps[]");
    assert_eq!(steps.len(), 2, "건너뛴 step 도 저널에 남는다: {journal}");
    assert_eq!(steps[0]["filledCount"], 1, "{journal}");
    assert!(steps[0].get("skipped").is_none(), "{journal}");
    assert_eq!(steps[1]["skipped"], true, "{journal}");
    assert_eq!(steps[1]["step"], 1, "0 기준 step 번호 유지: {journal}");
    assert_eq!(steps[1]["action"], "fill_fields", "{journal}");
    assert!(
        steps[1]["reason"].as_str().is_some_and(|r| !r.is_empty()),
        "왜 건너뛰었는지 없이 사라지면 '왜 안 바뀌었는지' 를 알 수 없다: {journal}"
    );
    assert!(out.path().exists(), "나머지 step 은 정상 저장된다");
}

#[test]
fn true_condition_runs_the_step() {
    let p = sample();
    let Some(field) = a_real_field_name() else {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    };
    let out = TempFileGuard::new(temp_path("cond-true", "hwp"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": { field.as_str(): "조건참" },
              "if": { "fieldExists": field.as_str() } },
        ],
        "assertions": { "verify": true },
    });
    let (code, journal) = run_plan("cond-true", &plan);
    assert_eq!(code, 0, "{journal}");
    let step = &journal["steps"][0];
    assert!(
        step.get("skipped").is_none(),
        "조건 참이면 실행한다: {journal}"
    );
    assert_eq!(step["filledCount"], 1, "{journal}");

    // 왕복 재독 — 조건 참인 step 이 실제로 문서를 바꿨는가.
    let fields = run(&["fields", out.path().to_str().unwrap(), "--json"]);
    assert_eq!(fields.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&fields.stdout).contains("조건참"),
        "산출물 재독에 새 값이 없다"
    );
}

#[test]
fn human_mode_reports_skipped_steps_and_does_not_count_them_as_applied() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = TempFileGuard::new(temp_path("human", "hwp"));
    let plan_path = TempFileGuard::new(temp_path("human", "json"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [
            { "action": "set_checkbox", "occurrence": 999999,
              "if": { "textFound": "이런문자열은문서에없다9999" } },
        ],
    });
    std::fs::write(plan_path.path(), serde_json::to_vec_pretty(&plan).unwrap()).unwrap();
    let output = run(&["run", plan_path.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("0 step 적용"),
        "건너뛴 step 을 적용한 것처럼 세면 '다 됐다'는 보고가 거짓이 된다: {text}"
    );
    assert!(
        text.contains("건너뜀"),
        "사람 모드에도 근거를 남긴다: {text}"
    );
}

#[test]
fn condition_syntax_error_is_usage_error_with_silent_stdout_in_human_mode() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = TempFileGuard::new(temp_path("syntax", "hwp"));
    let plan_path = TempFileGuard::new(temp_path("syntax", "json"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        // 두 조건을 나열하면 and 인지 or 인지가 계획서에 적혀 있지 않다 — 추측 금지.
        "steps": [ { "action": "set_checkbox", "occurrence": 0,
                     "if": { "fieldExists": "a", "textFound": "b" } } ],
    });
    std::fs::write(plan_path.path(), serde_json::to_vec_pretty(&plan).unwrap()).unwrap();
    let output = run(&["run", plan_path.path().to_str().unwrap()]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "조건 문법 오류 = 사용법 오류"
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로의 stdout 은 0바이트여야 한다: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(!out.path().exists(), "실행 0 — 산출물 부재");
}

// ── 5) 자기서술 등재 ────────────────────────────────────────────────────

#[test]
fn capabilities_and_mcp_declare_the_command() {
    let cap: serde_json::Value =
        serde_json::from_slice(&run(&["capabilities"]).stdout).expect("capabilities");
    let entry = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "export-plan-schema")
        .unwrap_or_else(|| panic!("capabilities 에 export-plan-schema 누락: {cap}"));
    assert_eq!(entry["json"], true, "{entry}");
    for flag in ["--json", "--bare", "-o"] {
        assert!(
            entry["flags"].as_array().unwrap().iter().any(|f| f == flag),
            "{flag} 선언 누락: {entry}"
        );
    }

    let mcp: serde_json::Value = serde_json::from_slice(&run(&["capabilities", "--mcp"]).stdout)
        .expect("capabilities --mcp");
    let tool = mcp["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_export_plan_schema")
        .unwrap_or_else(|| panic!("MCP 도구 누락: {mcp}"));
    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    assert!(tool["inputSchema"]["properties"].is_object(), "{tool}");
    // 필수가 없어도 빈 배열을 선언한다 — "필수 없음"과 "선언 누락"은 다른 상태다.
    assert!(
        tool["inputSchema"]["required"].is_array(),
        "required 배열 누락: {tool}"
    );
    assert_eq!(tool["cli"]["command"], "export-plan-schema", "{tool}");
    // 선언한 입력 속성은 전부 CLI 에 닿아야 한다 (선언만 하고 버리면 계약이 거짓말한다).
    let wired: Vec<&str> = tool["cli"]["optionalArgs"]
        .as_array()
        .map(|a| a.iter().filter_map(|o| o["when"].as_str()).collect())
        .unwrap_or_default();
    for key in tool["inputSchema"]["properties"]
        .as_object()
        .expect("properties")
        .keys()
    {
        assert!(
            wired.contains(&key.as_str()),
            "{key} 가 cli.optionalArgs 에 배선되지 않았다: {tool}"
        );
    }

    // hwp_run_plan 은 건너뛴 step 을 출력 계약으로 광고해야 한다 — 소비자가 저널에서
    // skipped 를 기대할 근거가 자기서술에 있어야 한다.
    let run_tool = mcp["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "hwp_run_plan")
        .expect("hwp_run_plan");
    assert!(
        run_tool["outputFields"]
            .as_array()
            .expect("outputFields")
            .iter()
            .any(|f| f == "steps[].skipped"),
        "{run_tool}"
    );
}

// ── 6) 조건 분기별 사례 ─────────────────────────────────────────────────

#[test]
fn field_exists_honours_the_occurrence_suffix() {
    // `이름[N]` 은 fill_fields 의 키 문법과 같은 어휘다. 조건이 이 표기를 모르면
    // "14칸 중 13번째가 있으면" 같은 판정을 계획서로 쓸 수 없다.
    let fields = sample_fields();
    let Some((name, total)) = duplicated_field(&fields) else {
        eprintln!("동명 누름틀이 있는 샘플이 없음 — 건너뜀");
        return;
    };
    let last = format!("{name}[{}]", total - 1);
    let past_end = format!("{name}[{total}]");

    let steps = journal_steps(
        "occ-suffix",
        serde_json::json!([
            // 마지막 순번은 존재한다 → 실행.
            { "action": "set_checkbox", "occurrence": 999999, "if": { "fieldExists": past_end } },
            { "action": "fill_fields", "data": { last.as_str(): "마지막칸" },
              "if": { "fieldExists": last } },
        ]),
    );
    assert_eq!(
        steps[0]["skipped"], true,
        "순번 {total} 은 범위 밖이므로 조건이 거짓이어야 한다: {steps}"
    );
    assert!(
        steps[1].get("skipped").is_none(),
        "마지막 순번은 존재하므로 조건이 참이어야 한다: {steps}"
    );
    assert_eq!(steps[1]["filledCount"], 1, "{steps}");
}

#[test]
fn field_equals_compares_the_current_value_of_the_right_occurrence() {
    // 동명 필드가 여럿일 때 순번을 무시하고 첫 칸만 보면, "13번째 칸이 이미 채워져
    // 있으면 건너뛰라"는 의도가 조용히 첫 칸 판정으로 바뀐다.
    let fields = sample_fields();
    let Some((idx_in_group, name, value)) = a_field_occurrence_with_value(&fields) else {
        eprintln!("값이 든 누름틀이 없음 — 건너뜀");
        return;
    };
    let spec = format!("{name}[{idx_in_group}]");
    let steps = journal_steps(
        "eq-occurrence",
        serde_json::json!([
            { "action": "set_checkbox", "occurrence": 999999,
              "if": { "fieldEquals": { "name": spec, "value": format!("{value}-불일치") } } },
        ]),
    );
    assert_eq!(steps[0]["skipped"], true, "값이 다르면 거짓: {steps}");
    assert!(
        steps[0]["reason"]
            .as_str()
            .is_some_and(|r| r.contains("현재값")),
        "무엇과 비교했는지 사유에 남아야 한다: {steps}"
    );
    // 조건이 참이면 step 이 실행되고, 이 step 은 선검증에서 걸린다(occurrence 범위 밖).
    // 즉 "조건 참 → 선검증 대상"이 증명된다. 그래서 여기서는 exit 2 를 기대한다.
    let out = TempFileGuard::new(temp_path("eq-true-validates", "hwp"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": sample().to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [ { "action": "set_checkbox", "occurrence": 999999,
                     "if": { "fieldEquals": { "name": spec, "value": value } } } ],
    });
    let (code, journal) = run_plan("eq-true-validates", &plan);
    assert_eq!(
        code, 2,
        "조건이 참이면 그 step 은 선검증을 받아야 한다 (면제는 거짓일 때만): {journal}"
    );
    assert!(!out.path().exists(), "선검증 실패 = 실행 0");
}

#[test]
fn text_found_is_case_sensitive() {
    // 대소문자를 뭉개면 "Total" 을 찾는 조건이 "total" 에도 걸린다 — 서식 판별에서
    // 그 차이는 다른 문서를 같은 문서로 보게 만든다.
    let text = sample_text();
    let (Some(token), Some(field)) = (an_ascii_word(&text), a_real_field_name()) else {
        eprintln!("본문 ASCII 낱말 또는 누름틀이 없음 — 건너뜀");
        return;
    };
    let flipped = flip_case(&token);
    if text.contains(&flipped) {
        eprintln!("대소문자를 뒤집은 낱말도 본문에 있음 — 건너뜀");
        return;
    }
    let steps = journal_steps(
        "case",
        serde_json::json!([
            // 뒤집은 낱말은 없다 → 거짓 → 건너뜀.
            { "action": "set_checkbox", "occurrence": 999999, "if": { "textFound": flipped } },
            // 원래 낱말은 있다 → 참 → 실행.
            { "action": "fill_fields", "data": { field.as_str(): "대소문자" },
              "if": { "textFound": token } },
        ]),
    );
    assert_eq!(
        steps[0]["skipped"], true,
        "대소문자가 다르면 찾지 못해야 한다 ('{flipped}'): {steps}"
    );
    assert!(
        steps[1].get("skipped").is_none(),
        "본문에 있는 낱말 '{token}' 을 찾지 못했다: {steps}"
    );
    assert_eq!(steps[1]["filledCount"], 1, "{steps}");
}

#[test]
fn every_action_can_carry_a_condition() {
    // 조건절은 step 4종 **전부**의 계약이다. 한 종류라도 빠지면 스키마가 거짓말한다.
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let schema = schema_body();
    let variants = schema["$defs"]["Step"]["oneOf"]
        .as_array()
        .expect("Step.oneOf");
    for variant in variants {
        let def_name = variant["$ref"]
            .as_str()
            .and_then(|r| r.strip_prefix("#/$defs/"))
            .expect("$defs 참조");
        let action = schema["$defs"][def_name]["properties"]["action"]["const"]
            .as_str()
            .expect("action const");
        // 필수 필드를 일부러 뺀 껍데기 step. 조건이 거짓이면 선검증 면제로 통과해야 한다.
        let steps = journal_steps(
            &format!("cond-action-{action}"),
            serde_json::json!([
                { "action": action, "if": { "fieldExists": "이런누름틀은없다9999" } }
            ]),
        );
        assert_eq!(
            steps[0]["skipped"], true,
            "action '{action}' 이 조건절을 받지 못한다: {steps}"
        );
        assert_eq!(steps[0]["action"], action, "{steps}");
    }
}

#[test]
fn skipped_steps_keep_their_index_and_order() {
    // 저널의 step 번호는 계획서의 인덱스다. 건너뛴 step 때문에 번호가 밀리면 소비자가
    // 저널 항목과 계획서 항목을 짝지을 수 없다.
    let Some(field) = a_real_field_name() else {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    };
    let steps = journal_steps(
        "order",
        serde_json::json!([
            { "action": "fill_fields", "data": { field.as_str(): "첫째" } },
            { "action": "set_checkbox", "occurrence": 999999,
              "if": { "textFound": "이런문자열은문서에없다9999" } },
            { "action": "replace_text", "find": "이런문자열도없다8888", "replace": "X",
              "if": { "fieldExists": "이런누름틀은없다9999" } },
            { "action": "fill_fields", "data": { field.as_str(): "넷째" } },
        ]),
    );
    let steps = steps.as_array().expect("steps[]");
    assert_eq!(steps.len(), 4, "건너뛴 step 도 자리를 지킨다: {steps:?}");
    for (i, step) in steps.iter().enumerate() {
        assert_eq!(step["step"], i, "step 번호가 계획서 인덱스와 어긋났다");
    }
    assert!(steps[0].get("skipped").is_none());
    assert_eq!(steps[1]["skipped"], true);
    assert_eq!(steps[2]["skipped"], true);
    assert!(steps[3].get("skipped").is_none());
}

#[test]
fn skipped_steps_do_not_change_the_visual_verification_target() {
    // changedPages 는 "눈으로 확인할 쪽"이다. 실행되지 않은 step 이 이 목록을 늘리면
    // 사람이 멀쩡한 쪽을 들여다보게 된다.
    let Some(field) = a_real_field_name() else {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    };
    let real_step = serde_json::json!(
        { "action": "fill_fields", "data": { field.as_str(): "변경" } }
    );
    let out_a = TempFileGuard::new(temp_path("pages-a", "hwp"));
    let plan_a = serde_json::json!({
        "planVersion": "1.0", "input": sample().to_str().unwrap(),
        "output": out_a.path().to_str().unwrap(), "steps": [real_step.clone()],
    });
    let (code_a, journal_a) = run_plan("pages-a", &plan_a);
    assert_eq!(code_a, 0, "{journal_a}");

    let out_b = TempFileGuard::new(temp_path("pages-b", "hwp"));
    let plan_b = serde_json::json!({
        "planVersion": "1.0", "input": sample().to_str().unwrap(),
        "output": out_b.path().to_str().unwrap(),
        "steps": [
            real_step,
            { "action": "set_cell", "table": 0, "row": 0, "col": 0, "text": "안바뀜",
              "if": { "textFound": "이런문자열은문서에없다9999" } },
        ],
    });
    let (code_b, journal_b) = run_plan("pages-b", &plan_b);
    assert_eq!(code_b, 0, "{journal_b}");
    assert_eq!(
        journal_a["changedPages"], journal_b["changedPages"],
        "건너뛴 step 이 눈검증 대상 쪽을 늘렸다: {} vs {}",
        journal_a["changedPages"], journal_b["changedPages"]
    );
}

#[test]
fn conditions_do_not_see_earlier_steps_edits() {
    // 조건은 **입력 문서** 기준이다. 앞 step 의 편집을 조건이 보게 되면 선검증(편집 전)과
    // 실행(편집 후)이 서로 다른 답을 내고, 검사를 통과한 계획이 실행에서 다르게 동작한다.
    let fields = sample_fields();
    let Some((name, _)) = fields.iter().find(|(_, v)| v.is_empty()).cloned() else {
        eprintln!("빈 누름틀이 없음 — 건너뜀");
        return;
    };
    let steps = journal_steps(
        "isolation",
        serde_json::json!([
            { "action": "fill_fields", "data": { name.as_str(): "새값" } },
            // 앞 step 이 방금 넣은 값을 조건이 보면 참이 된다 — 보면 안 된다.
            { "action": "set_checkbox", "occurrence": 999999,
              "if": { "fieldEquals": { "name": name.as_str(), "value": "새값" } } },
        ]),
    );
    assert_eq!(
        steps[1]["skipped"], true,
        "조건이 앞 step 의 편집 결과를 보고 있다 — 입력 문서 기준이어야 한다: {steps}"
    );
}

#[test]
fn all_steps_skipped_still_produces_an_honest_journal() {
    // 전부 건너뛴 계획도 실패가 아니다(조건이 다 거짓이었을 뿐). 다만 저널이 그 사실을
    // 말하지 않으면 "성공했는데 아무것도 안 바뀐" 상태가 설명 불가가 된다.
    let out = TempFileGuard::new(temp_path("all-skipped", "hwp"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": sample().to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [
            { "action": "set_checkbox", "occurrence": 999999,
              "if": { "textFound": "이런문자열은문서에없다9999" } },
            { "action": "fill_fields", "data": { "이런누름틀은없다9999": "값" },
              "if": { "fieldExists": "이런누름틀은없다9999" } },
        ],
        "assertions": { "verify": true },
    });
    let (code, journal) = run_plan("all-skipped", &plan);
    assert_eq!(code, 0, "{journal}");
    let steps = journal["steps"].as_array().expect("steps[]");
    assert_eq!(steps.len(), 2, "{journal}");
    assert!(steps.iter().all(|s| s["skipped"] == true), "{journal}");
    assert_eq!(
        journal["verify"]["identical"], true,
        "무편집 산출물도 자기검증을 통과해야 한다: {journal}"
    );
    assert!(out.path().exists(), "저장 자체는 정상 수행된다");
}

// ── 7) 스키마 상수 ↔ 실행기 드리프트 ────────────────────────────────────

#[test]
fn schema_pins_the_plan_version_the_engine_requires() {
    // 스키마가 const 로 박은 planVersion 과 실행기가 받는 값이 다르면, 스키마를 따라
    // 만든 계획이 전부 exit 2 가 된다.
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let schema = schema_body();
    let pinned = schema["$defs"]["Plan"]["properties"]["planVersion"]["const"]
        .as_str()
        .expect("planVersion const");
    let Some(field) = a_real_field_name() else {
        return;
    };
    for (version, want_ok) in [(pinned.to_string(), true), ("2.0".to_string(), false)] {
        let out = TempFileGuard::new(temp_path(&format!("ver-{version}"), "hwp"));
        let plan = serde_json::json!({
            "planVersion": version,
            "input": p.to_str().unwrap(),
            "output": out.path().to_str().unwrap(),
            "steps": [ { "action": "fill_fields", "data": { field.as_str(): "판번호" } } ],
        });
        let (code, journal) = run_plan(&format!("ver-{version}"), &plan);
        if want_ok {
            assert_eq!(
                code, 0,
                "스키마가 박은 planVersion '{version}' 을 실행기가 거절했다: {journal}"
            );
        } else {
            assert_eq!(code, 2, "실행기가 '{version}' 을 받아들였다: {journal}");
        }
    }
}

#[test]
fn schema_assertions_match_the_journal_echo() {
    // 저널은 판정에 쓴 단언을 그대로 되돌려준다. 스키마가 선언한 단언 이름과 그 에코가
    // 어긋나면, 계획서에 쓴 단언이 실제로 켜졌는지 확인할 방법이 없어진다.
    let Some(field) = a_real_field_name() else {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    };
    let schema = schema_body();
    let declared: std::collections::BTreeSet<String> = schema["$defs"]["Assertions"]["properties"]
        .as_object()
        .expect("Assertions.properties")
        .keys()
        .cloned()
        .collect();

    let out = TempFileGuard::new(temp_path("assert-echo", "hwp"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": sample().to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [ { "action": "fill_fields", "data": { field.as_str(): "단언" } } ],
        "assertions": { "notFoundEmpty": true, "verify": true },
    });
    let (code, journal) = run_plan("assert-echo", &plan);
    assert_eq!(code, 0, "{journal}");
    let echoed: std::collections::BTreeSet<String> = journal["assertions"]
        .as_object()
        .expect("저널 assertions 에코")
        .keys()
        .cloned()
        .collect();
    assert_eq!(
        declared, echoed,
        "스키마가 선언한 단언과 저널 에코가 다르다 — 스키마 {declared:?} / 저널 {echoed:?}"
    );
}

// ── 8) MCP 경로 ─────────────────────────────────────────────────────────

#[test]
fn mcp_serves_the_plan_schema_and_reports_skipped_steps() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    // (1) 스키마 도구 — MCP 호스트가 계획을 쓰기 전에 받아 가는 경로.
    let schema_frame = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "hwp_export_plan_schema", "arguments": { "bare": true } }
    });
    // (2) 조건부 계획 — 같은 서버에서 저널에 skipped 가 실려 돌아오는지.
    let out = TempFileGuard::new(temp_path("mcp-cond", "hwp"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [ { "action": "set_checkbox", "occurrence": 999999,
                     "if": { "textFound": "이런문자열은문서에없다9999" } } ],
    });
    let plan_frame = serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": { "name": "hwp_run_plan", "arguments": { "plan": plan } }
    });
    let responses = mcp_call(&[schema_frame, plan_frame]);

    let body: serde_json::Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .expect("스키마 본문");
    assert_eq!(responses[0]["result"]["isError"], false, "{}", responses[0]);
    assert_eq!(body["$ref"], "#/$defs/Plan", "bare 본문이 아니다: {body}");
    assert!(
        body["$defs"].as_object().is_some_and(|d| d.len() >= 9),
        "정의가 비었다: {body}"
    );

    let journal: serde_json::Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .expect("저널");
    assert_eq!(
        journal["steps"][0]["skipped"], true,
        "MCP 경로에도 skipped 가 실려야 한다: {journal}"
    );
    assert!(
        journal["steps"][0]["reason"]
            .as_str()
            .is_some_and(|r| !r.is_empty()),
        "{journal}"
    );
}

// ── 보조 ────────────────────────────────────────────────────────────────

/// 동명이 둘 이상인 누름틀 (이름, 개수).
fn duplicated_field(fields: &[(String, String)]) -> Option<(String, usize)> {
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for (name, _) in fields {
        *counts.entry(name.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .find(|(_, n)| *n > 1)
        .map(|(name, n)| (name.to_string(), n))
}

/// 값이 들어 있는 누름틀 하나 — (동명 그룹 내 순번, 이름, 값).
fn a_field_occurrence_with_value(fields: &[(String, String)]) -> Option<(usize, String, String)> {
    let mut seen: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for (name, value) in fields {
        let occurrence = *seen.entry(name.as_str()).or_insert(0);
        seen.insert(name.as_str(), occurrence + 1);
        if !value.is_empty() {
            return Some((occurrence, name.clone(), value.clone()));
        }
    }
    None
}

/// 본문에서 ASCII 낱말(길이 3 이상, 대소문자가 섞일 수 있는 것) 하나.
fn an_ascii_word(text: &str) -> Option<String> {
    text.split(|c: char| !c.is_ascii_alphabetic())
        .find(|w| w.len() >= 3 && w.chars().any(|c| c.is_ascii_uppercase()))
        .map(str::to_string)
}

/// 대소문자를 뒤집는다.
fn flip_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_uppercase() {
                c.to_ascii_lowercase()
            } else {
                c.to_ascii_uppercase()
            }
        })
        .collect()
}

/// `mcp-serve` 에 프레임을 순서대로 보내고 같은 수의 응답을 읽는다.
fn mcp_call(frames: &[serde_json::Value]) -> Vec<serde_json::Value> {
    use std::io::{BufRead, BufReader, Write};
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("mcp-serve 기동");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        for frame in frames {
            writeln!(stdin, "{frame}").expect("프레임 쓰기");
        }
        stdin.flush().expect("flush");
    }
    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let mut responses = Vec::new();
    for _ in frames {
        let mut line = String::new();
        let read = reader.read_line(&mut line).expect("응답 읽기");
        assert!(read > 0, "서버가 응답 없이 끊겼다");
        responses.push(serde_json::from_str(line.trim()).expect("JSON-RPC 응답"));
    }
    let _ = child.kill();
    let _ = child.wait();
    responses
}

/// 스키마 노드와 그 하위 `properties` 가 전부 설명을 갖는지 본다.
/// 순수 `$ref` 노드는 참조된 정의가 설명을 갖고 있으므로 면제한다.
fn check_described(node: &serde_json::Value, path: &str, missing: &mut Vec<String>) {
    let Some(map) = node.as_object() else {
        return;
    };
    let pure_ref = map.len() == 1 && map.contains_key("$ref");
    if !pure_ref
        && !map
            .get("description")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|d| !d.trim().is_empty())
    {
        missing.push(path.to_string());
    }
    if let Some(props) = map.get("properties").and_then(serde_json::Value::as_object) {
        for (key, value) in props {
            check_described(value, &format!("{path}.{key}"), missing);
        }
    }
    if let Some(extra) = map.get("additionalProperties") {
        if extra.is_object() {
            check_described(extra, &format!("{path}.additionalProperties"), missing);
        }
    }
}

/// `$ref` 를 재귀 수집한다.
fn collect_refs(value: &serde_json::Value, sink: &mut impl FnMut(&str)) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, item) in map {
                if key == "$ref" {
                    if let Some(name) = item.as_str().and_then(|p| p.strip_prefix("#/$defs/")) {
                        sink(name);
                    }
                } else {
                    collect_refs(item, sink);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_refs(item, sink);
            }
        }
        _ => {}
    }
}
