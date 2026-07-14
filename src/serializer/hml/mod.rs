mod body;
mod error;
mod fragments;
mod head;
mod preflight;
mod raw_fragment;
mod xml;

use crate::model::document::Document;
use crate::parser::HmlImportMetadata;

pub use error::{HmlExportError, HmlSaveBlocker};

use xml::XmlWriter;

pub(crate) use preflight::collect_blockers;

pub fn serialize_hml(
    document: &Document,
    metadata: &HmlImportMetadata,
) -> Result<Vec<u8>, HmlExportError> {
    preflight::validate_document(document, metadata)?;
    let mut writer = XmlWriter::default();
    writer.declaration();
    let mut root_attributes = vec![("Version", "2.91".to_string())];
    if let Some(sub_version) = &metadata.sub_version {
        root_attributes.push(("SubVersion", sub_version.clone()));
    }
    if let Some(style) = &metadata.style {
        root_attributes.push(("Style", style.clone()));
    }
    writer.open("HWPML", &root_attributes);
    head::write_head(&mut writer, document, metadata)?;
    body::write_body(&mut writer, document, metadata)?;
    write_tail(&mut writer, metadata);
    writer.close("HWPML");
    Ok(writer.finish())
}

fn write_tail(writer: &mut XmlWriter, metadata: &HmlImportMetadata) {
    writer.open("TAIL", &[]);
    fragments::write_at_anchor(writer, metadata, "TAIL", 0);
    writer.close("TAIL");
}
