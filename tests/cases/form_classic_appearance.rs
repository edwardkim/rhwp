//! 한컴 수동 PDF의 10pt 폼 글자 및 classic 2px 입체 프레임 계약.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FormType};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn original() -> DocumentCore {
    DocumentCore::from_bytes(include_bytes!("../../samples/hwpx/form-01.hwpx")).unwrap()
}
fn form_bbox(
    node: &RenderNode,
    kind: FormType,
) -> Option<rhwp::renderer::render_tree::BoundingBox> {
    if let RenderNodeType::FormObject(form) = &node.node_type {
        if form.form_type == kind {
            return Some(node.bbox);
        }
    }
    node.children.iter().find_map(|node| form_bbox(node, kind))
}
fn near(value: Option<&str>, expected: f64) -> bool {
    value
        .and_then(|s| s.parse::<f64>().ok())
        .is_some_and(|v| (v - expected).abs() < 0.02)
}
#[test]
fn enabled_captions_use_stored_ten_point_style_and_foreground() {
    let core = original();
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    for caption in ["명령 단추", "선택 상자", "계절 선택", "라디오 단추"] {
        let node = xml
            .descendants()
            .find(|n| n.has_tag_name("text") && n.text() == Some(caption))
            .unwrap();
        assert_eq!(
            node.attribute("fill"),
            Some("#000000"),
            "enabled 원래 글자색"
        );
        assert!(
            near(node.attribute("font-size"), 1000.0 * 96.0 / 7200.0),
            "저장 10pt를 높이 기반 임의 크기로 바꾸지 않음"
        );
    }
}
#[test]
fn raised_button_has_dark_bottom_edge_inside_original_bounds() {
    let core = original();
    let tree = core.build_page_render_tree(0).unwrap();
    let b = form_bbox(&tree.root, FormType::PushButton).unwrap();
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    assert!(
        xml.descendants().any(|n| n.has_tag_name("rect")
            && n.attribute("fill") == Some("#404040")
            && near(n.attribute("y"), b.y + b.height - 1.0)
            && near(n.attribute("height"), 1.0)),
        "classic 솟은 프레임의 어두운 아래 변"
    );
}
#[test]
fn edit_has_recessed_inner_edge_and_document_background() {
    let core = original();
    let tree = core.build_page_render_tree(0).unwrap();
    let b = form_bbox(&tree.root, FormType::Edit).unwrap();
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    assert!(
        xml.descendants().any(|n| n.has_tag_name("rect")
            && n.attribute("fill") == Some("#404040")
            && near(n.attribute("x"), b.x + 1.0)
            && near(n.attribute("y"), b.y + 1.0)),
        "들어간 프레임의 내부 좌상 변"
    );
    assert!(
        xml.descendants().any(|n| n.has_tag_name("rect")
            && n.attribute("fill") == Some("#f0f0f0")
            && near(n.attribute("x"), b.x)
            && near(n.attribute("y"), b.y)),
        "문서 Edit 배경색"
    );
}

#[test]
fn combo_label_origin_matches_independent_hancom_pdf() {
    let core = original();
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let label = xml
        .descendants()
        .find(|n| n.has_tag_name("text") && n.text() == Some("계절 선택"))
        .unwrap();
    // 사용자 PDF의 실제 text origin (87.36,193.68)pt를 같은 96dpi 좌표계로 변환.
    let x: f64 = label.attribute("x").unwrap().parse().unwrap();
    let baseline: f64 = label.attribute("y").unwrap().parse().unwrap();
    assert!((x - 87.36 * 4.0 / 3.0).abs() < 0.4);
    assert!((baseline - 193.68 * 4.0 / 3.0).abs() < 0.4);
}

#[test]
fn form_char_style_is_independent_of_context_until_follow_context() {
    for follow in [false, true] {
        let mut core = original();
        let doc = core.document_mut();
        let mut context_style = doc.doc_info.char_shapes[0].clone();
        context_style.base_size = 1600;
        let id = doc.doc_info.char_shapes.len() as u32;
        doc.doc_info.char_shapes.push(context_style);
        let para = &mut doc.sections[0].paragraphs[4];
        for reference in &mut para.char_shapes {
            reference.char_shape_id = id;
        }
        let Control::Form(form) = &mut para.controls[0] else {
            unreachable!()
        };
        form.properties.insert("CharShapeID".into(), "0".into());
        form.properties.insert(
            "FollowContext".into(),
            if follow { "1" } else { "0" }.into(),
        );
        // DocInfo를 추가한 합성 입력을 실제 저장/parser 경로로 열어 style snapshot을 만든다.
        let core = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
        let svg = core.render_page_svg_native(0).unwrap();
        let xml = roxmltree::Document::parse(&svg).unwrap();
        let label = xml
            .descendants()
            .find(|n| n.has_tag_name("text") && n.text() == Some("계절 선택"))
            .unwrap();
        assert!(near(
            label.attribute("font-size"),
            if follow { 1600.0 } else { 1000.0 } * 96.0 / 7200.0
        ));
    }
}

#[test]
fn explicit_frame_disabled_keeps_background_without_recessed_edges() {
    let mut core = original();
    let Control::Form(form) = &mut core.document_mut().sections[0].paragraphs[8].controls[0] else {
        unreachable!()
    };
    form.properties.insert("DrawFrame".into(), "0".into());
    let tree = core.build_page_render_tree(0).unwrap();
    let b = form_bbox(&tree.root, FormType::Edit).unwrap();
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    assert!(!xml.descendants().any(|n| n.has_tag_name("rect")
        && n.attribute("fill") == Some("#404040")
        && near(n.attribute("x"), b.x + 1.0)
        && near(n.attribute("y"), b.y + 1.0)));
    assert!(xml.descendants().any(|n| n.has_tag_name("rect")
        && n.attribute("fill") == Some("#f0f0f0")
        && near(n.attribute("x"), b.x)
        && near(n.attribute("y"), b.y)));
}
