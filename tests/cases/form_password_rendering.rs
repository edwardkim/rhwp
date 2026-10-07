//! 명시 PasswordChar는 표시만 가리고 편집 조회와 HWP/HWPX 원문을 유지한다.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FormType};
use rhwp::renderer::render_tree::{FormObjectNode, RenderNode, RenderNodeType};
use serde_json::Value;

const RAW: &str = "MASK_SENTINEL";

fn fixture(mask: Option<&str>, text: &str, inline: bool, surrounding: bool) -> DocumentCore {
    let mut core =
        DocumentCore::from_bytes(include_bytes!("../../samples/hwpx/form-01.hwpx")).unwrap();
    if surrounding {
        core.insert_text_native(0, 8, 0, "앞글").unwrap();
    }
    let Control::Form(form) = &mut core.document_mut().sections[0].paragraphs[8].controls[0] else {
        panic!("공개 form-01의 Edit 컨트롤")
    };
    assert_eq!(form.form_type, FormType::Edit);
    form.text = text.into();
    form.common.treat_as_char = inline;
    match mask {
        Some(mask) => {
            form.properties.insert("PasswordChar".into(), mask.into());
        }
        None => {
            form.properties.remove("PasswordChar");
        }
    }
    // 저장된 속성으로 문서를 다시 열어 실제 parser → layout 경로를 검사한다.
    DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap()
}

fn edit_node(node: &RenderNode) -> Option<(&FormObjectNode, &RenderNode)> {
    if let RenderNodeType::FormObject(form) = &node.node_type {
        if form.form_type == FormType::Edit {
            return Some((form, node));
        }
    }
    node.children.iter().find_map(edit_node)
}

fn edit_text(value: &Value) -> Option<&str> {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("formObject")
                && map.get("formType").and_then(Value::as_str) == Some("edit")
            {
                return map.get("text").and_then(Value::as_str);
            }
            map.values().find_map(edit_text)
        }
        Value::Array(items) => items.iter().find_map(edit_text),
        _ => None,
    }
}

fn assert_masked(core: &DocumentCore, reference: &DocumentCore, raw: &str, mask: &str) {
    let tree = core.build_page_render_tree(0).unwrap();
    let (form, node) = edit_node(&tree.root).expect("Edit 렌더 노드");
    assert_eq!(form.text, raw, "hit query에 필요한 원문");
    let hit: Value = serde_json::from_str(
        &core
            .get_form_object_at_native(
                0,
                node.bbox.x + node.bbox.width / 2.0,
                node.bbox.y + node.bbox.height / 2.0,
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(hit["text"], raw);
    let value: Value = serde_json::from_str(&core.get_form_value_native(0, 8, 0).unwrap()).unwrap();
    assert_eq!(value["text"], raw);
    for (actual, expected) in [
        (
            core.render_page_svg_native(0).unwrap(),
            reference.render_page_svg_native(0).unwrap(),
        ),
        (
            core.render_page_svg_legacy_native(0).unwrap(),
            reference.render_page_svg_legacy_native(0).unwrap(),
        ),
    ] {
        assert!(!actual.contains(raw), "암호 원문이 SVG에 표시됨");
        assert_eq!(actual, expected, "명시한 마스킹 문자만 표시");
    }
    let layer: Value = serde_json::from_str(&core.get_page_layer_tree_native(0).unwrap()).unwrap();
    assert_eq!(
        edit_text(&layer),
        Some(mask),
        "CanvasKit/replay 표시 payload"
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
                .unwrap(),
            "Native Skia도 같은 마스킹 문자열을 그림",
        );
    }
}

fn check_placement(inline: bool, surrounding: bool) {
    let mask = "*".repeat(RAW.chars().count());
    let core = fixture(Some("*"), RAW, inline, surrounding);
    let reference = fixture(Some(""), &mask, inline, surrounding);
    assert_masked(&core, &reference, RAW, &mask);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let Control::Form(form) = &reopened.document().sections[0].paragraphs[8].controls[0] else {
            panic!("저장 후 Edit 컨트롤")
        };
        assert_eq!(form.text, RAW);
        assert_eq!(
            form.properties.get("PasswordChar").map(String::as_str),
            Some("*")
        );
        // 저장 형식별 기존 배치 차이는 이번 표시 계약과 분리한다.
        let mut reference_document = reopened.document().clone();
        let Control::Form(form) = &mut reference_document.sections[0].paragraphs[8].controls[0]
        else {
            unreachable!()
        };
        form.text = mask.clone();
        form.properties.insert("PasswordChar".into(), "".into());
        let mut reference = DocumentCore::new_empty();
        reference.set_document(reference_document);
        assert_masked(&reopened, &reference, RAW, &mask);
    }
}

#[test]
fn password_edit_masks_empty_line_inline_form() {
    check_placement(true, false);
}

#[test]
fn password_edit_masks_text_line_inline_form() {
    check_placement(true, true);
}

#[test]
fn password_edit_masks_floating_form() {
    check_placement(false, false);
}

#[test]
fn plain_edit_text_remains_visible_and_queryable() {
    let core = fixture(Some(""), RAW, true, false);
    let tree = core.build_page_render_tree(0).unwrap();
    assert_eq!(edit_node(&tree.root).unwrap().0.text, RAW);
    assert!(core.render_page_svg_native(0).unwrap().contains(RAW));
    let layer: Value = serde_json::from_str(&core.get_page_layer_tree_native(0).unwrap()).unwrap();
    assert_eq!(edit_text(&layer), Some(RAW));
}
