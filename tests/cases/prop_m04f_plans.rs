//! [#5465] M04-f: 유효·무효 계획 표.
//!
//! 생성기 카탈로그를 `export-plan-schema` 키워드만으로 검사한다.
//! 스키마 4종 밖 action 은 거부한다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/proptest_m04f");

fn read_jsonl(rel: &str) -> Vec<Value> {
    let path = Path::new(ROOT).join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{rel}: {e}")))
        .collect()
}

fn resolve_ref<'a>(root: &'a Value, pointer: &str, path: &str) -> Result<&'a Value, String> {
    let trimmed = pointer
        .strip_prefix("#/")
        .ok_or_else(|| format!("{path}: $ref {pointer}"))?;
    let mut cur = root;
    for part in trimmed.split('/') {
        cur = cur
            .get(part)
            .ok_or_else(|| format!("{path}: $ref {pointer}"))?;
    }
    Ok(cur)
}

fn type_matches(ty: &str, instance: &Value) -> bool {
    match ty {
        "object" => instance.is_object(),
        "array" => instance.is_array(),
        "string" => instance.is_string(),
        "integer" => instance
            .as_i64()
            .map(|_| true)
            .or_else(|| instance.as_u64().map(|_| true))
            .unwrap_or(false),
        "number" => instance.is_number(),
        "boolean" => instance.is_boolean(),
        "null" => instance.is_null(),
        _ => false,
    }
}

/// `export-plan-schema` 가 실제로 쓰는 키워드만. prop_edit_plan 과 같다.
fn validate_against(
    root: &Value,
    schema: &Value,
    instance: &Value,
    path: &str,
) -> Result<(), String> {
    if let Some(pointer) = schema.get("$ref").and_then(Value::as_str) {
        let resolved = resolve_ref(root, pointer, path)?;
        validate_against(root, resolved, instance, path)?;
    }
    if let Some(ty) = schema.get("type") {
        let ok = match ty {
            Value::String(name) => type_matches(name, instance),
            Value::Array(names) => names.iter().any(|name| {
                name.as_str()
                    .is_some_and(|name| type_matches(name, instance))
            }),
            _ => true,
        };
        if !ok {
            return Err(format!("{path}: type {ty} 불일치"));
        }
    }
    if let Some(expected) = schema.get("const") {
        if instance != expected {
            return Err(format!("{path}: const {expected} 불일치"));
        }
    }
    if let Some(opts) = schema.get("oneOf").and_then(Value::as_array) {
        let mut matched = 0usize;
        let mut last_err = None;
        for opt in opts {
            match validate_against(root, opt, instance, path) {
                Ok(()) => matched += 1,
                Err(err) => last_err = Some(err),
            }
        }
        if matched != 1 {
            return Err(format!(
                "{path}: oneOf 일치 {matched} ({})",
                last_err.unwrap_or_else(|| "없음".into())
            ));
        }
    }
    if let Some(obj) = instance.as_object() {
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required {
                let key = key.as_str().expect("required");
                if !obj.contains_key(key) {
                    return Err(format!("{path}: 필수 {key} 없음"));
                }
            }
        }
        if let Some(props) = schema.get("properties").and_then(Value::as_object) {
            for (key, sub) in props {
                if let Some(value) = obj.get(key) {
                    validate_against(root, sub, value, &format!("{path}.{key}"))?;
                }
            }
        }
        let declared = schema.get("properties").and_then(Value::as_object);
        match schema.get("additionalProperties") {
            Some(Value::Bool(false)) => {
                if let Some(declared) = declared {
                    for key in obj.keys() {
                        if !declared.contains_key(key) {
                            return Err(format!("{path}: 추가 필드 {key}"));
                        }
                    }
                }
            }
            Some(sub) if sub.is_object() => {
                for (key, value) in obj {
                    if declared.is_none_or(|declared| !declared.contains_key(key)) {
                        validate_against(root, sub, value, &format!("{path}.{key}"))?;
                    }
                }
            }
            _ => {}
        }
        if let Some(min) = schema.get("minProperties").and_then(Value::as_u64) {
            if (obj.len() as u64) < min {
                return Err(format!("{path}: minProperties {min}"));
            }
        }
        if let Some(max) = schema.get("maxProperties").and_then(Value::as_u64) {
            if (obj.len() as u64) > max {
                return Err(format!("{path}: maxProperties {max}"));
            }
        }
    }
    if let Some(items) = schema.get("items") {
        let arr = instance.as_array().ok_or_else(|| format!("{path}: 배열"))?;
        if let Some(min) = schema.get("minItems").and_then(Value::as_u64) {
            if (arr.len() as u64) < min {
                return Err(format!("{path}: minItems {min}"));
            }
        }
        for (idx, item) in arr.iter().enumerate() {
            validate_against(root, items, item, &format!("{path}[{idx}]"))?;
        }
    }
    if let Some(s) = instance.as_str() {
        if let Some(min) = schema.get("minLength").and_then(Value::as_u64) {
            if (s.chars().count() as u64) < min {
                return Err(format!("{path}: minLength {min}"));
            }
        }
        if let Some(pat) = schema.get("pattern").and_then(Value::as_str) {
            let ok = match pat {
                "^[^\r\n\t]*$" => !s.chars().any(|ch| matches!(ch, '\r' | '\n' | '\t')),
                other => return Err(format!("{path}: pattern {other}")),
            };
            if !ok {
                return Err(format!("{path}: pattern {pat}"));
            }
        }
    }
    if instance.is_number() {
        let n = instance.as_f64().expect("number");
        if let Some(min) = schema.get("minimum").and_then(Value::as_f64) {
            if n < min {
                return Err(format!("{path}: minimum {min}"));
            }
        }
        if let Some(max) = schema.get("maximum").and_then(Value::as_f64) {
            if n > max {
                return Err(format!("{path}: maximum {max}"));
            }
        }
    }
    Ok(())
}

fn validate_plan(plan: &Value) -> Result<(), String> {
    let schema = rhwp::plan_schema::plan_schema();
    validate_against(&schema, &schema, plan, "$")
}

#[test]
fn valid_plan_catalog_matches_export_plan_schema() {
    let rows = read_jsonl("catalogs/valid_plans.jsonl");
    assert!(rows.len() >= 800);
    let mut failed = Vec::new();
    for row in &rows {
        if let Err(err) = validate_plan(&row["plan"]) {
            failed.push(format!("{}: {err}", row["id"]));
            if failed.len() >= 8 {
                break;
            }
        }
    }
    assert!(
        failed.is_empty(),
        "유효 계획이 스키마 거부:\n{}",
        failed.join("\n")
    );
}

#[test]
fn invalid_plan_catalog_is_rejected() {
    let rows = read_jsonl("catalogs/invalid_plans.jsonl");
    assert!(rows.len() >= 80);
    let mut passed = Vec::new();
    for row in &rows {
        assert_eq!(row["expected"], "schema_reject");
        if validate_plan(&row["plan"]).is_ok() {
            passed.push(row["id"].as_str().unwrap().to_string());
            if passed.len() >= 8 {
                break;
            }
        }
    }
    assert!(
        passed.is_empty(),
        "무효 계획이 스키마를 통과함: {}",
        passed.join(", ")
    );
}

#[test]
fn invented_actions_are_in_the_invalid_catalog() {
    let rows = read_jsonl("catalogs/invalid_plans.jsonl");
    let mut invented = 0usize;
    for row in &rows {
        if row["family"] != "unknown_action" {
            continue;
        }
        invented += 1;
        assert!(validate_plan(&row["plan"]).is_err());
        let action = row["plan"]["steps"][0]["action"].as_str().unwrap_or("");
        assert!(
            !matches!(
                action,
                "fill_fields" | "replace_text" | "set_cell" | "set_checkbox"
            ),
            "unknown_action 행이 기존 action 을 담음: {action}"
        );
    }
    assert!(invented >= 10, "발명 action 거부 행 {invented}");
}

#[test]
fn empty_find_and_cell_control_chars_are_rejected() {
    let rows = read_jsonl("catalogs/invalid_plans.jsonl");
    let mut empty_find = 0;
    let mut control = 0;
    for row in &rows {
        match row["family"].as_str().unwrap() {
            "replace_empty_find" => {
                empty_find += 1;
                assert!(validate_plan(&row["plan"]).is_err());
            }
            "set_cell_newline" | "set_cell_tab" | "set_cell_cr" => {
                control += 1;
                assert!(validate_plan(&row["plan"]).is_err());
            }
            _ => {}
        }
    }
    assert!(empty_find >= 4, "empty find {empty_find}");
    assert!(control >= 6, "control char {control}");
}

#[test]
fn valid_families_cover_four_actions_and_envelopes() {
    let rows = read_jsonl("catalogs/valid_plans.jsonl");
    let families: BTreeSet<_> = rows
        .iter()
        .map(|row| row["family"].as_str().unwrap())
        .collect();
    for family in [
        "fill_fields_axis",
        "replace_text_axis",
        "set_cell_axis",
        "set_checkbox_axis",
        "multi_step_envelope",
    ] {
        assert!(families.contains(family), "{family}");
    }
}
