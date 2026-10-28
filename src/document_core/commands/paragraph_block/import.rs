//! Cross-document preflight and staged boundary insertion.
mod remap;
mod resources;
use super::{
    invalid, owned, validation, ParagraphBlockBudget, ParagraphBlockLimits,
    RepeatParagraphBlockRequest,
};
use super::{ParagraphBlockCopy, ParagraphBlockMapping, ParagraphBlockPathStep as Step};
use crate::model::{
    identity::{used_instance_ids, Allocator},
    paragraph::Paragraph,
};
use crate::{document_core::DocumentCore, error::HwpError, model::document::Document};
pub use resources::ImportResourceCounts;
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportParagraphBlockResult {
    pub target_section: usize,
    pub inserted: Range<usize>,
    pub copies: Vec<ParagraphBlockCopy>,
    pub resources: ImportResourceCounts,
}

struct PreparedImport {
    result: ImportParagraphBlockResult,
    resources: Option<resources::Resources>,
    paragraphs: Vec<Paragraph>,
}

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
    /// Full native dry-run: stage the same resources, identities and return paths as execution.
    /// It is a snapshot query, not a persistent permit that can be applied to another head.
    pub fn preview_paragraph_block_import_native(
        &self,
        source: &Document,
        request: &ImportParagraphBlockRequest,
    ) -> Result<ImportParagraphBlockResult, HwpError> {
        Ok(self.prepare_paragraph_block_import(source, request)?.result)
    }

    /// No source mutation or whole-document clone. Every recoverable error precedes commit.
    pub fn import_paragraph_block_native(
        &mut self,
        source: &Document,
        request: &ImportParagraphBlockRequest,
    ) -> Result<ImportParagraphBlockResult, HwpError> {
        let prepared = self.prepare_paragraph_block_import(source, request)?;
        let Some(resources) = prepared.resources else {
            return Ok(prepared.result);
        };
        resources.reserve(&mut self.document)?;
        self.reserve_block_commit(request.target_section, prepared.paragraphs.len(), 0)?;
        let binary_changed = !resources.binaries.is_empty();
        resources.commit(&mut self.document);
        if binary_changed {
            self.bump_bin_data_epoch();
        }
        self.rebuild_resolved_styles();
        self.commit_block_content(
            request.target_section,
            request.insert_before,
            prepared.result.inserted.clone(),
            prepared.paragraphs,
            Vec::new(),
        );
        Ok(prepared.result)
    }

    fn prepare_paragraph_block_import(
        &self,
        source: &Document,
        request: &ImportParagraphBlockRequest,
    ) -> Result<PreparedImport, HwpError> {
        let budget = self.inspect_paragraph_block_import_native(source, request)?;
        let mut result = ImportParagraphBlockResult {
            target_section: request.target_section,
            inserted: budget.inserted,
            copies: vec![],
            resources: ImportResourceCounts::default(),
        };
        if request.count == 0 {
            return Ok(PreparedImport {
                result,
                resources: None,
                paragraphs: vec![],
            });
        }
        let source_request = RepeatParagraphBlockRequest {
            section_index: request.source_section,
            source_start: request.source_start,
            source_end: request.source_end,
            insert_before: request.source_end,
            count: request.count,
            limits: request.limits.block,
        };
        let reachable = validation::reachable_resources(source, &source_request)
            .map_err(|e| invalid(e.to_string()))?;
        // An implicit source outline must not silently become the target section's
        // explicit outline. Materializing the built-in definition needs its own contract.
        if source.sections[request.source_section]
            .section_def
            .outline_numbering_id
            == 0
            && self.document.sections[request.target_section]
                .section_def
                .outline_numbering_id
                != 0
            && reachable.iter().any(|resource| match resource {
                validation::Resource::Para(id) => {
                    let para = &source.doc_info.para_shapes[*id as usize];
                    para.head_type == crate::model::style::HeadType::Outline
                        && para.numbering_id == 0
                }
                _ => false,
            })
        {
            return Err(invalid(
                "implicit source outline cannot inherit a different target outline",
            ));
        }
        let resources = resources::Resources::prepare(
            source,
            &self.document,
            &reachable,
            source.sections[request.source_section]
                .section_def
                .outline_numbering_id,
            request.limits,
        )?;
        let original = &source.sections[request.source_section].paragraphs
            [request.source_start..request.source_end];
        let paths =
            validation::paths(original, &source_request).map_err(|e| invalid(e.to_string()))?;
        let mut allocator = Allocator {
            used: used_instance_ids(&self.document),
            next: 1,
        };
        let mut paragraphs = Vec::new();
        paragraphs
            .try_reserve_exact(budget.added_paragraphs)
            .map_err(|_| invalid("import staging allocation failed"))?;
        result
            .copies
            .try_reserve_exact(request.count)
            .map_err(|_| invalid("import result allocation failed"))?;
        for index in 0..request.count {
            let mut copy = original.to_vec();
            remap::paragraphs(&mut copy, &resources)?;
            super::super::clone_identity::reidentify_with_allocator(&mut copy, &mut allocator)?;
            let start = request.insert_before + index * original.len();
            let mut mappings = Vec::new();
            mappings
                .try_reserve_exact(paths.len())
                .map_err(|_| invalid("import mapping allocation failed"))?;
            for path in &paths {
                let mut destination = path.clone();
                let Some(Step::Paragraph(p)) = destination.first_mut() else {
                    return Err(invalid("invalid owned root"));
                };
                *p += start;
                mappings.push(ParagraphBlockMapping {
                    source: path.clone(),
                    destination,
                });
            }
            result.copies.push(ParagraphBlockCopy {
                range: start..start + original.len(),
                mappings,
            });
            paragraphs.extend(copy);
        }
        result.resources = resources.counts.clone();
        super::budget::measure(&paragraphs, 1, request.limits.block.max_structure_bytes, 0)?;
        // Native result is small and bounded by the existing mapping ceiling.
        // Serialize before commit so the future transport wrapper cannot first fail after mutation.
        serde_json::to_vec(&result).map_err(|e| invalid(e.to_string()))?;
        Ok(PreparedImport {
            result,
            resources: Some(resources),
            paragraphs,
        })
    }

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
