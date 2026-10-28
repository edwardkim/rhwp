//! 정상 한컴 form-01의 빈 selectedValue는 첫 목록 값을 표시한다.
//! 기준: 사용자가 제공한 form-01-2024.pdf 및 기존 HWP serializer의 Text 규칙.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FormType};
use rhwp::renderer::render_tree::{FormObjectNode, RenderNode, RenderNodeType};

const INITIAL: &str = "계절 선택";

fn combo(node: &RenderNode) -> Option<&FormObjectNode> {
    if let RenderNodeType::FormObject(form) = &node.node_type {
        if form.form_type == FormType::ComboBox {
            return Some(form);
        }
    }
    node.children.iter().find_map(combo)
}

fn fixture(text: &str, item: Option<&str>, inline: bool, surrounding: bool) -> DocumentCore {
    let mut core =
        DocumentCore::from_bytes(include_bytes!("../../samples/hwpx/form-01.hwpx")).unwrap();
    if surrounding {
        core.insert_text_native(0, 4, 0, "앞글").unwrap();
    }
    let Control::Form(form) = &mut core.document_mut().sections[0].paragraphs[4].controls[0] else {
        panic!("정상 원본 ComboBox")
    };
    form.text = text.into();
    form.common.treat_as_char = inline;
    match item {
        Some(item) => {
            form.properties.insert("listItem0".into(), item.into());
        }
        None => {
            form.properties.remove("listItem0");
        }
    }
    DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap()
}

fn assert_display(core: &DocumentCore, raw: &str, expected: &str) {
    let tree = core.build_page_render_tree(0).unwrap();
    let node = combo(&tree.root).expect("실제 배치된 ComboBox");
    assert_eq!(node.text, raw, "편집용 선택값은 그대로 보존");
    assert_eq!(node.display_or_text(), expected, "공통 표시 결과");
    let value: serde_json::Value =
        serde_json::from_str(&core.get_form_value_native(0, 4, 0).unwrap()).unwrap();
    assert_eq!(value["text"], raw, "조회로 초기 표시를 선택값에 쓰지 않음");
    // 표시 경로를 확인하되 합성 배치의 좌표가 다른 경우 전체 SVG를 동일시하지 않는다.
    if !expected.is_empty() {
        assert!(core.render_page_svg_native(0).unwrap().contains(expected));
        assert!(core
            .render_page_svg_legacy_native(0)
            .unwrap()
            .contains(expected));
    }
    if raw.is_empty() && expected == INITIAL {
        // 같은 배치에서 Text만 명시한 독립 대조군과 최종 렌더 출력이 같아야 한다.
        let mut doc = core.document().clone();
        let Control::Form(form) = &mut doc.sections[0].paragraphs[4].controls[0] else {
            unreachable!()
        };
        form.text = expected.into();
        let mut reference = DocumentCore::new_empty();
        reference.set_document(doc);
        assert_eq!(
            core.render_page_svg_native(0).unwrap(),
            reference.render_page_svg_native(0).unwrap()
        );
        assert_eq!(
            core.get_page_layer_tree_native(0).unwrap(),
            reference.get_page_layer_tree_native(0).unwrap()
        );
        #[cfg(feature = "native-skia")]
        {
            use rhwp::renderer::layer_renderer::LayerRasterRenderer;
            use rhwp::renderer::skia::SkiaLayerRenderer;
            let renderer = SkiaLayerRenderer::new();
            assert_eq!(
                renderer
                    .render_png(&core.build_page_layer_tree(0).unwrap())
                    .unwrap(),
                renderer
                    .render_png(&reference.build_page_layer_tree(0).unwrap())
                    .unwrap()
            );
        }
    }
}

#[test]
fn hancom_original_displays_initial_item_without_changing_selection() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/hwpx/form-01.hwpx")).unwrap();
    assert_display(&core, "", INITIAL);
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert_display(&reopened, "", INITIAL);
    // HWP Text는 기존 serializer가 첫 항목으로 저장한다. 재열기 후 표시도 같다.
    let reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert_display(&reopened, INITIAL, INITIAL);
}

#[test]
fn initial_item_reaches_all_form_placements() {
    for (inline, surrounding) in [(true, false), (true, true), (false, false)] {
        assert_display(
            &fixture("", Some(INITIAL), inline, surrounding),
            "",
            INITIAL,
        );
    }
}

#[test]
fn explicit_selection_and_free_input_override_initial_item() {
    for text in ["봄", "자유 입력"] {
        assert_display(&fixture(text, Some(INITIAL), true, false), text, text);
    }
}

#[test]
fn empty_or_absent_initial_item_stays_empty() {
    for item in [None, Some("")] {
        assert_display(&fixture("", item, true, false), "", "");
    }
}
