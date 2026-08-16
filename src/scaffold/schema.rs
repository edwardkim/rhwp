//! `scaffold_schema_v1` serde 모델.
//!
//! `rhwp scaffold` 의 입력 명세다. 에이전트가 **무(無)에서** 유효한 HWPX 문서를
//! 만들기 위해 작성하는 최상위 JSON 이며, `version="1"` 로 고정한다.
//!
//! 설계 원칙(왕복 정직성): 이 스키마는 rhwp 가 파싱해 되읽었을 때 **바이트 그대로**
//! 복원되는 기능만 노출한다 — 문서 제목, 개요 수준 제목(1~7), 본문 문단, 단순 표.
//! 미지 필드는 조용히 버리지 않고 [`Block`] 의 수동 `Deserialize` 로 즉시 거부한다
//! (`src/parser/ingest/schema.rs` 의 `StemBlock` 규약과 정합 — 기계 생성 입력은
//! 관용 파싱의 이득이 없고 실패는 빠를수록 싸다).

use serde::{Deserialize, Serialize};

/// 지원하는 스키마 버전.
pub const SCAFFOLD_SCHEMA_VERSION: &str = "1";

/// 문서 전체 — 에이전트가 작성하는 JSON 최상위.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldSpec {
    /// 스키마 버전 (현재 "1" 만 허용).
    pub version: String,

    /// 문서 제목. 있으면 본문 최상단에 가운데 정렬 제목 문단으로 출력한다.
    #[serde(default)]
    pub title: Option<String>,

    /// 기본 글꼴 이름.
    #[serde(default = "default_font")]
    pub font: String,

    /// 페이지 크기 (mm). 미지정 시 A4(210×297).
    #[serde(default = "default_page_size")]
    pub page_size: PageSize,

    /// 본문 블록 시퀀스 (제목/문단/표).
    #[serde(default)]
    pub blocks: Vec<Block>,
}

fn default_font() -> String {
    "함초롬바탕".to_string()
}

fn default_page_size() -> PageSize {
    PageSize {
        width_mm: 210.0,
        height_mm: 297.0,
    }
}

/// 페이지 크기 (mm 단위).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageSize {
    pub width_mm: f32,
    pub height_mm: f32,
}

/// 본문 블록.
///
/// `Deserialize` 는 수동 구현이다 — serde 의 internally-tagged enum 은
/// `deny_unknown_fields` 를 지원하지 않아 필드 오타·구조 착오(예: paragraph 에 `rows`)가
/// 조용히 무시된다. 전 필드 합집합([`RawBlock`], `deny_unknown_fields`)으로 받은 뒤 type
/// 별 허용 필드를 검증해, 틀린 입력은 무엇이 왜 틀렸는지 힌트가 붙은 오류로 즉시 실패한다.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Block {
    /// 개요 수준 제목(1~7). `export-structure` 가 개요 노드로 인식한다.
    Heading {
        /// 개요 수준 (1=최상위). 1 미만은 1로, 7 초과는 7로 클램프한다.
        level: u8,
        /// 제목 텍스트.
        text: String,
    },
    /// 본문 문단 (평문, 한글 포함).
    Paragraph {
        /// 문단 텍스트.
        text: String,
    },
    /// 단순 표 (행 × 열, 각 셀은 평문 텍스트).
    Table {
        /// 행 목록. 각 행은 셀 텍스트의 목록이다. 행마다 길이가 다르면 최대 열 수에
        /// 맞춰 빈 셀로 채운다(직사각 정규화).
        rows: Vec<Vec<String>>,
    },
}

/// [`Block`] 전 변형의 필드 합집합 — 미지 필드 거부와 type 별 검증의 중간층.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    level: Option<u8>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    rows: Option<Vec<Vec<String>>>,
}

impl<'de> Deserialize<'de> for Block {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let raw = RawBlock::deserialize(deserializer)?;
        let forbid =
            |present: bool, block: &str, field: &str, hint: &str| -> Result<(), D::Error> {
                if present {
                    Err(D::Error::custom(format!(
                        "{block} 블록에 허용되지 않는 필드 '{field}' — {hint}"
                    )))
                } else {
                    Ok(())
                }
            };
        match raw.block_type.as_str() {
            "heading" => {
                forbid(
                    raw.rows.is_some(),
                    "heading",
                    "rows",
                    "표는 type:\"table\" 블록을 쓰세요",
                )?;
                let text = raw
                    .text
                    .ok_or_else(|| D::Error::custom("heading 블록에 'text' 필드가 필요합니다"))?;
                let level = raw.level.ok_or_else(|| {
                    D::Error::custom("heading 블록에 'level' 필드가 필요합니다 (1~7)")
                })?;
                Ok(Block::Heading { level, text })
            }
            "paragraph" => {
                forbid(
                    raw.level.is_some(),
                    "paragraph",
                    "level",
                    "level 은 heading 블록 전용입니다",
                )?;
                forbid(
                    raw.rows.is_some(),
                    "paragraph",
                    "rows",
                    "표는 type:\"table\" 블록을 쓰세요",
                )?;
                let text = raw
                    .text
                    .ok_or_else(|| D::Error::custom("paragraph 블록에 'text' 필드가 필요합니다"))?;
                Ok(Block::Paragraph { text })
            }
            "table" => {
                forbid(
                    raw.level.is_some(),
                    "table",
                    "level",
                    "level 은 heading 블록 전용입니다",
                )?;
                forbid(
                    raw.text.is_some(),
                    "table",
                    "text",
                    "셀 내용은 'rows' 안에 넣으세요",
                )?;
                let rows = raw
                    .rows
                    .ok_or_else(|| D::Error::custom("table 블록에 'rows' 필드가 필요합니다"))?;
                Ok(Block::Table { rows })
            }
            other => Err(D::Error::custom(format!(
                "알 수 없는 블록 type '{other}' (지원: heading|paragraph|table)"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> &'static str {
        r#"{
            "version": "1",
            "title": "2026년 1분기 실적 보고서",
            "font": "함초롬바탕",
            "blocks": [
                {"type": "heading", "level": 1, "text": "개요"},
                {"type": "paragraph", "text": "본 문서는 자동 생성되었습니다."},
                {"type": "heading", "level": 2, "text": "매출 현황"},
                {"type": "table", "rows": [["항목", "값"], ["매출", "100"]]}
            ]
        }"#
    }

    #[test]
    fn parse_sample() {
        let spec: ScaffoldSpec = serde_json::from_str(sample_json()).unwrap();
        assert_eq!(spec.version, "1");
        assert_eq!(spec.title.as_deref(), Some("2026년 1분기 실적 보고서"));
        assert_eq!(spec.blocks.len(), 4);
        assert!(matches!(spec.blocks[0], Block::Heading { level: 1, .. }));
        assert!(matches!(spec.blocks[1], Block::Paragraph { .. }));
        assert!(matches!(spec.blocks[3], Block::Table { .. }));
    }

    #[test]
    fn defaults_apply() {
        let spec: ScaffoldSpec = serde_json::from_str(r#"{"version":"1","blocks":[]}"#).unwrap();
        assert_eq!(spec.font, "함초롬바탕");
        assert_eq!(spec.page_size.width_mm, 210.0);
        assert!(spec.title.is_none());
    }

    #[test]
    fn paragraph_with_rows_is_rejected() {
        let e = serde_json::from_str::<Block>(r#"{"type":"paragraph","text":"a","rows":[["x"]]}"#)
            .unwrap_err()
            .to_string();
        assert!(
            e.contains("paragraph 블록에 허용되지 않는 필드 'rows'"),
            "{e}"
        );
    }

    #[test]
    fn heading_without_level_is_rejected() {
        let e = serde_json::from_str::<Block>(r#"{"type":"heading","text":"제목"}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("'level'"), "{e}");
    }

    #[test]
    fn unknown_block_type_is_rejected() {
        let e = serde_json::from_str::<Block>(r#"{"type":"image","text":"x"}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("알 수 없는 블록 type 'image'"), "{e}");
    }

    #[test]
    fn unknown_field_is_rejected() {
        let e = serde_json::from_str::<Block>(r#"{"type":"paragraph","text":"a","bold":true}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("bold"), "{e}");
    }

    #[test]
    fn top_level_typo_is_rejected() {
        let e = serde_json::from_str::<ScaffoldSpec>(r#"{"version":"1","fnt":"바탕","blocks":[]}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("fnt"), "{e}");
    }
}
