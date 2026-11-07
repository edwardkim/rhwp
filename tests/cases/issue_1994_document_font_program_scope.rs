//! 관계형: 사용자 선택 원본의 자연 전진과 다른 원본의 저장 메트릭 소유를 구분한다.
//! Native/fresh WASM 일치율: 비해당. 공개 API로 생성한 합성 문서의 프로그램 선택 계약이다.
//! 주석 갱신: 2026-11-07 20:36 KST. 사설 글꼴 설치·실물 px 좌표에 의존하지 않는다.
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
