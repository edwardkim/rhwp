//! [#4545] 정책 게이트 — 6년 축(반입 판정의 기계화) 코어.
//!
//! ## 왜 연산자 4개뿐인가
//!
//! 정책은 감사 대상 문서다. 표현식 언어(스크립트·정규식·산술)를 넣는 순간
//! 정책은 튜링 완전해지고, 튜링 완전한 정책은 감사 불가능한 정책이다.
//! `eq`·`in`·`gte`·`lte` 4개로 부족한 표현력은 규칙을 늘려 해결한다
//! (설계서 §2.1 결정 그대로).
//!
//! ## 왜 미지 키가 로드 시점 오류인가
//!
//! 판정 키 오타(`reproducd`)를 조용히 넘기면 그 규칙은 영원히 평가되지 않는
//! 항상-참이 된다 — 게이트가 뚫린 줄도 모르게. 고정 사전(JUDGMENT_KEYS)
//! 밖의 키는 정책 로드 시점에 사용법 오류다.
//!
//! ## 재계산 원칙
//!
//! 게이트는 캡슐에 적힌 자기 신고를 읽지 않는다 — 판정 재료는 전부 검증
//! 축의 **재계산**(계보 걷기·서명 검증·앵커 조회·재실행)에서 온다. 신고
//! 기반 게이트는 게이트가 아니다.

use std::collections::BTreeMap;

/// 판정 키 고정 사전 — 이 목록이 정책이 쓸 수 있는 좌변의 전부다.
///
/// 값의 산출처(재계산 경로)는 cmd_gate 가 책임진다. 새 검증 축이 생기면
/// 여기와 cmd_gate 의 계산기가 함께 늘어야 하며, 사전에 없는 키는 정책
/// 로드가 거부한다.
pub const JUDGMENT_KEYS: &[&str] = &[
    "reproduced",    // bool — 계획 재실행 산출이 영수증과 일치 (--deep 재계산)
    "lineageValid",  // bool — 머리부터 뿌리까지 체인 무결
    "lineageDepth",  // number — 체인 길이
    "signerVerdict", // string — valid|invalid|unknownKey|revoked|malformed|unsigned
    "signerKeyId",   // string|null — 사이드카의 키 식별자
    "anchoredOk",    // bool — 앵커 로그 등재 여부
];

const OPS: &[&str] = &["eq", "in", "gte", "lte"];

/// 파싱·검증된 정책.
pub struct Policy {
    pub name: String,
    pub default_deny: bool,
    pub rules: Vec<Rule>,
}

pub struct Rule {
    pub id: String,
    /// (판정 키, 연산자, 기대값)
    pub require: Vec<(String, String, serde_json::Value)>,
}

/// 정책 파일을 파싱하고 키·연산자를 사전 대조한다 — 위반은 Err(사용법 오류).
pub fn parse(text: &str) -> Result<Policy, String> {
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("정책 JSON 파싱 실패 - {e}"))?;
    if v["kind"] != "admissionPolicy" {
        return Err("정책 kind 가 admissionPolicy 가 아닙니다".to_string());
    }
    let name = v["name"].as_str().unwrap_or("(이름 없음)").to_string();
    let default_deny = match v["defaultVerdict"].as_str() {
        None | Some("deny") => true,
        Some("allow") => false,
        Some(other) => return Err(format!("미지 defaultVerdict: {other} (deny|allow)")),
    };
    let mut rules = Vec::new();
    for (idx, rule) in v["rules"]
        .as_array()
        .ok_or("정책에 rules 배열이 필요합니다")?
        .iter()
        .enumerate()
    {
        let id = rule["id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("R{idx}"));
        let mut require = Vec::new();
        let req = rule["require"]
            .as_object()
            .ok_or_else(|| format!("{id}: require 객체가 필요합니다"))?;
        for (key, cond) in req {
            if !JUDGMENT_KEYS.contains(&key.as_str()) {
                return Err(format!(
                    "{id}: 미지 판정 키 `{key}` — 사전: {JUDGMENT_KEYS:?} (오타는 항상-참 구멍이 되므로 로드 시점에 거부한다)"
                ));
            }
            let cond = cond
                .as_object()
                .ok_or_else(|| format!("{id}.{key}: {{연산자: 값}} 객체가 필요합니다"))?;
            for (op, expected) in cond {
                if !OPS.contains(&op.as_str()) {
                    return Err(format!("{id}.{key}: 미지 연산자 `{op}` — 허용: {OPS:?}"));
                }
                require.push((key.clone(), op.clone(), expected.clone()));
            }
        }
        rules.push(Rule { id, require });
    }
    Ok(Policy {
        name,
        default_deny,
        rules,
    })
}

/// 규칙이 참조하는 판정 키 집합 — 지연 계산의 근거.
pub fn referenced_keys(policy: &Policy) -> std::collections::BTreeSet<String> {
    policy
        .rules
        .iter()
        .flat_map(|r| r.require.iter().map(|(k, _, _)| k.clone()))
        .collect()
}

fn cmp_num(a: &serde_json::Value, b: &serde_json::Value) -> Option<std::cmp::Ordering> {
    a.as_f64().zip(b.as_f64()).map(|(x, y)| x.total_cmp(&y))
}

/// 평가 — (verdict allow?, violations).
///
/// 판정값이 `None`(재료 미지정 — 예: --keyring 없이 signer 규칙)이면 그
/// 규칙은 위반으로 떨어지고 actual 에 사유가 실린다: 게이트는 모르는 것을
/// 통과시키지 않는다.
pub fn evaluate(
    policy: &Policy,
    judgments: &BTreeMap<String, Option<serde_json::Value>>,
) -> (bool, Vec<serde_json::Value>) {
    let mut violations = Vec::new();
    for rule in &policy.rules {
        for (key, op, expected) in &rule.require {
            let actual = judgments.get(key).cloned().flatten();
            let pass = match (op.as_str(), &actual) {
                (_, None) => false,
                ("eq", Some(a)) => a == expected,
                ("in", Some(a)) => expected.as_array().is_some_and(|arr| arr.contains(a)),
                ("gte", Some(a)) => {
                    cmp_num(a, expected).is_some_and(|o| o != std::cmp::Ordering::Less)
                }
                ("lte", Some(a)) => {
                    cmp_num(a, expected).is_some_and(|o| o != std::cmp::Ordering::Greater)
                }
                _ => false,
            };
            if !pass {
                violations.push(serde_json::json!({
                    "rule": rule.id,
                    "key": key,
                    "op": op,
                    "expected": expected,
                    "actual": match actual {
                        Some(a) => a,
                        None => serde_json::json!("unavailable(판정 재료 미지정 — 필요한 플래그를 확인하라)"),
                    },
                }));
            }
        }
    }
    let allow = if policy.rules.is_empty() {
        // deny 기본: 명시 허용 규칙이 하나도 없으면 통과가 아니다.
        !policy.default_deny
    } else {
        violations.is_empty()
    };
    (allow, violations)
}
