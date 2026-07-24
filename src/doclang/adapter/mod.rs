//! Parser adapter — the **only** module layer permitted to import rhwp types.
//!
//! This module lowers a parsed rhwp `Document` into the crate's own
//! rhwp-agnostic Semantic IR (`crate::doclang::ir`).  Keeping rhwp types strictly
//! contained here means a rhwp version change only requires updating this
//! subtree, leaving the IR, writer, and loss modules untouched.
//!
//! [`build_sir`] is the orchestration entry point: it detects the input format,
//! parses it via rhwp, enforces the v1 Non-Goal boundary (encrypted /
//! distribution / unsupported-format rejection), then walks sections and
//! paragraphs through the sibling adapters to assemble a [`SirDocument`].

pub(crate) mod control;
pub(crate) mod geometry;
pub(crate) mod geometry_pass;
pub mod inline;
pub(crate) mod paragraph;
pub mod resources;
pub(crate) mod table;

use crate::model::document::Document;
use crate::parser::{detect_format, parse_document, FileFormat, ParseError};

use crate::doclang::error::ConvertError;
use crate::doclang::ir::prov::LocationMap;
use crate::doclang::ir::{Section, SirDocument};
use crate::doclang::loss::report::LossReport;
use crate::doclang::options::Mode;

/// Build the v2 `<location>` map for already-parsed `document` + raw `data`.
///
/// Re-layouts every page via the render tree and joins bounding boxes onto the
/// IR provenance keys. Returns an empty/partial map on layout failure and
/// records the failure into `loss` so the caller can tell why coverage is empty.
/// Only called when `ConvertOptions::with_location` is set.
pub(crate) fn build_location_map(
    data: &[u8],
    document: &Document,
    loss: &mut crate::doclang::loss::report::LossReport,
) -> LocationMap {
    geometry_pass::build_location_map(data, document, loss)
}

/// Detect, parse, and lower HWP/HWPX bytes into the Semantic IR.
///
/// # Format policy (v1 Non-Goal boundary)
///
/// 1. **Format detection** — only `Hwp` (HWP 5.0 binary) and `Hwpx` proceed.
///    `Hwp3`, `Hml`, `DrmProtected`, `Empty`, and `Unknown` are rejected with
///    [`ConvertError::UnsupportedFormat`] *before* parsing, giving a precise
///    error rather than a deep parse failure.
/// 2. **Encrypted documents** — rhwp itself refuses these
///    ([`ParseError::EncryptedDocument`]); we map that to
///    [`ConvertError::EncryptedDocument`].  Any other parse error becomes
///    [`ConvertError::Parse`].
/// 3. **Distribution (배포용) documents** — rhwp parses these *successfully*
///    (it decrypts the ViewText stream), so there is no capability gap.  v1
///    excludes them **by policy**: we check `document.header.distribution`
///    explicitly and return [`ConvertError::DistributionDocumentUnsupported`].
///
/// `mode` selects lean vs preserve lowering; the returned [`LossReport`] is
/// populated in lean mode with every property that DocLang cannot express.
pub fn build_sir(data: &[u8], mode: Mode) -> Result<(SirDocument, LossReport), ConvertError> {
    let (sir, loss, _doc) = build_sir_with_document(data, mode)?;
    Ok((sir, loss))
}

/// Like [`build_sir`] but also returns the parsed rhwp [`Document`], so callers
/// that need the model again (e.g. the v2 geometry pass) avoid a second parse.
pub(crate) fn build_sir_with_document(
    data: &[u8],
    mode: Mode,
) -> Result<(SirDocument, LossReport, Document), ConvertError> {
    // (1) Format gate — exhaustive match, reject everything that is not a
    // supported binary/HWPX container before attempting a parse.
    match detect_format(data) {
        FileFormat::Hwp | FileFormat::Hwpx => {}
        FileFormat::Hwp3 => return Err(ConvertError::UnsupportedFormat("HWP 3.x")),
        FileFormat::Hml => return Err(ConvertError::UnsupportedFormat("HWPML (HML)")),
        FileFormat::DrmProtected => {
            return Err(ConvertError::UnsupportedFormat("DRM-protected document"))
        }
        FileFormat::Empty => return Err(ConvertError::UnsupportedFormat("empty file")),
        FileFormat::Unknown => return Err(ConvertError::UnsupportedFormat("unknown file format")),
    }

    // (2) Parse — map rhwp's encrypted-document refusal to our typed error;
    // fold everything else into Parse(msg).
    let document = parse_document(data).map_err(|e| match e {
        ParseError::EncryptedDocument => ConvertError::EncryptedDocument,
        other => ConvertError::Parse(other.to_string()),
    })?;

    // (3) Distribution policy exclusion (NOT a capability gap — rhwp parsed it
    // fine; we decline to convert it in v1).
    if document.header.distribution {
        return Err(ConvertError::DistributionDocumentUnsupported);
    }

    // (4) Lower sections → paragraphs → blocks, with a document-wide footnote
    // counter so footnote numbers stay sequential across the whole document and
    // a document-wide thread counter so multi-column `<thread>` ids are unique.
    let mut loss = LossReport::new();
    let doc_info = &document.doc_info;
    let mut footnote_counter = 1usize;
    let mut thread_counter = 0usize;

    let mut sections: Vec<Section> = Vec::with_capacity(document.sections.len());
    for (si, section) in document.sections.iter().enumerate() {
        // Lower each paragraph independently, keeping the per-paragraph block
        // stream so the section assembler can correlate column-break markers
        // (carried on the paragraph) with the blocks they introduce.
        let mut per_para: Vec<Vec<crate::doclang::ir::Block>> =
            Vec::with_capacity(section.paragraphs.len());
        for (pi, para) in section.paragraphs.iter().enumerate() {
            let location = format!("s{}/p{}", si, pi);
            // Stamp model provenance `(si, pi)` so the geometry pass can later
            // join render-tree bounding boxes onto these body blocks.
            let blocks = paragraph::convert_paragraph_prov(
                para,
                &document,
                doc_info,
                mode,
                &mut footnote_counter,
                &location,
                &mut loss,
                Some((si, pi)),
            );
            per_para.push(blocks);
        }
        // Assemble the section: insert `<thread>` markers for multi-column flow
        // (and record the column-layout loss) when present, then apply list
        // grouping over the full block stream so that consecutive list-item
        // paragraphs coalesce into one Block::List.
        let section_loc = format!("s{}", si);
        let blocks = paragraph::assemble_section_with_threads(
            &section.paragraphs,
            per_para,
            &mut thread_counter,
            &section_loc,
            &mut loss,
        );
        sections.push(Section { blocks });
    }

    let sir = SirDocument {
        sections,
        doclang_version: crate::doclang::DOCLANG_VERSION,
    };
    Ok((sir, loss, document))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_format_rejected() {
        // Random bytes match no known signature.
        let err = build_sir(
            &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
            Mode::Lean,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ConvertError::UnsupportedFormat("unknown file format")
        ));
    }

    #[test]
    fn hwp3_format_rejected() {
        // HWP 3.0 binary prefix.
        let mut data = b"HWP Document File".to_vec();
        data.extend_from_slice(&[0u8; 16]);
        let err = build_sir(&data, Mode::Lean).unwrap_err();
        assert!(matches!(err, ConvertError::UnsupportedFormat("HWP 3.x")));
    }

    #[test]
    fn legacy_hwpml_format_rejected() {
        // Minimal HWPML XML preamble recognised by rhwp's detector.
        let data = br#"<?xml version="1.0" encoding="UTF-8"?><HWPML Version="2.8"></HWPML>"#;
        // Only assert rejection if rhwp classifies it as LegacyHwpml; otherwise
        // it falls through to Unknown — both are UnsupportedFormat, which is the
        // behaviour we care about.
        let err = build_sir(data, Mode::Lean).unwrap_err();
        assert!(matches!(err, ConvertError::UnsupportedFormat(_)));
    }

    #[test]
    fn truncated_cfb_is_parse_error_not_panic() {
        // Valid CFB signature so the format gate passes, but the body is
        // garbage → rhwp returns a parse error (never an encrypted/distribution
        // error, and never a panic).
        let mut data = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        data.extend_from_slice(&[0u8; 64]);
        let err = build_sir(&data, Mode::Lean).unwrap_err();
        assert!(
            matches!(err, ConvertError::Parse(_)),
            "expected Parse error, got {err:?}"
        );
    }
}
