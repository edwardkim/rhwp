//! `ingest_schema_v1.json` serde 모델.
//!
//! Claude Code Skill ↔ rhwp Rust 본체 인터페이스. version="1" 고정.

use serde::{Deserialize, Serialize};

/// 문서 전체 — Skill이 작성하는 JSON 최상위.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestDocument {
    /// 스키마 버전 (현재 "1" 만 허용).
    pub version: String,

    /// 페이지 크기 (mm).
    #[serde(default = "default_page_size")]
    pub page_size: PageSize,

    /// 기본 폰트 이름. Skill이 시험지에서 인식한 폰트 또는 fallback.
    #[serde(default = "default_font")]
    pub default_font: String,

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
pub struct PageSize {
    pub width_mm: f32,
    pub height_mm: f32,
}

/// 한 문제 단위.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// 문제 번호 (보통 1~30).
    pub number: u32,

    /// 지문 — 첫 줄 또는 전체 (stem_blocks가 있으면 stem_blocks가 우선, stem은 fallback).
    pub stem: String,

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// 선택지 항목.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// 표시 라벨 (예: "①" U+2460).
    pub label: String,
    /// 본문 텍스트 (label 없이).
    pub text: String,
}

/// 이미지 메타.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            "questions": [
                {
                    "number": 1,
                    "stem": "다음 글의 주제로 가장 적절한 것은?",
                    "stem_blocks": [
                        {"type": "text", "text": "다음 글의 주제로 가장 적절한 것은?"}
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
        assert_eq!(doc.questions.len(), 1);
        assert_eq!(doc.questions[0].number, 1);
        assert_eq!(doc.questions[0].choices.len(), 5);
        assert_eq!(doc.questions[0].choices[0].label, "①");
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
    }

    #[test]
    fn test_placement_serde() {
        let p_str = serde_json::to_string(&Placement::Between).unwrap();
        assert_eq!(p_str, r#""between""#);
        let p: Placement = serde_json::from_str(r#""inline""#).unwrap();
        assert_eq!(p, Placement::Inline);
    }
}
