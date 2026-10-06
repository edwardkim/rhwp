//! Bounded transport for cross-document import. Source ownership stays external.
use super::{invalid, ImportParagraphBlockRequest, TEMPLATE_REQUEST_MAX_BYTES};
use crate::{document_core::DocumentCore, error::HwpError, model::document::Document};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImportOptions {
    request: ImportParagraphBlockRequest,
    #[serde(default)]
    dry_run: bool,
}

impl DocumentCore {
    /// Same full resource/identity preparation as native import, without a source clone.
    /// This edits memory only. Source bytes, file access and saving are not JSON options.
    pub fn import_paragraph_block_json_native(
        &mut self,
        source: &Document,
        options_json: &str,
    ) -> Result<String, HwpError> {
        if options_json.len() > TEMPLATE_REQUEST_MAX_BYTES {
            return Err(invalid("template request JSON exceeds 8 MiB"));
        }
        let options: ImportOptions = serde_json::from_str(options_json)
            .map_err(|e| invalid(format!("import options JSON: {e}")))?;
        let result = if options.dry_run {
            self.preview_paragraph_block_import_native(source, &options.request)?
        } else {
            self.import_paragraph_block_native(source, &options.request)?
        };
        // Native staging already validates serializability before commit. The envelope
        // contains only bounded paths/counts and constants, never source document text.
        Ok(serde_json::json!({
            "schemaVersion": crate::schema_registry::ENVELOPE_SCHEMA_VERSION,
            "dryRun": options.dry_run, "changedPages": null,
            "untrustedContent": true, "untrustedFields": ["operationResult"],
            "operationResult": {"action": "import_paragraph_block", "result": result},
        })
        .to_string())
    }
}
