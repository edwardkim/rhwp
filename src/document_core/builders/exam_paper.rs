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
                .push(make_text_para(&apply_number_prefix(q, &q.stem)));
        } else {
            for (b_idx, block) in q.stem_blocks.iter().enumerate() {
                match block {
                    StemBlock::Text { text } => {
                        let prefixed = if b_idx == 0 {
                            apply_number_prefix(q, text)
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

/// 첫 stem 텍스트에 `{q.number}. ` 접두어를 적용하는 정책 결정.
///
/// **v2 정책 (#660 후속, code review 권고 반영)**: 휴리스틱(`text.starts_with('[')` 등)
/// 제거. `Question.auto_number` 명시 필드(default `true`)에 따라 결정한다.
/// - `auto_number == true` (default): `{n}. ` 자동 prepend
/// - `auto_number == false`: Skill이 명시적으로 prefix 또는 그룹 지시문을 작성한 것으로
///   간주해 그대로 사용
fn apply_number_prefix(q: &crate::parser::ingest::schema::Question, text: &str) -> String {
    if q.auto_number {
        format!("{}. {}", q.number, text)
    } else {
        text.to_string()
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
    fn test_build_stem_with_auto_number_false_explicit_prefix() {
        // v2 정책: Skill이 명시적으로 "2. ㉠..." 작성 + auto_number=false → 그대로 출력.
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 2,
                "stem": "㉠에 해당하는 내용으로 가장 적절한 것은?",
                "auto_number": false,
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
    fn test_build_stem_with_auto_number_false_group_directive() {
        // v2 정책: 첫 stem_block이 "[1~3] 다음 글을 ..." 그룹 지시문 + auto_number=false → 그대로 출력.
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 1,
                "stem": "윗글의 내용과 일치하지 않는 것은?",
                "auto_number": false,
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

    #[test]
    fn test_build_stem_auto_number_default_true_omitted() {
        // v2 정책: auto_number 미지정 시 기본 true → "{n}. " 자동 prepend.
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 7,
                "stem": "주제로 가장 적절한 것은?",
                "stem_blocks": [
                    {"type": "text", "text": "주제로 가장 적절한 것은?"}
                ],
                "choices": [{"label": "①", "text": "X"}],
                "media": []
            }]
        }"#;
        let ingest = parse_ingest_str(json).unwrap();
        let doc = build_exam_paper(&ingest);
        assert_eq!(
            doc.sections[0].paragraphs[0].text,
            "7. 주제로 가장 적절한 것은?"
        );
    }

    #[test]
    fn test_build_stem_auto_number_true_with_boxed_bracket_text() {
        // v2 정책 회귀: v1 휴리스틱이 잘못 처리하던 "[보기]"로 시작하는 비-그룹지시문이
        // auto_number=true (default)에서 정상적으로 prefix됨.
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 12,
                "stem": "[보기]를 참고하여 …",
                "stem_blocks": [
                    {"type": "text", "text": "[보기]를 참고하여 다음을 분석한 것은?"}
                ],
                "choices": [{"label": "①", "text": "X"}],
                "media": []
            }]
        }"#;
        let ingest = parse_ingest_str(json).unwrap();
        let doc = build_exam_paper(&ingest);
        // v2: auto_number 기본 true → "12. [보기]를 참고하여 ..."
        assert_eq!(
            doc.sections[0].paragraphs[0].text,
            "12. [보기]를 참고하여 다음을 분석한 것은?"
        );
    }

    #[test]
    fn test_build_stem_auto_number_double_digit_number() {
        // v2 정책: multi-digit 문제 번호 (10번 이상)에서 정상 동작.
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 25,
                "stem": "정답은?",
                "stem_blocks": [
                    {"type": "text", "text": "정답은?"}
                ],
                "choices": [{"label": "①", "text": "X"}],
                "media": []
            }]
        }"#;
        let ingest = parse_ingest_str(json).unwrap();
        let doc = build_exam_paper(&ingest);
        assert_eq!(doc.sections[0].paragraphs[0].text, "25. 정답은?");
    }

    #[test]
    fn test_build_stem_only_auto_number_false() {
        // v2 정책: stem_blocks 비어있을 때도 auto_number=false 적용 — stem 그대로.
        let json = r#"{
            "version": "1",
            "questions": [{
                "number": 3,
                "stem": "3. ㉠과 ㉡의 관계로 가장 적절한 것은?",
                "auto_number": false,
                "choices": [{"label": "①", "text": "X"}],
                "media": []
            }]
        }"#;
        let ingest = parse_ingest_str(json).unwrap();
        let doc = build_exam_paper(&ingest);
        assert_eq!(
            doc.sections[0].paragraphs[0].text,
            "3. ㉠과 ㉡의 관계로 가장 적절한 것은?"
        );
    }
}
