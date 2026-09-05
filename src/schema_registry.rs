//! 공개 `rhwp` 제품의 스키마 버전 단일 출처.

/// 명령별 `--json` 봉투 최상위 `schemaVersion`.
///
/// 바인딩(python `SUPPORTED_SCHEMA_VERSION` · node `SUPPORTED_SCHEMA_VERSION`)이
/// 정확히 이 값과 대조한다 — 이 값을 올리면 바인딩 상수·호환 계층을 같은
/// 릴리스에서 함께 올려야 한다.
pub const ENVELOPE_SCHEMA_VERSION: &str = "1.0";

/// scaffold 축 — `rhwp scaffold` 입력 명세(`scaffold_schema_v1`)의 판.
/// 소비처는 `src/scaffold/schema.rs` 의 재수출이 유일하다.
pub const SCAFFOLD_SCHEMA_VERSION: &str = "1";

/// #4962 W3가 기존 10k POC usage projection과 대사할 때 유지하는 legacy schema 판.
pub(crate) const LEGACY_FONT_LAYOUT_HABITS_SCHEMA_VERSION: &str = "poc-font-layout-habits-v2";

/// #4966의 정본 폰트 규칙 레지스트리에서 생성하는 backend projection의 schema 판.
pub(crate) const FONT_RULE_PROJECTION_SCHEMA_VERSION: &str = "1.0";

/// 공개 `rhwp` 제품의 릴리스 semver.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
