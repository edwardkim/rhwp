//! PDF signature/feature contracts; no typesetting golden or PDF fidelity claim.
use rhwp::parser::{detect_format, FileFormat};

#[test]
fn pdf_signature_is_detected_even_without_import_feature() {
    assert_eq!(detect_format(b"%PDF-1.4\n"), FileFormat::Pdf);
    for bytes in [b"%PDF".as_slice(), b"prefix%PDF-1.4", b"pdf"] {
        assert_eq!(detect_format(bytes), FileFormat::Unknown);
    }
}

#[test]
fn non_pdf_public_wrapper_has_no_pdf_import_diagnostics() {
    let document =
        rhwp::wasm_api::HwpDocument::from_bytes(include_bytes!("../../samples/hwp_table_test.hwp"))
            .unwrap();
    assert_eq!(document.get_pdf_import_diagnostics(), "null");
}

#[cfg(not(feature = "pdf-import"))]
#[test]
fn default_build_rejects_pdf_with_actionable_feature_code() {
    use rhwp::parser::{parse_document, parse_document_with_password, ParseError};
    let bytes = b"%PDF-1.4\n";
    for result in [
        parse_document(bytes),
        parse_document_with_password(bytes, b"unused"),
    ] {
        assert!(matches!(
            result,
            Err(ParseError::UnsupportedFormat {
                code: "PDF_IMPORT_DISABLED",
                format: "PDF",
                ..
            })
        ));
    }
    for result in [
        rhwp::DocumentCore::from_bytes(bytes),
        rhwp::DocumentCore::from_bytes_with_password(bytes, b"unused"),
    ] {
        let error = result
            .err()
            .expect("feature-off import must fail")
            .to_string();
        assert!(error.contains("PDF_IMPORT_DISABLED"), "{error}");
    }
    let error = rhwp::service::DocumentService::new()
        .open_bytes(bytes, &rhwp::service::OpenOptions::new())
        .err()
        .unwrap();
    assert!(
        matches!(
            error,
            rhwp::service::ServiceError::UnsupportedFormat {
                detected: FileFormat::Pdf
            }
        ),
        "disabled import is not a damaged document"
    );
    let error = error.to_string();
    assert!(error.contains("PDF_IMPORT_DISABLED"), "{error}");
}
