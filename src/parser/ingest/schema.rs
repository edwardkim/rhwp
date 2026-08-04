//! `ingest_schema_v1.json` serde 모델.
//!
//! Claude Code Skill ↔ rhwp Rust 본체 인터페이스. version="1" 고정.

use serde::{Deserialize, Serialize};

/// 문서 전체 — Skill이 작성하는 JSON 최상위.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngestDocument {
    /// 스키마 버전 (현재 "1" 만 허용).
    pub version: String,

    /// 페이지 크기 (mm).
    #[serde(default = "default_page_size")]
    pub page_size: PageSize,

    /// 기본 폰트 이름. Skill이 시험지에서 인식한 폰트 또는 fallback.
    #[serde(default = "default_font")]
    pub default_font: String,

    /// 반복 머리말 텍스트.
    #[serde(default)]
    pub header_text: Option<String>,

    /// 반복 꼬리말 텍스트.
    #[serde(default)]
    pub footer_text: Option<String>,

    /// 시험지 형식 라벨(예: 홀수형/짝수형).
    #[serde(default)]
    pub form_label: Option<String>,

    /// 여러 문제가 공유하는 지문 목록.
    #[serde(default)]
    pub passages: Vec<Passage>,

    /// 시험문제 목록.
    pub questions: Vec<Question>,
}

fn default_page_size() -> PageSize {
    PageSize {
        width_mm: 210.0,
        height_mm: 297.0,
    }
}

fn default_font() -> String {
    "함초롬바탕".to_string()
}

/// 페이지 크기 (mm 단위).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageSize {
    pub width_mm: f32,
    pub height_mm: f32,
}

/// 여러 문제가 공유하는 지문.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Passage {
    /// 지문 ID. `Question.passage_ref` 에서 참조한다.
    pub id: String,
    /// 공유 지문 블록.
    #[serde(default)]
    pub blocks: Vec<StemBlock>,
}

/// 한 문제 단위.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    /// 문제 번호 (보통 1~30).
    pub number: u32,

    /// 지문 — 첫 줄 또는 전체 (stem_blocks가 있으면 stem_blocks가 우선, stem은 fallback).
    pub stem: String,

    /// 공유 지문 ID 참조. 같은 passage는 builder가 첫 참조 위치에 한 번만 출력한다.
    #[serde(default)]
    pub passage_ref: Option<String>,

    /// 지문을 텍스트/이미지 블록 시퀀스로 표현 (선택사항).
    /// 비어있으면 stem 한 줄로 처리.
    #[serde(default)]
    pub stem_blocks: Vec<StemBlock>,

    /// 선택지 5개 (또는 그 이하).
    pub choices: Vec<Choice>,

    /// 이 문제와 연관된 미디어(이미지) 목록.
    #[serde(default)]
    pub media: Vec<Media>,

    /// `true`면 빌더가 첫 stem 텍스트 앞에 `{number}. `를 자동 prepend.
    /// Skill이 stem_blocks 첫 텍스트에 명시적으로 번호 또는 그룹 지시문(`[1~3] …`)을
    /// 작성한 경우 `false`로 설정해 중복 prefix를 회피한다.
    /// 미지정 시 기본 `true`.
    #[serde(default = "default_auto_number")]
    pub auto_number: bool,
}

fn default_auto_number() -> bool {
    true
}

/// 지문 내 블록 (텍스트 또는 이미지).
///
/// [#3358] `Deserialize` 는 수동 구현이다 — serde 의 internally-tagged enum 은
/// `deny_unknown_fields` 를 지원하지 않아, 필드명 오타·구조 착오(예: boxed 에 `text`)가
/// 조용히 무시되고 **내용이 소리 없이 유실**됐다. 전 필드 합집합([`RawStemBlock`],
/// `deny_unknown_fields`)으로 받은 뒤 type 별 허용 필드를 검증해, 틀린 입력은
/// 무엇이 왜 틀렸는지 힌트가 붙은 오류로 즉시 실패한다 (ingest 는 기계 생성 입력이라
/// 관용 파싱의 이득이 없고, 실패는 빠를수록 싸다).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StemBlock {
    /// 텍스트 단락.
    Text { text: String },
    /// 이미지 참조 (자르기/배치는 media 항목에서 결정).
    Image {
        /// `media[].id` 와 일치해야 한다.
        #[serde(rename = "ref")]
        ref_: String,
        /// 위치 힌트 (선택).
        #[serde(default)]
        placement: Placement,
    },
    /// 테두리/배경이 있는 보기 박스.
    Boxed {
        /// 박스 제목. 예: `<보기>`.
        #[serde(default)]
        title: Option<String>,
        /// 박스 내부 블록.
        #[serde(default)]
        blocks: Vec<StemBlock>,
    },
}

/// [#3358] StemBlock 전 변형의 필드 합집합 — 미지 필드 거부와 type 별 검증의 중간층.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStemBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(rename = "ref", default)]
    ref_: Option<String>,
    #[serde(default)]
    placement: Option<Placement>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    blocks: Option<Vec<StemBlock>>,
}

impl<'de> Deserialize<'de> for StemBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let raw = RawStemBlock::deserialize(deserializer)?;
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
            "text" => {
                forbid(
                    raw.ref_.is_some(),
                    "text",
                    "ref",
                    "이미지는 type:\"image\" 블록을 쓰세요",
                )?;
                forbid(
                    raw.placement.is_some(),
                    "text",
                    "placement",
                    "placement 는 image 블록 전용입니다",
                )?;
                forbid(
                    raw.title.is_some(),
                    "text",
                    "title",
                    "title 은 boxed 블록 전용입니다",
                )?;
                forbid(
                    raw.blocks.is_some(),
                    "text",
                    "blocks",
                    "blocks 는 boxed 블록 전용입니다",
                )?;
                let text = raw
                    .text
                    .ok_or_else(|| D::Error::custom("text 블록에 'text' 필드가 필요합니다"))?;
                Ok(StemBlock::Text { text })
            }
            "image" => {
                forbid(
                    raw.text.is_some(),
                    "image",
                    "text",
                    "본문은 type:\"text\" 블록으로 넣으세요",
                )?;
                forbid(
                    raw.title.is_some(),
                    "image",
                    "title",
                    "title 은 boxed 블록 전용입니다",
                )?;
                forbid(
                    raw.blocks.is_some(),
                    "image",
                    "blocks",
                    "blocks 는 boxed 블록 전용입니다",
                )?;
                let ref_ = raw.ref_.ok_or_else(|| {
                    D::Error::custom("image 블록에 'ref' 필드가 필요합니다 (media[].id 참조)")
                })?;
                Ok(StemBlock::Image {
                    ref_,
                    placement: raw.placement.unwrap_or_default(),
                })
            }
            "boxed" => {
                forbid(
                    raw.text.is_some(),
                    "boxed",
                    "text",
                    "박스 내용은 'blocks' 배열의 text 블록으로 넣으세요",
                )?;
                forbid(
                    raw.ref_.is_some(),
                    "boxed",
                    "ref",
                    "이미지는 blocks 안의 image 블록으로 넣으세요",
                )?;
                forbid(
                    raw.placement.is_some(),
                    "boxed",
                    "placement",
                    "placement 는 image 블록 전용입니다",
                )?;
                Ok(StemBlock::Boxed {
                    title: raw.title,
                    blocks: raw.blocks.unwrap_or_default(),
                })
            }
            other => Err(D::Error::custom(format!(
                "알 수 없는 블록 type '{other}' (지원: text|image|boxed)"
            ))),
        }
    }
}

/// 선택지 항목.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    /// 표시 라벨 (예: "①" U+2460).
    pub label: String,
    /// 본문 텍스트 (label 없이).
    pub text: String,
}

/// 이미지 메타.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Media {
    /// 미디어 ID (예: "img/q1_passage.png").
    /// `--media-dir` 기준 상대 경로로 해석한다.
    pub id: String,
    /// 원본 픽셀 폭.
    pub natural_w: u32,
    /// 원본 픽셀 높이.
    pub natural_h: u32,
    /// 출력 폭 (mm). 미지정 시 본문폭의 70%.
    #[serde(default)]
    pub target_w_mm: Option<f32>,
    /// 배치 위치.
    #[serde(default)]
    pub placement: Placement,
}

/// 이미지 배치 위치 결정.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Placement {
    /// 지문과 선택지 사이.
    #[default]
    Between,
    /// 지문 위.
    Above,
    /// 지문 아래(선택지 다음).
    Below,
    /// 지문 텍스트 내 인라인 (글자처럼 흐름).
    Inline,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_minimal_json() -> &'static str {
        r#"{
            "version": "1",
            "page_size": {"width_mm": 210.0, "height_mm": 297.0},
            "default_font": "함초롬바탕",
            "header_text": "국어 영역",
            "footer_text": "1/20",
            "form_label": "홀수형",
            "passages": [
                {
                    "id": "p1-2",
                    "blocks": [
                        {"type": "text", "text": "[1~2] 다음 글을 읽고 물음에 답하시오."},
                        {"type": "text", "text": "공유 지문입니다."}
                    ]
                }
            ],
            "questions": [
                {
                    "number": 1,
                    "passage_ref": "p1-2",
                    "stem": "다음 글의 주제로 가장 적절한 것은?",
                    "stem_blocks": [
                        {"type": "text", "text": "다음 글의 주제로 가장 적절한 것은?"},
                        {
                            "type": "boxed",
                            "title": "<보기>",
                            "blocks": [
                                {"type": "text", "text": "보기 본문"}
                            ]
                        }
                    ],
                    "choices": [
                        {"label": "①", "text": "환경 보호"},
                        {"label": "②", "text": "도시 생활"},
                        {"label": "③", "text": "전통 음식"},
                        {"label": "④", "text": "기술 발전"},
                        {"label": "⑤", "text": "진로 탐색"}
                    ],
                    "media": []
                }
            ]
        }"#
    }

    #[test]
    fn test_parse_minimal() {
        let doc: IngestDocument = serde_json::from_str(sample_minimal_json()).unwrap();
        assert_eq!(doc.version, "1");
        assert_eq!(doc.header_text.as_deref(), Some("국어 영역"));
        assert_eq!(doc.footer_text.as_deref(), Some("1/20"));
        assert_eq!(doc.form_label.as_deref(), Some("홀수형"));
        assert_eq!(doc.passages.len(), 1);
        assert_eq!(doc.passages[0].id, "p1-2");
        assert_eq!(doc.questions.len(), 1);
        assert_eq!(doc.questions[0].number, 1);
        assert_eq!(doc.questions[0].passage_ref.as_deref(), Some("p1-2"));
        assert_eq!(doc.questions[0].choices.len(), 5);
        assert_eq!(doc.questions[0].choices[0].label, "①");
        assert!(matches!(
            &doc.questions[0].stem_blocks[1],
            StemBlock::Boxed { .. }
        ));
    }

    #[test]
    fn test_parse_legacy_minimal_defaults() {
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 1,
                "stem": "기존 입력",
                "choices": [{"label": "①", "text": "A"}]
            }]
        }"#;
        let doc: IngestDocument = serde_json::from_str(json).unwrap();

        assert!(doc.header_text.is_none());
        assert!(doc.footer_text.is_none());
        assert!(doc.form_label.is_none());
        assert!(doc.passages.is_empty());
        assert!(doc.questions[0].passage_ref.is_none());
        assert!(doc.questions[0].stem_blocks.is_empty());
        assert!(doc.questions[0].media.is_empty());
        assert!(doc.questions[0].auto_number);
    }

    #[test]
    fn test_placement_default() {
        assert_eq!(Placement::default(), Placement::Between);
    }

    #[test]
    fn test_roundtrip() {
        let doc: IngestDocument = serde_json::from_str(sample_minimal_json()).unwrap();
        let s = serde_json::to_string(&doc).unwrap();
        let doc2: IngestDocument = serde_json::from_str(&s).unwrap();
        assert_eq!(doc.questions.len(), doc2.questions.len());
        assert_eq!(doc.questions[0].stem, doc2.questions[0].stem);
        assert_eq!(doc.passages.len(), doc2.passages.len());
    }

    #[test]
    fn test_placement_serde() {
        let p_str = serde_json::to_string(&Placement::Between).unwrap();
        assert_eq!(p_str, r#""between""#);
        let p: Placement = serde_json::from_str(r#""inline""#).unwrap();
        assert_eq!(p, Placement::Inline);
    }

    // ── [#3358] 미지 필드 거부 — 침묵 유실 대신 즉시 실패 ────────────────────

    /// 관찰된 실제 사고 형태: boxed 에 text 를 주면 종전에는 빈 박스가 조용히 생겼다.
    #[test]
    fn boxed_with_text_is_rejected_with_hint() {
        let e = serde_json::from_str::<StemBlock>(r#"{"type":"boxed","text":"소속: 성명:"}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("boxed 블록에 허용되지 않는 필드 'text'"), "{e}");
        assert!(e.contains("blocks"), "힌트가 있어야 합니다: {e}");
    }

    #[test]
    fn unknown_block_field_is_rejected() {
        let e = serde_json::from_str::<StemBlock>(r#"{"type":"text","text":"a","bold":true}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("bold"), "{e}");
    }

    #[test]
    fn unknown_block_type_is_rejected() {
        let e = serde_json::from_str::<StemBlock>(r#"{"type":"table"}"#)
            .unwrap_err()
            .to_string();
        assert!(e.contains("알 수 없는 블록 type 'table'"), "{e}");
    }

    #[test]
    fn top_level_typo_is_rejected() {
        let json = r#"{"version":"1","defaul_font":"바탕","questions":[]}"#;
        let e = serde_json::from_str::<IngestDocument>(json)
            .unwrap_err()
            .to_string();
        assert!(e.contains("defaul_font"), "{e}");
    }

    #[test]
    fn question_typo_is_rejected() {
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 1, "stem": "Q", "choice": [], "choices": []
            }]
        }"#;
        let e = serde_json::from_str::<IngestDocument>(json)
            .unwrap_err()
            .to_string();
        assert!(e.contains("choice"), "{e}");
    }

    /// 필드를 전부 채운 정상 블록 3형은 종전과 동일하게 파싱된다.
    #[test]
    fn valid_blocks_still_parse() {
        let b: StemBlock =
            serde_json::from_str(r#"{"type":"image","ref":"img/a.png","placement":"below"}"#)
                .unwrap();
        assert!(matches!(b, StemBlock::Image { .. }));
        let b: StemBlock = serde_json::from_str(
            r#"{"type":"boxed","title":"<보기>","blocks":[{"type":"text","text":"본문"}]}"#,
        )
        .unwrap();
        assert!(matches!(b, StemBlock::Boxed { .. }));
    }
}
