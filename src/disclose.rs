//! [#4551] 선택적 공개 — 8년 축(가림 캡슐·부분 개봉) 코어.
//!
//! ## 문제와 해법
//!
//! 계보 증명은 공개하고 싶고 내용은 비밀이다 — 캡슐의 `plan` 에는 편집
//! 문자열(find/replace·채움 값)이 평문으로 실려 공개 시 샌다. 해법은 값을
//! **커밋** `sha256(값‖salt)` 으로 치환한 가림 캡슐과, 값·salt 를 담은 비밀
//! **개봉(opening)** 파일의 분리다. 해시 축 검증(체인·앵커·감사 회계)은
//! 가림본에도 그대로 돌고, deep 재현만 개봉 보유자의 전유물이 된다.
//!
//! ## 설계 급소 — 바이트 완전 복원
//!
//! 개봉 파일은 원본 `planText` 원문을 함께 보관한다(어차피 비밀 보관물이다).
//! 그래서 `restore` 는 캡슐을 **바이트 단위로 원본과 동일하게** 되살릴 수
//! 있고, 원본의 분리 서명(4년 축)이 복원본에서 그대로 valid 다 — 가림·복원이
//! 서명 축과 어긋나지 않는 유일한 길이다.
//!
//! ## salt 가 필수인 이유
//!
//! 커밋이 `sha256(값)` 뿐이면 저엔트로피 필드(짧은 문구·경로)는 사전 대입으로
//! 전수 복원된다. 필드마다 독립 32바이트 salt 를 도구가 강제 생성한다 — 수동
//! salt 는 받지 않는다.

use sha2::{Digest, Sha256};

/// 개봉 파일의 `kind`.
pub const OPENING_KIND: &str = "capsuleOpening";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// 커밋 = sha256(값 UTF-8 ‖ salt hex ASCII).
pub fn commit(value: &str, salt_hex: &str) -> String {
    sha256_hex(format!("{value}{salt_hex}").as_bytes())
}

/// 구조 골격 키 — 가리지 않는다(설계서: 골격 공개가 검증의 전제).
fn is_structural(key: &str) -> bool {
    matches!(key, "planVersion" | "action")
}

/// plan 의 문자열 잎을 전부 커밋으로 치환하고 (경로 → (값, salt)) 를 모은다.
///
/// 경로는 JSON 포인터(`/steps/0/find`) — 개봉·검증이 같은 좌표계를 쓴다.
pub fn redact_plan(
    plan: &mut serde_json::Value,
    path: &str,
    key_hint: &str,
    openings: &mut Vec<(String, String, String)>,
) -> Result<(), String> {
    match plan {
        serde_json::Value::String(s) => {
            if is_structural(key_hint) {
                return Ok(());
            }
            let mut salt = [0u8; 32];
            getrandom::fill(&mut salt).map_err(|e| format!("salt 생성 실패: {e}"))?;
            let salt_hex = {
                let mut hex = String::with_capacity(64);
                for b in salt {
                    use std::fmt::Write as _;
                    let _ = write!(hex, "{b:02x}");
                }
                hex
            };
            let committed = commit(s, &salt_hex);
            openings.push((path.to_string(), s.clone(), salt_hex));
            *plan = serde_json::json!({ "committed": committed });
            Ok(())
        }
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                redact_plan(v, &format!("{path}/{k}"), k, openings)?;
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for (i, v) in items.iter_mut().enumerate() {
                redact_plan(v, &format!("{path}/{i}"), key_hint, openings)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// 가림 plan 에서 JSON 포인터의 `committed` 값을 읽는다.
pub fn committed_at<'a>(plan: &'a serde_json::Value, pointer: &str) -> Option<&'a str> {
    plan.pointer(pointer)?.get("committed")?.as_str()
}

/// 가림 plan 의 committed 잎 수 — unopened 계수용.
pub fn committed_count(plan: &serde_json::Value) -> usize {
    match plan {
        serde_json::Value::Object(map) => {
            if map.len() == 1 && map.contains_key("committed") {
                1
            } else {
                map.values().map(committed_count).sum()
            }
        }
        serde_json::Value::Array(items) => items.iter().map(committed_count).sum(),
        _ => 0,
    }
}
