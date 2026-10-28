//! Preserve the existing #877 parser contract through the public entry point.
//! Source unit moved by T08; this is a record-alignment check, not a page-count pin.

#[test]
fn original_hwp3_sample16_preserves_record_alignment() {
    let bytes = include_bytes!("../../samples/hwp3-sample16.hwp");
    let document = rhwp::parse_document(bytes).expect("sample16 parse failed");
    let paragraphs: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
    // #877 independently compared this input with the HWP5 conversion (1058 paragraphs).
    // Retain the original regression's lower bound and execute the committed input.
    assert!(
        paragraphs >= 1000,
        "ch=6/7/8 alignment: {paragraphs} paragraphs"
    );
}
