mod adapter;
pub mod encoding;
mod envelope;
pub mod error;
mod reader;
mod warnings;

use crate::model::document::Document;

pub use encoding::HmlEncoding;
pub use envelope::PreservedFragment;
pub use error::HmlError;
pub use reader::HmlLimits;
pub use warnings::{HmlWarning, HmlWarningCode};

const SIGNATURE_PROBE_LIMIT: usize = 64 * 1024;

pub fn detect_hml_signature(bytes: &[u8]) -> bool {
    encoding::decode_prefix(bytes, SIGNATURE_PROBE_LIMIT)
        .is_some_and(|xml| reader::has_hwpml_root(&xml))
}

pub struct HmlMetadata {
    pub hwpml_version: Option<String>,
    pub sub_version: Option<String>,
    pub style: Option<String>,
    pub encoding: HmlEncoding,
    pub resource_count: usize,
}

pub struct HmlParseResult {
    pub document: Document,
    pub metadata: HmlMetadata,
    pub warnings: Vec<HmlWarning>,
    pub preserved_fragments: Vec<PreservedFragment>,
}

pub fn parse_hml(bytes: &[u8]) -> Result<HmlParseResult, HmlError> {
    parse_hml_with_limits(bytes, &HmlLimits::default())
}

pub fn parse_hml_with_limits(bytes: &[u8], limits: &HmlLimits) -> Result<HmlParseResult, HmlError> {
    let decoded = encoding::decode(bytes, limits.max_xml_bytes)?;
    let source = reader::read_hml(&decoded.text, limits)?;
    let version = Some(source.version.clone());
    let sub_version = source.sub_version.clone();
    let style = source.style.clone();
    let resource_count = source.resource_count;
    let warnings = source.warnings.clone();
    let preserved_fragments = source.preserved_fragments.clone();
    let document = adapter::into_document(source)?;
    Ok(HmlParseResult {
        document,
        metadata: HmlMetadata {
            hwpml_version: version,
            sub_version,
            style,
            encoding: decoded.encoding,
            resource_count,
        },
        warnings,
        preserved_fragments,
    })
}
