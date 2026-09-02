use crate::cli::protocol::*;

/// [#3719 §6-8] step 조건절 판정 — `Ok(None)` = 조건 참(실행), `Ok(Some(사유))` =
/// 조건 거짓(건너뜀), `Err(사유)` = 조건 **문법** 오류(계획 자체가 무효).
///
/// 거짓과 문법 오류를 같은 축으로 접으면 오타 하나가 "조건이 거짓이었다"로 둔갑해
/// 계획이 조용히 아무 일도 하지 않고 성공을 보고한다. 그래서 두 축을 나눈다 —
/// 거짓은 정상 판정(exit 0, skipped 저널), 문법 오류는 `invalid` + exit 2 다.
///
/// 판정은 **입력 문서** 기준이다. 앞 step 의 편집 결과를 조건이 보게 하면 선검증(실행 전)
/// 과 실행(편집 후)이 서로 다른 답을 낼 수 있고, 그러면 "검사를 통과한 계획이 실행에서
/// 다르게 동작"한다.
pub(super) fn evaluate_step_condition(
    condition: &serde_json::Value,
    doc: &rhwp::wasm_api::HwpDocument,
    name_counts: &std::collections::HashMap<String, usize>,
    name_values: &std::collections::HashMap<String, Vec<String>>,
) -> Result<Option<String>, String> {
    let Some(map) = condition.as_object() else {
        return Err(
            "if 는 { fieldExists | fieldEquals | textFound } 중 하나를 담은 객체여야 합니다"
                .to_string(),
        );
    };
    // 조건 두 개를 나열하면 and 인지 or 인지가 계획서 어디에도 적혀 있지 않다.
    // 추측해서 실행하는 대신 거절한다 — 되돌릴 수 없는 쓰기의 전제 조건이다.
    if map.len() != 1 {
        return Err(format!(
            "if 는 조건을 정확히 하나만 담아야 합니다 (현재 {}개: {}) — 둘 이상은 and/or 가 정의돼 있지 않습니다",
            map.len(),
            map.keys().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    let (key, value) = map.iter().next().expect("길이 1");
    match key.as_str() {
        "fieldExists" => {
            let Some(spec) = value.as_str().filter(|s| !s.is_empty()) else {
                return Err(
                    "if.fieldExists 는 비어 있지 않은 필드 이름 문자열이어야 합니다".to_string(),
                );
            };
            let (name, occurrence) = parse_field_key(spec);
            let total = name_counts.get(name).copied().unwrap_or(0);
            if occurrence < total {
                Ok(None)
            } else {
                Ok(Some(format!(
                    "조건 fieldExists '{}' 불충족 — 문서의 동명 누름틀 {}개",
                    spec, total
                )))
            }
        }
        "fieldEquals" => {
            let Some(operand) = value.as_object() else {
                return Err(
                    "if.fieldEquals 는 {\"name\":<필드 이름>, \"value\":<비교값>} 객체여야 합니다"
                        .to_string(),
                );
            };
            if let Some(unknown) = operand
                .keys()
                .find(|k| k.as_str() != "name" && k.as_str() != "value")
            {
                return Err(format!(
                    "if.fieldEquals 에 알 수 없는 키: {} (name·value 만 받습니다)",
                    unknown
                ));
            }
            let (Some(spec), Some(expected)) = (
                operand.get("name").and_then(|v| v.as_str()),
                operand.get("value").and_then(|v| v.as_str()),
            ) else {
                return Err("if.fieldEquals 의 name·value 는 둘 다 문자열이어야 합니다".to_string());
            };
            if spec.is_empty() {
                return Err("if.fieldEquals 의 name 이 비어 있습니다".to_string());
            }
            let (name, occurrence) = parse_field_key(spec);
            match name_values.get(name).and_then(|v| v.get(occurrence)) {
                Some(actual) if actual == expected => Ok(None),
                Some(actual) => Ok(Some(format!(
                    "조건 fieldEquals '{}' == '{}' 불충족 — 현재값 '{}'",
                    spec, expected, actual
                ))),
                None => Ok(Some(format!(
                    "조건 fieldEquals '{}' == '{}' 불충족 — 해당 누름틀이 없습니다",
                    spec, expected
                ))),
            }
        }
        "textFound" => {
            let Some(needle) = value.as_str().filter(|s| !s.is_empty()) else {
                return Err("if.textFound 는 비어 있지 않은 문자열이어야 합니다".to_string());
            };
            // 한 건만 확인하면 되므로 limit 1 — 존재 판정에 전건 수집은 낭비다.
            if doc.grep(needle, true, Some(1)).is_empty() {
                Ok(Some(format!(
                    "조건 textFound '{}' 불충족 — 본문에서 찾지 못했습니다",
                    needle
                )))
            } else {
                Ok(None)
            }
        }
        other => Err(format!(
            "알 수 없는 조건: {} (fieldExists·fieldEquals·textFound)",
            other
        )),
    }
}
