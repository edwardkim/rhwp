//! PDF paint order preserves covered text without moving it above later marks.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{control::Control, shape::ShapeObject},
    DocumentCore,
};
const PDF: &[u8] = include_bytes!("../../pdf/issue7966/text-paint-order-controls.pdf");
fn check(core: &DocumentCore) {
    assert_eq!(core.page_count(), 3);
    for (page, section) in core.document().sections.iter().enumerate() {
        let mut marks = Vec::new();
        for c in section.paragraphs.iter().flat_map(|p| &p.controls) {
            match c {
                Control::Picture(p) => marks.push((p.common.z_order, "image".to_string())),
                Control::Shape(s) => {
                    if let ShapeObject::Rectangle(r) = s.as_ref() {
                        let label = r
                            .drawing
                            .text_box
                            .as_ref()
                            .map(|b| {
                                b.paragraphs
                                    .iter()
                                    .map(|p| p.text.as_str())
                                    .collect::<String>()
                            })
                            .unwrap_or_else(|| "rectangle".into());
                        marks.push((r.common.z_order, label));
                    }
                }
                _ => {}
            }
        }
        marks.sort_by_key(|m| m.0);
        let expected: Vec<&str> = (0..if page == 2 { 2 } else { 1 })
            .flat_map(|_| ["FIRST", "rectangle", "SECOND", "image"])
            .collect();
        assert_eq!(
            marks.iter().map(|m| m.1.as_str()).collect::<Vec<_>>(),
            expected
        );
        assert!(marks.windows(2).all(|w| w[0].0 < w[1].0));
        let svg = core.render_page_svg_native(page as u32).unwrap();
        let xml = roxmltree::Document::parse(&svg).unwrap();
        let text: String = xml
            .descendants()
            .filter(|n| n.is_text() && n.ancestors().any(|a| a.has_tag_name("text")))
            .filter_map(|n| n.text())
            .collect();
        assert_eq!(
            text.matches("FIRST").count(),
            if page == 2 { 2 } else { 1 },
            "covered source text remains editable and renderable in its proper plane"
        );
        assert_eq!(
            text.matches("SECOND").count(),
            if page == 2 { 2 } else { 1 }
        );
    }
}
#[test]
fn covered_text_keeps_its_source_owner_and_paint_order_through_lowering_and_saves() {
    let core = DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    check(&core);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        check(&DocumentCore::from_bytes(&bytes).unwrap());
    }
}
