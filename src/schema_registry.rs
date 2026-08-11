//! [#4329] 스키마 버전 레지스트리 — 전 버전 축의 **단일 출처**(R83)이자 버전
//! 정책(R67)의 단일 적용점.
//!
//! ## 왜 한 곳인가
//!
//! 이 저장소의 계약 버전은 네 축이다 — 봉투(명령별 `schemaVersion`) · IR
//! (`irSchemaVersion`) · capabilities(`capabilitiesSchemaVersion`) · 계획
//! (`planSchemaVersion`) — 그리고 릴리스 semver(`CARGO_PKG_VERSION`)가 그 위를
//! 덮는다. #4329 이전에는 이 값들이 각자 파일의 상수와 리터럴로 흩어져 있었다
//! (봉투 리터럴만 8개 파일 ~67사이트). 흩어진 값은 두 가지를 막는다: ① 버전
//! 정책을 바꿀 때 몇 곳을 고쳐야 하는지 아무도 모른다, ② 외부 소비자가 "무엇이
//! 언제 올랐는가"를 기계로 대조할 자리가 없다. ②의 실측 결과가 소비자의 스테일
//! 버전 고정이다(#4327 §3 — 외부 스킬 저장소가 v0.7.3 스냅샷의 표면 인식으로
//! 편집 축을 자체 CLI 로 갈라 나갔다).
//!
//! 여기 선언된 값이 곧 계약이고, 다른 곳의 버전 문자열은 전부 이 모듈의 재수출
//! 또는 참조다. 산개 재발은 `tests/schema_registry_contract.rs` 의 소스 스캔
//! 가드가 잡는다. 소비자를 향한 노출은 `capabilities` 봉투의 `schemaRegistry`
//! (`registry_value()`)다 — 한 번의 호출로 전 축을 대조한다(#4327 U2).
//!
//! ## 진화 규약 (요약)
//!
//! 네 축 공통: **필드 추가 = minor, 의미 변경·삭제 = major.** major 는 분기 회고
//! 승인 없이 금지 — 각 축 문서(ir_schema·capabilities_schema·plan_schema 모듈
//! 머리말)에 나뉘어 있던 같은 문장을 이 모듈이 승계한다. semver 와의 연동 규칙
//! 전문은 mydocs/tech/agent_runtime/version_policy.md 가 canonical 이다.

use serde_json::{json, Value};

/// 명령별 `--json` 봉투 최상위 `schemaVersion`.
///
/// 바인딩(python `SUPPORTED_SCHEMA_VERSION` · node `SUPPORTED_SCHEMA_VERSION`)이
/// 정확히 이 값과 대조한다 — 이 값을 올리면 바인딩 상수·호환 계층을 같은
/// 릴리스에서 함께 올려야 한다(version_policy.md §3).
pub const ENVELOPE_SCHEMA_VERSION: &str = "1.0";

/// 공개 IR 스키마 버전 — `export-ir-schema` 봉투의 `irSchemaVersion`.
pub const IR_SCHEMA_VERSION: &str = "1.0";

/// capabilities 스키마 버전 — `export-capabilities-schema` 봉투의
/// `capabilitiesSchemaVersion`. 봉투 축과 **분리**된 전역 버전이다.
///
/// - 1.1: 세션 도구 선언 확장 계열.
/// - 1.2: McpTool 에 annotations(MCP 표준 ToolAnnotations) 추가 (#4220 T3, minor).
/// - 1.3: capabilities 봉투에 `schemaRegistry`(이 레지스트리의 자기서술) 추가
///   (#4329 U2, minor).
pub const CAPABILITIES_SCHEMA_VERSION: &str = "1.3";

/// 계획서 문법 버전 — `export-plan-schema` 봉투의 `planSchemaVersion`.
///
/// - 1.1: 계획서 루트에 선택 `preconditions.inputSha256`(CAS — 실행 전 입력 해시
///   전제, #4378 R22) 추가 (minor).
pub const PLAN_SCHEMA_VERSION: &str = "1.1";

/// [#4509] 서명 축 — 키 파일(ed25519Key)·분리 서명(capsuleSignature)·키
/// 등록부(keyring) **파일 형식**의 판. 봉투 축과 별개로 도는 교환 파일
/// 버전이라 capabilities 의 schemaRegistry 축 집합에는 싣지 않는다(그 집합은
/// 봉투 계약 축으로 고정 — 해당 가드 참조).
pub const SIGNING_SCHEMA_VERSION: &str = "1.0";

/// 릴리스 semver — Cargo.toml 의 단일 출처를 컴파일 시점에 읽는다.
/// `rhwp::version()` 과 같은 원천이므로 두 값은 구조적으로 같다.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// 레지스트리의 기계 소비용 자기서술 — `capabilities` 봉투의 `schemaRegistry` 로
/// 노출된다(#4327 U2, 외부 소비자 대사 채널).
///
/// 소비자 계약: `axes[].axis` 는 {"envelope","ir","capabilities","plan"} 고정
/// 집합이고(추가 = capabilities minor), `axes[].version` 은 위 상수들과 항상
/// 일치한다(`tests/schema_registry_contract.rs` 가 실행 봉투로 고정). 정책 전문은
/// `policy` 가 가리키는 저장소 경로에서 읽는다.
pub fn registry_value() -> Value {
    const BUMP_RULE: &str =
        "필드 추가 = minor, 의미 변경·삭제 = major (major 는 분기 회고 승인 필요)";
    json!({
        "crateVersion": crate_version(),
        "axes": [
            {
                "axis": "envelope",
                "version": ENVELOPE_SCHEMA_VERSION,
                "surface": "모든 --json 봉투 최상위 schemaVersion",
                "bump": BUMP_RULE,
            },
            {
                "axis": "ir",
                "version": IR_SCHEMA_VERSION,
                "surface": "export-ir-schema 봉투의 irSchemaVersion",
                "bump": BUMP_RULE,
            },
            {
                "axis": "capabilities",
                "version": CAPABILITIES_SCHEMA_VERSION,
                "surface": "export-capabilities-schema 봉투의 capabilitiesSchemaVersion",
                "bump": BUMP_RULE,
            },
            {
                "axis": "plan",
                "version": PLAN_SCHEMA_VERSION,
                "surface": "export-plan-schema 봉투의 planSchemaVersion (run 계획서 문법)",
                "bump": BUMP_RULE,
            },
        ],
        "policy": "mydocs/tech/agent_runtime/version_policy.md",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 축 집합과 crateVersion 이 자기서술과 일치한다 — 실행 봉투 쪽 고정은
    /// tests/schema_registry_contract.rs, 여기서는 값 조립 자체를 고정한다.
    #[test]
    fn registry_value_carries_all_axes_and_crate_version() {
        let v = registry_value();
        assert_eq!(v["crateVersion"], crate_version());
        let axes: Vec<&str> = v["axes"]
            .as_array()
            .expect("axes 배열")
            .iter()
            .map(|a| a["axis"].as_str().expect("axis 문자열"))
            .collect();
        assert_eq!(axes, ["envelope", "ir", "capabilities", "plan"]);
        for a in v["axes"].as_array().unwrap() {
            assert!(a["version"].as_str().is_some_and(|s| !s.is_empty()));
            assert!(a["surface"].as_str().is_some_and(|s| !s.is_empty()));
            assert!(a["bump"].as_str().is_some_and(|s| !s.is_empty()));
        }
    }

    /// 정책 문서 경로가 실물과 이어져 있다 — 문서가 이사하면 여기서 끊긴 것이
    /// 드러나야 소비자에게 죽은 경로를 광고하지 않는다.
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn policy_path_points_to_existing_document() {
        let policy = registry_value()["policy"].as_str().unwrap().to_string();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(&policy);
        assert!(path.exists(), "정책 문서가 없다: {policy}");
    }
}
