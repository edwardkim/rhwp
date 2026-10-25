use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FormObject, FormType};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn document(kind: FormType, inline: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    let mut form = FormObject {
        form_type: kind,
        name: "선택할 양식".into(),
        caption: "라디오 단추".into(),
        enabled: true,
        width: 8504,
        height: 1984,
        ..Default::default()
    };
    form.common.width = form.width;
    form.common.height = form.height;
    form.common.treat_as_char = inline;
    let para = &mut doc.sections[0].paragraphs[0];
    para.controls = vec![Control::Form(Box::new(form))];
    para.char_count = 9;
    para.has_para_text = true;
    core.set_document(doc);
    core
}

fn form_box(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::FormObject(_)) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(form_box)
}

#[test]
fn inline_forms_have_distinct_carets_and_copy_the_selected_control() {
    for kind in [
        FormType::PushButton,
        FormType::CheckBox,
        FormType::RadioButton,
        FormType::ComboBox,
        FormType::Edit,
    ] {
        let mut core = document(kind, true);
        assert!(core.document().sections[0].paragraphs[0].controls[0].is_logical_inline());
        let tree = core.build_page_render_tree(0).unwrap();
        let bbox = form_box(&tree.root).unwrap();
        let public = HwpDocument::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
        assert_eq!(public.get_logical_length(0, 0).unwrap(), 1);
        let rects: serde_json::Value =
            serde_json::from_str(&public.get_selection_rects(0, 0, 0, 0, 1).unwrap()).unwrap();
        assert!(
            rects
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["width"].as_f64().unwrap_or(0.0) > 0.0),
            "양식만 고른 블록을 표시한다"
        );
        for (fraction, expected) in [(0.1, 0), (0.9, 1)] {
            let hit: serde_json::Value = serde_json::from_str(
                &core
                    .hit_test_native(
                        0,
                        bbox.x + bbox.width * fraction,
                        bbox.y + bbox.height / 2.0,
                    )
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(hit["charOffset"], expected);
        }
        let before: serde_json::Value =
            serde_json::from_str(&core.get_cursor_rect_native(0, 0, 0).unwrap()).unwrap();
        let after: serde_json::Value =
            serde_json::from_str(&core.get_cursor_rect_native(0, 0, 1).unwrap()).unwrap();
        assert!((before["x"].as_f64().unwrap() - bbox.x).abs() < 0.1);
        assert!((after["x"].as_f64().unwrap() - bbox.x - bbox.width).abs() < 0.1);
        core.copy_selection_logical_native(0, 0, 0, 0, 1).unwrap();
        core.paste_internal_native(0, 0, 0).unwrap();
        assert_eq!(
            core.document().sections[0]
                .paragraphs
                .iter()
                .flat_map(|p| &p.controls)
                .filter(|c| matches!(c, Control::Form(_)))
                .count(),
            2,
            "선택한 양식 개체를 실제로 복제한다"
        );
    }
}

#[test]
fn floating_forms_do_not_take_an_inline_slot() {
    let core = document(FormType::RadioButton, false);
    assert!(!core.document().sections[0].paragraphs[0].controls[0].is_logical_inline());
}

#[test]
fn radio_outer_bottom_right_edge_remains_visible_on_white_paper() {
    let core = document(FormType::RadioButton, true);
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    assert!(xml
        .descendants()
        .any(|n| n.has_tag_name("polyline") && n.attribute("stroke") == Some("#a0a0a0")));
    assert!(!xml
        .descendants()
        .any(|n| n.has_tag_name("polyline") && n.attribute("stroke") == Some("#fafafa")));
}
