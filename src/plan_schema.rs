//! [#3719 §6-4] `export-plan-schema` — 계획서(`rhwp run`)의 JSON Schema 를 기계 산출한다.
//!
//! `run --json` 이 계획을 **검사**한다면 이 스키마는 계획을 **만드는** 쪽의 정답지다.
//! 에이전트는 지금까지 계획서의 모양을 도구 설명 한 문단(`hwp_run_plan.description`)에서
//! 역추론했다. 한 문단은 "무엇이 필수인지"·"어떤 값이 허용되는지"를 담지 못하므로,
//! 모델은 그럴듯한 필드명을 지어내고 실행기는 그때서야 `invalid[]` 로 되돌려보낸다.
//! 왕복 한 번이 실패 한 번이다 — 계획을 **쓰기 전에** 읽을 수 있는 계약이 필요하다.
//!
//! ## 왜 손으로 쓰는가
//!
//! `run_plan_engine` 은 계획서를 `serde_json::Value` 로 직접 읽는다(파생할 타입이 없다).
//! 실제 계획 예시에서 역추론하면 "그 예시가 마침 쓰지 않은 선택 필드"가 계약에서 통째로
//! 빠진다(`caseSensitive`·`keepStyle`·`occurrence` 가 그런 부류다). 여기 적은 목록이
//! 곧 "우리가 계획 생성기에게 약속하는 계획서 표면"이며, 판정자(`run_plan_engine`)의
//! 선검증 분기와 1:1로 대응한다.
//!
//! ## 판별 유니온
//!
//! step 4종은 `action` 을 판별자로 하는 `oneOf` 다. 각 분기가 `action` 을 `const` 로
//! 고정하므로 정확히 하나만 통과한다 — 소비자(코드 생성기)는 이 모양을 태그드 유니온으로
//! 그대로 옮길 수 있다.
//!
//! ## 열린 객체와 닫힌 객체
//!
//! 계획서 본체·step 은 `additionalProperties: true`(추가-전용 진화)다. 반면 **조건절은
//! 닫혀 있다** — 실행기가 모르는 조건 키를 `invalid` 로 거부하기 때문이다. 스키마가
//! 실행기보다 관대하면 "스키마는 통과하는데 실행은 거부하는" 계획이 생기고, 그건
//! 계약이 아니라 함정이다.
//!
//! ## 버저닝
//!
//! `planSchemaVersion` 은 봉투 `schemaVersion` 과 **분리**된 계획서 문법의 버전이다.
//! 필드 추가 = minor, 의미 변경·삭제 = major. 계획서 자신이 싣는 `planVersion` 과도
//! 다른 축이다(`planVersion` 은 계획서가 선언하는 값, 이쪽은 스키마 문서의 판번호).

use serde_json::{json, Value};

use crate::schema_registry::ENVELOPE_SCHEMA_VERSION;

/// 계획 스키마 버전 — 단일 출처는 [`crate::schema_registry`](#4329). 여기서는
/// 재수출만 해 기존 호출부 경로를 보존한다. 봉투 schemaVersion·계획서
/// planVersion(아래 `REQUIRED_PLAN_VERSION`, 계획 파일이 선언하는 문법 수용
/// 게이트)과는 여전히 독립적으로 진화한다.
pub use crate::schema_registry::PLAN_SCHEMA_VERSION;

/// 계획서가 선언해야 하는 `planVersion` 값 — 실행기가 이 값만 받는다.
const REQUIRED_PLAN_VERSION: &str = "1.0";

/// JSON Schema draft — 소비자(코드 생성기)가 파서를 고를 수 있게 명시한다.
const SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// `$defs` 참조 하나.
fn r(name: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{name}") })
}

/// 설명이 달린 `$defs` 참조. 2020-12 는 `$ref` 옆 키워드를 허용하므로, 참조라는 이유로
/// 설명을 잃지 않는다 — 참조된 정의의 일반 설명과 "이 자리에서의 뜻"은 다른 정보다.
fn r_doc(name: &str, description: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{name}"), "description": description })
}

/// 설명이 달린 원시 타입.
fn prim(ty: &str, description: &str) -> Value {
    json!({ "type": ty, "description": description })
}

/// 비어 있으면 뜻이 없는 문자열.
///
/// 조건 피연산자는 빈 문자열을 받으면 실행기가 사용법 오류로 거부한다. 스키마도 같은
/// 제약을 내보내야 검증기를 통과한 계획이 실행 단계에서 막히지 않는다.
fn nonempty_string(description: &str) -> Value {
    json!({ "type": "string", "minLength": 1, "description": description })
}

/// 값이 고정된 문자열 — 판별 유니온의 판별자와 버전 상수에 쓴다.
fn constant(value: &str, description: &str) -> Value {
    json!({ "type": "string", "const": value, "description": description })
}

/// 0 이상 정수 — 순번·표 좌표는 음수를 가질 수 없다.
fn uint(description: &str) -> Value {
    json!({ "type": "integer", "minimum": 0, "description": description })
}

/// 배열 타입. `min_items` 로 "비어 있으면 무의미한 배열"을 스키마 수준에서 막는다.
fn array_of(items: Value, min_items: usize, description: &str) -> Value {
    json!({
        "type": "array",
        "items": items,
        "minItems": min_items,
        "description": description,
    })
}

/// 열린 객체 — 필수 필드를 명시하되 추가 필드를 허용한다.
///
/// `additionalProperties: true` 는 의도적이다. 계획서는 **추가-전용 진화** 계약이라
/// 새 선택 필드가 붙어도 기존 계획서가 계속 유효해야 한다. false 로 두면 step 종류를
/// 하나 늘릴 때마다 예전 스키마를 든 생성기가 전부 동시에 거짓 위반을 낸다.
fn object(properties: Value, required: &[&str], description: &str) -> Value {
    json!({
        "type": "object",
        "description": description,
        "properties": properties,
        "required": required,
        "additionalProperties": true,
    })
}

/// 닫힌 객체 — 선언되지 않은 키를 거부한다.
///
/// 실행기가 모르는 키를 거부하는 자리에만 쓴다(조건절). 스키마가 실행기보다 관대하면
/// 통과하는 계획이 실행에서 거부되고, 소비자는 어느 쪽을 믿어야 할지 알 수 없다.
fn closed_object(properties: Value, required: &[&str], description: &str) -> Value {
    json!({
        "type": "object",
        "description": description,
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

// ── 계획서 봉투 ──────────────────────────────────────────────────────────

fn plan_def() -> Value {
    object(
        json!({
            "planVersion": constant(
                REQUIRED_PLAN_VERSION,
                "계획서 문법 판번호. 실행기는 \"1.0\" 만 받고 그 밖의 값은 exit 2 로 거절한다.",
            ),
            "input": prim("string", "원본 문서 경로 (HWP/HWPX/HML). 실행기는 이 파일을 읽기만 한다."),
            "output": prim(
                "string",
                "산출 경로. 전 step 이 인메모리로 적용되고 단언을 통과한 뒤 **단 한 번** 여기에 쓴다. \
                 확장자가 산출 형식을 정한다(생략 시 입력 형식 보존).",
            ),
            "steps": array_of(
                r("Step"),
                1,
                "적용할 편집 목록. 선언 순서대로 인메모리에 적용된다. 비어 있으면 exit 2 — \
                 아무것도 하지 않는 계획은 의도가 아니라 실수다.",
            ),
            "assertions": r_doc(
                "Assertions",
                "저장 전에 통과해야 하는 단언. 생략하면 기본값으로 판정한다.",
            ),
            "preconditions": r_doc(
                "Preconditions",
                "[#4378 R22] 실행 전 전제(CAS). 하나라도 어긋나면 실행 0·저장 0 으로 \
                 거절한다(exit 2, invalid[].code=preconditionFailed).",
            ),
            "dryRun": json!({
                "type": "boolean",
                "default": false,
                "description": "참이면 선검증만 하고 저장하지 않는다(디스크 무변경). 저널 대신 \
                                preview[] 를 낸다 — 각 항목은 step·action 에 더해 조건이 거짓이면 \
                                skipped:true·reason 을 싣는다. `run --dry-run` 플래그와 동등하다.",
            }),
        }),
        &["planVersion", "input", "output", "steps"],
        "`rhwp run` 이 받는 편집 계획서. 도구 호출을 체이닝하는 대신 의도 하나를 선언하면 \
         실행기가 전 step 의 실행 가능성을 먼저 판정하고(불가 시 실행 0·exit 2), \
         인메모리 원자 실행 뒤 단언 통과 시에만 저장한다.",
    )
}

/// [#4378 R22] 실행 전 전제 — 계획이 세워진 시점의 문서 상태를 고정한다.
fn preconditions_def() -> Value {
    object(
        json!({
            "inputSha256": prim(
                "string",
                "input 파일 전체의 SHA-256(64자리 16진, 대소문자 무관). 실행 시점의 \
                 실제 해시와 다르면 — 계획 수립 후 다른 에이전트/사람이 문서를 바꿨다는 \
                 뜻이므로 — 아무것도 적용하지 않고 invalid[](code=preconditionFailed)로 \
                 거절한다. 경합 유실(#3905: 두 exit 0 이 편집 하나를 무신호로 지움)의 \
                 차단기다. 해시는 `rhwp-agent fingerprint` 또는 sha256sum 으로 얻는다.",
            ),
        }),
        &["inputSha256"],
        "실행 전 전제. 현재 축은 inputSha256 하나이며, 축 추가는 minor 다.",
    )
}

fn assertions_def() -> Value {
    object(
        json!({
            "notFoundEmpty": json!({
                "type": "boolean",
                "default": true,
                "description": "지목한 대상이 하나도 빠지지 않았음을 요구한다. 선검증이 구조적으로 \
                                보장하므로(없는 대상은 실행 전에 invalid) 저널에 계약 표기로 실린다.",
            }),
            "verify": json!({
                "type": "boolean",
                "default": false,
                "description": "저장 직전 산출 바이트를 다시 읽어 인메모리 IR 과 대조한다. \
                                차이가 있으면 exit 3 이고 **디스크는 무변경**이다.",
            }),
        }),
        &[],
        "사후 단언 — 실패하면 저장하지 않는다. 계획이 스스로 검증 조건을 들고 다니게 해서 \
         \"성공했다고 보고했는데 산출물이 깨진\" 상태를 구조적으로 없앤다.",
    )
}

// ── step 판별 유니온 ─────────────────────────────────────────────────────

fn step_def() -> Value {
    json!({
        "description": "편집 step 하나. `action` 이 판별자인 태그드 유니온이며, 4종 전부 \
                        선택 필드 `if`(조건절)를 받는다.",
        "oneOf": [
            r("FillFieldsStep"),
            r("ReplaceTextStep"),
            r("SetCellStep"),
            r("SetCheckboxStep"),
        ],
    })
}

/// step 분기 하나를 만든다.
///
/// `action`(판별자)과 `if`(조건절)를 **여기서 한 번만** 붙인다. 분기마다 손으로 적으면
/// 새 step 종류가 조건절을 빠뜨린 채 추가될 수 있다 — 4종 전부가 조건을 받는다는 것이
/// §6-8 의 계약이므로 그 사실을 조립 지점에 고정한다.
fn step_variant(
    action: &str,
    action_doc: &str,
    extra: Value,
    required: &[&str],
    doc: &str,
) -> Value {
    let mut properties = serde_json::Map::new();
    properties.insert("action".to_string(), constant(action, action_doc));
    if let Some(map) = extra.as_object() {
        for (key, value) in map {
            properties.insert(key.clone(), value.clone());
        }
    }
    // `if` 는 JSON Schema 키워드이기도 하지만 여기서는 `properties` 아래의 **이름**이라
    // 충돌하지 않는다. 계획서가 쓰는 실제 키가 `if` 이므로 다른 이름을 쓸 수 없다.
    properties.insert(
        "if".to_string(),
        r_doc(
            "StepCondition",
            "선택 조건. 거짓이면 이 step 을 건너뛰고 저널에 skipped:true 로 남긴다. \
             거짓인 step 은 선검증에서도 면제된다(없는 필드를 채우는 step 이라도 위반이 아니다).",
        ),
    );
    let mut required_all = vec!["action".to_string()];
    required_all.extend(required.iter().map(|s| s.to_string()));
    object(
        Value::Object(properties),
        &required_all.iter().map(String::as_str).collect::<Vec<_>>(),
        doc,
    )
}

fn fill_fields_step_def() -> Value {
    step_variant(
        "fill_fields",
        "누름틀(필드)에 값을 채우는 step 임을 지목한다.",
        json!({
            "data": json!({
                "type": "object",
                "description": "필드 이름 → 채울 값. 같은 이름이 여러 번 나오는 서식은 \
                                `이름[순번]`(0 기준)으로 지목한다(예: \"피규제집단명[13]\"). \
                                순번 없이 이름만 주면 첫 칸을 채우고 저널의 ambiguous 로 알린다.",
                "additionalProperties": {
                    "type": ["string", "number", "boolean"],
                    "description": "채울 값. 문자열이 아니면 JSON 표기 그대로 문자열화해 넣는다.",
                },
            }),
        }),
        &["data"],
        "누름틀 채우기 (메일머지). 화면상 구별되지 않는 이름의 쌍둥이 필드가 있으면 채우되 \
         저널의 `confusable` 로 반드시 보고한다.",
    )
}

fn replace_text_step_def() -> Value {
    step_variant(
        "replace_text",
        "본문 문자열을 치환하는 step 임을 지목한다.",
        json!({
            "find": json!({
                "type": "string",
                "minLength": 1,
                "description": "찾을 문자열. 빈 문자열은 거부한다(문서 전체를 뜻하게 되므로).",
            }),
            "replace": prim("string", "바꿔 넣을 문자열. 빈 문자열이면 삭제다."),
            "caseSensitive": json!({
                "type": "boolean",
                "default": true,
                "description": "대소문자를 구별할지. 기본은 구별한다.",
            }),
            "occurrence": uint(
                "0 기준 일치 순번. 주면 그 한 건만, 생략하면 전건을 치환한다. \
                 범위를 벗어나면 선검증에서 거부한다.",
            ),
        }),
        &["find", "replace"],
        "문자열 치환 (기관명 변경·연도 갱신·용어 정비). 일치 0건이면 선검증에서 거부한다 \
         — 조용히 0건 치환을 성공으로 보고하지 않는다.",
    )
}

fn set_cell_step_def() -> Value {
    step_variant(
        "set_cell",
        "표 한 칸의 내용을 바꾸는 step 임을 지목한다.",
        json!({
            "table": uint("0 기준 표 번호 (문서 순서)."),
            "row": json!({
                "type": "integer",
                "minimum": 0,
                "maximum": 65535,
                "description": "0 기준 행 번호. HWP 격자 주소는 u16 이라 65535 를 넘으면 거부한다.",
            }),
            "col": json!({
                "type": "integer",
                "minimum": 0,
                "maximum": 65535,
                "description": "0 기준 열 번호. HWP 격자 주소는 u16 이라 65535 를 넘으면 거부한다.",
            }),
            "text": json!({
                "type": "string",
                // 실행기가 \r·\n·\t 를 선검증에서 거부한다 — 스키마도 같은 말을 해야
                // 검증기를 통과한 계획이 실행에서 막히지 않는다.
                "pattern": "^[^\r\n\t]*$",
                "description": "칸에 기록할 한 줄 값. 줄바꿈·탭은 거부한다(칸 구조가 깨지므로).",
            }),
            "keepStyle": json!({
                "type": "boolean",
                "default": false,
                "description": "참이면 기존 글자색을 유지한다. 기본은 검정으로 바꾼다 \
                                — 서식 안내문의 회색 예시 글자를 그대로 두면 제출본에 남는다.",
            }),
        }),
        &["table", "row", "col", "text"],
        "표 칸 기록. 좌표는 실행 시점에 다시 해석하므로 앞 step 의 편집으로 밀린 표도 안전하다. \
         병합으로 덮인 칸은 앵커 좌표 안내와 함께 거부한다.",
    )
}

fn set_checkbox_step_def() -> Value {
    step_variant(
        "set_checkbox",
        "빈 체크박스를 표시하는 step 임을 지목한다.",
        json!({
            "occurrence": uint(
                "0 기준 빈 체크박스(□) 순번. 범위를 벗어나면 선검증에서 거부한다.",
            ),
        }),
        &["occurrence"],
        "체크박스 표시 — 문서의 n 번째 빈 체크박스 □ 를 ☑ 로 바꾼다.",
    )
}

// ── 조건절 (§6-8) ────────────────────────────────────────────────────────

fn step_condition_def() -> Value {
    let mut def = closed_object(
        json!({
            "fieldExists": nonempty_string(
                "그 이름의 누름틀이 문서에 있으면 참. `이름[순번]`(0 기준)으로 특정 순번의 \
                 존재를 물을 수도 있다 — 동명 필드가 몇 개인지에 따라 참·거짓이 갈린다.",
            ),
            "fieldEquals": r_doc(
                "FieldEqualsCondition",
                "지목한 누름틀의 **현재 값**이 주어진 값과 정확히 같으면 참.",
            ),
            "textFound": nonempty_string(
                "그 문자열이 본문에 한 건이라도 있으면 참 (대소문자 구별).",
            ),
        }),
        &[],
        "step 조건절. 정확히 한 종류만 쓴다 — 두 조건을 함께 주면 and 인지 or 인지가 \
         계획서에 적혀 있지 않으므로 거부한다. 판정은 **입력 문서 기준으로 실행 전에 한 번** \
         이루어지며, 앞 step 의 편집 결과가 뒤 step 의 조건을 바꾸지 않는다 \
         (선검증과 실행이 같은 판정을 보게 하기 위한 것이다).",
    );
    // 실행기는 조건 **개수**를 세어 둘 이상이면 거부한다. 스키마가 그 말을 하지 않으면
    // 검증기를 통과한 계획이 실행에서 막힌다 — 설명문에 적는 것만으로는 부족하다.
    if let Some(map) = def.as_object_mut() {
        map.insert("minProperties".to_string(), json!(1));
        map.insert("maxProperties".to_string(), json!(1));
    }
    def
}

// ── dry-run 미리보기 (§6-11) ─────────────────────────────────────────────

fn preview_step_def() -> Value {
    object(
        json!({
            "step": uint("0 기준 step 순번 — 계획서 steps[] 의 인덱스와 같다. 건너뛴 step 도 \
                          자리를 지키므로 저널 항목과 계획서 항목을 순번으로 짝지을 수 있다."),
            "action": prim(
                "string",
                "그 step 의 action (fill_fields·replace_text·set_cell·set_checkbox 중 하나).",
            ),
            "skipped": json!({
                "type": "boolean",
                "description": "참이면 `if` 조건이 거짓이라 이 step 은 실행되지 않는다 — 선검증도 \
                                면제된다. 거짓(또는 생략)이면 정상적으로 실행 가능하다고 판정된 것이다.",
            }),
            "reason": prim(
                "string",
                "`skipped` 가 참일 때만 실리는 사유 — 어느 조건이 왜 거짓이었는지. 실행 가능한 \
                 step 에는 없다.",
            ),
        }),
        &["step", "action"],
        "`dryRun:true` 저널의 `preview[]` 원소 하나. `skipped` 가 없거나 거짓이면 그 step 은 실제 \
         실행에서도 적용될 것이라는 뜻이고, 참이면 `reason` 과 함께 건너뛸 것이라는 뜻이다 — \
         dry-run 이 예고하는 실행 개수와 `run`(실제 실행)이 보고할 적용 개수가 같은 말을 하도록 한다.",
    )
}

fn field_equals_condition_def() -> Value {
    closed_object(
        json!({
            "name": nonempty_string(
                "누름틀 이름. `이름[순번]`(0 기준)으로 동명 필드 중 하나를 지목할 수 있다.",
            ),
            "value": prim("string", "비교할 값. 문자열 완전 일치로만 판정한다."),
        }),
        &["name", "value"],
        "`fieldEquals` 의 피연산자 — 어떤 필드의 값이 무엇과 같은지.",
    )
}

// ── 조립 ─────────────────────────────────────────────────────────────────

/// `$defs` 의 정의 수 — 봉투가 규모를 미리 알리는 데 쓴다.
fn definition_count(schema: &Value) -> usize {
    schema
        .get("$defs")
        .and_then(|d| d.as_object())
        .map(|o| o.len())
        .unwrap_or(0)
}

/// `rhwp run` 계획서 전체의 JSON Schema.
pub fn plan_schema() -> Value {
    // 정의가 늘면 json! 매크로 재귀 한도에 걸린다 — 맵으로 조립한다.
    let defs: serde_json::Map<String, Value> = [
        ("Plan", plan_def()),
        ("Preconditions", preconditions_def()),
        ("Assertions", assertions_def()),
        ("Step", step_def()),
        ("FillFieldsStep", fill_fields_step_def()),
        ("ReplaceTextStep", replace_text_step_def()),
        ("SetCellStep", set_cell_step_def()),
        ("SetCheckboxStep", set_checkbox_step_def()),
        ("StepCondition", step_condition_def()),
        ("FieldEqualsCondition", field_equals_condition_def()),
        ("PreviewStep", preview_step_def()),
    ]
    .into_iter()
    .map(|(name, def)| (name.to_string(), def))
    .collect();

    json!({
        "$schema": SCHEMA_DIALECT,
        // [#4329] $id 의 버전 조각도 레지스트리 상수에서 파생 — 리터럴 산개 금지.
        "$id": format!("https://github.com/edwardkim/rhwp/schema/plan/{PLAN_SCHEMA_VERSION}"),
        "title": "rhwp 편집 계획서",
        "planSchemaVersion": PLAN_SCHEMA_VERSION,
        "description":
            "`rhwp run` 이 실행하는 선언적 편집 계획서의 공개 계약. 계획을 **만드는** 쪽이 \
             읽는 문서다 — 필수·선택 필드와 허용 값이 실행기의 선검증 분기와 1:1로 대응한다. \
             진화 규약: 필드 추가 = minor, 의미 변경·삭제 = major.",
        "$ref": "#/$defs/Plan",
        // 표준 JSON Schema 키워드가 아니지만(x- 접두어 벤더 확장), `dryRun:true` 저널의
        // `preview[]` 원소 모양을 이 스키마 문서 안에서 가리킬 유일한 자리다 — Plan 자체는
        // 입력을 서술하고 preview 는 출력이라, Plan.properties 안에 넣으면 "계획서가
        // 받는 필드"로 오독된다. $ref 를 여기 두면 정의 도달성 검사(고아 $defs 없음)도
        // 그대로 통과한다.
        "x-dryRunPreview": r_doc(
            "PreviewStep",
            "`dryRun:true` 로 실행했을 때 저널의 `preview[]` 배열 원소 하나의 모양. 입력 \
             계획서가 받는 필드가 아니라 실행기가 돌려주는 저널의 계약이다.",
        ),
        "$defs": defs,
    })
}

/// `export-plan-schema` 봉투 — 스키마 본문과 메타를 함께 싣는다.
///
/// `definitionCount` 는 소비자가 코드 생성 규모를 먼저 가늠하고, 스키마가 통째로 비는
/// 회귀를 한 숫자로 잡게 한다.
pub fn envelope() -> Value {
    let schema = plan_schema();
    let def_count = definition_count(&schema);
    json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "planSchemaVersion": PLAN_SCHEMA_VERSION,
        "dialect": SCHEMA_DIALECT,
        "definitionCount": def_count,
        "schema": schema,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 닫힌 객체와 그 사유 — 사유 없는 예외는 허용목록이 아니라 구멍이다.
    const CLOSED_DEFS: &[(&str, &str)] = &[
        (
            "StepCondition",
            "실행기가 모르는 조건 키를 invalid 로 거부한다 — 스키마가 더 관대하면 통과한 계획이 실행에서 막힌다",
        ),
        (
            "FieldEqualsCondition",
            "피연산자가 name·value 둘뿐이다 — 셋째 키는 뜻이 정의돼 있지 않다",
        ),
    ];

    #[test]
    fn schema_has_root_and_defs() {
        let schema = plan_schema();
        assert_eq!(schema["$ref"], "#/$defs/Plan");
        assert!(schema["$defs"]["Plan"].is_object());
        assert!(schema["$defs"]["StepCondition"].is_object());
    }

    #[test]
    fn every_ref_resolves_to_a_definition() {
        // 끊어진 $ref 는 코드 생성기를 즉시 망가뜨린다 — 스키마의 최소 건전성이다.
        let schema = plan_schema();
        let defs = schema["$defs"].as_object().expect("$defs");
        let mut missing = Vec::new();
        collect_refs(&schema, &mut |name| {
            if !defs.contains_key(name) {
                missing.push(name.to_string());
            }
        });
        assert!(missing.is_empty(), "정의되지 않은 참조: {missing:?}");
    }

    #[test]
    fn definitions_are_reachable_from_root() {
        // 아무도 가리키지 않는 정의는 죽은 계약이다.
        let schema = plan_schema();
        let defs = schema["$defs"].as_object().expect("$defs");
        let mut referenced = std::collections::HashSet::new();
        referenced.insert("Plan".to_string());
        collect_refs(&schema, &mut |name| {
            referenced.insert(name.to_string());
        });
        let orphans: Vec<&String> = defs.keys().filter(|k| !referenced.contains(*k)).collect();
        assert!(orphans.is_empty(), "아무도 참조하지 않는 정의: {orphans:?}");
    }

    #[test]
    fn objects_are_open_unless_listed_with_a_reason() {
        // 추가-전용 진화 계약: step 종류가 늘어도 기존 계획서가 깨지면 안 된다.
        // 닫아야 하는 자리는 사유와 함께 CLOSED_DEFS 에 등재한다.
        let schema = plan_schema();
        let defs = schema["$defs"].as_object().expect("$defs");
        for (name, def) in defs {
            if def["type"] != "object" {
                continue;
            }
            let closed = CLOSED_DEFS.iter().find(|(n, _)| n == name);
            match closed {
                Some((_, why)) => {
                    assert_eq!(
                        def["additionalProperties"], false,
                        "{name} 은 닫힌 객체로 등재됐는데 실제로는 열려 있다 ({why})"
                    );
                }
                None => assert_eq!(
                    def["additionalProperties"], true,
                    "{name} 이 추가 필드를 막고 있다 — 계획서는 추가-전용 진화다. \
                     정말 닫아야 하면 CLOSED_DEFS 에 사유와 함께 등재하라"
                ),
            }
        }
        for (name, why) in CLOSED_DEFS {
            assert!(defs.contains_key(*name), "낡은 CLOSED_DEFS 항목: {name}");
            assert!(!why.trim().is_empty(), "{name} 의 닫은 사유가 비었습니다");
        }
    }

    #[test]
    fn step_union_is_discriminated_by_action() {
        // 판별자가 없으면 oneOf 는 소비자에게 "넷 중 뭔지 직접 알아내라"는 말이다.
        let schema = plan_schema();
        let variants = schema["$defs"]["Step"]["oneOf"]
            .as_array()
            .expect("Step.oneOf");
        assert_eq!(variants.len(), 4, "step 4종");
        let mut actions = Vec::new();
        for variant in variants {
            let name = variant["$ref"]
                .as_str()
                .and_then(|p| p.strip_prefix("#/$defs/"))
                .expect("oneOf 분기는 $defs 참조");
            let def = &schema["$defs"][name];
            let action = def["properties"]["action"]["const"]
                .as_str()
                .unwrap_or_else(|| panic!("{name} 에 action const 가 없다"));
            assert!(
                def["required"]
                    .as_array()
                    .expect("required")
                    .iter()
                    .any(|r| r == "action"),
                "{name} 이 action 을 필수로 선언하지 않았다"
            );
            // 조건부 step 은 4종 전부의 계약이다 — 하나라도 빠지면 유니온이 거짓말한다.
            assert!(
                def["properties"]["if"]["$ref"] == "#/$defs/StepCondition",
                "{name} 이 조건절 `if` 를 받지 않는다"
            );
            actions.push(action.to_string());
        }
        actions.sort();
        assert_eq!(
            actions,
            ["fill_fields", "replace_text", "set_cell", "set_checkbox"]
        );
    }

    #[test]
    fn every_definition_and_property_carries_a_description() {
        // 설명 없는 정의는 모델에게 없는 정의다 — 이름만으로는 무엇을 넣을지 알 수 없다.
        let schema = plan_schema();
        let mut missing = Vec::new();
        for (name, def) in schema["$defs"].as_object().expect("$defs") {
            check_described(def, name, &mut missing);
        }
        assert!(missing.is_empty(), "설명 없는 스키마 노드: {missing:?}");
    }

    #[test]
    fn envelope_reports_definition_count() {
        let env = envelope();
        assert_eq!(env["schemaVersion"], "1.0");
        assert_eq!(env["planSchemaVersion"], PLAN_SCHEMA_VERSION);
        assert_eq!(env["dialect"], SCHEMA_DIALECT);
        let count = env["definitionCount"].as_u64().expect("definitionCount");
        assert!(count >= 9, "정의가 너무 적다: {count}");
        assert_eq!(env["schema"]["$ref"], "#/$defs/Plan");
    }

    /// 스키마 노드와 그 하위 `properties` 가 전부 설명을 갖는지 본다.
    /// 순수 `$ref` 노드(키가 `$ref` 하나뿐)는 참조된 정의가 설명을 갖고 있으므로 면제한다.
    fn check_described(node: &Value, path: &str, missing: &mut Vec<String>) {
        let Some(map) = node.as_object() else {
            return;
        };
        let pure_ref = map.len() == 1 && map.contains_key("$ref");
        if !pure_ref
            && !map
                .get("description")
                .and_then(Value::as_str)
                .is_some_and(|d| !d.trim().is_empty())
        {
            missing.push(path.to_string());
        }
        if let Some(props) = map.get("properties").and_then(Value::as_object) {
            for (key, value) in props {
                check_described(value, &format!("{path}.{key}"), missing);
            }
        }
        // `additionalProperties` 가 스키마면 그것도 값의 계약이다.
        if let Some(extra) = map.get("additionalProperties") {
            if extra.is_object() {
                check_described(extra, &format!("{path}.additionalProperties"), missing);
            }
        }
    }

    /// `$ref` 를 재귀 수집한다.
    fn collect_refs(value: &Value, sink: &mut impl FnMut(&str)) {
        match value {
            Value::Object(map) => {
                for (key, item) in map {
                    if key == "$ref" {
                        if let Some(path) = item.as_str() {
                            if let Some(name) = path.strip_prefix("#/$defs/") {
                                sink(name);
                            }
                        }
                    } else {
                        collect_refs(item, sink);
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    collect_refs(item, sink);
                }
            }
            _ => {}
        }
    }
}
