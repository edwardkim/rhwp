//! [#3762] `export-ir-schema` — 외부 바인딩 코드 생성의 단일 출처 계약 (M18 착수 조건).
//!
//! 이 스키마가 깨지면 바인딩 세대가 통째로 잘못된 코드를 만든다. 그래서 여기서
//! 검증하는 것은 "JSON 이 나오나"가 아니라 **스키마로서 성립하나**다 — 끊어진
//! 참조·고아 정의·닫힌 객체는 전부 실패다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp")
}

fn describe(args: &[&str], o: &Output) -> String {
    format!(
        "명령: rhwp {}\nexit: {:?}\nstderr:\n{}",
        args.join(" "),
        o.status.code(),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn schema_envelope() -> serde_json::Value {
    let args = ["export-ir-schema"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    serde_json::from_slice(&output.stdout).expect("봉투 JSON")
}

/// 봉투 스키마 — 소비자가 버전과 방언을 먼저 확인할 수 있어야 한다.
#[test]
fn envelope_declares_version_and_dialect() {
    let v = schema_envelope();
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["irSchemaVersion"].is_string(), "{v}");
    // 부분 문자열 검사(`.contains("json-schema.org")`)는 호스트를 판정하지 못한다 —
    // https://json-schema.org.evil.com/... 같은 값도 통과하므로 계약 검사로 쓸 수 없다.
    // 방언은 src/ir_schema.rs 의 SCHEMA_DIALECT 로 고정된 값이니 정확히 대조한다.
    assert_eq!(
        v["dialect"], "https://json-schema.org/draft/2020-12/schema",
        "방언을 명시해야 소비자가 파서를 고를 수 있다: {v}"
    );
    assert!(v["definitionCount"].as_u64().unwrap_or(0) >= 25, "{v}");
    assert!(v["schema"].is_object(), "{v}");
}

/// IR 버전은 봉투 버전과 **분리**돼 있다 — 명령별 봉투와 전역 IR 은 따로 진화한다.
#[test]
fn ir_schema_version_is_independent_of_envelope_version() {
    let v = schema_envelope();
    let schema = &v["schema"];
    assert_eq!(
        schema["irSchemaVersion"], v["irSchemaVersion"],
        "스키마 본문과 봉투의 IR 버전이 어긋난다: {v}"
    );
}

/// 루트가 Document 를 가리킨다.
#[test]
fn schema_root_points_at_document() {
    let v = schema_envelope();
    let schema = &v["schema"];
    assert_eq!(schema["$ref"], "#/$defs/Document", "{schema}");
    assert!(schema["$defs"]["Document"].is_object(), "{schema}");
}

/// 끊어진 `$ref` 는 코드 생성기를 즉시 망가뜨린다.
#[test]
fn every_reference_resolves() {
    let v = schema_envelope();
    let schema = &v["schema"];
    let defs = schema["$defs"].as_object().expect("$defs");
    let mut missing = Vec::new();
    collect_refs(schema, &mut |name| {
        if !defs.contains_key(name) {
            missing.push(name.to_string());
        }
    });
    assert!(missing.is_empty(), "정의되지 않은 참조: {missing:?}");
}

/// 아무도 가리키지 않는 정의는 죽은 계약이다.
#[test]
fn no_orphan_definitions() {
    let v = schema_envelope();
    let schema = &v["schema"];
    let defs = schema["$defs"].as_object().expect("$defs");
    let mut referenced: HashSet<String> = HashSet::new();
    referenced.insert("Document".to_string());
    collect_refs(schema, &mut |name| {
        referenced.insert(name.to_string());
    });
    let orphans: Vec<&String> = defs.keys().filter(|k| !referenced.contains(*k)).collect();
    assert!(orphans.is_empty(), "아무도 참조하지 않는 정의: {orphans:?}");
}

/// IR 은 **추가-전용 진화**다 — 객체가 추가 필드를 막으면 필드 하나 늘 때마다
/// 모든 바인딩이 동시에 깨진다.
#[test]
fn objects_permit_additional_properties() {
    let v = schema_envelope();
    let defs = v["schema"]["$defs"].as_object().expect("$defs");
    for (name, def) in defs {
        if def["type"] == "object" {
            assert_eq!(
                def["additionalProperties"], true,
                "{name} 이 추가 필드를 막고 있다"
            );
        }
    }
}

/// 편집 API 가 쓰는 좌표 타입이 스키마에 있어야 한다 — 없으면 바인딩이 표·누름틀을
/// 다룰 수 없다.
#[test]
fn schema_covers_edit_surface_types() {
    let v = schema_envelope();
    let defs = v["schema"]["$defs"].as_object().expect("$defs");
    for required in [
        "Document",
        "Section",
        "Paragraph",
        "TableControl",
        "TableCell",
        "FieldRange",
        "CharShape",
        "ParaShape",
        "Provenance",
    ] {
        assert!(
            defs.contains_key(required),
            "{required} 정의 누락: {defs:?}"
        );
    }
}

/// 모든 정의에 설명이 달려 있어야 한다 — 생성된 바인딩의 docstring 원천이다.
#[test]
fn every_definition_is_documented() {
    let v = schema_envelope();
    let defs = v["schema"]["$defs"].as_object().expect("$defs");
    let undocumented: Vec<&String> = defs
        .iter()
        .filter(|(_, d)| d["description"].as_str().unwrap_or("").trim().is_empty())
        .map(|(k, _)| k)
        .collect();
    assert!(undocumented.is_empty(), "설명 없는 정의: {undocumented:?}");
}

/// `--bare` 는 봉투 없이 스키마 본문만 — JSON Schema 도구에 바로 먹인다.
#[test]
fn bare_flag_emits_schema_body_only() {
    let args = ["export-ir-schema", "--bare"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("스키마");
    assert!(v["$schema"].is_string(), "{v}");
    assert!(v["$defs"].is_object(), "{v}");
    // 봉투 필드가 섞이면 안 된다.
    assert!(v["definitionCount"].is_null(), "봉투가 섞였다: {v}");
}

/// `-o` 로 파일에 쓰고, `--json` 이면 stdout 은 기계 계약을 유지한다.
#[test]
fn output_file_keeps_stdout_machine_readable() {
    let target = std::env::temp_dir().join(format!(
        "rhwp-irschema-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let target_str = target.to_str().unwrap().to_string();
    let args = ["export-ir-schema", "-o", &target_str, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("봉투");
    assert_eq!(v["output"], target_str, "{v}");
    assert!(v["bytes"].as_u64().unwrap_or(0) > 0, "{v}");

    let written = std::fs::read_to_string(&target).expect("파일");
    let parsed: serde_json::Value = serde_json::from_str(&written).expect("파일 JSON");
    assert!(parsed["schema"]["$defs"].is_object(), "{parsed}");

    let _ = std::fs::remove_file(&target);
}

/// 알 수 없는 옵션은 사용법 오류(2), stdout 0바이트.
#[test]
fn unknown_option_is_usage_error() {
    let args = ["export-ir-schema", "--없는옵션"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

/// `-o` 뒤에 경로가 없으면 사용법 오류.
#[test]
fn missing_output_path_is_usage_error() {
    let args = ["export-ir-schema", "-o"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty());
}

/// 쓸 수 없는 경로는 런타임 오류(1) — 사용법 오류와 구분한다.
#[test]
fn unwritable_output_is_runtime_error() {
    let bad: PathBuf = PathBuf::from("없는폴더-irschema")
        .join("깊은")
        .join("경로.json");
    let bad_str = bad.to_str().unwrap().to_string();
    let args = ["export-ir-schema", "-o", &bad_str];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
}

/// capabilities 가 이 명령을 json 명령으로 선언한다 (드리프트 가드와 정합).
#[test]
fn capabilities_declares_export_ir_schema() {
    let output = run(&["capabilities"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("capabilities");
    let cmd = v["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "export-ir-schema")
        .expect("export-ir-schema 수록");
    assert_eq!(cmd["json"], true, "{cmd}");
    let fields = cmd["recordFields"].as_array().expect("recordFields");
    for want in ["irSchemaVersion", "schema", "definitionCount"] {
        assert!(fields.iter().any(|f| f == want), "{want} 누락: {cmd}");
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
