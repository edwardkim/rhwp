//! #6852: 일반 사각형을 글상자 마스크로 추정해 원본 실선을 지우지 않는다.
//! 실제 HWP/HWPX 5쪽의 앞쪽 흰 사각형과 뒤쪽 그림자 사각형을 구분한다.
//! 내부 변형은 분기 경계 시험이며 한컴 출력의 정답지로 사용하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{DrawingObjAttr, ShapeObject, TextBox};
use rhwp::model::style::FillType;

const HWP: &str = "samples/issue6797/156160455-social-pig-farm-income.hwp";
const HWPX: &str = "samples/hwpx/156160455-social-pig-farm-income.hwpx";

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .expect("원본 fixture 필수");
    DocumentCore::from_bytes(&bytes).expect("원본 파싱")
}

fn check_rectangles(core: &DocumentCore, foreground_stroke: bool) {
    assert_eq!(core.page_count(), 11, "원본 쪽수 보존");
    let svg = core.render_page_svg_native(4).expect("5쪽 SVG");
    let xml = roxmltree::Document::parse(&svg).expect("SVG XML");
    let coord =
        |node: roxmltree::Node<'_, '_>, name| node.attribute(name).unwrap().parse::<f64>().unwrap();
    let boxes: Vec<_> = xml
        .descendants()
        .filter(|node| node.has_tag_name("rect"))
        .filter(|node| {
            node.attribute("x").is_some()
                && node.attribute("y").is_some()
                && (75.0..80.0).contains(&coord(*node, "x"))
                && (129.0..133.0).contains(&coord(*node, "y"))
                && (16.0..16.1).contains(&coord(*node, "width"))
                && (17.0..17.1).contains(&coord(*node, "height"))
        })
        .collect();
    assert_eq!(boxes.len(), 2, "제목 앞 사각형 2개를 정확히 식별");
    let shadow = boxes[0];
    let foreground = boxes[1];
    assert_eq!(shadow.attribute("fill"), Some("#282828"));
    assert_eq!(shadow.attribute("stroke"), Some("#000000"));
    assert!((coord(shadow, "stroke-width") - 0.5).abs() < 1e-8);
    for (node, x, y) in [
        (shadow, 78.10666666666667, 131.28),
        (foreground, 75.58666666666667, 129.4933333333333),
    ] {
        assert!((coord(node, "x") - x).abs() < 1e-6, "x 유지");
        assert!((coord(node, "y") - y).abs() < 1e-6, "y 유지");
    }
    assert_eq!(
        foreground.attribute("stroke"),
        foreground_stroke.then_some("#000000"),
        "앞쪽 사각형의 명시된 선을 글상자 보정으로 제거해서는 안 된다"
    );
    if foreground_stroke {
        assert!((coord(foreground, "stroke-width") - 0.5).abs() < 1e-8);
    }
}

fn variant(change: impl FnOnce(&mut DrawingObjAttr)) -> DocumentCore {
    let mut core = load(HWP);
    let mut doc = core.document().clone();
    let Control::Shape(shape) = &mut doc.sections[0].paragraphs[48].controls[0] else {
        panic!("대상은 그리기 컨트롤");
    };
    let ShapeObject::Group(group) = shape.as_mut() else {
        panic!("대상은 묶음");
    };
    let ShapeObject::Rectangle(rect) = &mut group.children[1] else {
        panic!("앞쪽 자식은 사각형");
    };
    assert!(rect.drawing.text_box.is_none());
    assert_eq!(rect.drawing.border_line.attr & 0x3f, 1);
    assert_eq!(rect.drawing.fill.solid.unwrap().background_color, 0xffffff);
    change(&mut rect.drawing);
    core.set_document(doc);
    core
}

#[test]
fn original_hwp_foreground_and_shadow_keep_solid_strokes() {
    check_rectangles(&load(HWP), true);
}

#[test]
fn original_hwpx_foreground_and_shadow_keep_solid_strokes() {
    check_rectangles(&load(HWPX), true);
}

#[test]
fn ordinary_unfilled_rectangle_is_not_a_textbox() {
    let core = variant(|drawing| drawing.fill = Default::default());
    check_rectangles(&core, true);
}

#[test]
fn explicit_no_line_is_not_promoted_to_solid() {
    let core = variant(|drawing| drawing.border_line.attr &= !0x3f);
    check_rectangles(&core, false);
}

#[test]
fn empty_and_whitespace_textboxes_keep_their_strokes() {
    for text in ["", "   "] {
        let core = variant(|drawing| {
            drawing.fill = Default::default();
            drawing.text_box = Some(TextBox {
                paragraphs: vec![rhwp::model::paragraph::Paragraph {
                    text: text.to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            });
        });
        check_rectangles(&core, true);
    }
}

#[test]
fn existing_textbox_branch_is_unchanged_not_a_new_nonprinting_rule() {
    let core = variant(|drawing| {
        drawing.fill.fill_type = FillType::None;
        drawing.fill.solid = None;
        drawing.text_box = Some(TextBox {
            paragraphs: vec![rhwp::model::paragraph::Paragraph {
                text: "A".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        });
    });
    // 기존 A 분기의 동작 범위만 보호한다. 한컴 비인쇄 의미를 증명하는 테스트가 아니다.
    check_rectangles(&core, false);
}
