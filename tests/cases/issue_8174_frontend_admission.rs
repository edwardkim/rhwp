//! Public replacements for frontend helper-unit admission/rollback checks.
//! No test-only production API, copied allocator or private serializer callbacks.
#![cfg(feature = "pdf-import")]

use rhwp::{
    parser::pdf::{parse_pdf_with_policy, PdfImportOptions, PdfImportPolicy},
    PdfImportSession,
};

#[test]
fn public_recognition_and_text_admission_never_publish_a_partial_document() {
    for source in [
        include_bytes!("../../pdf/issue7998/paragraph-origins.pdf").as_slice(),
        include_bytes!("../../pdf/issue7998/cell-nested.pdf").as_slice(),
        include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf").as_slice(),
    ] {
        let options = PdfImportOptions {
            include_table_inference: true,
            ..Default::default()
        };
        let expected = parse_pdf_with_policy(source, &options, PdfImportPolicy::default()).unwrap();
        let expected_document = rhwp::serializer::serialize_hwpx(&expected.document).unwrap();
        let expected_metadata = serde_json::to_value(&expected.metadata.pages).unwrap();
        let consumed = expected.metadata.resource_usage.decoded_bytes;
        assert!(consumed > 16);
        for cap in [1, consumed / 16, consumed / 2] {
            let mut policy = PdfImportPolicy::default();
            policy.backend.decoded_bytes = cap;
            let error = parse_pdf_with_policy(source, &options, policy)
                .err()
                .expect("partial IR must not be published");
            assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
            let recovered =
                parse_pdf_with_policy(source, &options, PdfImportPolicy::default()).unwrap();
            assert_eq!(
                rhwp::serializer::serialize_hwpx(&recovered.document).unwrap(),
                expected_document
            );
            assert_eq!(
                serde_json::to_value(&recovered.metadata.pages).unwrap(),
                expected_metadata
            );
        }
        // The fixed public metadata schema must retain numbers/strings/arrays,
        // independently encoded by serde_json rather than the internal value builder.
        assert!(expected_metadata.is_array());
        assert!(!expected.metadata.pages[0].source_spans.is_empty());
        assert_eq!(expected.metadata.pages[0].source_page, 0);
    }
}

fn info() -> rhwp::model::document::DocInfo {
    let mut info = rhwp::parse_document(include_bytes!("../../samples/hwp_table_test.hwp"))
        .unwrap()
        .doc_info;
    for faces in &mut info.font_faces {
        faces.push(rhwp::model::style::Font {
            name: "원본 글꼴 UTF-8 한글😀 ".repeat(2048),
            ..Default::default()
        });
    }
    info
}

#[test]
fn first_save_admits_real_doc_info_before_copying_and_preserves_the_live_ir() {
    let source = include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf");
    let mut ordinary = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut ordinary_core = ordinary.open(source, &Default::default()).unwrap();
    let initial = ordinary.resource_usage().decoded_bytes;
    ordinary_core.document_mut().doc_info = info();
    let expected = ordinary
        .export_hwpx(&mut ordinary_core, 4 * 1024 * 1024)
        .unwrap();
    assert!(ordinary.resource_usage().decoded_bytes > initial + 16_384);

    let cap = initial + 16_384;
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = cap;
    let mut limited = PdfImportSession::new(policy).unwrap();
    let mut core = limited.open(source, &Default::default()).unwrap();
    core.document_mut().doc_info = info();
    let before = live_state(core.document());
    assert!(limited.export_hwpx(&mut core, 4 * 1024 * 1024).is_err());
    assert_eq!(live_state(core.document()), before);
    assert!(limited.resource_usage().decoded_bytes <= cap);
    assert!(limited.resource_usage().failure.is_some());
    assert!(limited.export_hwp(&mut core, 4 * 1024 * 1024).is_err());
    assert_eq!(live_state(core.document()), before);

    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut restored = fresh.open(source, &Default::default()).unwrap();
    restored.document_mut().doc_info = info();
    assert_eq!(
        fresh.export_hwpx(&mut restored, 4 * 1024 * 1024).unwrap(),
        expected
    );
}

fn live_state(
    document: &rhwp::model::document::Document,
) -> (Vec<String>, Vec<Vec<String>>, Option<Vec<u8>>) {
    (
        document
            .sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .map(|p| p.text.clone())
            .collect(),
        document
            .doc_info
            .font_faces
            .iter()
            .map(|faces| faces.iter().map(|f| f.name.clone()).collect())
            .collect(),
        document.doc_info.raw_stream.clone(),
    )
}
