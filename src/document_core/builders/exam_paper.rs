//! 시험문제 Document IR 빌더 (Task #660 본 작업 1단계).
//!
//! [`IngestDocument`]를 시험문제 표준 layout의 [`Document`] IR로 변환한다.
//!
//! 본 단계(#660)는 **텍스트 위주**:
//! - 지문(stem) + 선택지(①~⑤) 텍스트 직접 포함 (spike #654 결정 정책)
//! - 이미지는 `[이미지: <ref>]` placeholder 텍스트로 대체
//! - placement 4모드(`between`/`above`/`below`/`inline`) 실제 IR 매핑은 #661에서 구현
//!
//! 후속 단계:
//! - #661: placement 4모드 IR 매핑 + Picture/BinData 빌드 (단 출력은 #182 의존)
//! - #182: HWPX writer Picture 직렬화 분기 추가 (별도 작업자)

use crate::model::document::{Document, Section};
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::parser::ingest::schema::{IngestDocument, StemBlock};

/// [`IngestDocument`] → [`Document`] IR 변환.
///
/// 시험지 layout:
/// - 각 문제: `{번호}. {지문}` + (이미지 placeholder) + ① ~ ⑤ 선택지 + 빈 문단(다음 문제 간격)
/// - 마지막 문제는 끝 빈 문단 없음
pub fn build_exam_paper(ingest: &IngestDocument) -> Document {
    let mut doc = Document::default();
    doc.sections.push(Section::default());

    let total_questions = ingest.questions.len();
    for (q_idx, q) in ingest.questions.iter().enumerate() {
        // 1. stem
        if q.stem_blocks.is_empty() {
            // stem_blocks 미제공 시 stem 한 줄 사용
            doc.sections[0]
                .paragraphs
                .push(make_text_para(&apply_number_prefix(q.number, &q.stem)));
        } else {
            for (b_idx, block) in q.stem_blocks.iter().enumerate() {
                match block {
                    StemBlock::Text { text } => {
                        let prefixed = if b_idx == 0 {
                            apply_number_prefix(q.number, text)
                        } else {
                            text.clone()
                        };
                        doc.sections[0].paragraphs.push(make_text_para(&prefixed));
                    }
                    StemBlock::Image { ref_, .. } => {
                        // 본 단계 placeholder. Picture/BinData 본격 빌드는 #661, 직렬화는 #182.
                        doc.sections[0]
                            .paragraphs
                            .push(make_text_para(&format!("[이미지: {ref_}]")));
                    }
                }
            }
        }

        // 2. 선택지 ①~⑤ — spike #654 결정 정책: 텍스트 직접 포함
        for choice in &q.choices {
            let line = format!("{} {}", choice.label, choice.text);
            doc.sections[0].paragraphs.push(make_text_para(&line));
        }

        // 3. 문제 간 빈 문단 (마지막 문제 제외)
        if q_idx + 1 < total_questions {
            doc.sections[0].paragraphs.push(Paragraph::new_empty());
        }
    }

    doc
}

/// 첫 stem 텍스트에 `{number}. ` 접두어를 추가하되, 사용자가 이미 명시적으로
/// 번호 또는 그룹 지시문(`[1~3]` 등)을 포함했으면 그대로 둔다.
///
/// e2e #663 검증에서 발견 — Skill이 `2. ㉠에 해당하는 ...` 처럼 작성한 경우
/// 빌더가 또 `2. ` prefix를 추가해 `2. 2. ㉠...` 중복 출력되던 문제 정정.
fn apply_number_prefix(number: u32, text: &str) -> String {
    let auto_prefix = format!("{number}. ");
    if text.starts_with(&auto_prefix) || text.starts_with('[') {
        text.to_string()
    } else {
        format!("{auto_prefix}{text}")
    }
}

/// 한 줄짜리 텍스트 Paragraph 생성 헬퍼.
fn make_text_para(text: &str) -> Paragraph {
    let utf16_len: u32 = text.encode_utf16().count() as u32;
    Paragraph {
        text: text.to_string(),
        char_count: utf16_len + 1, // +1: 문단 끝 마커
        char_offsets: (0..utf16_len).collect(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 1000,
            text_height: 1000,
            baseline_distance: 850,
            line_spacing: 600,
            segment_width: 50000,
            tag: 0x00060000,
            ..Default::default()
        }],
        para_shape_id: 0,
        style_id: 0,
        has_para_text: true,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ingest::parse_ingest_str;

    fn minimal_ingest_json() -> &'static str {
        r#"{
            "version": "1",
            "page_size": {"width_mm": 210.0, "height_mm": 297.0},
            "default_font": "함초롬바탕",
            "questions": [{
                "number": 1,
                "stem": "다음 글의 주제는?",
                "stem_blocks": [{"type": "text", "text": "다음 글의 주제는?"}],
                "choices": [
                    {"label": "①", "text": "A"},
                    {"label": "②", "text": "B"}
                ],
                "media": []
            }]
        }"#
    }

    #[test]
    fn test_build_single_question() {
        let ingest = parse_ingest_str(minimal_ingest_json()).unwrap();
        let doc = build_exam_paper(&ingest);
        assert_eq!(doc.sections.len(), 1);
        // 1 stem + 2 choice = 3 paragraph (마지막 문제이므로 끝 빈 문단 없음)
        assert_eq!(doc.sections[0].paragraphs.len(), 3);
        assert_eq!(doc.sections[0].paragraphs[0].text, "1. 다음 글의 주제는?");
        assert_eq!(doc.sections[0].paragraphs[1].text, "① A");
        assert_eq!(doc.sections[0].paragraphs[2].text, "② B");
    }

    #[test]
    fn test_build_with_image_placeholder() {
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 1,
                "stem": "S",
                "stem_blocks": [
                    {"type": "text", "text": "S"},
                    {"type": "image", "ref": "img/q1.png", "placement": "between"}
                ],
                "choices": [{"label": "①", "text": "A"}],
                "media": []
            }]
        }"#;
        let ingest = parse_ingest_str(json).unwrap();
        let doc = build_exam_paper(&ingest);
        assert_eq!(doc.sections[0].paragraphs.len(), 3);
        assert_eq!(doc.sections[0].paragraphs[0].text, "1. S");
        assert_eq!(doc.sections[0].paragraphs[1].text, "[이미지: img/q1.png]");
        assert_eq!(doc.sections[0].paragraphs[2].text, "① A");
    }

    #[test]
    fn test_build_multiple_questions_separator() {
        let json = r#"{
            "version": "1",
            "questions": [
                {
                    "number": 1,
                    "stem": "Q1",
                    "stem_blocks": [{"type": "text", "text": "Q1"}],
                    "choices": [{"label": "①", "text": "A"}],
                    "media": []
                },
                {
                    "number": 2,
                    "stem": "Q2",
                    "stem_blocks": [{"type": "text", "text": "Q2"}],
                    "choices": [{"label": "①", "text": "B"}],
                    "media": []
                }
            ]
        }"#;
        let ingest = parse_ingest_str(json).unwrap();
        let doc = build_exam_paper(&ingest);
        // Q1: 1 stem + 1 choice + 1 빈 문단(간격) + Q2: 1 stem + 1 choice = 5 paragraph
        assert_eq!(doc.sections[0].paragraphs.len(), 5);
        assert_eq!(doc.sections[0].paragraphs[0].text, "1. Q1");
        assert_eq!(doc.sections[0].paragraphs[1].text, "① A");
        assert_eq!(doc.sections[0].paragraphs[2].text, ""); // 빈 문단
        assert_eq!(doc.sections[0].paragraphs[3].text, "2. Q2");
        assert_eq!(doc.sections[0].paragraphs[4].text, "① B");
    }

    #[test]
    fn test_build_stem_without_blocks() {
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 5,
                "stem": "단순 stem",
                "choices": [{"label": "①", "text": "X"}],
                "media": []
            }]
        }"#;
        let ingest = parse_ingest_str(json).unwrap();
        let doc = build_exam_paper(&ingest);
        assert_eq!(doc.sections[0].paragraphs[0].text, "5. 단순 stem");
    }

    #[test]
    fn test_build_stem_with_explicit_number_prefix_no_duplication() {
        // e2e #663 회귀: Skill이 stem 첫 블록에 "2. ..."로 작성해도 빌더가 또 "2. "를
        // 추가하지 않음.
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 2,
                "stem": "㉠에 해당하는 내용으로 가장 적절한 것은?",
                "stem_blocks": [
                    {"type": "text", "text": "2. ㉠에 해당하는 내용으로 가장 적절한 것은?"}
                ],
                "choices": [{"label": "①", "text": "X"}],
                "media": []
            }]
        }"#;
        let ingest = parse_ingest_str(json).unwrap();
        let doc = build_exam_paper(&ingest);
        assert_eq!(
            doc.sections[0].paragraphs[0].text,
            "2. ㉠에 해당하는 내용으로 가장 적절한 것은?"
        );
    }

    #[test]
    fn test_build_stem_with_group_directive_no_prefix() {
        // e2e #663 회귀: 첫 stem_block이 "[1~3] 다음 글을 ..." 형식의 그룹 지시문이면
        // 빌더가 "1. " prefix를 강제로 붙이지 않음.
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 1,
                "stem": "윗글의 내용과 일치하지 않는 것은?",
                "stem_blocks": [
                    {"type": "text", "text": "[1~3] 다음 글을 읽고 물음에 답하시오."},
                    {"type": "text", "text": "본문..."}
                ],
                "choices": [{"label": "①", "text": "X"}],
                "media": []
            }]
        }"#;
        let ingest = parse_ingest_str(json).unwrap();
        let doc = build_exam_paper(&ingest);
        assert_eq!(
            doc.sections[0].paragraphs[0].text,
            "[1~3] 다음 글을 읽고 물음에 답하시오."
        );
    }
}
