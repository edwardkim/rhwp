//! Changing character size after an empty paragraph must not create a stored page break.

use rhwp::document_core::DocumentCore;

#[test]
fn char_size_after_empty_paragraph_stays_on_one_page() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp"))
        .expect("load blank HWP");
    core.insert_text_native(0, 0, 0, "Title").unwrap();
    core.apply_char_format_native(0, 0, 0, 5, r#"{"fontSize":2000}"#)
        .unwrap();
    for (index, text) in ["A", "B", "", "C", "D"].iter().enumerate() {
        core.insert_paragraph_native(0, index + 1).unwrap();
        if !text.is_empty() {
            core.insert_text_native(0, index + 1, 0, text).unwrap();
        }
    }
    for index in [2, 4] {
        core.apply_char_format_native(0, index, 0, 1, r#"{"fontSize":1300,"bold":true}"#)
            .unwrap();
    }

    let paragraphs = &core.document().sections[0].paragraphs;
    let empty_end = paragraphs[3].line_segs.last().unwrap();
    let following_start = paragraphs[4].line_segs.first().unwrap();
    assert!(
        following_start.vertical_pos >= empty_end.vertical_pos + empty_end.line_height,
        "following paragraph starts before the empty paragraph ends"
    );
    assert_eq!(core.page_count(), 1, "editing must not add a page");
    core.delete_paragraph_native(0, 3).unwrap();
    assert_eq!(core.page_count(), 1, "removing the empty paragraph must not add a page");
    let reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert_eq!(reopened.page_count(), 1, "saved HWP must stay on one page");
}

#[test]
fn real_stored_zero_still_starts_a_page() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample16-hwp5.hwp"))
        .expect("load stored multi-page HWP");
    assert_eq!(
        core.page_count(),
        64,
        "stored page boundaries must remain intact"
    );
}
