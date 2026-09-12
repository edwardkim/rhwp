//! Copy-local fill preflight and detached model edits followed by one insertion.
use super::{
    validation, ParagraphBlockBudget, ParagraphBlockPathStep as Step,
    ParagraphBlockValidationError as Error, RepeatParagraphBlockRequest,
};
use crate::{document_core::DocumentCore, model::paragraph::Paragraph};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

const MAX_TARGETS: usize = 10_000;
const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;

/// Offsets are Unicode scalar indices, like Paragraph::insert_text_at, not UTF-16.
#[derive(Debug, Clone)]
pub enum TemplateFillTarget {
    TextRange {
        path: Vec<Step>,
        start: usize,
        end: usize,
    },
    /// A closed, same-paragraph ClickHere range, not a global field occurrence.
    Field {
        path: Vec<Step>,
        field_range_index: usize,
    },
}

#[derive(Debug, Clone)]
pub struct TemplateBinding {
    pub key: String,
    pub target: TemplateFillTarget,
}

#[derive(Debug, Clone)]
pub struct TemplateFillRequest {
    /// Count must equal records.len(); addresses refer to the input document.
    pub block: RepeatParagraphBlockRequest,
    pub bindings: Vec<TemplateBinding>,
    pub records: Vec<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateFillPreview {
    pub block: ParagraphBlockBudget,
    pub target_count: usize,
    /// UTF-8 replacement payload only. Derived runs/offsets must be budgeted at staging.
    pub replacement_text_bytes: usize,
}

struct Selection<'a> {
    path: &'a [Step],
    range: std::ops::Range<usize>,
}

fn error(path: &[Step], code: &str, detail: impl Into<String>) -> Error {
    Error {
        code: code.into(),
        path: path.to_vec(),
        detail: detail.into(),
    }
}

fn add_budget(total: &mut usize, amount: usize, limit: usize) -> Result<(), Error> {
    *total = total
        .checked_add(amount)
        .filter(|n| *n <= limit)
        .ok_or_else(|| error(&[], "fillBudget", "fill byte budget exceeded"))?;
    Ok(())
}

fn path(target: &TemplateFillTarget) -> &[Step] {
    match target {
        TemplateFillTarget::TextRange { path, .. } | TemplateFillTarget::Field { path, .. } => path,
    }
}

/// Closed interval contact is rejected for field boundaries and competing insertions.
fn touches(a: &std::ops::Range<usize>, b: &std::ops::Range<usize>) -> bool {
    a.start <= b.end && b.start <= a.end
}

fn select<'a>(binding: &'a TemplateBinding, para: &Paragraph) -> Result<Selection<'a>, Error> {
    let path = path(&binding.target);
    let (range, field_index) = match &binding.target {
        TemplateFillTarget::TextRange { start, end, .. } => (*start..*end, None),
        TemplateFillTarget::Field {
            field_range_index, ..
        } => {
            let fr = para
                .field_ranges
                .get(*field_range_index)
                .ok_or_else(|| error(path, "fillField", "field range does not exist"))?;
            if fr.inner_slot_count != 0 {
                return Err(error(path, "fillControl", "field contains control slots"));
            }
            (fr.start_char_idx..fr.end_char_idx, Some(*field_range_index))
        }
    };
    if range.start > range.end || range.end > para.text.chars().count() {
        return Err(error(
            path,
            "fillRange",
            "text range is outside the paragraph",
        ));
    }
    if !para.orphan_field_ends.is_empty() {
        return Err(error(
            path,
            "fillField",
            "cross-paragraph fields require a separate fill contract",
        ));
    }
    if para.controls.iter().enumerate().any(|(index, control)| {
        matches!(control, crate::model::control::Control::Field(_))
            && !para.field_ranges.iter().any(|fr| fr.control_idx == index)
    }) {
        return Err(error(
            path,
            "fillField",
            "field begin is not closed in this paragraph",
        ));
    }
    for (index, fr) in para.field_ranges.iter().enumerate() {
        if Some(index) != field_index && touches(&range, &(fr.start_char_idx..fr.end_char_idx)) {
            return Err(error(
                path,
                "fillField",
                "text target touches another field boundary",
            ));
        }
    }
    // A position fallback cannot prove that a replacement preserves control anchors.
    // Empty or stale offsets must not authorize destructive text replacement.
    if !para.controls.is_empty() {
        if para.char_offsets.len() != para.text.chars().count()
            || para.char_offsets.windows(2).any(|w| w[0] >= w[1])
        {
            return Err(error(
                path,
                "fillControl",
                "control positions lack exact character offsets",
            ));
        }
        let allowed = field_index.map(|i| para.field_ranges[i].control_idx);
        for (index, position) in para.control_text_positions().into_iter().enumerate() {
            if Some(index) != allowed && range.start <= position && position <= range.end {
                return Err(error(
                    path,
                    "fillControl",
                    "text target touches a control anchor",
                ));
            }
        }
    }
    Ok(Selection { path, range })
}

impl DocumentCore {
    /// Fill detached copies, then insert once. Any returned error leaves this core unchanged.
    /// This does not fill the source or flatten cell paragraphs, and does not use clipboard.
    pub fn repeat_and_fill_paragraph_block_native(
        &mut self,
        request: &TemplateFillRequest,
    ) -> Result<super::RepeatParagraphBlockResult, crate::error::HwpError> {
        let preview = self
            .validate_template_fill_native(request)
            .map_err(|e| super::invalid(e.to_string()))?;
        if request.block.count == 0 {
            return self.repeat_paragraph_block_prepared(&request.block, preview.block, |_, _| {
                Ok(Vec::new())
            });
        }
        // Bound scalar/UTF-16 scratch arrays as well as payload before cloning/editing.
        // This is a conservative working-structure budget, not a process RSS promise.
        let mut estimated = preview.block.structure_bytes;
        let scratch = preview
            .replacement_text_bytes
            .checked_mul(32)
            .ok_or_else(|| super::invalid("fill derived size overflow"))?;
        add_budget(
            &mut estimated,
            scratch,
            request.block.limits.max_structure_bytes,
        )
        .map_err(|e| super::invalid(e.to_string()))?;
        let source = &self.document.sections[request.block.section_index].paragraphs
            [request.block.source_start..request.block.source_end];
        let paras: HashMap<_, _> = validation::paragraphs(source, &request.block)
            .map_err(|e| super::invalid(e.to_string()))?
            .into_iter()
            .collect();
        let mut edits = Vec::new();
        for binding in &request.bindings {
            let p = paras
                .get(path(&binding.target))
                .ok_or_else(|| super::invalid("validated fill target disappeared"))?;
            let selection = select(binding, p).map_err(|e| super::invalid(e.to_string()))?;
            edits.push((binding, selection.range));
        }
        // All coordinates refer to the input. Right-to-left edits preserve later anchors.
        edits.sort_by(|a, b| b.1.start.cmp(&a.1.start));
        let mut measured = 0;
        let mut work_bytes = 0;
        self.repeat_paragraph_block_prepared(&request.block, preview.block, |copy, index| {
            let mut tables = Vec::new();
            for (binding, range) in &edits {
                let para =
                    super::template_edit::paragraph(copy, path(&binding.target), &mut tables)?;
                let value = &request.records[index][&binding.key];
                let target_style = para.char_shape_id_at(range.start);
                // Existing text also feeds char/offset scratch arrays. Payload alone
                // does not bound editing a long field with a short replacement.
                let mut working = estimated;
                let existing_scratch = para
                    .text
                    .len()
                    .checked_mul(32)
                    .ok_or_else(|| super::invalid("fill existing text size overflow"))?;
                add_budget(
                    &mut working,
                    existing_scratch,
                    request.block.limits.max_structure_bytes,
                )
                .map_err(|e| super::invalid(format!("fill copy {index}: {e}")))?;
                add_budget(
                    &mut work_bytes,
                    para.text.len(),
                    request.block.limits.max_structure_bytes,
                )
                .map_err(|e| super::invalid(format!("fill copy {index}: {e}")))?;
                match &binding.target {
                    TemplateFillTarget::Field {
                        field_range_index, ..
                    } => Self::replace_field_text_model(para, *field_range_index, value)?,
                    TemplateFillTarget::TextRange { .. } => {
                        para.delete_text_at(range.start, range.end - range.start);
                        para.insert_text_at(range.start, value);
                        para.replace_line_segs(Vec::new());
                    }
                }
                // Deletion intentionally keeps the surviving right run's style.
                // A template replacement instead inherits the selected start style,
                // while range application preserves the unselected right neighbor.
                if let Some(style) = target_style {
                    para.apply_char_shape_range(
                        range.start,
                        range.start + value.chars().count(),
                        style,
                    );
                }
            }
            let cost = super::owned::source(
                copy,
                request.block.limits.max_nodes / request.block.count,
                request.block.limits.max_depth,
            )?;
            let bytes = super::budget::measure(
                copy,
                1,
                request.block.limits.max_structure_bytes,
                cost.skipped_bytes,
            )?;
            add_budget(
                &mut measured,
                bytes,
                request.block.limits.max_structure_bytes,
            )
            .map_err(|e| super::invalid(format!("fill copy {index}: {e}")))?;
            Ok(tables)
        })
    }

    /// Validate all records and source-local selections without cloning or editing IR.
    /// This is a preflight only: no copies, generated IDs, files or layout are produced.
    pub fn validate_template_fill_native(
        &self,
        request: &TemplateFillRequest,
    ) -> Result<TemplateFillPreview, Error> {
        let r = &request.block;
        // Validate addresses/limits even for the no-op without scanning source content.
        self.validate_paragraph_block_native(&RepeatParagraphBlockRequest {
            count: 0,
            ..r.clone()
        })?;
        if request.records.len() != r.count || request.bindings.len() > MAX_TARGETS {
            return Err(error(
                &[],
                "fillCardinality",
                "record count mismatch or too many bindings",
            ));
        }
        if r.count > r.limits.max_copies {
            return Err(error(&[], "fillBudget", "copy count limit exceeded"));
        }
        let target_count = r
            .count
            .checked_mul(request.bindings.len())
            .filter(|n| *n <= MAX_TARGETS)
            .ok_or_else(|| error(&[], "fillBudget", "expanded target limit exceeded"))?;
        let mut keys = HashSet::new();
        let mut input_bytes = 0;
        for binding in &request.bindings {
            let p = path(&binding.target);
            if p.len() > r.limits.max_depth {
                return Err(error(&[], "fillBudget", "target path depth exceeded"));
            }
            add_budget(&mut input_bytes, std::mem::size_of_val(p), MAX_INPUT_BYTES)?;
            add_budget(&mut input_bytes, binding.key.len(), MAX_INPUT_BYTES)?;
            if binding.key.is_empty() || !keys.insert(binding.key.as_str()) {
                return Err(error(p, "fillKey", "binding key is empty or duplicated"));
            }
        }
        let mut fill_bytes = 0;
        for (index, record) in request.records.iter().enumerate() {
            if record.len() != keys.len() || record.keys().any(|key| !keys.contains(key.as_str())) {
                return Err(error(
                    &[],
                    "fillKey",
                    format!("record {index} has missing or extra keys"),
                ));
            }
            for (key, value) in record {
                add_budget(&mut input_bytes, key.len(), MAX_INPUT_BYTES)?;
                add_budget(&mut input_bytes, value.len(), MAX_INPUT_BYTES)?;
                add_budget(&mut fill_bytes, value.len(), r.limits.max_structure_bytes)?;
            }
        }
        let block = self.validate_paragraph_block_native(r)?;
        let mut total = block.structure_bytes;
        add_budget(&mut total, fill_bytes, r.limits.max_structure_bytes)?;
        if r.count == 0 {
            return Ok(TemplateFillPreview {
                block,
                target_count,
                replacement_text_bytes: fill_bytes,
            });
        }
        let source =
            &self.document().sections[r.section_index].paragraphs[r.source_start..r.source_end];
        let paragraphs: HashMap<_, _> = validation::paragraphs(source, r)?.into_iter().collect();
        let mut selections: HashMap<&[Step], Vec<Selection<'_>>> = HashMap::new();
        let mut inspected_bytes = 0;
        for binding in &request.bindings {
            let p = path(&binding.target);
            let para = paragraphs
                .get(p)
                .ok_or_else(|| error(p, "fillPath", "target is not a source-owned paragraph"))?;
            // Bound repeated scans of a long paragraph across many bindings too.
            add_budget(
                &mut inspected_bytes,
                para.text.len(),
                r.limits.max_structure_bytes,
            )?;
            let selected = select(binding, para)?;
            let previous = selections.entry(selected.path).or_default();
            if previous.iter().any(|old| {
                let a = &old.range;
                let b = &selected.range;
                if a.is_empty() || b.is_empty() {
                    touches(a, b)
                } else {
                    a.start < b.end && b.start < a.end
                }
            }) {
                return Err(error(p, "fillOverlap", "fill targets overlap"));
            }
            previous.push(selected);
        }
        Ok(TemplateFillPreview {
            block,
            target_count,
            replacement_text_bytes: fill_bytes,
        })
    }
}
