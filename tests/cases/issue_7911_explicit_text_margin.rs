//! HWPX presence semantics only; no screen coordinates or visual baseline.
use rhwp::model::{
    control::Control,
    shape::{RectangleShape, TextBox},
};
use std::io::Read;

fn document(explicit: bool, margin: i16) -> rhwp::model::document::Document {
    let spec = serde_json::from_value(serde_json::json!({"version":"1", "blocks":[]})).unwrap();
    let mut doc = rhwp::scaffold::build_scaffold(&spec);
    let mut rect = RectangleShape::default();
    rect.common.width = 10_000;
    rect.common.height = 2_000;
    rect.drawing.text_box = Some(TextBox {
        text_margin_explicit: explicit,
        margin_left: margin,
        paragraphs: vec![rhwp::model::paragraph::Paragraph::new_empty()],
        ..Default::default()
    });
    doc.sections[0].paragraphs[0].controls = vec![Control::Shape(Box::new(
        rhwp::model::shape::ShapeObject::Rectangle(rect),
    ))];
    doc.sections[0].paragraphs[0].char_count = 9;
    doc.sections[0].paragraphs[0].control_mask = 0x800;
    doc
}

fn text_box(doc: &rhwp::model::document::Document) -> &TextBox {
    let boxes: Vec<_> = doc
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            Control::Shape(s) => s.drawing().and_then(|d| d.text_box.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(boxes.len(), 1);
    boxes[0]
}

#[test]
fn absent_and_explicit_zero_margin_survive_distinctly() {
    for (explicit, margin, expected) in [(false, 0, false), (true, 0, true), (false, 100, true)] {
        let bytes = rhwp::serializer::serialize_hwpx(&document(explicit, margin)).unwrap();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).unwrap();
        let mut xml = String::new();
        zip.by_name("Contents/section0.xml")
            .unwrap()
            .read_to_string(&mut xml)
            .unwrap();
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        assert_eq!(
            parsed
                .descendants()
                .filter(|n| n
                    .has_tag_name(("http://www.hancom.co.kr/hwpml/2011/paragraph", "textMargin")))
                .count(),
            usize::from(expected)
        );
        let reopened = rhwp::parse_document(&bytes).unwrap();
        let tb = text_box(&reopened);
        assert_eq!(tb.text_margin_explicit, expected);
        assert_eq!(tb.margin_left, margin);
        let second = rhwp::serializer::serialize_hwpx(&reopened).unwrap();
        let final_doc = rhwp::parse_document(&second).unwrap();
        assert_eq!(text_box(&final_doc).text_margin_explicit, expected);
    }
}
