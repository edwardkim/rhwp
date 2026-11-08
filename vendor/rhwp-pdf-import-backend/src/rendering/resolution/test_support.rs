//! Test-only helpers shared across the resolution-stage unit tests.
//!
//! The colour, pipeline, and context test modules each used to carry an
//! identical `fixture_doc` builder that emits a minimal valid PDF with an
//! empty `/Pages` tree — enough for `PdfDocument::from_bytes` to parse,
//! which is the only thing the resolver tests need from a doc handle
//! (`ColorResolver` only dereferences `doc` when an ICCBased space or a
//! stream-backed tint transform sends it down `doc.resolve_object`,
//! cases the tests construct their own objects for). Consolidating into
//! one helper here means the fixture lives in one place and the three
//! call-sites just import it.
//!
//! Note: the module itself is gated `#[cfg(test)]` in `mod.rs`, so this
//! file only compiles for test builds; no inner `#![cfg(test)]` needed.

use crate::document::PdfDocument;

/// Minimal one-object-set PDF that parses cleanly into a [`PdfDocument`].
///
/// The body has a Catalog pointing at an empty Pages tree; nothing else.
/// Resolver stages never traverse the page tree, so the document body is
/// just a stub.
pub(crate) fn fixture_doc() -> PdfDocument {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\n");
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 3\n0000000000 65535 f \n");
    buf.extend_from_slice(format!("{:010} 00000 n \n", cat_off).as_bytes());
    buf.extend_from_slice(format!("{:010} 00000 n \n", pages_off).as_bytes());
    buf.extend_from_slice(
        format!("trailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    PdfDocument::from_bytes(buf).expect("fixture PDF parses")
}
