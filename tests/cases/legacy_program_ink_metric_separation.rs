//! 관계형: 기존 명시 글꼴 공급의 윤곽 보존과 자연 전진 선택은 독립적인 계약이다.
//! 주석 갱신: 2026-11-08 19:45 KST. 공개 문서·글꼴 환경 API의 합성 계약이다.
//! 실문서 좌표·설치 글꼴·쪽수·golden·시각 일치율에는 의존하지 않는다.
//! sample16 실제 Print 대조와 원 PR 이력은 native-weight-r46g-analysis.md의 R55 절을 따른다.
use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};
fn run(node: &RenderNode) -> &TextRunNode {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == "S/W -----" {
            return run;
        }
    }
    node.children.iter().find_map(find).expect("원문 글줄")
}
fn find(node: &RenderNode) -> Option<&TextRunNode> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == "S/W -----" {
            return Some(run);
        }
    }
    node.children.iter().find_map(find)
}
fn core() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    for font in doc.doc_info.font_faces.iter_mut().flatten() {
        font.name = "HCI Poppy".into();
        font.alt_type = 2;
        font.native_program = None;
    }
    core.set_document(doc);
    core.insert_text_native(0, 0, 0, "S/W -----").unwrap();
    core
}
#[test]
fn legacy_named_program_ink_is_independent_of_natural_metric_selection() {
    let mut core = core();
    let mut positions = None;
    for preserve in [true, false] {
        let environment = if preserve {
            r#"{"id":"source","substitutions":{"HCI Poppy":"HCI Poppy"},"preserveSourceMetrics":true}"#
        } else {
            r#"{"id":"legacy","substitutions":{"HCI Poppy":"HCI Poppy"}}"#
        };
        core.set_font_environment(Some(FontEnvironment::from_json(environment).unwrap()))
            .unwrap();
        let tree = core.build_page_render_tree(0).unwrap();
        let run = run(&tree.root);
        assert_eq!(
            run.style.preserve_glyph_ink, !preserve,
            "기존 명시 공급의 일반 글자는 원 전진 칸에 윤곽을 누르지 않는다"
        );
        let current = EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style);
        if let Some(ref before) = positions {
            assert_eq!(
                &current, before,
                "윤곽 보존이 leader의 원본 전진을 바꾸지 않는다"
            );
        } else {
            positions = Some(current);
        }
    }
}
