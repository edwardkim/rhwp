//! Fixed forms keep identity and stage only roots containing selected targets.
use super::{
    invalid,
    repeat::table_keys,
    template::{path, FillEdits},
    ParagraphBlockLimits, ParagraphBlockPathStep as Step, RepeatParagraphBlockRequest,
    TemplateBinding,
};
use crate::{
    document_core::{DocumentCore, TableTextReflowKey},
    error::HwpError,
    model::{event::DocumentEvent, paragraph::Paragraph},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Explicit request-local scope. Targets use paragraph indices relative to start.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateScope {
    pub section_index: usize,
    pub start: usize,
    pub end: usize,
    #[serde(default)]
    pub limits: ParagraphBlockLimits,
}

impl TemplateScope {
    /// Internal reuse of the bounded ownership/reference validator, not a copy operation.
    /// Fixed filling initially accepts the same closed supported tree as block copying.
    pub(super) fn inspection(&self) -> RepeatParagraphBlockRequest {
        RepeatParagraphBlockRequest {
            section_index: self.section_index,
            source_start: self.start,
            source_end: self.end,
            insert_before: self.end,
            count: 1,
            limits: self.limits,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FillTemplateRequest {
    pub scope: TemplateScope,
    pub bindings: Vec<TemplateBinding>,
    pub record: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FillTemplateResult {
    pub section_index: usize,
    /// Absolute owning paragraph indices. No paragraphs are inserted or removed.
    pub paragraphs: Vec<usize>,
    pub target_count: usize,
}

struct PreparedForm {
    result: FillTemplateResult,
    staged: Vec<Paragraph>,
    roots: Vec<usize>,
    old_keys: Vec<TableTextReflowKey>,
    reflowed: Vec<TableTextReflowKey>,
}

impl DocumentCore {
    /// Prepare and discard fixed-form edits without reallocating IDs or using clipboard.
    /// An empty binding+record pair validates the scope then makes no changes.
    pub fn preview_fill_template_native(
        &self,
        request: &FillTemplateRequest,
    ) -> Result<FillTemplateResult, HwpError> {
        Ok(self.prepare_template_form(request)?.result)
    }

    fn prepare_template_form(
        &self,
        request: &FillTemplateRequest,
    ) -> Result<PreparedForm, HwpError> {
        let scope = &request.scope;
        let inspection = scope.inspection();
        let preview = self
            .validate_template_parts(
                &inspection,
                &request.bindings,
                std::slice::from_ref(&request.record),
            )
            .map_err(|e| invalid(e.to_string()))?;
        let mut edits = FillEdits::prepare(self, &inspection, &request.bindings, &preview)?;
        let roots: Vec<usize> = request
            .bindings
            .iter()
            .map(|binding| match path(&binding.target).first() {
                Some(Step::Paragraph(index)) => Ok(*index),
                _ => Err(invalid("fill path has no paragraph root")),
            })
            .collect::<Result<BTreeSet<_>, _>>()?
            .into_iter()
            .collect();
        let result = FillTemplateResult {
            section_index: scope.section_index,
            paragraphs: roots.iter().map(|i| scope.start + i).collect(),
            target_count: request.bindings.len(),
        };
        if roots.is_empty() {
            return Ok(PreparedForm {
                result,
                roots,
                staged: vec![],
                old_keys: vec![],
                reflowed: vec![],
            });
        }
        // Never clone the whole Document or untouched roots in the scope.
        let source = &self.document.sections[scope.section_index].paragraphs;
        let mut staged = Vec::new();
        staged
            .try_reserve_exact(roots.len())
            .map_err(|_| invalid("form allocation failed"))?;
        let mut old_keys = Vec::new();
        for root in &roots {
            let para = &source[scope.start + root];
            old_keys.extend(table_keys(std::slice::from_ref(para), &inspection)?);
            staged.push(para.clone());
        }
        let mut reflowed = edits.apply_to_roots(
            &mut staged,
            &request.record,
            scope.limits,
            scope.limits.max_nodes,
            Some(&roots),
        )?;
        let new_keys = table_keys(&staged, &inspection)?;
        if old_keys.len() != new_keys.len() {
            return Err(invalid("form table ownership mismatch"));
        }
        reflowed.extend(old_keys.iter().zip(new_keys).filter_map(|(old, new)| {
            self.render_normalization
                .text_reflowed_tables
                .contains(old)
                .then_some(new)
        }));
        Ok(PreparedForm {
            result,
            staged,
            roots,
            old_keys,
            reflowed,
        })
    }

    /// Fill only selected owning roots, after the shared immutable preparation succeeds.
    pub fn fill_template_native(
        &mut self,
        request: &FillTemplateRequest,
    ) -> Result<FillTemplateResult, HwpError> {
        let scope = &request.scope;
        let PreparedForm {
            result,
            staged,
            roots,
            old_keys,
            reflowed,
        } = self.prepare_template_form(request)?;
        if roots.is_empty() {
            return Ok(result);
        }
        let event = DocumentEvent::TemplateFilled {
            section: scope.section_index,
            paragraphs: result.paragraphs.clone(),
            targets: result.target_count,
        };
        // Every recoverable error precedes mutation. Box addresses remain stable on move.
        self.render_normalization
            .text_reflowed_tables
            .try_reserve(reflowed.len())
            .map_err(|_| invalid("form provenance allocation failed"))?;
        self.event_log
            .try_reserve(1)
            .map_err(|_| invalid("form event allocation failed"))?;
        for (index, para) in result.paragraphs.iter().zip(staged) {
            self.document.sections[scope.section_index].paragraphs[*index] = para;
        }
        self.document.sections[scope.section_index].raw_stream = None;
        for old in old_keys {
            self.render_normalization.text_reflowed_tables.remove(&old);
        }
        self.render_normalization
            .text_reflowed_tables
            .extend(reflowed);
        // Only text-edited body roots need body-width reflow. A cell/textbox edit
        // must not reinterpret the untouched host's stored control-line geometry.
        for root in &roots {
            if request
                .bindings
                .iter()
                .any(|binding| path(&binding.target) == [Step::Paragraph(*root)])
            {
                self.reflow_paragraph(scope.section_index, scope.start + root);
            }
        }
        let hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[scope.section_index].paragraphs,
            result.paragraphs[0],
            None,
            None,
            &self.styles,
            self.dpi,
            hwp3_layout,
        );
        self.recompose_section(scope.section_index);
        self.paginate_if_needed();
        self.event_log.push(event);
        Ok(result)
    }
}
