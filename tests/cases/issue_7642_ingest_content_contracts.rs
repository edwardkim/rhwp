//! 제거된 시험지 빌더 검사 14개의 내용·소유·직렬화 계약을 새 이름으로 보존한다.
//! 기대 내용은 각 ingest JSON의 문제·지문·선택지 순서에서 정한다.
//! 렌더러·화면 좌표·이미지·골든을 검사하지 않는 구조/API 계약이다.
//! 실제 HWP/HWPX의 시각 통과는 전쪽 Native/fresh WASM TSV·직접 판독으로 별도 확인한다.

use rhwp::document_core::builders::exam_paper::build_exam_paper;
use rhwp::model::control::Control;
use rhwp::parser::ingest::parse_ingest_str;

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
fn issue7642_ingest_single_question() {
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
fn issue7642_ingest_single_question_serializes_to_hwpx() {
    let ingest = parse_ingest_str(minimal_ingest_json()).unwrap();
    let doc = build_exam_paper(&ingest);
    let bytes = rhwp::serializer::serialize_hwpx(&doc).expect("구조 직렬화");
    let restored = rhwp::parser::parse_document(&bytes).expect("독립 파서로 재읽기");
    let texts: Vec<_> = restored.sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect();
    assert_eq!(
        texts,
        ["1. 다음 글의 주제는?", "① A", "② B"],
        "문제·선택지 순서 및 누락·중복"
    );
}

#[test]
fn issue7642_ingest_with_image_placeholder() {
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
fn issue7642_ingest_multiple_questions_separator() {
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
fn issue7642_ingest_stem_without_blocks() {
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
fn issue7642_ingest_stem_with_auto_number_false_explicit_prefix() {
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
fn issue7642_ingest_stem_with_auto_number_false_group_directive() {
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
fn issue7642_ingest_stem_auto_number_default_true_omitted() {
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
fn issue7642_ingest_stem_auto_number_true_with_boxed_bracket_text() {
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
fn issue7642_ingest_stem_auto_number_double_digit_number() {
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
fn issue7642_ingest_shared_passage_emits_once() {
    let json = r#"{
        "version": "1",
        "passages": [{
            "id": "p1-2",
            "blocks": [
                {"type": "text", "text": "[1~2] 다음 글을 읽고 물음에 답하시오."},
                {"type": "text", "text": "공유 지문"}
            ]
        }],
        "questions": [
            {
                "number": 1,
                "passage_ref": "p1-2",
                "stem": "Q1",
                "choices": [{"label": "①", "text": "A"}],
                "media": []
            },
            {
                "number": 2,
                "passage_ref": "p1-2",
                "stem": "Q2",
                "choices": [{"label": "①", "text": "B"}],
                "media": []
            }
        ]
    }"#;
    let ingest = parse_ingest_str(json).unwrap();
    let doc = build_exam_paper(&ingest);
    let texts: Vec<&str> = doc.sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect();

    assert_eq!(
        texts,
        vec![
            "[1~2] 다음 글을 읽고 물음에 답하시오.",
            "공유 지문",
            "1. Q1",
            "① A",
            "",
            "2. Q2",
            "① B"
        ]
    );
}

#[test]
fn issue7642_ingest_boxed_stem_block_uses_boxed_para_shape() {
    let json = r#"{
        "version": "1",
        "questions": [{
            "number": 3,
            "stem": "다음 보기에서 알 수 있는 것은?",
            "stem_blocks": [
                {"type": "text", "text": "다음 보기에서 알 수 있는 것은?"},
                {
                    "type": "boxed",
                    "title": "<보기>",
                    "blocks": [
                        {"type": "text", "text": "보기 본문"}
                    ]
                }
            ],
            "choices": [{"label": "①", "text": "A"}],
            "media": []
        }]
    }"#;
    let ingest = parse_ingest_str(json).unwrap();
    let doc = build_exam_paper(&ingest);

    assert_eq!(doc.doc_info.border_fills.len(), 2);
    assert_eq!(doc.doc_info.para_shapes.len(), 2);
    // [#3355] 기본 borderFill(첫 엔트리)은 무테두리여야 한다 — 기본 글자모양이
    // 참조하므로 실선이면 모든 텍스트 런에 상자가 그려진다.
    assert!(
        doc.doc_info.border_fills[0]
            .borders
            .iter()
            .all(|b| b.line_type == rhwp::model::style::BorderLineType::None),
        "기본 borderFill 은 4면 모두 선 없음이어야 합니다"
    );
    // boxed 블록용(둘째 엔트리)은 실선 테두리를 유지한다.
    assert!(
        doc.doc_info.border_fills[1]
            .borders
            .iter()
            .all(|b| b.line_type == rhwp::model::style::BorderLineType::Solid),
        "boxed borderFill 은 실선을 유지해야 합니다"
    );
    assert_eq!(
        doc.sections[0].paragraphs[0].text,
        "3. 다음 보기에서 알 수 있는 것은?"
    );
    assert_eq!(doc.sections[0].paragraphs[1].text, "<보기>");
    let boxed_shape = doc.sections[0].paragraphs[1].para_shape_id;
    assert_ne!(boxed_shape, doc.sections[0].paragraphs[0].para_shape_id);
    assert_eq!(doc.sections[0].paragraphs[2].text, "보기 본문");
    assert_eq!(
        doc.sections[0].paragraphs[2].para_shape_id, boxed_shape,
        "보기 제목과 본문은 같은 박스 소유"
    );
}

#[test]
fn issue7642_ingest_header_footer_metadata_creates_controls() {
    let json = r#"{
        "version": "1",
        "header_text": "국어 영역",
        "footer_text": "1/20",
        "form_label": "홀수형",
        "questions": [{
            "number": 1,
            "stem": "Q",
            "choices": [{"label": "①", "text": "A"}],
            "media": []
        }]
    }"#;
    let ingest = parse_ingest_str(json).unwrap();
    let doc = build_exam_paper(&ingest);
    let controls = &doc.sections[0].paragraphs[0].controls;

    assert_eq!(controls.len(), 2);
    match &controls[0] {
        Control::Header(header) => {
            assert_eq!(header.paragraphs[0].text, "홀수형 국어 영역");
            assert!(header.text_width > 0);
        }
        other => panic!("expected header control, got {other:?}"),
    }
    match &controls[1] {
        Control::Footer(footer) => {
            assert_eq!(footer.paragraphs[0].text, "1/20");
            assert!(footer.text_width > 0);
        }
        other => panic!("expected footer control, got {other:?}"),
    }
}

#[test]
fn issue7642_ingest_stem_only_auto_number_false() {
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
