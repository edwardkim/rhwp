use super::{
    invalid, owned, validation, ParagraphBlockBudget, ParagraphBlockPathStep as Step,
    RepeatParagraphBlockRequest,
};
use crate::{
    document_core::{DocumentCore, TableTextReflowKey},
    error::HwpError,
    model::{
        control::Control,
        event::DocumentEvent,
        identity::{used_instance_ids, Allocator},
        paragraph::Paragraph,
    },
};
use serde::Serialize;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphBlockMapping {
    /// Relative to the original block, not a DSEL address.
    pub source: Vec<Step>,
    /// Absolute body paragraph followed by owned steps, immediately after insertion.
    pub destination: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphBlockCopy {
    pub range: Range<usize>,
    pub mappings: Vec<ParagraphBlockMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepeatParagraphBlockResult {
    pub section_index: usize,
    pub inserted: Range<usize>,
    pub source_after: Range<usize>,
    pub copies: Vec<ParagraphBlockCopy>,
}

fn table_keys(
    paras: &[Paragraph],
    request: &RepeatParagraphBlockRequest,
) -> Result<Vec<TableTextReflowKey>, HwpError> {
    let mut keys = Vec::new();
    owned::inspect_source(
        paras,
        request.limits.max_nodes / request.count,
        request.limits.max_depth,
        |node| {
            if let owned::Node::Control(Control::Table(t)) = node {
                keys.push(TableTextReflowKey::from_table(t));
            }
            Ok(())
        },
    )?;
    Ok(keys)
}

impl DocumentCore {
    /// Repeat a supported whole-paragraph block without clipboard or boundary merges.
    /// Every returned Err precedes the single insertion. Process/OOM abort is not rollback.
    pub fn repeat_paragraph_block_native(
        &mut self,
        request: &RepeatParagraphBlockRequest,
    ) -> Result<RepeatParagraphBlockResult, HwpError> {
        let budget = self
            .validate_paragraph_block_native(request)
            .map_err(|e| invalid(e.to_string()))?;
        self.repeat_paragraph_block_prepared(request, budget, |_, _| Ok(Vec::new()))
    }

    /// The callback only edits detached copies. It cannot observe or mutate this core.
    pub(super) fn repeat_paragraph_block_prepared(
        &mut self,
        request: &RepeatParagraphBlockRequest,
        budget: ParagraphBlockBudget,
        mut fill: impl FnMut(&mut [Paragraph], usize) -> Result<Vec<TableTextReflowKey>, HwpError>,
    ) -> Result<RepeatParagraphBlockResult, HwpError> {
        let mut result = RepeatParagraphBlockResult {
            section_index: request.section_index,
            inserted: request.insert_before..budget.inserted_end,
            source_after: budget.source_start_after..budget.source_end_after,
            copies: Vec::new(),
        };
        if request.count == 0 {
            return Ok(result);
        }
        // Preflight bounded the entire original owned tree and reference arrays.
        // Reuse the shared identity namespaces, reserving the document only once.
        let mut allocator = Allocator {
            used: used_instance_ids(&self.document),
            next: 1,
        };
        let source = &self.document.sections[request.section_index].paragraphs
            [request.source_start..request.source_end];
        let paths = validation::paths(source, request).map_err(|e| invalid(e.to_string()))?;
        let source_tables = table_keys(source, request)?;
        let reflow_flags: Vec<_> = source_tables
            .iter()
            .map(|key| self.render_normalization.text_reflowed_tables.contains(key))
            .collect();
        let mut staged = Vec::new();
        staged
            .try_reserve_exact(budget.added_paragraphs)
            .map_err(|_| invalid("paragraph staging allocation failed"))?;
        result
            .copies
            .try_reserve_exact(request.count)
            .map_err(|_| invalid("copy result allocation failed"))?;
        let mut inherited = Vec::new();
        for copy_index in 0..request.count {
            // The original remains immutably borrowed for every copy. No JSON roundtrip.
            let mut copy = source.to_vec();
            super::super::clone_identity::reidentify_with_allocator(&mut copy, &mut allocator)?;
            inherited.extend(fill(&mut copy, copy_index)?);
            let copied_tables = table_keys(&copy, request)?;
            if copied_tables.len() != source_tables.len() {
                return Err(invalid("copied table ownership mismatch"));
            }
            inherited.extend(
                copied_tables
                    .into_iter()
                    .zip(&reflow_flags)
                    .filter_map(|(key, reflowed)| reflowed.then_some(key)),
            );
            let start = request.insert_before + copy_index * source.len();
            let mut mappings = Vec::new();
            mappings
                .try_reserve_exact(paths.len())
                .map_err(|_| invalid("path result allocation failed"))?;
            for path in &paths {
                let mut destination = path.clone();
                let Some(Step::Paragraph(index)) = destination.first_mut() else {
                    return Err(invalid("owned path has no paragraph root"));
                };
                *index += start;
                mappings.push(ParagraphBlockMapping {
                    source: path.clone(),
                    destination,
                });
            }
            result.copies.push(ParagraphBlockCopy {
                range: start..start + source.len(),
                mappings,
            });
            staged.extend(copy);
        }
        // All recoverable errors and complete return values precede mutation.
        self.render_normalization
            .text_reflowed_tables
            .try_reserve(inherited.len())
            .map_err(|_| invalid("reflow provenance allocation failed"))?;
        self.event_log
            .try_reserve(1)
            .map_err(|_| invalid("event allocation failed"))?;
        let section = &mut self.document.sections[request.section_index];
        section
            .paragraphs
            .try_reserve(budget.added_paragraphs)
            .map_err(|_| invalid("destination allocation failed"))?;
        section
            .paragraphs
            .splice(request.insert_before..request.insert_before, staged);
        section.raw_stream = None;
        self.render_normalization
            .text_reflowed_tables
            .extend(inherited);
        // Existing editor flow rule: fresh host vpos joins the surrounding flow.
        // No empty paragraph insertion, style substitution or table frame reset.
        let hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[request.section_index].paragraphs,
            request.insert_before,
            Some(result.inserted.clone()),
            None,
            &self.styles,
            self.dpi,
            hwp3_layout,
        );
        self.recompose_section(request.section_index);
        self.paginate_if_needed();
        self.event_log.push(DocumentEvent::ContentPasted {
            section: request.section_index,
            para: request.insert_before,
        });
        Ok(result)
    }
}
