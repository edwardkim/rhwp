//! Destination preservation contracts; no host application or file is launched.
use rhwp::document_core::hyperlink::HyperlinkTarget;
use rhwp::DocumentCore;

#[test]
fn all_destination_kinds_keep_original_spelling_through_hwp_and_hwpx() {
    for uri in [
        "https://example.com/한글?x=1",
        "mailto:user@example.com",
        "#목표",
        "../문서.pdf",
        r"C:\docs\문서.hwp",
        "rhwp-app:open/document",
    ] {
        let target = HyperlinkTarget::body(0, 0);
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "Link target").unwrap();
        core.insert_hyperlink_native(&target, 0, 4, uri).unwrap();
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            let links = reopened.hyperlinks_native(&target).unwrap();
            assert_eq!(links.len(), 1, "{uri}: link ownership survives saving");
            assert_eq!(
                links[0].uri, uri,
                "{uri}: destination spelling survives saving"
            );
            assert_eq!(reopened.page_count(), 1);
        }
    }
}
