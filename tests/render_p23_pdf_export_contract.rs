use rhwp::error::HwpError;
use rhwp::wasm_api::HwpDocument;
use std::path::Path;

fn sample_doc() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/re-03-latin-only-hancom.hwp");
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    HwpDocument::from_bytes(&bytes).expect("load PDF export fixture")
}

fn assert_pdf_bytes(bytes: &[u8]) {
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|idx| idx + 1)
        .unwrap_or(0);
    let trimmed = &bytes[..end];

    assert!(
        bytes.starts_with(b"%PDF-"),
        "PDF export should return a PDF header"
    );
    assert!(
        trimmed.ends_with(b"%%EOF"),
        "PDF export should return a complete PDF file"
    );
}

#[test]
fn document_core_exports_single_page_pdf() {
    let doc = sample_doc();

    let pdf = doc
        .render_page_pdf_native(0)
        .expect("single page PDF export should succeed");

    assert_pdf_bytes(&pdf);
}

#[test]
fn document_core_exports_explicit_page_selection_pdf() {
    let doc = sample_doc();

    let pdf = doc
        .render_pages_pdf_native(&[0])
        .expect("page selection PDF export should succeed");

    assert_pdf_bytes(&pdf);
}

#[test]
fn document_core_exports_full_document_pdf() {
    let doc = sample_doc();

    let pdf = doc
        .render_document_pdf_native()
        .expect("full document PDF export should succeed");

    assert_pdf_bytes(&pdf);
}

#[test]
fn document_core_pdf_export_rejects_empty_page_selection() {
    let doc = sample_doc();

    let err = doc
        .render_pages_pdf_native(&[])
        .expect_err("empty page selection should be rejected");

    assert!(
        matches!(err, HwpError::RenderError(ref message) if message.contains("at least one page")),
        "unexpected error: {err:?}"
    );
}

#[test]
fn document_core_pdf_export_preserves_page_range_errors() {
    let doc = sample_doc();

    let err = doc
        .render_pages_pdf_native(&[0, 99])
        .expect_err("out-of-range pages should be rejected");

    assert!(
        matches!(err, HwpError::PageOutOfRange(99)),
        "unexpected error: {err:?}"
    );
}
