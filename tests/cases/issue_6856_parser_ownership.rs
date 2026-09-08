//! #6856: parser ownership contracts using in-memory sections, not Hancom fixtures.
use rhwp::model::control::Control;
use rhwp::model::document::Section;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{
    Caption, GroupShape, RectangleControlKind, RectangleShape, ShapeObject, TextBox,
};
use rhwp::parser::body_text::parse_body_text_section;
use rhwp::parser::hwpx::section::parse_hwpx_section;
use rhwp::serializer::body_text::serialize_section;

fn empty_textbox() -> RectangleShape {
    let mut rect = RectangleShape::default();
    rect.drawing.text_box = Some(TextBox {
        paragraphs: vec![Paragraph::default()],
        ..Default::default()
    });
    rect
}

fn roundtrip(shape: ShapeObject) -> ShapeObject {
    let section = Section {
        paragraphs: vec![Paragraph {
            controls: vec![Control::Shape(Box::new(shape))],
            ..Default::default()
        }],
        ..Default::default()
    };
    let parsed = parse_body_text_section(&serialize_section(&section)).unwrap();
    match parsed.paragraphs[0].controls[0].clone() {
        Control::Shape(shape) => *shape,
        _ => panic!("shape missing"),
    }
}

#[test]
fn hwp_caption_does_not_replace_owned_empty_area() {
    for owned in [false, true] {
        let mut rect = if owned {
            empty_textbox()
        } else {
            RectangleShape::default()
        };
        rect.drawing.caption = Some(Caption {
            paragraphs: vec![Paragraph {
                text: "caption".into(),
                ..Default::default()
            }],
            ..Default::default()
        });
        let ShapeObject::Rectangle(parsed) = roundtrip(ShapeObject::Rectangle(rect)) else {
            panic!("rectangle missing")
        };
        assert_eq!(
            parsed.drawing.caption.unwrap().paragraphs[0].text,
            "caption"
        );
        assert_eq!(parsed.drawing.text_box.is_some(), owned);
        if owned {
            assert_eq!(parsed.drawing.text_box.unwrap().paragraphs.len(), 1);
        }
    }
}

#[test]
fn hwp_group_sibling_textbox_does_not_reclassify_plain_rectangle() {
    for reversed in [false, true] {
        let mut children = vec![
            ShapeObject::Rectangle(RectangleShape::default()),
            ShapeObject::Rectangle(empty_textbox()),
        ];
        if reversed {
            children.reverse();
        }
        let ShapeObject::Group(parsed) = roundtrip(ShapeObject::Group(GroupShape {
            children,
            ..Default::default()
        })) else {
            panic!("group missing")
        };
        assert_eq!(parsed.children.len(), 2);
        for (i, child) in parsed.children.iter().enumerate() {
            let ShapeObject::Rectangle(rect) = child else {
                panic!("rectangle missing")
            };
            let expected = if (i == 1) != reversed {
                RectangleControlKind::TextBox
            } else {
                RectangleControlKind::Rectangle
            };
            assert_eq!(rect.control_kind(), expected);
        }
    }
}

#[test]
fn hwpx_nested_textbox_keeps_paragraphs_under_their_owner() {
    let xml = r#"<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section" xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"><hp:p><hp:run><hp:rect><hp:drawText><hp:subList><hp:p><hp:run><hp:t>outer</hp:t><hp:rect><hp:drawText><hp:subList><hp:p><hp:run><hp:t>inner</hp:t></hp:run></hp:p></hp:subList></hp:drawText></hp:rect></hp:run></hp:p></hp:subList></hp:drawText></hp:rect><hp:rect></hp:rect></hp:run></hp:p></hs:sec>"#;
    let parsed = parse_hwpx_section(xml).unwrap();
    let controls = &parsed.paragraphs[0].controls;
    assert_eq!(controls.len(), 2);
    let Control::Shape(outer) = &controls[0] else {
        panic!("outer missing")
    };
    let outer = outer.drawing().unwrap().text_box.as_ref().unwrap();
    assert_eq!(outer.paragraphs.len(), 1);
    assert_eq!(outer.paragraphs[0].text, "outer");
    let Control::Shape(inner) = &outer.paragraphs[0].controls[0] else {
        panic!("inner missing")
    };
    let inner = inner.drawing().unwrap().text_box.as_ref().unwrap();
    assert_eq!(inner.paragraphs.len(), 1);
    assert_eq!(inner.paragraphs[0].text, "inner");
    let Control::Shape(sibling) = &controls[1] else {
        panic!("sibling missing")
    };
    assert!(sibling.drawing().unwrap().text_box.is_none());
}
