//! Empty-element and explicit empty paragraphs must preserve the same owned IR.
use rhwp::model::{control::Control, paragraph::Paragraph};
use rhwp::parser::hwpx::section::parse_hwpx_section;

fn parse_owned(empty: bool) -> Vec<Paragraph> {
    let close = if empty { "/>" } else { "></hp:p>" };
    let xml = format!(
        r#"<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section" xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"><hp:p><hp:run><hp:rect><hp:drawText><hp:subList><hp:p id="2147483648" paraPrIDRef="7" styleIDRef="3" pageBreak="1" columnBreak="1"{close}<hp:p><hp:run><hp:t>following</hp:t></hp:run></hp:p></hp:subList></hp:drawText></hp:rect><hp:rect></hp:rect></hp:run></hp:p></hs:sec>"#
    );
    let section = parse_hwpx_section(&xml).unwrap();
    assert_eq!(section.paragraphs[0].controls.len(), 2, "sibling swallowed");
    let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
        panic!("shape missing")
    };
    shape
        .drawing()
        .unwrap()
        .text_box
        .as_ref()
        .unwrap()
        .paragraphs
        .clone()
}

#[test]
fn self_closing_owned_paragraph_preserves_attributes_and_following_content() {
    let expected = parse_owned(false);
    let actual = parse_owned(true);
    assert_eq!(actual.len(), 2, "empty paragraph must not disappear");
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(&expected) {
        assert_eq!(actual.text, expected.text);
        assert_eq!(actual.para_shape_id, expected.para_shape_id);
        assert_eq!(actual.style_id, expected.style_id);
        assert_eq!(actual.raw_break_type, expected.raw_break_type);
        assert_eq!(actual.raw_header_extra, expected.raw_header_extra);
        assert_eq!(actual.has_para_text, expected.has_para_text);
        assert_eq!(actual.controls.len(), expected.controls.len());
    }
    assert_eq!(actual[0].para_shape_id, 7);
    assert_eq!(actual[0].style_id, 3);
    assert_eq!(actual[0].raw_break_type, 12);
    assert_eq!(
        &actual[0].raw_header_extra[6..10],
        &2147483648_u32.to_le_bytes()
    );
    assert!(actual[0].text.is_empty());
    assert_eq!(actual[1].text, "following");
}
