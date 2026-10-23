//! #8166 양식 개체의 캡션은 개체 영역 밖으로 그려지지 않는다.
//!
//! 기준: form-01 의 체크 상자 폭만 3000·1100 HWPUNIT 로 줄인 저장본을 한컴 웹기안기로 인쇄하면
//! 캡션이 개체 오른쪽 경계에서 잘린다(1100 은 체크 표시만 남는다).
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FormType};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const CAPTION: &str = "선택 상자";

fn checkbox_bbox(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::FormObject(form) = &node.node_type {
        if form.form_type == FormType::CheckBox {
            return Some(node.bbox);
        }
    }
    node.children.iter().find_map(checkbox_bbox)
}

fn with_checkbox_width(width: u32) -> DocumentCore {
    let mut core =
        DocumentCore::from_bytes(include_bytes!("../../samples/hwpx/form-01.hwpx")).unwrap();
    let form = core.document_mut().sections[0]
        .paragraphs
        .iter_mut()
        .flat_map(|para| para.controls.iter_mut())
        .find_map(|control| match control {
            Control::Form(form) if form.form_type == FormType::CheckBox => Some(form),
            _ => None,
        })
        .expect("form-01 의 체크 상자");
    assert_eq!(form.caption, CAPTION);
    form.width = width;
    form.common.width = width;
    DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap()
}

/// SVG 에서 캡션 `<text>` 를 감싼 가장 안쪽 clip 의 사각형을 돌려준다.
fn caption_clip_rect(svg: &str) -> (f64, f64, f64, f64) {
    let text_at = svg
        .find(&format!(">{CAPTION}</text>"))
        .expect("캡션 글자 출력");
    let group_at = svg[..text_at]
        .rfind("<g clip-path=\"url(#")
        .expect("캡션을 감싼 clip 그룹");
    let id_start = group_at + "<g clip-path=\"url(#".len();
    let id = &svg[id_start..id_start + svg[id_start..].find(')').unwrap()];
    assert!(
        !svg[group_at..text_at].contains("</g>"),
        "clip 그룹이 캡션 전에 닫히면 안 된다"
    );
    let def = &svg[svg
        .find(&format!("<clipPath id=\"{id}\">"))
        .expect("clip 정의")..];
    let attr = |name: &str| -> f64 {
        let start = def.find(&format!("{name}=\"")).unwrap() + name.len() + 2;
        def[start..start + def[start..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    (attr("x"), attr("y"), attr("width"), attr("height"))
}

fn assert_caption_clipped_to_form(width: u32) {
    let core = with_checkbox_width(width);
    let tree = core.build_page_render_tree(0).unwrap();
    let bbox = checkbox_bbox(&tree.root).expect("배치된 체크 상자");
    let expected_width = f64::from(width) * 96.0 / 7200.0;
    assert!(
        (bbox.width - expected_width).abs() < 0.5,
        "개체 영역은 저장 폭 {width}HU 를 따른다: {bbox:?}"
    );

    let (x, y, w, h) = caption_clip_rect(&core.render_page_svg_native(0).unwrap());
    assert!(
        (x - bbox.x).abs() < 0.01
            && (y - bbox.y).abs() < 0.01
            && (w - bbox.width).abs() < 0.01
            && (h - bbox.height).abs() < 0.01,
        "캡션은 체크 상자 영역으로 잘려야 한다: clip=({x}, {y}, {w}, {h}) bbox={bbox:?}"
    );
}

#[test]
fn checkbox_caption_is_clipped_where_it_overflows_the_control() {
    assert_caption_clipped_to_form(3000);
}

#[test]
fn checkbox_caption_is_clipped_when_the_control_only_fits_the_box() {
    assert_caption_clipped_to_form(1100);
}

#[test]
fn checkbox_caption_inside_a_wide_control_keeps_the_same_clip_contract() {
    assert_caption_clipped_to_form(9921);
}
