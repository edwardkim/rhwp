//! Cross-document structural preflight. This query never imports resources or mutates a core.
use super::{
    invalid, owned, validation, ParagraphBlockBudget, ParagraphBlockLimits,
    RepeatParagraphBlockRequest,
};
use crate::{document_core::DocumentCore, error::HwpError, model::document::Document};
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Request ceilings, not file-format limits or an RSS guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportParagraphBlockLimits {
    #[serde(default)]
    pub block: ParagraphBlockLimits,
    pub max_resource_metadata_bytes: usize,
    pub max_binary_bytes: usize,
}

impl Default for ImportParagraphBlockLimits {
    fn default() -> Self {
        Self {
            block: ParagraphBlockLimits::default(),
            max_resource_metadata_bytes: 32 * 1024 * 1024,
            max_binary_bytes: 64 * 1024 * 1024,
        }
    }
}

impl ImportParagraphBlockLimits {
    fn validate(&self) -> Result<(), HwpError> {
        self.block.validate()?;
        let hard = Self::default();
        for (name, value, ceiling) in [
            (
                "maxResourceMetadataBytes",
                self.max_resource_metadata_bytes,
                hard.max_resource_metadata_bytes,
            ),
            (
                "maxBinaryBytes",
                self.max_binary_bytes,
                hard.max_binary_bytes,
            ),
        ] {
            if value == 0 || value > ceiling {
                return Err(invalid(format!("{name} must be in 1..={ceiling}")));
            }
        }
        Ok(())
    }
}

/// Source and target indices belong to different documents, before this request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportParagraphBlockRequest {
    pub source_section: usize,
    pub source_start: usize,
    pub source_end: usize,
    pub target_section: usize,
    pub insert_before: usize,
    pub count: usize,
    #[serde(default)]
    pub limits: ImportParagraphBlockLimits,
}

/// Structural and reference preflight only. Resource byte loading, ID-space capacity,
/// staged remapping and save compatibility still have to pass before insertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportParagraphBlockPreview {
    pub target_section: usize,
    pub inserted: Range<usize>,
    pub added_paragraphs: usize,
    pub added_nodes: usize,
    pub structure_bytes: usize,
    pub mapping_bytes: usize,
    pub source_document_nodes: usize,
    pub target_document_nodes: usize,
    /// Always false for this structural query; not an executable import permit.
    pub resources_prepared: bool,
}

impl DocumentCore {
    /// Inspect a foreign block without constructing/cloning a source DocumentCore.
    /// No BinData load, DocInfo write, identity allocation, clipboard or event mutation.
    pub fn inspect_paragraph_block_import_native(
        &self,
        source: &Document,
        request: &ImportParagraphBlockRequest,
    ) -> Result<ImportParagraphBlockPreview, HwpError> {
        request.limits.validate()?;
        let target = self
            .document
            .sections
            .get(request.target_section)
            .ok_or_else(|| invalid("target section index out of range"))?;
        if request.insert_before > target.paragraphs.len() {
            return Err(invalid("target paragraph boundary out of range"));
        }
        // Reuse B's immutable source/reference contract. A synthetic source-end
        // boundary does not impose same-document insertion rules on the real target.
        let source_request = RepeatParagraphBlockRequest {
            section_index: request.source_section,
            source_start: request.source_start,
            source_end: request.source_end,
            insert_before: request.source_end,
            count: request.count,
            limits: request.limits.block,
        };
        let budget = validation::validate_document_block(source, &source_request)
            .map_err(|error| invalid(format!("source: {error}")))?;
        let inserted_end = request
            .insert_before
            .checked_add(budget.added_paragraphs)
            .ok_or_else(|| invalid("target paragraph boundary overflow"))?;
        target
            .paragraphs
            .len()
            .checked_add(budget.added_paragraphs)
            .ok_or_else(|| invalid("target paragraph count overflow"))?;
        let target_nodes = if request.count == 0 {
            0
        } else {
            let remaining = request
                .limits
                .block
                .max_document_nodes
                .checked_sub(budget.document_nodes)
                .ok_or_else(|| invalid("combined document node budget exceeded"))?;
            owned::document(&self.document, remaining)?
        };
        Ok(preview(request, budget, inserted_end, target_nodes))
    }
}

fn preview(
    request: &ImportParagraphBlockRequest,
    budget: ParagraphBlockBudget,
    end: usize,
    target_nodes: usize,
) -> ImportParagraphBlockPreview {
    ImportParagraphBlockPreview {
        target_section: request.target_section,
        inserted: request.insert_before..end,
        added_paragraphs: budget.added_paragraphs,
        added_nodes: budget.added_nodes,
        structure_bytes: budget.structure_bytes,
        mapping_bytes: budget.mapping_bytes,
        source_document_nodes: budget.document_nodes,
        target_document_nodes: target_nodes,
        resources_prepared: false,
    }
}
