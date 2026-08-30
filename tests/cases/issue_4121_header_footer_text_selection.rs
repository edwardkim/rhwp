//! [#4121] 머리말/꼬리말 텍스트 선택의 코어/WASM 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::header_footer::HeaderFooterApply;
use rhwp::model::style::Alignment;
use rhwp::wasm_api::HwpDocument;

fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/field-01.hwp"))
        .expect("sample")
}

fn header_with_two_paragraphs(first: &str, second: &str) -> HwpDocument {
    let mut doc = HwpDocument::from_bytes(&sample()).expect("parse");
    doc.create_header_footer_native(0, true, 0)
        .expect("양쪽 머리말 생성");
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, first)
        .expect("첫 문단 입력");
    let first_len = first.chars().count();
    doc.split_paragraph_in_header_footer_native(0, true, 0, 0, first_len, None)
        .expect("문단 분할");
    doc.insert_text_in_header_footer_native(0, true, 0, 1, 0, second)
        .expect("둘째 문단 입력");
    doc
}

fn header_text(doc: &HwpDocument) -> String {
    let raw = doc
        .get_header_footer_native(0, true, 0)
        .expect("머리말 조회");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("머리말 JSON");
    value["text"].as_str().unwrap_or_default().to_string()
}

#[test]
fn newly_created_header_and_footer_use_body_default_justification() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank2010 생성");
    for (is_header, apply_to) in [(true, 0), (false, 1), (true, 2)] {
        doc.create_header_footer_native(0, is_header, apply_to)
            .expect("HF 생성");
    }

    let expected = [
        (true, HeaderFooterApply::Both),
        (false, HeaderFooterApply::Even),
        (true, HeaderFooterApply::Odd),
    ];
    for (is_header, apply_to) in expected {
        let apply_to_u8 = match apply_to {
            HeaderFooterApply::Both => 0,
            HeaderFooterApply::Even => 1,
            HeaderFooterApply::Odd => 2,
        };
        let paragraph = doc.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|paragraph| paragraph.controls.iter())
            .find_map(|control| match control {
                Control::Header(header) if is_header && header.apply_to == apply_to => {
                    header.paragraphs.first()
                }
                Control::Footer(footer) if !is_header && footer.apply_to == apply_to => {
                    footer.paragraphs.first()
                }
                _ => None,
            })
            .expect("생성한 HF 문단");
        let shape = &doc.document().doc_info.para_shapes[paragraph.para_shape_id as usize];
        assert_eq!(
            shape.alignment,
            Alignment::Justify,
            "새 HF는 종류와 적용 범위에 관계없이 본문 기본 양쪽 정렬로 시작해야 함: header={is_header}, apply={apply_to:?}"
        );

        let props = doc
            .get_para_properties_in_hf_native(0, is_header, apply_to_u8, 0)
            .expect("HF 문단 속성 조회");
        let props: serde_json::Value = serde_json::from_str(&props).expect("HF 문단 속성 JSON");
        assert_eq!(
            props["alignment"], "justify",
            "생성 직후 문단 속성도 본문 기본 양쪽 정렬을 노출해야 함"
        );
    }

    let text = "가나다 라 바 아";
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, text)
        .expect("공백 포함 머리말 입력");
    let rects = doc
        .get_selection_rects_in_header_footer_native(0, true, 0, 0, 0, 0, 0, text.chars().count())
        .expect("공백 포함 머리말 선택 영역");
    let rects: serde_json::Value = serde_json::from_str(&rects).expect("선택 영역 JSON");
    let width = rects[0]["width"].as_f64().expect("첫 선택 영역 폭");
    assert!(
        width < 200.0,
        "새 HF의 짧은 마지막 줄은 공백을 영역 전체로 늘리면 안 됨: width={width}"
    );
}

#[test]
fn selection_rects_span_header_paragraphs_and_reject_missing_preview_target() {
    let doc = header_with_two_paragraphs("ABCDE", "FGHIJ");

    let raw = doc
        .get_selection_rects_in_header_footer_native(0, true, 0, 0, 0, 1, 1, 3)
        .expect("양쪽 머리말 선택 사각형");
    let rects: serde_json::Value = serde_json::from_str(&raw).expect("rect JSON");
    let rects = rects.as_array().expect("rect array");
    assert!(
        rects.len() >= 2,
        "다문단 선택은 둘 이상의 run을 덮어야 함: {raw}"
    );
    assert!(rects.iter().all(|rect| rect["pageIndex"] == 0));
    assert!(rects
        .iter()
        .all(|rect| rect["width"].as_f64().unwrap_or(0.0) > 0.0));

    let first_rect = &rects[0];
    let hit_raw = doc
        .hit_test_in_header_footer_native(
            0,
            true,
            first_rect["x"].as_f64().unwrap_or(0.0) + 1.0,
            first_rect["y"].as_f64().unwrap_or(0.0) + 1.0,
        )
        .expect("머리말 hit-test");
    let hit: serde_json::Value = serde_json::from_str(&hit_raw).expect("hit JSON");
    assert_eq!(hit["sectionIndex"], 0);
    assert_eq!(hit["applyTo"], 0);

    assert!(
        doc.get_selection_rects_in_header_footer_native(0, true, 1, 0, 0, 1, 1, 3)
            .is_err(),
        "대표 페이지라도 존재하지 않는 HF target은 거부해야 함"
    );

    assert!(
        doc.get_selection_rects_in_header_footer_native(0, true, 0, 0, 0, 0, 1, 99)
            .is_err(),
        "active target의 유효하지 않은 문자 범위는 잘라 그리지 않고 거부해야 함"
    );

    assert!(
        doc.get_cursor_rect_in_header_footer_native(0, true, 1, 0, 1, 0)
            .is_err(),
        "preferred page의 active target과 다른 HF에서 캐럿을 찾으면 안 됨"
    );
}

#[test]
fn odd_and_even_footer_rects_follow_each_pages_resolved_target() {
    let mut doc = HwpDocument::from_bytes(&sample()).expect("parse");
    for apply_to in [0, 1, 2] {
        doc.create_header_footer_native(0, false, apply_to)
            .expect("꼬리말 생성");
        doc.insert_text_in_header_footer_native(
            0,
            false,
            apply_to,
            0,
            0,
            match apply_to {
                1 => "EVEN",
                2 => "ODD",
                _ => "BOTH",
            },
        )
        .expect("꼬리말 텍스트");
    }
    doc.insert_text_native(0, 0, 0, &"가나다라마바사 ".repeat(4_000))
        .expect("여러 쪽 본문 생성");
    assert!(doc.page_count() >= 2, "홀짝 검증에는 두 쪽 이상이 필요함");

    let mut seen = std::collections::BTreeSet::new();
    for page_num in 0..doc.page_count().min(4) {
        let target_raw = doc
            .get_header_footer_edit_target_native(page_num, false)
            .expect("쪽별 꼬리말 target");
        let target: serde_json::Value = serde_json::from_str(&target_raw).expect("target JSON");
        let apply_to = target["applyTo"].as_u64().unwrap_or(0) as u8;
        seen.insert(apply_to);

        let correct = doc
            .get_selection_rects_in_header_footer_native(0, false, apply_to, page_num, 0, 0, 0, 1)
            .expect("active target rect");
        let correct_rects: serde_json::Value = serde_json::from_str(&correct).expect("rect JSON");
        assert!(
            !correct_rects.as_array().unwrap().is_empty(),
            "page={page_num}, target={apply_to}: {correct}"
        );

        let wrong_apply = if apply_to == 1 { 2 } else { 1 };
        let wrong = doc
            .get_selection_rects_in_header_footer_native(
                0,
                false,
                wrong_apply,
                page_num,
                0,
                0,
                0,
                1,
            )
            .expect("inactive target 선택 영역");
        if page_num == 0 {
            let wrong_rects: serde_json::Value =
                serde_json::from_str(&wrong).expect("preview rect JSON");
            assert!(
                !wrong_rects.as_array().unwrap().is_empty(),
                "구역 첫 페이지는 실제 active target과 달라도 선택한 HF를 대표 투영해야 함: {wrong}"
            );
        } else {
            assert_eq!(wrong, "[]");
        }
    }
    assert_eq!(seen, std::collections::BTreeSet::from([1, 2]));

    let preview_raw = doc
        .get_header_footer_preview_page_native(0)
        .expect("대표 편집 페이지");
    let preview: serde_json::Value = serde_json::from_str(&preview_raw).expect("preview JSON");
    assert_eq!(preview["pageIndex"], 0);

    let actual_before = doc
        .get_header_footer_edit_target_native(0, false)
        .expect("대표 페이지의 실제 target");
    let even_cursor_raw = doc
        .get_cursor_rect_in_header_footer_native(0, false, 1, 0, 2, 0)
        .expect("짝수 꼬리말을 구역 첫 페이지에 가상 투영한 커서");
    let even_cursor: serde_json::Value =
        serde_json::from_str(&even_cursor_raw).expect("cursor JSON");
    assert_eq!(even_cursor["pageIndex"], 0);
    let hit_raw = doc
        .hit_test_in_header_footer_target_native(
            0,
            0,
            false,
            1,
            even_cursor["x"].as_f64().unwrap_or_default(),
            even_cursor["y"].as_f64().unwrap_or_default() + 1.0,
        )
        .expect("대표 페이지의 명시적 짝수 target hit-test");
    let hit: serde_json::Value = serde_json::from_str(&hit_raw).expect("hit JSON");
    assert_eq!(hit["sectionIndex"], 0);
    assert_eq!(hit["applyTo"], 1);
    assert_eq!(
        doc.get_header_footer_edit_target_native(0, false)
            .expect("대표 투영 후 실제 target"),
        actual_before,
        "대표 편집 preview는 pagination active target을 바꾸면 안 됨"
    );

    let saved = doc.export_hwp().expect("preview 조회 후 저장");
    let reparsed = HwpDocument::from_bytes(&saved).expect("preview 조회 후 재파스");
    assert_eq!(
        reparsed
            .get_header_footer_edit_target_native(0, false)
            .expect("재파스 실제 target"),
        actual_before,
        "비인쇄 preview는 저장/출력 의미를 바꾸면 안 됨"
    );
}

#[test]
fn field_display_text_keeps_header_footer_model_offsets() {
    let mut doc = HwpDocument::from_bytes(&sample()).expect("parse");
    doc.create_header_footer_native(0, true, 0)
        .expect("양쪽 머리말 생성");
    doc.insert_field_in_hf_native(0, true, 0, 0, 0, 3)
        .expect("파일 이름 필드 삽입");

    let raw = doc
        .get_selection_rects_in_header_footer_native(0, true, 0, 0, 0, 0, 0, 1)
        .expect("필드 선택 사각형");
    let rects: serde_json::Value = serde_json::from_str(&raw).expect("rect JSON");
    let rect = rects
        .as_array()
        .and_then(|values| values.first())
        .expect("필드 표시 문자열도 선택 사각형을 가져야 함");

    let hit_raw = doc
        .hit_test_in_header_footer_native(
            0,
            true,
            rect["x"].as_f64().unwrap_or_default()
                + rect["width"].as_f64().unwrap_or_default() * 0.75,
            rect["y"].as_f64().unwrap_or_default() + 1.0,
        )
        .expect("필드 표시 문자열 hit-test");
    let hit: serde_json::Value = serde_json::from_str(&hit_raw).expect("hit JSON");
    assert!(
        hit["charOffset"].as_u64().unwrap_or(u64::MAX) <= 1,
        "표시 문자열 길이가 모델의 단일 필드 marker offset을 넘기면 안 됨: {hit_raw}"
    );
}

#[test]
fn replacement_is_atomic_across_paragraphs_and_preserves_suffix() {
    let mut doc = header_with_two_paragraphs("Alpha", "Beta");

    let raw = doc
        .replace_range_in_header_footer_native(0, true, 0, 1, 2, 0, 2, "X\nY")
        .expect("다문단 치환");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("치환 JSON");
    assert_eq!(value["hfParaIndex"], 1);
    assert_eq!(value["charOffset"], 1);
    assert_eq!(header_text(&doc), "AlX\nYta");

    let before = header_text(&doc);
    assert!(
        doc.replace_range_in_header_footer_native(0, true, 0, 0, 0, 1, 99, "bad")
            .is_err(),
        "유효하지 않은 끝 offset은 mutation 전에 거부해야 함"
    );
    assert_eq!(
        header_text(&doc),
        before,
        "실패한 치환은 문서를 바꾸면 안 됨"
    );

    let saved = doc.export_hwp().expect("치환 문서 저장");
    let reparsed = HwpDocument::from_bytes(&saved).expect("치환 문서 재파스");
    assert_eq!(header_text(&reparsed), "AlX\nYta");
}

#[test]
fn copy_selection_preserves_paragraph_break_and_plain_text() {
    let mut doc = header_with_two_paragraphs("Alpha", "Beta");

    let raw = doc
        .copy_selection_in_header_footer_native(0, true, 0, 0, 1, 1, 3)
        .expect("다문단 복사");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("복사 JSON");
    assert_eq!(value["text"], "lpha\nBet");
    assert_eq!(doc.get_clipboard_text_native(), "lpha\nBet");
    assert_eq!(
        header_text(&doc),
        "Alpha\nBeta",
        "복사는 원문을 바꾸면 안 됨"
    );
}

#[test]
fn char_format_applies_only_to_the_selected_header_range() {
    let mut doc = header_with_two_paragraphs("Alpha", "Beta");
    let before_raw = doc
        .get_char_properties_in_header_footer_native(0, true, 0, 0, 0)
        .expect("선택 밖 속성");
    let before: serde_json::Value = serde_json::from_str(&before_raw).expect("속성 JSON");
    let old_bold = before["bold"].as_bool().unwrap_or(false);
    let target_bold = !old_bold;

    doc.apply_char_format_in_header_footer_native(
        0,
        true,
        0,
        0,
        1,
        1,
        2,
        &format!(r#"{{"bold":{target_bold}}}"#),
    )
    .expect("다문단 부분 서식");

    for (para_idx, offset) in [(0, 1), (1, 1)] {
        let raw = doc
            .get_char_properties_in_header_footer_native(0, true, 0, para_idx, offset)
            .expect("선택 속성");
        let value: serde_json::Value = serde_json::from_str(&raw).expect("속성 JSON");
        assert_eq!(value["bold"], target_bold);
    }

    let outside_raw = doc
        .get_char_properties_in_header_footer_native(0, true, 0, 0, 0)
        .expect("선택 밖 속성");
    let outside: serde_json::Value = serde_json::from_str(&outside_raw).expect("속성 JSON");
    assert_eq!(outside["bold"], old_bold, "선택 밖 글자 서식은 유지돼야 함");

    let saved = doc.export_hwp().expect("서식 문서 저장");
    let reparsed = HwpDocument::from_bytes(&saved).expect("서식 문서 재파스");
    let selected_raw = reparsed
        .get_char_properties_in_header_footer_native(0, true, 0, 1, 1)
        .expect("재파스 선택 속성");
    let selected: serde_json::Value =
        serde_json::from_str(&selected_raw).expect("재파스 속성 JSON");
    assert_eq!(selected["bold"], target_bold);
}
