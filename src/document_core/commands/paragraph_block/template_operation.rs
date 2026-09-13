//! Shared request and full dry-run boundary for CLI, MCP and WASM.
use super::{
    invalid, FillTemplateRequest, FillTemplateResult, RepeatParagraphBlockResult,
    RepeatTableRowsRequest, RepeatTableRowsResult, TemplateFillRequest,
};
use crate::{document_core::DocumentCore, error::HwpError};
use serde::{Deserialize, Serialize};

pub const TEMPLATE_REQUEST_MAX_BYTES: usize = 8 * 1024 * 1024;

/// One atomic operation. Multi-step address rebinding is intentionally not implied.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "action",
    content = "request",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum TemplateOperation {
    FillTemplate(FillTemplateRequest),
    RepeatAndFillParagraphBlock(TemplateFillRequest),
    RepeatAndFillTableRows(RepeatTableRowsRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "action", content = "result", rename_all = "snake_case")]
pub enum TemplateOperationResult {
    FillTemplate(FillTemplateResult),
    RepeatAndFillParagraphBlock(RepeatParagraphBlockResult),
    RepeatAndFillTableRows(RepeatTableRowsResult),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TemplateOptions {
    operation: TemplateOperation,
    #[serde(default)]
    dry_run: bool,
}

struct BoundedJson(Vec<u8>);
impl std::io::Write for BoundedJson {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > TEMPLATE_REQUEST_MAX_BYTES.saturating_sub(self.0.len()) {
            return Err(std::io::Error::other("template request JSON exceeds 8 MiB"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl TemplateOperation {
    /// Existing run plans are Values. Bound the encoded request before cloning/decoding it.
    pub fn from_parts(action: &str, request: &serde_json::Value) -> Result<Self, HwpError> {
        #[derive(Serialize)]
        struct Wire<'a> {
            action: &'a str,
            request: &'a serde_json::Value,
        }
        let mut encoded = BoundedJson(Vec::new());
        serde_json::to_writer(&mut encoded, &Wire { action, request })
            .map_err(|e| invalid(format!("template request JSON: {e}")))?;
        Self::from_json(std::str::from_utf8(&encoded.0).expect("JSON encoder emits UTF-8"))
    }

    /// Input workload only, not measured allocation, RSS or a layout prediction.
    pub fn workload(&self) -> Result<serde_json::Value, HwpError> {
        let (bindings, records): (_, &[_]) = match self {
            Self::FillTemplate(r) => (&r.bindings, std::slice::from_ref(&r.record)),
            Self::RepeatAndFillParagraphBlock(r) => (&r.bindings, r.records.as_slice()),
            Self::RepeatAndFillTableRows(r) => (&r.bindings, r.records.as_slice()),
        };
        let targets = bindings
            .len()
            .checked_mul(records.len())
            .ok_or_else(|| invalid("template workload overflow"))?;
        let bytes = records
            .iter()
            .flat_map(|r| r.values())
            .try_fold(0usize, |n, value| n.checked_add(value.len()))
            .ok_or_else(|| invalid("template workload overflow"))?;
        Ok(
            serde_json::json!({"records":records.len(),"targets":targets,"replacementTextBytes":bytes}),
        )
    }

    /// Cap bytes before JSON allocation. Serde's recursion limit is retained.
    pub fn from_json(json: &str) -> Result<Self, HwpError> {
        if json.len() > TEMPLATE_REQUEST_MAX_BYTES {
            return Err(invalid("template request JSON exceeds 8 MiB"));
        }
        serde_json::from_str(json).map_err(|e| invalid(format!("template request JSON: {e}")))
    }

    pub fn is_action(action: &str) -> bool {
        matches!(
            action,
            "fill_template" | "repeat_and_fill_paragraph_block" | "repeat_and_fill_table_rows"
        )
    }
}

impl DocumentCore {
    /// JSON adapter shared by the WASM wrapper and native boundary contracts.
    /// Results contain paths/counts only, never document text or saved-file claims.
    pub fn apply_template_operation_json_native(
        &mut self,
        options_json: &str,
    ) -> Result<String, HwpError> {
        if options_json.len() > TEMPLATE_REQUEST_MAX_BYTES {
            return Err(invalid("template request JSON exceeds 8 MiB"));
        }
        let options: TemplateOptions = serde_json::from_str(options_json)
            .map_err(|e| invalid(format!("template options JSON: {e}")))?;
        let workload = options.operation.workload()?;
        let result = if options.dry_run {
            self.preview_template_operation_native(&options.operation)?
        } else {
            self.execute_template_operation_native(&options.operation)?
        };
        Ok(serde_json::json!({
            "schemaVersion": "1.0", "dryRun": options.dry_run,
            "changedPages": null, "untrustedContent": true,
            "untrustedFields": ["operationResult"], "operationResult": result,
            "workload": workload,
        })
        .to_string())
    }

    /// Full detached preparation, not just address validation. No mutation or layout.
    pub fn preview_template_operation_native(
        &self,
        operation: &TemplateOperation,
    ) -> Result<TemplateOperationResult, HwpError> {
        Ok(match operation {
            TemplateOperation::FillTemplate(r) => {
                TemplateOperationResult::FillTemplate(self.preview_fill_template_native(r)?)
            }
            TemplateOperation::RepeatAndFillParagraphBlock(r) => {
                TemplateOperationResult::RepeatAndFillParagraphBlock(
                    self.preview_repeat_and_fill_paragraph_block_native(r)?,
                )
            }
            TemplateOperation::RepeatAndFillTableRows(r) => {
                TemplateOperationResult::RepeatAndFillTableRows(
                    self.preview_repeat_and_fill_table_rows_native(r)?,
                )
            }
        })
    }

    pub fn execute_template_operation_native(
        &mut self,
        operation: &TemplateOperation,
    ) -> Result<TemplateOperationResult, HwpError> {
        Ok(match operation {
            TemplateOperation::FillTemplate(r) => {
                TemplateOperationResult::FillTemplate(self.fill_template_native(r)?)
            }
            TemplateOperation::RepeatAndFillParagraphBlock(r) => {
                TemplateOperationResult::RepeatAndFillParagraphBlock(
                    self.repeat_and_fill_paragraph_block_native(r)?,
                )
            }
            TemplateOperation::RepeatAndFillTableRows(r) => {
                TemplateOperationResult::RepeatAndFillTableRows(
                    self.repeat_and_fill_table_rows_native(r)?,
                )
            }
        })
    }
}
