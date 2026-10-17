//! Request image-placement accounting and preservation, not pixel-position pins.
#![cfg(feature = "pdf-import")]

use rhwp::parser::pdf::{
    parse_pdf_with_options, parse_pdf_with_policy, PdfImportOptions, PdfImportPolicy,
};
const PDF: &[u8] = include_bytes!("../../pdf/issue7966/image-placement-controls.pdf");

#[test]
fn repeated_images_preserve_every_source_page_and_paint_placement() {
    let parsed = parse_pdf_with_options(PDF, &Default::default()).unwrap();
    assert_eq!(parsed.metadata.source_page_count, 3);
    let core = rhwp::DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    assert_eq!(core.page_count(), 3);
    for page in 0..3 {
        // The independent fixture emits 33 Do/BI paint operations on each page.
        let svg = core.render_page_svg_native(page).unwrap();
        assert_eq!(
            svg.matches("<image ").count(),
            33,
            "page {page}: repeated resource paints must not collapse or disappear"
        );
    }
}

#[test]
fn image_policy_counts_xobject_form_and_inline_before_decoding() {
    for page in 0..3 {
        let options = PdfImportOptions {
            source_pages: Some(vec![page]),
            ..PdfImportOptions::default()
        };
        let policy = PdfImportPolicy {
            max_document_image_placements: 33,
            ..Default::default()
        };
        assert_eq!(
            parse_pdf_with_policy(PDF, &options, policy)
                .unwrap()
                .document
                .sections
                .len(),
            1
        );
        let error = parse_pdf_with_policy(
            PDF,
            &options,
            PdfImportPolicy {
                max_document_image_placements: 32,
                ..policy
            },
        )
        .err()
        .unwrap();
        assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
        let details = error.diagnostics.unwrap();
        assert_eq!(details.source_page, Some(page));
        let limit = details.frontend_limit.unwrap();
        assert_eq!(limit.reason, "document image placements");
        assert_eq!(
            (limit.used, limit.requested, limit.limit),
            (Some(0), Some(33), Some(32))
        );
        // A rejected request cannot poison the following request.
        assert!(parse_pdf_with_policy(PDF, &options, policy).is_ok());
    }
}

#[test]
fn image_placement_budget_accumulates_only_across_selected_pages() {
    let policy = PdfImportPolicy {
        max_document_image_placements: 65,
        ..Default::default()
    };
    let error = parse_pdf_with_policy(PDF, &Default::default(), policy)
        .err()
        .unwrap();
    let details = error.diagnostics.unwrap();
    assert_eq!(details.source_page, Some(1));
    let limit = details.frontend_limit.unwrap();
    assert_eq!(
        (limit.used, limit.requested, limit.limit),
        (Some(33), Some(33), Some(65))
    );
    let options = PdfImportOptions {
        source_pages: Some(vec![0, 2]),
        ..PdfImportOptions::default()
    };
    let exact = PdfImportPolicy {
        max_document_image_placements: 66,
        ..policy
    };
    let result = parse_pdf_with_policy(PDF, &options, exact).unwrap();
    assert_eq!(
        result
            .metadata
            .pages
            .iter()
            .map(|p| p.source_page)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(result.document.sections.len(), 2);
    assert!(parse_pdf_with_policy(
        PDF,
        &options,
        PdfImportPolicy {
            max_document_image_placements: 0,
            ..policy
        }
    )
    .is_err());
}
