//! 관계형: 사용자 선택 원본의 자연 전진과 다른 원본의 저장 메트릭 소유를 구분한다.
//! Native/fresh WASM 일치율: 비해당. 공개 API로 생성한 합성 문서의 프로그램 선택 계약이다.
//! 주석 갱신: 2026-11-08 00:47 KST. 사설 글꼴 설치·실물 px 좌표에 의존하지 않는다.
use rhwp::{
    model::style::Font,
    renderer::{
        font_environment::FontEnvironment,
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};
use serde_json::{json, Value};

fn find<'a>(node: &'a RenderNode, text: &str) -> Option<&'a TextRunNode> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == text {
            return Some(run);
        }
    }
    node.children.iter().find_map(|child| find(child, text))
}

fn environment(selected: bool) -> Value {
    let mut value = json!({"id":"automatic", "substitutions":{"옥수수":"Corn"},
        "preserveSourceMetrics":true});
    if selected {
        value["id"] = json!("user-selected");
        value["substitutions"]["나눔고딕"] = json!("Haansoft Batang");
        value["trueTypeFaces"] = json!(["Haansoft Batang"]);
        value["replaceSourceMetricsFor"] = json!(["나눔고딕"]);
    }
    value
}

#[test]
fn selected_source_metrics_do_not_change_the_other_original_program() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "선택원본").unwrap();
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.insert_text_native(0, 1, 0, "유지원본").unwrap();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        *fonts = vec![
            Font {
                name: "나눔고딕".into(),
                alt_type: 1,
                ..Font::default()
            },
            Font {
                name: "옥수수".into(),
                alt_type: 2,
                ..Font::default()
            },
        ];
    }
    let mut first = document.doc_info.char_shapes[0].clone();
    first.font_ids = [0; 7];
    let mut second = first.clone();
    second.font_ids = [1; 7];
    document.doc_info.char_shapes = vec![first, second];
    core.set_document(document);
    core.set_char_shape_id_native(0, 0, 0, 4, 0).unwrap();
    core.set_char_shape_id_native(0, 1, 0, 4, 1).unwrap();
    let original = core.export_hwp_native().unwrap();
    core.set_font_environment(Some(
        FontEnvironment::from_json(&environment(false).to_string()).unwrap(),
    ))
    .unwrap();
    let before = core.build_page_render_tree(0).unwrap();
    let untouched = find(&before.root, "유지원본").unwrap();
    core.set_font_environment(Some(
        FontEnvironment::from_json(&environment(true).to_string()).unwrap(),
    ))
    .unwrap();
    let after = core.build_page_render_tree(0).unwrap();
    let selected = find(&after.root, "선택원본").unwrap();
    let preserved = find(&after.root, "유지원본").unwrap();
    assert!(
        selected.style.preserve_glyph_ink,
        "선택한 원본은 자연 프로그램 윤곽을 사용한다"
    );
    assert_eq!(
        preserved.style, untouched.style,
        "다른 원본의 실제 paint 스타일을 보존한다"
    );
    assert_eq!(
        EmbeddedTextMeasurer.compute_char_positions(&preserved.text, &preserved.style),
        EmbeddedTextMeasurer.compute_char_positions(&untouched.text, &untouched.style),
        "다른 원본의 실제 글자 전진도 보존한다",
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        original,
        "세션 선택은 저장 원본을 변경하지 않는다"
    );
}

#[test]
fn source_metric_overrides_reject_unmapped_and_duplicate_sources() {
    for invalid in [json!(["없는 원본"]), json!(["나눔고딕", "나눔고딕"])] {
        let mut value = environment(true);
        value["replaceSourceMetricsFor"] = invalid;
        assert!(FontEnvironment::from_json(&value.to_string()).is_err());
    }
}

#[test]
fn omitted_natural_program_selection_keeps_legacy_leader() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "-----. ").unwrap();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        for font in fonts {
            font.name = "한양신명조".into();
        }
    }
    core.set_document(document);
    let legacy = json!({"id":"legacy-reference", "substitutions":{"한양신명조":"HYSinMyeongJo-Medium"},
        "trueTypeFaces":["HYSinMyeongJo-Medium"]});
    let mut legacy_positions = None;
    for choice in [None, Some(false), Some(true)] {
        let mut value = legacy.clone();
        if let Some(preserve) = choice {
            value["preserveSourceMetrics"] = json!(preserve);
            if preserve {
                value["replaceSourceMetricsFor"] = json!(["한양신명조"]);
            }
        }
        let environment = FontEnvironment::from_json(&value.to_string()).unwrap();
        let serialized = serde_json::to_value(&environment).unwrap();
        assert_eq!(
            serialized.get("preserveSourceMetrics"),
            value.get("preserveSourceMetrics"),
            "생략과 명시 false를 세션 직렬화 왕복에서도 구분한다"
        );
        core.set_font_environment(Some(
            FontEnvironment::from_json(&serialized.to_string()).unwrap(),
        ))
        .unwrap();
        let tree = core.build_page_render_tree(0).unwrap();
        let run = find(&tree.root, "-----. ").unwrap();
        assert_eq!(
            run.style.preserve_glyph_ink,
            choice.is_some(),
            "기존 메트릭 선언은 자연 윤곽 선택을 대신하지 않는다"
        );
        let positions = EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style);
        if choice.is_none() {
            legacy_positions = Some(positions);
        } else {
            assert!(
                positions[5] > legacy_positions.as_ref().unwrap()[5],
                "명시 자연 전진은 기존 leader와 별개로 유지한다"
            );
        }
    }
    let mut invalid = legacy;
    invalid["preserveSourceMetrics"] = Value::Null;
    assert!(FontEnvironment::from_json(&invalid.to_string()).is_err());
}

fn selected_program_core() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "프로그램 선택").unwrap();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        *fonts = vec![Font {
            name: "선택 원본".into(),
            alt_type: 1,
            ..Font::default()
        }];
    }
    document.doc_info.char_shapes[0].font_ids = [0; 7];
    core.set_document(document);
    core
}

#[test]
fn supplied_face_query_precedes_shared_family_without_changing_source() {
    let mut core = selected_program_core();
    let saved = core.export_hwp_native().unwrap();
    let selected = core
        .build_font_environment_from_available_programs(
            &json!([{"query":"선택 원본","family":"공유 계열", "programId":"specific-bold-face", "trueType":true}]).to_string(),
        )
        .unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(&selected).unwrap()))
        .unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    let run = find(&tree.root, "프로그램 선택").unwrap();
    assert_eq!(
        run.style.font_family.split(',').next(),
        Some("선택 원본"),
        "공유 family의 기본 굵기로 바꾸지 않고 공급한 face 조회 이름을 먼저 출력한다"
    );
    assert!(run
        .style
        .font_family
        .split(',')
        .any(|name| name == "공유 계열"));
    assert_eq!(core.export_hwp_native().unwrap(), saved);
}

#[test]
fn slot_selected_face_query_precedes_shared_family() {
    let mut core = selected_program_core();
    let environment = json!({"id":"slot-program", "substitutions":{}, "preserveSourceMetrics":true,
        "trueTypeFaces":["공유 계열"],"documentSubstitutions":[{
            "source":"선택 원본","languageSlot":0,"fontId":0,"target":"공유 계열",
            "programId":"selected-slot-program","query":"슬롯의 실제 face"}]});
    core.set_font_environment(Some(
        FontEnvironment::from_json(&environment.to_string()).unwrap(),
    ))
    .unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    let run = find(&tree.root, "프로그램 선택").unwrap();
    assert_eq!(
        run.style.font_family.split(',').next(),
        Some("슬롯의 실제 face"),
        "슬롯이 고른 프로그램 조회 이름을 family보다 먼저 보존한다"
    );
}

#[test]
fn automatic_hft_program_identity_preserves_its_canonical_paint_alias() {
    let mut core = selected_program_core();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        *fonts = vec![Font {
            name: "한양신명조".into(),
            alt_type: 2,
            ..Font::default()
        }];
    }
    core.set_document(document);
    let saved = core.export_hwp_native().unwrap();
    let selected = core
        .build_font_environment_from_available_programs(
            &json!([{"query":"한양신명조", "family":"HY Sinmyeongjo",
                "programId":"same-original-hft", "trueType":false, "openTypeCff":true}])
            .to_string(),
        )
        .unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(&selected).unwrap()))
        .unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    let run = find(&tree.root, "프로그램 선택").unwrap();
    assert_eq!(
        run.style.font_family.split(',').next(),
        Some("HY Sinmyeongjo"),
        "자동 HFT 인증은 다른 프로그램에도 대응하는 원본 별칭을 재승격하지 않는다"
    );
    let mut explicit: Value = serde_json::from_str(&selected).unwrap();
    explicit["preserveSourceMetrics"] = json!(false);
    core.set_font_environment(Some(
        FontEnvironment::from_json(&explicit.to_string()).unwrap(),
    ))
    .unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    assert_eq!(
        find(&tree.root, "프로그램 선택")
            .unwrap()
            .style
            .font_family
            .split(',')
            .next(),
        Some("한양신명조"),
        "명시적으로 원본 전진을 바꾼 선택은 실제 조회 이름을 보존한다"
    );
    assert_eq!(core.export_hwp_native().unwrap(), saved);
}
