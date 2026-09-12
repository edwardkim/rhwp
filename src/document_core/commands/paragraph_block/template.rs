//! Read-only compilation of copy-local fill targets. No mutation or layout acceptance.
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
