//! `rhwp scaffold` — 스펙(JSON) → 유효 HWPX 문서 생성.
//!
//! rhwp 의 읽기/편집 축과 직교하는 **생성(authoring)** 축이다. 에이전트가 문서를
//! *소비*하는 데서 나아가 구조화된 명세로부터 문서를 *만든다*. 입력은 문서가 아니라
//! 호출자(사용자/에이전트)가 작성한 계획서이므로 출력 봉투는 신뢰 불가 표지가 붙지
//! 않는다(`src/provenance.rs` 의 `scaffold` 항목 참조).
//!
//! 지원 요소는 **왕복 검증으로 통과한 것만** 노출한다 — 문서 제목, 개요 수준 제목(1~7),
//! 개요 번호 모양, 본문 문단(한글 포함), 단순 표(행×열 텍스트 셀). 각 요소는 `serialize_hwpx` 로 쓴 뒤
//! `parse_hwpx` 로 되읽어 내용이 그대로 복원됨을 이 모듈의 테스트가 증명한다.

pub mod builder;
pub mod schema;

pub use builder::build_scaffold;
pub use schema::{
    Block, CellAlign, CellAlignSpec, OutlineLevel, OutlineNumFormat, PageSize, ScaffoldSpec,
    OUTLINE_DEFAULTS, SCAFFOLD_SCHEMA_VERSION,
};

use crate::error::HwpError;

/// JSON 바이트로부터 [`ScaffoldSpec`]을 파싱한다.
pub fn parse_scaffold_bytes(bytes: &[u8]) -> Result<ScaffoldSpec, HwpError> {
    let spec: ScaffoldSpec = serde_json::from_slice(bytes)
        .map_err(|e| HwpError::InvalidFile(format!("scaffold JSON 파싱 실패: {e}")))?;
    validate_version(&spec)?;
    validate_blocks(&spec)?;
    validate_outline_numbering(&spec)?;
    Ok(spec)
}

/// 문자열로부터 [`ScaffoldSpec`]을 파싱한다.
pub fn parse_scaffold_str(s: &str) -> Result<ScaffoldSpec, HwpError> {
    let spec: ScaffoldSpec = serde_json::from_str(s)
        .map_err(|e| HwpError::InvalidFile(format!("scaffold JSON 파싱 실패: {e}")))?;
    validate_version(&spec)?;
    validate_blocks(&spec)?;
    validate_outline_numbering(&spec)?;
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

/// 개요 번호 형식 문자열의 최대 길이(글자). 한/글 문단 번호 대화상자가 받는 길이 안쪽이다.
const OUTLINE_TEXT_MAX_CHARS: usize = 32;

/// `outline_numbering` 은 1~7개이고, 각 `text` 는 비어 있지 않으며 `^` 뒤에는 자기 수준
/// 이하의 숫자(`^1`~`^L`)만 온다. 잘못 쓰면 한/글이 번호를 그리지 못하거나 엉뚱한 수준을
/// 세므로 즉시 거부한다.
fn validate_outline_numbering(spec: &ScaffoldSpec) -> Result<(), HwpError> {
    let Some(levels) = &spec.outline_numbering else {
        return Ok(());
    };
    if levels.is_empty() || levels.len() > 7 {
        return Err(HwpError::InvalidFile(format!(
            "outline_numbering 은 수준 1 부터 1~7개여야 합니다 (받은 개수 {})",
            levels.len()
        )));
    }
    for (i, lv) in levels.iter().enumerate() {
        let level = i + 1;
        let where_ = format!("outline_numbering[{i}] (수준 {level})");
        if lv.text.trim().is_empty() {
            return Err(HwpError::InvalidFile(format!(
                "{where_}: text 가 비었습니다 (예: \"^{level}.\")"
            )));
        }
        if lv.text.chars().count() > OUTLINE_TEXT_MAX_CHARS {
            return Err(HwpError::InvalidFile(format!(
                "{where_}: text 는 {OUTLINE_TEXT_MAX_CHARS}자 이하여야 합니다"
            )));
        }
        let mut chars = lv.text.chars();
        while let Some(c) = chars.next() {
            if c != '^' {
                continue;
            }
            match chars.next().and_then(|d| d.to_digit(10)) {
                Some(d) if (1..=level as u32).contains(&d) => {}
                _ => {
                    return Err(HwpError::InvalidFile(format!(
                        "{where_}: '^' 뒤에는 1~{level} 의 수준 번호가 와야 합니다 ({:?})",
                        lv.text
                    )))
                }
            }
        }
    }
    Ok(())
}

/// [#7232] 열 단위 `cell_align` 의 길이는 그 표의 열 수와 같아야 한다.
///
/// 열 수는 행 중 가장 긴 것이다(빌더의 직사각 정규화와 같은 규칙). 길이가 어긋나면
/// 어느 열이 무슨 정렬을 받는지가 호출자의 의도와 조용히 달라지므로 즉시 거부한다.
fn validate_blocks(spec: &ScaffoldSpec) -> Result<(), HwpError> {
    for (i, block) in spec.blocks.iter().enumerate() {
        let Block::Table { rows, cell_align } = block else {
            continue;
        };
        let Some(CellAlignSpec::PerColumn(list)) = cell_align else {
            continue;
        };
        let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0).max(1);
        if list.len() != col_count {
            return Err(HwpError::InvalidFile(format!(
                "blocks[{i}] (table): cell_align 목록 길이 {} 가 열 수 {col_count} 와 다릅니다 \
                 (표 전체에 같은 정렬을 주려면 목록 대신 값 하나를 쓰세요)",
                list.len()
            )));
        }
    }
    Ok(())
}
