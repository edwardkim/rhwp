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

/// 지원하는 스키마 버전 — 정의는 버전 단일 출처(#4329)인
/// `schema_registry` 에 있고 여기서는 재수출만 한다.
pub use crate::schema_registry::SCAFFOLD_SCHEMA_VERSION;

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

/// 표 셀 문단의 가로 정렬.
///
/// 이름과 값은 편집 경로 `rhwp edit insert-table --alignments left,center,right`
/// (`src/cli/commands/edit/tables/insert.rs`)와 같은 낱말을 쓴다. 그쪽이 지정하지 않을 때
/// 커서 문단의 정렬을 물려받듯, 여기서 생략하면 본문 문단과 같은 `justify` 다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellAlign {
    /// 양쪽 정렬 — scaffold 본문 문단과 같은 기본값.
    #[default]
    Justify,
    Left,
    Center,
    Right,
}

impl CellAlign {
    /// 조판 IR 의 정렬로 옮긴다.
    pub fn to_alignment(self) -> crate::model::style::Alignment {
        use crate::model::style::Alignment;
        match self {
            CellAlign::Justify => Alignment::Justify,
            CellAlign::Left => Alignment::Left,
            CellAlign::Center => Alignment::Center,
            CellAlign::Right => Alignment::Right,
        }
    }
}

/// `cell_align` 값 — 표 전체 하나 또는 열 단위 목록.
///
/// 열 단위 목록은 편집 경로의 `column_alignments` 와 같은 축이다. 길이는 그 표의 열 수와
/// 같아야 하며, 어긋나면 [`crate::scaffold::parse_scaffold_str`] 이 즉시 거부한다
/// (기계 생성 입력은 조용한 보정보다 빠른 실패가 싸다).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum CellAlignSpec {
    /// 표 안 모든 셀에 같은 정렬.
    All(CellAlign),
    /// 열 단위 정렬 (0번째가 첫 열).
    PerColumn(Vec<CellAlign>),
}

/// 낱말 하나를 [`CellAlign`] 으로.
fn parse_cell_align(word: &str) -> Result<CellAlign, String> {
    match word.trim() {
        "justify" => Ok(CellAlign::Justify),
        "left" => Ok(CellAlign::Left),
        "center" => Ok(CellAlign::Center),
        "right" => Ok(CellAlign::Right),
        other => Err(format!(
            "알 수 없는 cell_align 값 '{other}' (지원: justify|left|center|right)"
        )),
    }
}

/// `untagged` 자동 구현은 실패를 *"data did not match any variant"* 로만 알려 준다.
/// 이 스키마의 규약(무엇이 왜 틀렸는지 힌트를 붙여 즉시 실패)에 맞춰 직접 읽는다.
impl<'de> Deserialize<'de> for CellAlignSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(word) => parse_cell_align(&word)
                .map(CellAlignSpec::All)
                .map_err(D::Error::custom),
            serde_json::Value::Array(items) => items
                .iter()
                .map(|item| {
                    item.as_str()
                        .ok_or_else(|| {
                            format!("cell_align 목록의 항목은 문자열이어야 합니다: {item}")
                        })
                        .and_then(parse_cell_align)
                })
                .collect::<Result<Vec<_>, String>>()
                .map(CellAlignSpec::PerColumn)
                .map_err(D::Error::custom),
            other => Err(D::Error::custom(format!(
                "cell_align 은 문자열(justify|left|center|right) 또는 그 목록이어야 합니다: {other}"
            ))),
        }
    }
}

impl CellAlignSpec {
    /// `col` 번째 열의 정렬.
    pub fn for_column(&self, col: usize) -> CellAlign {
        match self {
            CellAlignSpec::All(a) => *a,
            CellAlignSpec::PerColumn(list) => list.get(col).copied().unwrap_or_default(),
        }
    }
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
        /// [#7232] 셀 문단의 가로 정렬. 생략하면 `justify`(종전 동작).
        /// 좁은 열에 긴 영문 토큰이 오면 한/글이 양쪽 정렬을 맞추려 글자 사이를 벌리므로,
        /// 코드명·경로·SQL 이 많은 표는 `left` 를 지정한다.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cell_align: Option<CellAlignSpec>,
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
    #[serde(default)]
    cell_align: Option<CellAlignSpec>,
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
                forbid(
                    raw.cell_align.is_some(),
                    "heading",
                    "cell_align",
                    "cell_align 은 table 블록 전용입니다",
                )?;
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
                forbid(
                    raw.cell_align.is_some(),
                    "paragraph",
                    "cell_align",
                    "cell_align 은 table 블록 전용입니다",
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
                Ok(Block::Table {
                    rows,
                    cell_align: raw.cell_align,
                })
            }
            other => Err(D::Error::custom(format!(
                "알 수 없는 블록 type '{other}' (지원: heading|paragraph|table)"
            ))),
        }
    }
}
