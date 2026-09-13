//! Whole-paragraph block automation with read-only preflight and staged insertion.
mod budget;
mod import;
mod owned;
mod repeat;
mod rows_geometry;
mod template;
mod template_edit;
mod template_form;
mod template_operation;
mod template_rows;
mod template_select;
mod validation;
pub use import::{
    ImportParagraphBlockLimits, ImportParagraphBlockPreview, ImportParagraphBlockRequest,
};
pub use repeat::{ParagraphBlockCopy, ParagraphBlockMapping, RepeatParagraphBlockResult};
pub use template::{TemplateBinding, TemplateFillPreview, TemplateFillRequest, TemplateFillTarget};
pub use template_form::{FillTemplateRequest, FillTemplateResult, TemplateScope};
pub use template_operation::{
    TemplateOperation, TemplateOperationResult, TEMPLATE_REQUEST_MAX_BYTES,
};
pub use template_rows::{RepeatTableRowsRequest, RepeatTableRowsResult};
pub use validation::{ParagraphBlockPathStep, ParagraphBlockValidationError};

use crate::{document_core::DocumentCore, error::HwpError};
use serde::{Deserialize, Serialize};

/// Request-local ceilings, not file-format limits or an RSS guarantee.
/// Callers may lower, but cannot raise, the hard ceilings in `Default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParagraphBlockLimits {
    pub max_copies: usize,
    pub max_paragraphs: usize,
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_structure_bytes: usize,
    pub max_mapping_bytes: usize,
    pub max_document_nodes: usize,
}

impl Default for ParagraphBlockLimits {
    fn default() -> Self {
        Self {
            max_copies: 1_000,
            max_paragraphs: 10_000,
            max_nodes: 100_000,
            max_depth: 64,
            max_structure_bytes: 32 * 1024 * 1024,
            max_mapping_bytes: 8 * 1024 * 1024,
            max_document_nodes: 1_000_000,
        }
    }
}

impl ParagraphBlockLimits {
    fn validate(self) -> Result<(), HwpError> {
        let hard = Self::default();
        for (name, value, ceiling) in [
            ("maxCopies", self.max_copies, hard.max_copies),
            ("maxParagraphs", self.max_paragraphs, hard.max_paragraphs),
            ("maxNodes", self.max_nodes, hard.max_nodes),
            ("maxDepth", self.max_depth, hard.max_depth),
            (
                "maxStructureBytes",
                self.max_structure_bytes,
                hard.max_structure_bytes,
            ),
            (
                "maxMappingBytes",
                self.max_mapping_bytes,
                hard.max_mapping_bytes,
            ),
            (
                "maxDocumentNodes",
                self.max_document_nodes,
                hard.max_document_nodes,
            ),
        ] {
            if value == 0 || value > ceiling {
                return Err(invalid(format!("{name} must be in 1..={ceiling}")));
            }
        }
        Ok(())
    }
}

/// All indices refer to the document before this request. Source is [start,end).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepeatParagraphBlockRequest {
    pub section_index: usize,
    pub source_start: usize,
    pub source_end: usize,
    pub insert_before: usize,
    pub count: usize,
    #[serde(default)]
    pub limits: ParagraphBlockLimits,
}

/// Conservative, architecture-local owned-structure cost, not serialized file size.
/// Successful budgeting does NOT yet imply that controls/references are clonable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphBlockBudget {
    pub added_paragraphs: usize,
    pub structure_bytes: usize,
    pub added_nodes: usize,
    pub owned_depth: usize,
    pub mapping_bytes: usize,
    pub document_nodes: usize,
    pub source_start_after: usize,
    pub source_end_after: usize,
    pub inserted_end: usize,
}

fn invalid(message: impl Into<String>) -> HwpError {
    HwpError::RenderError(format!("paragraph block: {}", message.into()))
}

impl DocumentCore {
    /// Inspect address/option/arithmetic and owned-memory budgets without cloning
    /// a paragraph, loading BinData, or touching caches, events, batch or clipboard.
    /// Strict control support, owned-path and reference validation is a separate
    /// required gate before insertion; this method alone never authorizes a clone.
    pub fn paragraph_block_budget_native(
        &self,
        request: &RepeatParagraphBlockRequest,
    ) -> Result<ParagraphBlockBudget, HwpError> {
        document_block_budget(&self.document, request)
    }
}

fn document_block_budget(
    document: &crate::model::document::Document,
    request: &RepeatParagraphBlockRequest,
) -> Result<ParagraphBlockBudget, HwpError> {
    let r = request;
    r.limits.validate()?;
    let section = document
        .sections
        .get(r.section_index)
        .ok_or_else(|| invalid("section index out of range"))?;
    if r.source_start >= r.source_end || r.source_end > section.paragraphs.len() {
        return Err(invalid(
            "source must be a nonempty in-range paragraph interval",
        ));
    }
    if r.insert_before > section.paragraphs.len()
        || (r.source_start < r.insert_before && r.insert_before < r.source_end)
    {
        return Err(invalid("destination is out of range or inside the source"));
    }
    let added = (r.source_end - r.source_start)
        .checked_mul(r.count)
        .ok_or_else(|| invalid("paragraph count overflow"))?;
    if r.count > r.limits.max_copies || added > r.limits.max_paragraphs {
        return Err(invalid("copy/paragraph budget exceeded"));
    }
    let inserted_end = r
        .insert_before
        .checked_add(added)
        .ok_or_else(|| invalid("destination overflow"))?;
    let shift = if r.insert_before <= r.source_start {
        added
    } else {
        0
    };
    let start = r
        .source_start
        .checked_add(shift)
        .ok_or_else(|| invalid("source start overflow"))?;
    let end = r
        .source_end
        .checked_add(shift)
        .ok_or_else(|| invalid("source end overflow"))?;
    // A zero-copy request does not inspect unsupported or large source content.
    let (structure_bytes, added_nodes, owned_depth, mapping_bytes, document_nodes) = if r.count == 0
    {
        (0, 0, 0, 0, 0)
    } else {
        let source = &section.paragraphs[r.source_start..r.source_end];
        let cost = owned::source(source, r.limits.max_nodes / r.count, r.limits.max_depth)?;
        let nodes = cost
            .nodes
            .checked_mul(r.count)
            .ok_or_else(|| invalid("owned node count overflow"))?;
        let mappings = cost
            .mapping_bytes
            .checked_mul(r.count)
            .ok_or_else(|| invalid("mapping size overflow"))?;
        if mappings > r.limits.max_mapping_bytes {
            return Err(invalid("mapping byte budget exceeded"));
        }
        let bytes = budget::measure(
            source,
            r.count,
            r.limits.max_structure_bytes,
            cost.skipped_bytes,
        )?;
        let document_nodes = owned::document(document, r.limits.max_document_nodes)?;
        (bytes, nodes, cost.depth, mappings, document_nodes)
    };
    Ok(ParagraphBlockBudget {
        added_paragraphs: added,
        structure_bytes,
        added_nodes,
        owned_depth,
        mapping_bytes,
        document_nodes,
        source_start_after: start,
        source_end_after: end,
        inserted_end,
    })
}
