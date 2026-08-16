//! 공개 `rhwp` 제품의 스키마 레지스트리 파사드.
//!
//! 계약 축 정의와 JSON 조립은 `rhwp-contracts`가 소유하지만, 공개
//! `crateVersion`은 루트 제품 크레이트의 버전이어야 한다. 내부 크레이트의
//! `CARGO_PKG_VERSION`이 봉투로 새지 않도록 이 경계에서 제품 버전을 주입한다.

pub use rhwp_contracts::schema_registry::{
    CAPABILITIES_SCHEMA_VERSION, ENVELOPE_SCHEMA_VERSION, IR_SCHEMA_VERSION, PLAN_SCHEMA_VERSION,
    SIGNING_SCHEMA_VERSION,
};

/// 공개 `rhwp` 제품의 릴리스 semver.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// 공개 제품 버전을 포함한 기계 소비용 스키마 레지스트리.
pub fn registry_value() -> serde_json::Value {
    rhwp_contracts::schema_registry::registry_value_with_crate_version(crate_version())
}
