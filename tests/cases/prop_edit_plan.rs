//! [#5363] M04-1: proptest 편집 계획 시퀀스 생성기.
//!
//! `rhwp run` 이 이미 받는 step 4종(`fill_fields` · `replace_text` · `set_cell` ·
//! `set_checkbox`)만 조합한다. DocumentCore 편집 로직을 발명하지 않고, 생성된
//! 계획서는 JSON 왕복과 `export-plan-schema` 정적 검증만 본다.
//! HWPX/HWP5 IrDiff-0 왕복 property 본체는 M04-2/3.
#![cfg(not(target_arch = "wasm32"))]

use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

const PLAN_VERSION: &str = "1.0";
const ACTIONS: [&str; 4] = ["fill_fields", "replace_text", "set_cell", "set_checkbox"];
const PATHS: &[&str] = &[
    "in.hwp",
    "in.hwpx",
    "form.hml",
    "samples/field-01.hwp",
    "out.hwpx",
];
const FIELD_NAMES: &[&str] = &["이름", "주소", "신청인", "피규제집단명", "title", "name"];
const FIND_NEEDLES: &[&str] = &["한글", "기관명", "2024", "TODO", "example"];
const CELL_TEXTS: &[&str] = &["", "서울", "완료", "1,000", "n/a"];
const REPLACE_TEXTS: &[&str] = &["", "한국", "2026", "DONE"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditPlan {
    plan_version: String,
    input: String,
    output: String,
    steps: Vec<EditStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assertions: Option<Assertions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dry_run: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Assertions {
    #[serde(skip_serializing_if = "Option::is_none")]
    not_found_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verify: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum EditStep {
    FillFields {
        data: Map<String, Value>,
        #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
        cond: Option<StepCondition>,
    },
    ReplaceText {
        find: String,
        replace: String,
        #[serde(rename = "caseSensitive", skip_serializing_if = "Option::is_none")]
        case_sensitive: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        occurrence: Option<u32>,
        #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
        cond: Option<StepCondition>,
    },
    SetCell {
        table: u32,
        row: u16,
        col: u16,
        text: String,
        #[serde(rename = "keepStyle", skip_serializing_if = "Option::is_none")]
        keep_style: Option<bool>,
        #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
        cond: Option<StepCondition>,
    },
    SetCheckbox {
        occurrence: u32,
        #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
        cond: Option<StepCondition>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum StepCondition {
    FieldExists(String),
    FieldEquals { name: String, value: String },
    TextFound(String),
}

fn arb_path() -> impl Strategy<Value = String> {
    prop::sample::select(PATHS).prop_map(str::to_string)
}

fn arb_field_key() -> impl Strategy<Value = String> {
    (
        prop::sample::select(FIELD_NAMES),
        proptest::option::of(0u32..4),
    )
        .prop_map(|(name, occ)| match occ {
            Some(n) => format!("{name}[{n}]"),
            None => (*name).to_string(),
        })
}

fn arb_field_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        prop::sample::select(REPLACE_TEXTS).prop_map(|s| Value::String((*s).to_string())),
        (0i64..10_000).prop_map(|n| json!(n)),
        any::<bool>().prop_map(Value::Bool),
    ]
}

fn arb_condition() -> impl Strategy<Value = StepCondition> {
    prop_oneof![
        arb_field_key().prop_map(StepCondition::FieldExists),
        (arb_field_key(), prop::sample::select(REPLACE_TEXTS)).prop_map(|(name, value)| {
            StepCondition::FieldEquals {
                name,
                value: (*value).to_string(),
            }
        }),
        prop::sample::select(FIND_NEEDLES).prop_map(|s| StepCondition::TextFound((*s).to_string())),
    ]
}

fn arb_fill_fields() -> impl Strategy<Value = EditStep> {
    (
        proptest::collection::btree_map(arb_field_key(), arb_field_value(), 1..4),
        proptest::option::of(arb_condition()),
    )
        .prop_map(|(data, cond)| EditStep::FillFields {
            data: data.into_iter().collect(),
            cond,
        })
}

fn arb_replace_text() -> impl Strategy<Value = EditStep> {
    (
        prop::sample::select(FIND_NEEDLES),
        prop::sample::select(REPLACE_TEXTS),
        proptest::option::of(any::<bool>()),
        proptest::option::of(0u32..8),
        proptest::option::of(arb_condition()),
    )
        .prop_map(
            |(find, replace, case_sensitive, occurrence, cond)| EditStep::ReplaceText {
                find: (*find).to_string(),
                replace: (*replace).to_string(),
                case_sensitive,
                occurrence,
                cond,
            },
        )
}

fn arb_set_cell() -> impl Strategy<Value = EditStep> {
    (
        0u32..8,
        0u16..64,
        0u16..32,
        prop::sample::select(CELL_TEXTS),
        proptest::option::of(any::<bool>()),
        proptest::option::of(arb_condition()),
    )
        .prop_map(
            |(table, row, col, text, keep_style, cond)| EditStep::SetCell {
                table,
                row,
                col,
                text: (*text).to_string(),
                keep_style,
                cond,
            },
        )
}

fn arb_set_checkbox() -> impl Strategy<Value = EditStep> {
    (0u32..8, proptest::option::of(arb_condition()))
        .prop_map(|(occurrence, cond)| EditStep::SetCheckbox { occurrence, cond })
}

fn arb_step() -> impl Strategy<Value = EditStep> {
    prop_oneof![
        arb_fill_fields(),
        arb_replace_text(),
        arb_set_cell(),
        arb_set_checkbox(),
    ]
}

fn arb_assertions() -> impl Strategy<Value = Assertions> {
    (
        proptest::option::of(any::<bool>()),
        proptest::option::of(any::<bool>()),
    )
        .prop_map(|(not_found_empty, verify)| Assertions {
            not_found_empty,
            verify,
        })
}

fn arb_valid_plan() -> impl Strategy<Value = EditPlan> {
    (
        arb_path(),
        arb_path(),
        proptest::collection::vec(arb_step(), 1..5),
        proptest::option::of(arb_assertions()),
        proptest::option::of(any::<bool>()),
    )
        .prop_map(|(input, output, steps, assertions, dry_run)| EditPlan {
            plan_version: PLAN_VERSION.to_string(),
            input,
            output,
            steps,
            assertions,
            dry_run,
        })
}

fn valid_seed_plan() -> EditPlan {
    EditPlan {
        plan_version: PLAN_VERSION.to_string(),
        input: "in.hwp".into(),
        output: "out.hwpx".into(),
        steps: vec![
            EditStep::FillFields {
                data: Map::from_iter([("이름".into(), json!("홍길동"))]),
                cond: None,
            },
            EditStep::ReplaceText {
                find: "기관명".into(),
                replace: "한국".into(),
                case_sensitive: Some(true),
                occurrence: None,
                cond: Some(StepCondition::TextFound("기관명".into())),
            },
            EditStep::SetCell {
                table: 0,
                row: 1,
                col: 2,
                text: "서울".into(),
                keep_style: Some(false),
                cond: None,
            },
            EditStep::SetCheckbox {
                occurrence: 0,
                cond: Some(StepCondition::FieldExists("신청인".into())),
            },
        ],
        assertions: Some(Assertions {
            not_found_empty: Some(true),
            verify: Some(false),
        }),
        dry_run: Some(true),
    }
}

fn arb_invalid_plan() -> impl Strategy<Value = Value> {
    let seed = serde_json::to_value(valid_seed_plan()).expect("seed");
    prop_oneof![
        Just(json!({
            "planVersion": "0.9",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "fill_fields", "data": {"이름": "x"}}],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [],
        })),
        Just(json!({
            "planVersion": "1.0",
            "output": "out.hwp",
            "steps": [{"action": "fill_fields", "data": {"이름": "x"}}],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "explode", "data": {}}],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "replace_text", "find": "", "replace": "x"}],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "set_cell", "table": 0, "row": 0, "col": 0, "text": "a\nb"}],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "set_cell", "table": 0, "row": -1, "col": 0, "text": "x"}],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "set_checkbox"}],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "fill_fields"}],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{
                "action": "fill_fields",
                "data": {"이름": "x"},
                "if": {"fieldExists": "이름", "textFound": "한글"}
            }],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{
                "action": "replace_text",
                "find": "한글",
                "replace": "x",
                "if": {}
            }],
        })),
        Just(json!({
            "planVersion": "1.0",
            "input": "in.hwp",
            "steps": "not-an-array",
        })),
        Just(seed.clone()).prop_map(|mut plan| {
            plan["planVersion"] = json!("2.0");
            plan
        }),
        Just(seed).prop_map(|mut plan| {
            plan["steps"] = json!([]);
            plan
        }),
    ]
}

fn schema_actions(schema: &Value) -> Vec<String> {
    let variants = schema["$defs"]["Step"]["oneOf"]
        .as_array()
        .expect("Step.oneOf");
    let mut actions = Vec::new();
    for variant in variants {
        let name = variant["$ref"]
            .as_str()
            .and_then(|p| p.strip_prefix("#/$defs/"))
            .expect("oneOf $ref");
        let action = schema["$defs"][name]["properties"]["action"]["const"]
            .as_str()
            .expect("action const")
            .to_string();
        actions.push(action);
    }
    actions.sort();
    actions
}

fn resolve_ref<'a>(root: &'a Value, pointer: &str, path: &str) -> Result<&'a Value, String> {
    let trimmed = pointer
        .strip_prefix("#/")
        .ok_or_else(|| format!("{path}: 지원하지 않는 $ref {pointer}"))?;
    let mut cur = root;
    for part in trimmed.split('/') {
        cur = cur
            .get(part)
            .ok_or_else(|| format!("{path}: $ref {pointer} 를 풀 수 없습니다"))?;
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

/// `export-plan-schema` 가 실제로 쓰는 키워드만 검사한다.
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
            return Err(format!("{path}: type {ty} 불일치 ({instance})"));
        }
    }

    if let Some(expected) = schema.get("const") {
        if instance != expected {
            return Err(format!("{path}: const {expected} 불일치 ({instance})"));
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
                "{path}: oneOf 일치 {matched}개 (마지막 거부: {})",
                last_err.unwrap_or_else(|| "없음".into())
            ));
        }
    }

    if let Some(obj) = instance.as_object() {
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required {
                let key = key.as_str().expect("required 키");
                if !obj.contains_key(key) {
                    return Err(format!("{path}: 필수 필드 {key} 없음"));
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
                            return Err(format!("{path}: 추가 필드 {key} 거부"));
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
                return Err(format!("{path}: minProperties {min} (실제 {})", obj.len()));
            }
        }
        if let Some(max) = schema.get("maxProperties").and_then(Value::as_u64) {
            if (obj.len() as u64) > max {
                return Err(format!("{path}: maxProperties {max} (실제 {})", obj.len()));
            }
        }
    }

    if let Some(items) = schema.get("items") {
        let arr = instance
            .as_array()
            .ok_or_else(|| format!("{path}: 배열이어야 합니다"))?;
        if let Some(min) = schema.get("minItems").and_then(Value::as_u64) {
            if (arr.len() as u64) < min {
                return Err(format!("{path}: minItems {min} (실제 {})", arr.len()));
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
                other => {
                    return Err(format!("{path}: 지원하지 않는 pattern {other}"));
                }
            };
            if !ok {
                return Err(format!("{path}: pattern {pat} 불일치"));
            }
        }
    }

    if instance.is_number() {
        let n = instance.as_f64().expect("number");
        if let Some(min) = schema.get("minimum").and_then(Value::as_f64) {
            if n < min {
                return Err(format!("{path}: minimum {min} (실제 {n})"));
            }
        }
        if let Some(max) = schema.get("maximum").and_then(Value::as_f64) {
            if n > max {
                return Err(format!("{path}: maximum {max} (실제 {n})"));
            }
        }
    }

    Ok(())
}

fn validate_plan_schema(plan: &Value) -> Result<(), String> {
    let schema = rhwp::plan_schema::plan_schema();
    validate_against(&schema, &schema, plan, "$")
}

fn prop_config() -> ProptestConfig {
    ProptestConfig {
        cases: 32,
        max_shrink_iters: 64,
        ..ProptestConfig::default()
    }
}

#[test]
fn generator_actions_match_export_plan_schema() {
    let schema = rhwp::plan_schema::plan_schema();
    assert_eq!(schema["$ref"], "#/$defs/Plan");
    assert_eq!(schema_actions(&schema), ACTIONS);
}

#[test]
fn seed_plan_roundtrips_and_matches_schema() {
    let plan = valid_seed_plan();
    let text = serde_json::to_string_pretty(&plan).expect("serialize");
    let back: EditPlan = serde_json::from_str(&text).expect("deserialize");
    assert_eq!(plan, back);
    let value: Value = serde_json::from_str(&text).expect("value");
    assert_eq!(value["planVersion"], PLAN_VERSION);
    validate_plan_schema(&value).expect("schema");
}

#[test]
fn handwritten_invalid_plans_are_rejected() {
    let cases = [
        json!({"planVersion": "1.0", "input": "a.hwp", "output": "b.hwp", "steps": []}),
        json!({"planVersion": "9.9", "input": "a.hwp", "output": "b.hwp",
            "steps": [{"action": "fill_fields", "data": {"이름": "x"}}]}),
        json!({"planVersion": "1.0", "input": "a.hwp", "output": "b.hwp",
            "steps": [{"action": "nope"}]}),
        json!({"planVersion": "1.0", "input": "a.hwp", "output": "b.hwp",
            "steps": [{"action": "replace_text", "find": "", "replace": "x"}]}),
        json!({"planVersion": "1.0", "input": "a.hwp", "output": "b.hwp",
            "steps": [{"action": "set_cell", "table": 0, "row": 0, "col": 0, "text": "x\t"}]}),
    ];
    for case in cases {
        assert!(
            validate_plan_schema(&case).is_err(),
            "통과하면 안 됨: {case}"
        );
    }
}

proptest! {
    #![proptest_config(prop_config())]

    #[test]
    fn generated_plans_serialize_and_match_schema(plan in arb_valid_plan()) {
        let text = serde_json::to_string(&plan).expect("serialize");
        let back: EditPlan = serde_json::from_str(&text).expect("deserialize");
        prop_assert_eq!(&plan, &back);
        let value: Value = serde_json::from_str(&text).expect("value");
        let again: EditPlan = serde_json::from_value(value.clone()).expect("from_value");
        prop_assert_eq!(plan, again);
        if let Err(err) = validate_plan_schema(&value) {
            return Err(TestCaseError::fail(format!("{err}\n{value}")));
        }
    }

    #[test]
    fn invalid_plans_are_rejected_not_executed(plan in arb_invalid_plan()) {
        prop_assert!(
            validate_plan_schema(&plan).is_err(),
            "무효 계획이 스키마를 통과함: {plan}"
        );
    }
}
