//! #6856: 18가지 유무 동치류. 정상 한컴 fixture 18건을 뜻하지 않는다.
use rhwp::model::control::{AutoNumber, Control};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{Caption, RectangleControlKind, RectangleShape, TextBox};
use rhwp::model::style::{FillType, ImageFill};
use rhwp::parser::hwpx::section::parse_hwpx_section;
use rhwp::renderer::render_tree::{BoundingBox, RectangleNode, RenderNode, RenderNodeType};

#[test]
fn eighteen_variations_have_two_rectangles_and_sixteen_textboxes() {
    let mut counts = [0, 0];
    for background in [false, true] {
        for content in 0..9 {
            let mut rect = RectangleShape::default();
            if background {
                rect.drawing.fill.fill_type = FillType::Image;
                rect.drawing.fill.image = Some(ImageFill {
                    bin_data_id: 1,
                    ..Default::default()
                });
            }
            if content > 0 {
                let mask = content - 1;
                let mut paragraph = Paragraph::default();
                if mask & 1 != 0 {
                    paragraph.text = "본문".into();
                }
                if mask & 2 != 0 {
                    paragraph
                        .controls
                        .push(Control::Picture(Box::default()));
                }
                if mask & 4 != 0 {
                    paragraph
                        .controls
                        .push(Control::AutoNumber(AutoNumber::default()));
                }
                rect.drawing.text_box = Some(TextBox {
                    paragraphs: vec![paragraph],
                    ..Default::default()
                });
            }
            let expected = if content == 0 {
                RectangleControlKind::Rectangle
            } else {
                RectangleControlKind::TextBox
            };
            assert_eq!(
                rect.control_kind(),
                expected,
                "background={background} content={content}"
            );
            // NONE/SOLID 등의 선 속성을 식별에 사용하지 않는다.
            for attr in [0, 1, u32::MAX] {
                rect.drawing.border_line.attr = attr;
                assert_eq!(rect.control_kind(), expected);
            }
            counts[usize::from(content > 0)] += 1;
        }
    }
    assert_eq!(counts, [2, 16]);
}

#[test]
fn owned_zero_paragraph_area_is_not_absence() {
    let mut rect = RectangleShape::default();
    rect.drawing.text_box = Some(TextBox::default());
    assert_eq!(rect.control_kind(), RectangleControlKind::TextBox);
}

#[test]
fn caption_is_not_an_owned_textbox() {
    let mut rect = RectangleShape::default();
    rect.drawing.caption = Some(Caption::default());
    assert_eq!(rect.control_kind(), RectangleControlKind::Rectangle);
}

#[test]
fn structural_marker_survives_empty_content_without_duplicate() {
    let bounds = BoundingBox::new(0.0, 0.0, 100.0, 50.0);
    let mut outer = RenderNode::new(
        1,
        RenderNodeType::Rectangle(RectangleNode::new(0.0, Default::default(), None)),
        bounds,
    );
    // 도형 장식용 RectangleNode에는 원본 컨트롤 부호를 자동으로 붙이지 않는다.
    assert_eq!(outer.control_code_label(), None);
    let mut content = RenderNode::new(2, RenderNodeType::TextBox, bounds);
    let mut nested = RenderNode::new(
        3,
        RenderNodeType::Rectangle(RectangleNode::new(0.0, Default::default(), None)),
        bounds,
    );
    nested.set_rectangle_control_kind(RectangleControlKind::Rectangle);
    content.children.push(nested);
    outer.children.push(content);
    outer.set_rectangle_control_kind(RectangleControlKind::TextBox);
    assert_eq!(outer.control_code_label(), Some("[글상자]"));
    assert_eq!(outer.children[0].control_code_label(), None);
    assert_eq!(
        outer.children[0].children[0].control_code_label(),
        Some("[사각형]")
    );
    outer.children.clear();
    assert_eq!(outer.control_code_label(), Some("[글상자]"));
}

#[test]
fn hwpx_absent_empty_and_picture_only_areas_are_distinct() {
    for (draw, expected) in [
        ("", RectangleControlKind::Rectangle),
        ("<hp:drawText/>", RectangleControlKind::TextBox),
        ("<hp:drawText><hp:subList><hp:p><hp:run><hp:pic id=\"2\"><hc:img binaryItemIDRef=\"image1\"/></hp:pic></hp:run></hp:p></hp:subList></hp:drawText>", RectangleControlKind::TextBox),
    ] {
        let xml = format!("<hs:sec xmlns:hs=\"http://www.hancom.co.kr/hwpml/2011/section\" xmlns:hp=\"http://www.hancom.co.kr/hwpml/2011/paragraph\" xmlns:hc=\"http://www.hancom.co.kr/hwpml/2011/core\"><hp:p><hp:run><hp:rect>{draw}</hp:rect></hp:run></hp:p></hs:sec>");
        let section = parse_hwpx_section(&xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else { panic!("shape missing"); };
        let rhwp::model::shape::ShapeObject::Rectangle(rect) = shape.as_ref() else { panic!("rectangle missing"); };
        assert_eq!(rect.control_kind(), expected);
        if draw.contains("hp:pic") {
            let tb = rect.drawing.text_box.as_ref().unwrap();
            assert!(tb.paragraphs.iter().any(|p| p.controls.iter().any(|c| matches!(c, Control::Picture(_)))));
        }
    }
}
