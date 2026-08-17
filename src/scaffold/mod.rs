//! `rhwp scaffold` — 스펙(JSON) → 유효 HWPX 문서 생성.
//!
//! rhwp 의 읽기/편집 축과 직교하는 **생성(authoring)** 축이다. 에이전트가 문서를
//! *소비*하는 데서 나아가 구조화된 명세로부터 문서를 *만든다*. 입력은 문서가 아니라
//! 호출자(사용자/에이전트)가 작성한 계획서이므로 출력 봉투는 신뢰 불가 표지가 붙지
//! 않는다(`src/provenance.rs` 의 `scaffold` 항목 참조).
//!
//! 지원 요소는 **왕복 검증으로 통과한 것만** 노출한다 — 문서 제목, 개요 수준 제목(1~7),
//! 본문 문단(한글 포함), 단순 표(행×열 텍스트 셀). 각 요소는 `serialize_hwpx` 로 쓴 뒤
//! `parse_hwpx` 로 되읽어 내용이 그대로 복원됨을 이 모듈의 테스트가 증명한다.

pub mod builder;
pub mod schema;

pub use builder::build_scaffold;
pub use schema::{Block, PageSize, ScaffoldSpec, SCAFFOLD_SCHEMA_VERSION};

use crate::error::HwpError;

/// JSON 바이트로부터 [`ScaffoldSpec`]을 파싱한다.
pub fn parse_scaffold_bytes(bytes: &[u8]) -> Result<ScaffoldSpec, HwpError> {
    let spec: ScaffoldSpec = serde_json::from_slice(bytes)
        .map_err(|e| HwpError::InvalidFile(format!("scaffold JSON 파싱 실패: {e}")))?;
    validate_version(&spec)?;
    Ok(spec)
}

/// 문자열로부터 [`ScaffoldSpec`]을 파싱한다.
pub fn parse_scaffold_str(s: &str) -> Result<ScaffoldSpec, HwpError> {
    let spec: ScaffoldSpec = serde_json::from_str(s)
        .map_err(|e| HwpError::InvalidFile(format!("scaffold JSON 파싱 실패: {e}")))?;
    validate_version(&spec)?;
    Ok(spec)
}

fn validate_version(spec: &ScaffoldSpec) -> Result<(), HwpError> {
    if spec.version != SCAFFOLD_SCHEMA_VERSION {
        return Err(HwpError::InvalidFile(format!(
            "지원하지 않는 scaffold 스키마 버전 '{}' (지원: \"{}\")",
            spec.version, SCAFFOLD_SCHEMA_VERSION
        )));
    }
    Ok(())
}
