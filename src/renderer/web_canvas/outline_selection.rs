//! Atomic outline preparation for the explicit Canvas2D preview.

use std::collections::{HashMap, HashSet};

use crate::paint::{
    validate_text_variant_scope, LayerNodeKind, PageLayerTree, PaintOp, TextVariantKind,
};
use crate::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantRejectReport, TextVariantSelectionOptions,
    TextVariantSelectionReport, VariantRejectReason, VariantSelectedReason,
};

use super::glyph_outline::{self, PreparedGlyphOutline};

const MAX_PREPARED_BYTES: usize = 32 * 1024 * 1024;
const MAX_PREPARED_PARTS: usize = 4096;
const MAX_REPORTS: usize = 256;

#[derive(Default)]
pub(super) struct OutlineReplay {
    prepared: HashMap<usize, PreparedGlyphOutline>,
    selected_sources: HashSet<u32>,
    pub reports: Vec<TextVariantSelectionReport>,
    pub reports_dropped: usize,
}

impl OutlineReplay {
    pub fn prepared(&self, op: &PaintOp) -> Option<&PreparedGlyphOutline> {
        self.prepared.get(&(op as *const PaintOp as usize))
    }

    pub fn suppresses_fallback(&self, op: &PaintOp) -> bool {
        matches!(op, PaintOp::TextRun { source: Some(source), .. }
            if self.selected_sources.contains(&source.id.0))
    }

    pub fn release_prepared(&mut self) {
        // Pointer keys are valid only during this synchronous tree replay.
        self.prepared.clear();
        self.selected_sources.clear();
    }
}

fn reject_selected(report: &mut TextVariantSelectionReport, reason: VariantRejectReason) {
    if report.selected_variant_kind != Some(TextVariantKind::GlyphOutline) {
        return;
    }
    if let Some(variant_id) = report.selected_variant_id.take() {
        report.rejected_variants.push(TextVariantRejectReport {
            variant_id,
            variant_kind: TextVariantKind::GlyphOutline,
            reasons: vec![reason],
        });
    }
    report.selected_variant_id = Some("textRun".into());
    report.selected_variant_kind = Some(TextVariantKind::TextRun);
    report.selected_reason = VariantSelectedReason::DefaultTextRunFallback;
    report.fallback_required = true;
}

pub(super) fn prepare_tree(tree: &PageLayerTree, scale: f64) -> OutlineReplay {
    let mut state = OutlineReplay {
        reports: analyze_text_variant_selection(
            tree,
            TextVariantSelectionOptions::canvas2d_strict_outline(),
        ),
        ..Default::default()
    };
    state.reports_dropped = state.reports.len().saturating_sub(MAX_REPORTS);
    if validate_text_variant_scope(tree).is_err() {
        for report in &mut state.reports {
            reject_selected(report, VariantRejectReason::IncompleteVariantSet);
        }
        state.reports.truncate(MAX_REPORTS);
        return state;
    }

    let groups = state
        .reports
        .iter()
        .enumerate()
        .filter(|(_, report)| report.selected_variant_kind == Some(TextVariantKind::GlyphOutline))
        .map(|(index, report)| (report.equivalence_group.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut parts = HashMap::<usize, Vec<usize>>::new();
    let mut sources = HashMap::<usize, u32>::new();
    let mut retained_bytes = 0usize;
    let mut nodes = vec![&tree.root];
    while let Some(node) = nodes.pop() {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => nodes.extend(children.iter().rev()),
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                nodes.push(child);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    let PaintOp::GlyphOutline { bbox, outline, .. } = op else {
                        continue;
                    };
                    let Some(&index) = groups.get(&outline.variant.equivalence_group) else {
                        continue;
                    };
                    let report = &state.reports[index];
                    if report.selected_variant_kind != Some(TextVariantKind::GlyphOutline)
                        || report.selected_variant_id.as_deref()
                            != Some(outline.variant.variant_id.as_str())
                    {
                        continue;
                    }
                    let prepared = (state.prepared.len() < MAX_PREPARED_PARTS)
                        .then(|| {
                            glyph_outline::prepare(
                                outline,
                                *bbox,
                                &tree.resources,
                                scale,
                                MAX_PREPARED_BYTES - retained_bytes,
                            )
                        })
                        .flatten();
                    if let Some(prepared) = prepared {
                        let cost = prepared.byte_cost();
                        if cost <= MAX_PREPARED_BYTES - retained_bytes {
                            retained_bytes += cost;
                            let key = op as *const PaintOp as usize;
                            state.prepared.insert(key, prepared);
                            parts.entry(index).or_default().push(key);
                            sources.insert(index, outline.source.id.0);
                            continue;
                        }
                    }
                    if let Some(keys) = parts.remove(&index) {
                        for key in keys {
                            if let Some(prepared) = state.prepared.remove(&key) {
                                retained_bytes -= prepared.byte_cost();
                            }
                        }
                    }
                    sources.remove(&index);
                    reject_selected(
                        &mut state.reports[index],
                        VariantRejectReason::UnsupportedOutlinePayload,
                    );
                }
            }
        }
    }
    state.selected_sources.extend(sources.into_values());
    state.reports.truncate(MAX_REPORTS);
    state
}
