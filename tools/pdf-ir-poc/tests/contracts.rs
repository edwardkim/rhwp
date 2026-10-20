use rhwp::model::{control::Control, shape::ShapeObject};
use rhwp_pdf_oxide_poc::compile_page;

fn input(relative: &str) -> Vec<u8> {
    let repo = std::env::var("RHWP_POC_REPO").expect("set RHWP_POC_REPO to the rhwp checkout");
    std::fs::read(std::path::Path::new(&repo).join(relative)).unwrap()
}

fn check_roundtrip(source: &str, text_boxes: usize, images: usize, rules: usize) {
    let compiled = compile_page(&input(source), 0).unwrap();
    let loaded = rhwp::parse_document(&compiled.hwpx).unwrap();
    let original = &compiled.document.sections[0].paragraphs[0].controls;
    let restored: Vec<_> = loaded.sections[0].paragraphs[0]
        .controls
        .iter()
        .filter(|c| !matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)))
        .collect();
    assert_eq!(original.len(), restored.len());
    assert_eq!(compiled.report["lowering"]["textBoxes"], text_boxes);
    assert_eq!(compiled.report["lowering"]["pictures"], images);
    assert_eq!(compiled.report["lowering"]["straightRules"], rules);
    for (a, b) in original.iter().zip(restored) {
        match (a, b) {
            (Control::Shape(a), Control::Shape(b)) => {
                assert_eq!(
                    std::mem::discriminant(a.as_ref()),
                    std::mem::discriminant(b.as_ref())
                );
                assert_eq!(a.common().horizontal_offset, b.common().horizontal_offset);
                assert_eq!(a.common().vertical_offset, b.common().vertical_offset);
                assert_eq!(a.common().width, b.common().width);
                assert_eq!(a.common().height, b.common().height);
                match (a.as_ref(), b.as_ref()) {
                    (ShapeObject::Rectangle(a), ShapeObject::Rectangle(b)) => {
                        let a = a.drawing.text_box.as_ref().unwrap();
                        let b = b.drawing.text_box.as_ref().unwrap();
                        assert_eq!(a.paragraphs.len(), b.paragraphs.len());
                        assert_eq!(a.paragraphs[0].text, b.paragraphs[0].text);
                        assert_eq!(a.paragraphs[0].char_offsets, b.paragraphs[0].char_offsets);
                    }
                    (ShapeObject::Line(a), ShapeObject::Line(b)) => {
                        assert_eq!(a.start.x, b.start.x);
                        assert_eq!(a.start.y, b.start.y);
                        assert_eq!(a.end.x, b.end.x);
                        assert_eq!(a.end.y, b.end.y);
                    }
                    _ => panic!("unexpected shape in this POC"),
                }
            }
            (Control::Picture(a), Control::Picture(b)) => {
                assert_eq!(a.common.horizontal_offset, b.common.horizontal_offset);
                assert_eq!(a.common.vertical_offset, b.common.vertical_offset);
                assert_eq!(a.common.width, b.common.width);
                assert_eq!(a.common.height, b.common.height);
                assert_eq!(a.img_dim, b.img_dim);
                assert_eq!(a.image_attr.bin_data_id, b.image_attr.bin_data_id);
                assert_eq!(a.border_x, b.border_x);
                assert_eq!(a.border_y, b.border_y);
                assert_eq!(b.border_x, [0, 0, b.common.width as i32, 0]);
                assert_eq!(
                    b.border_y,
                    [
                        b.common.width as i32,
                        b.common.height as i32,
                        0,
                        b.common.height as i32
                    ]
                );
                assert_eq!((b.crop.right as u32, b.crop.bottom as u32), b.img_dim);
                assert!(b.crop.right > 0 && b.crop.bottom > 0);
            }
            _ => panic!("object type changed in serialization"),
        }
    }
    assert_eq!(
        compiled.document.bin_data_content.len(),
        loaded.bin_data_content.len()
    );
    for (a, b) in compiled
        .document
        .bin_data_content
        .iter()
        .zip(&loaded.bin_data_content)
    {
        assert_eq!(a.data.load(), b.data.load());
    }
}

#[test]
fn non_pdf_input_is_rejected() {
    assert!(compile_page(b"not a pdf", 0).is_err());
}

#[test]
fn selected_page_bounds_are_checked() {
    assert!(compile_page(&input("pdf/re-align-center-hancom-hwp-2020.pdf"), 1).is_err());
}

#[test]
fn text_page_ir_and_utf16_offsets_survive_hwpx() {
    check_roundtrip("pdf/re-align-center-hancom-hwp-2020.pdf", 3, 0, 0);
}

#[test]
fn rules_images_geometry_and_text_survive_hwpx() {
    check_roundtrip(
        "pdf/issue5875/nested_table_text_caption-2020.pdf",
        95,
        2,
        14,
    );
}

#[test]
fn text_boxes_have_explicit_zero_margins_and_single_line_reflow() {
    let compiled = compile_page(
        &input("pdf/issue5875/nested_table_text_caption-2020.pdf"),
        0,
    )
    .unwrap();
    assert_eq!(compiled.report["lowering"]["explicitZeroTextMargins"], 95);
    let loaded = rhwp::parse_document(&compiled.hwpx).unwrap();
    for c in &loaded.sections[0].paragraphs[0].controls {
        if let Control::Shape(s) = c {
            if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                assert_eq!(
                    (
                        tb.margin_left,
                        tb.margin_right,
                        tb.margin_top,
                        tb.margin_bottom
                    ),
                    (0, 0, 0, 0)
                );
                for p in &tb.paragraphs {
                    let ps = &loaded.doc_info.para_shapes[p.para_shape_id as usize];
                    assert_eq!(ps.attr2 & rhwp::model::style::PARA_ATTR2_LINE_WRAP_MASK, 1);
                    assert_eq!(p.line_segs.len(), 1);
                }
            }
        }
    }
}

#[test]
fn painted_axis_rules_preserve_the_stroked_area_and_other_objects() {
    let bytes = input("pdf/issue5875/nested_table_text_caption-2020.pdf");
    let normal = compile_page(&bytes, 0).unwrap();
    let painted = rhwp_pdf_oxide_poc::compile_page_painted_rules(&bytes, 0).unwrap();
    assert_eq!(painted.report["lowering"]["paintedRuleRectangles"], 14);
    let a = &normal.document.sections[0].paragraphs[0].controls;
    let b = &painted.document.sections[0].paragraphs[0].controls;
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
        if let Control::Shape(a) = a {
            if let ShapeObject::Line(line) = a.as_ref() {
                let Control::Shape(b) = b else {
                    panic!("painted area must be a shape")
                };
                let ShapeObject::Rectangle(rect) = b.as_ref() else {
                    panic!("axis stroke area must be rectangular")
                };
                assert!(rect.common.width > 0 && rect.common.height > 0);
                assert!(rect.drawing.text_box.is_none());
                let stroke = line.drawing.border_line.width;
                if line.common.width == 0 {
                    assert_eq!(rect.common.width, stroke as u32);
                    assert_eq!(rect.common.height, line.common.height);
                    assert!(
                        (rect.common.horizontal_offset as i64 * 2 + stroke as i64
                            - line.common.horizontal_offset as i64 * 2)
                            .abs()
                            <= 1
                    );
                } else {
                    assert_eq!(rect.common.height, stroke as u32);
                    assert_eq!(rect.common.width, line.common.width);
                    assert!(
                        (rect.common.vertical_offset as i64 * 2 + stroke as i64
                            - line.common.vertical_offset as i64 * 2)
                            .abs()
                            <= 1
                    );
                }
                assert_eq!(
                    rect.drawing.fill.solid.as_ref().unwrap().background_color,
                    line.drawing.border_line.color
                );
                continue;
            }
        }
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
    }
    let reopened = rhwp::parse_document(&painted.hwpx).unwrap();
    assert_eq!(reopened.bin_data_content.len(), 2);
    let count=reopened.sections[0].paragraphs[0].controls.iter().filter(|c|
        matches!(c,Control::Shape(s) if matches!(s.as_ref(),ShapeObject::Rectangle(r) if r.drawing.text_box.is_none()))).count();
    assert_eq!(count, 14);
}
