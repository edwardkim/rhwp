//! Issue #4961: bounded, read-only layout and paint font decision trace.

use serde::Deserialize;
use serde_json::json;

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::renderer::composer::is_lang_neutral;
use crate::renderer::font_decision::*;
use crate::renderer::layout::trace_char_width_decisions;
use crate::renderer::px_to_hwpunit;
use crate::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};
use crate::renderer::style_resolver::{
    detect_lang_category, lookup_font_name_decision, FontSubstitutionBoundary,
};

const DEFAULT_MAX_CHARACTERS: usize = 1024;
const MAX_CHARACTERS: usize = 4096;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TraceOptions {
    max_characters: Option<usize>,
}

fn options(options_json: &str) -> Result<TraceLimits, HwpError> {
    let parsed: TraceOptions = if options_json.trim().is_empty() {
        TraceOptions::default()
    } else {
        serde_json::from_str(options_json).map_err(|error| {
            HwpError::RenderError(format!("font decision trace options: {error}"))
        })?
    };
    let max_characters = parsed.max_characters.unwrap_or(DEFAULT_MAX_CHARACTERS);
    if !(1..=MAX_CHARACTERS).contains(&max_characters) {
        return Err(HwpError::RenderError(format!(
            "font decision trace maxCharacters must be in 1..={MAX_CHARACTERS}"
        )));
    }
    Ok(TraceLimits { max_characters })
}

fn collect_runs<'a>(node: &'a RenderNode, runs: &mut Vec<&'a TextRunNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        runs.push(run);
    }
    for child in &node.children {
        collect_runs(child, runs);
    }
}

fn run_language_slots(
    chars: &[char],
    neutral_only_fallback: Option<usize>,
) -> Vec<(usize, Option<usize>)> {
    if chars.is_empty() {
        return Vec::new();
    }
    // composer::split_runs_by_lang와 같은 초기 언어 및 중립 문자 상속 규칙.
    let all_default = chars.iter().all(|&ch| detect_lang_category(ch) == 0);
    let mut current = if chars.iter().all(|&ch| is_lang_neutral(ch)) {
        neutral_only_fallback.unwrap_or(0)
    } else {
        chars
            .iter()
            .map(|&ch| detect_lang_category(ch))
            .find(|&lang| lang != 0 || all_default)
            .unwrap_or(0)
    };
    chars
        .iter()
        .map(|&ch| {
            if is_lang_neutral(ch) {
                (current, Some(current))
            } else {
                current = detect_lang_category(ch);
                (current, None)
            }
        })
        .collect()
}

fn run_style_language_slot(core: &DocumentCore, run: &TextRunNode) -> Option<usize> {
    let style = core.styles.char_styles.get(run.char_shape_id? as usize)?;
    (0..crate::renderer::style_resolver::LANG_COUNT).find(|&slot| {
        style.font_family_for_lang(slot) == run.style.font_family
            && (style.letter_spacing_for_lang(slot) - run.style.letter_spacing).abs() < 1e-9
            && (style.ratio_for_lang(slot) - run.style.ratio).abs() < 1e-9
    })
}

fn nested_path(run: &TextRunNode) -> Vec<usize> {
    run.cell_context
        .as_ref()
        .map(|context| {
            context
                .path
                .iter()
                .flat_map(|entry| [entry.control_index, entry.cell_index, entry.cell_para_index])
                .collect()
        })
        .unwrap_or_default()
}

fn source_paragraph_index(run: &TextRunNode) -> Option<usize> {
    run.cell_context
        .as_ref()
        .map(|context| context.parent_para_index)
        .or(run.para_index)
        // Header/footer and note layout use usize::MAX-relative internal markers.
        // They are not document paragraph coordinates and differ on wasm32/native.
        .filter(|index| *index < usize::MAX.saturating_sub(4096))
}

fn source_matches(core: &DocumentCore, run: &TextRunNode, offset: usize, ch: char) -> bool {
    let (Some(section), Some(paragraph), Some(char_start)) = (
        run.section_index,
        source_paragraph_index(run),
        run.char_start,
    ) else {
        return false;
    };
    let path: Vec<(usize, usize, usize)> = run
        .cell_context
        .as_ref()
        .map(|context| {
            context
                .path
                .iter()
                .map(|entry| (entry.control_index, entry.cell_index, entry.cell_para_index))
                .collect()
        })
        .unwrap_or_default();
    core.resolve_control_para(section, paragraph, &path)
        .ok()
        .and_then(|para| para.text.chars().nth(char_start + offset))
        .is_some_and(|source| source == ch)
}

fn measurement_provenance() -> Result<ProvenanceDecision, HwpError> {
    linked_provenance(
        json!({
            "sourceBoundaryId": "rust-measurement.estimate-width",
            "candidateKind": "predicate",
            "sourceFace": null,
            "targetOrPolicy": "estimate cluster and character advances using embedded metrics and guarded heuristics",
            "conditions": {},
            "order": null,
        }),
        "rust-measurement",
        "unknown",
        "inferred",
        vec!["The predicate selects advances; it does not prove glyph identity.".into()],
    )
    .map_err(|error| HwpError::RenderError(format!("font trace provenance: {error}")))
}

fn style_provenance(
    boundary: FontSubstitutionBoundary,
    source: &str,
    target: &str,
    _language_slot: usize,
) -> Result<ProvenanceDecision, HwpError> {
    let language = boundary.language_condition(source);
    linked_provenance(
        json!({
            "sourceBoundaryId": boundary.source_boundary_id(),
            "candidateKind": "finite-mapping",
            "sourceFace": source,
            "targetOrPolicy": target,
            "conditions": { "languageSlot": language },
            "order": null,
        }),
        "rust-style-resolution",
        "style-fallback",
        "verified-by-test",
        vec!["Name compatibility does not establish SFNT identity or metric equivalence.".into()],
    )
    .map_err(|error| HwpError::RenderError(format!("font trace provenance: {error}")))
}

fn subst_font_provenance(
    source: &str,
    target: &str,
    language_slot: usize,
) -> Result<ProvenanceDecision, HwpError> {
    unlinked_provenance(
        json!({
            "sourceBoundaryId": "rust-style-resolution.document-subst-font",
            "candidateKind": "document-declared-substitution",
            "sourceFace": source,
            "targetOrPolicy": target,
            "conditions": { "languageSlot": language_slot.to_string() },
            "order": null,
        }),
        "rust-style-resolution",
    )
    .map_err(|error| HwpError::RenderError(format!("font trace provenance: {error}")))
}

fn metric_alias_provenance(source: &str, target: &str) -> Result<ProvenanceDecision, HwpError> {
    let is_surrogate =
        source == "HY각헤드라인M" || target == "Pretendard" || target == "Noto Serif KR";
    linked_provenance(
        json!({
            "sourceBoundaryId": "rust-metric.metric-alias",
            "candidateKind": "finite-mapping",
            "sourceFace": source,
            "targetOrPolicy": target,
            "conditions": {},
            "order": null,
        }),
        "rust-metric",
        if is_surrogate {
            "metric-surrogate"
        } else {
            "unknown"
        },
        if is_surrogate {
            "historical"
        } else {
            "unknown"
        },
        if is_surrogate {
            vec!["Alias selection does not prove glyph-outline identity.".into()]
        } else {
            Vec::new()
        },
    )
    .map_err(|error| HwpError::RenderError(format!("font trace provenance: {error}")))
}

fn metric_entry_provenance(
    metric: &crate::renderer::font_metrics_data::MetricLookupDecision<'_>,
) -> Result<ProvenanceDecision, HwpError> {
    linked_provenance(
        json!({
            "sourceBoundaryId": "rust-metric.metric-table",
            "candidateKind": "metric-entry",
            "sourceFace": metric.metric.name,
            "targetOrPolicy": format!("metric-entry:{}", metric.entry_index),
            "conditions": {
                "bold": metric.metric.bold,
                "italic": metric.metric.italic,
                "emSize": metric.metric.em_size,
                "latinRangeSymbol": format!("FONT_{}_LATIN_RANGES", metric.entry_index),
                "hangulSymbol": metric.metric.hangul.as_ref().map(|_| format!("FONT_{}_HANGUL", metric.entry_index)),
            },
            "order": null,
        }),
        "rust-metric",
        "metric-entry",
        "verified-by-test",
        Vec::new(),
    )
    .map_err(|error| HwpError::RenderError(format!("font trace provenance: {error}")))
}

fn transform_steps(
    run: &TextRunNode,
    ch: char,
    decision: &crate::renderer::layout::CharWidthDecision<'_>,
) -> Vec<DecisionStep> {
    if matches!(
        decision.width_source,
        "clusterContinuation" | "inlineObjectPlaceholder" | "hwpPuaFiller"
    ) {
        return vec![DecisionStep {
            kind: "zeroAdvance".into(),
            input: Some(ch.to_string()),
            output: Some("0".into()),
            reason: Some(decision.width_source.into()),
        }];
    }
    if decision.width_source == "tabAdvance" {
        return vec![DecisionStep {
            kind: "tabContextAdvance".into(),
            input: Some(ch.to_string()),
            output: Some(decision.final_width_px.to_string()),
            reason: Some("computedFromCurrentPositionAndTabPolicy".into()),
        }];
    }
    let mut steps = Vec::new();
    if decision.metric.is_some_and(|metric| metric.bold_fallback) {
        steps.push(DecisionStep {
            kind: "boldFallback".into(),
            input: Some("bold".into()),
            output: Some("regularMetricAdvance".into()),
            reason: Some("fauxBoldDoesNotChangeLayoutAdvance".into()),
        });
    }
    if (run.style.ratio - 1.0).abs() > f64::EPSILON {
        steps.push(DecisionStep {
            kind: "ratio".into(),
            input: Some(decision.base_width_px.to_string()),
            output: Some(run.style.ratio.to_string()),
            reason: Some("documentCharacterWidthRatio".into()),
        });
    }
    for (kind, value, applies) in [
        ("letterSpacing", run.style.letter_spacing, true),
        ("extraCharacterSpacing", run.style.extra_char_spacing, true),
        ("extraWordSpacing", run.style.extra_word_spacing, ch == ' '),
        (
            "extraDashAdvance",
            run.style.extra_dash_advance,
            decision.dash_leader,
        ),
    ] {
        if applies && value.abs() > f64::EPSILON {
            steps.push(DecisionStep {
                kind: kind.into(),
                input: Some(value.to_string()),
                output: None,
                reason: Some("layoutSpacingAdjustment".into()),
            });
        }
    }
    if decision.dash_leader {
        steps.push(DecisionStep {
            kind: "dashLeaderClamp".into(),
            input: Some(decision.base_width_px.to_string()),
            output: Some(decision.final_width_px.to_string()),
            reason: Some("elasticDashLeader".into()),
        });
    }
    if decision.negative_spacing_clamped {
        steps.push(DecisionStep {
            kind: "negativeSpacingClamp".into(),
            input: None,
            output: Some(decision.final_width_px.to_string()),
            reason: Some("preventReverseAdvance".into()),
        });
    }
    steps
}

impl DocumentCore {
    pub fn get_font_decision_trace_native(
        &self,
        page_num: u32,
        options_json: &str,
    ) -> Result<String, HwpError> {
        let native_unavailable_reason =
            if cfg!(all(not(target_arch = "wasm32"), feature = "native-skia")) {
                "nativeRendererSnapshotRequired"
            } else {
                "nativeSkiaFeatureUnavailable"
            };
        self.get_font_decision_trace_with_native_observer(
            page_num,
            options_json,
            None,
            native_unavailable_reason,
        )
    }

    pub(crate) fn get_font_decision_trace_with_native_observer(
        &self,
        page_num: u32,
        options_json: &str,
        native_observer: Option<
            &dyn Fn(&str, char, bool, bool) -> crate::renderer::font_decision::BackendDecision,
        >,
        native_unavailable_reason: &'static str,
    ) -> Result<String, HwpError> {
        if page_num >= self.page_count() {
            return Err(HwpError::PageOutOfRange(page_num));
        }
        let requested_limits = options(options_json)?;
        let applied_limits = requested_limits;
        let tree = self.build_page_tree(page_num)?;
        let mut runs = Vec::new();
        collect_runs(&tree.root, &mut runs);
        let characters_seen = runs
            .iter()
            .map(|run| run.text.chars().count())
            .sum::<usize>();
        let mut records = Vec::with_capacity(characters_seen.min(applied_limits.max_characters));
        let mut source_mapping_mismatch = false;
        let mut ledger_rule_missing = false;
        'runs: for (run_index, run) in runs.iter().enumerate() {
            let chars: Vec<char> = run.text.chars().collect();
            let language_slots = run_language_slots(&chars, run_style_language_slot(self, run));
            let width_decisions = trace_char_width_decisions(&run.text, &run.style);
            for (run_offset, ((&ch, &(language_slot, inherited)), width)) in chars
                .iter()
                .zip(language_slots.iter())
                .zip(width_decisions.iter())
                .enumerate()
            {
                if records.len() >= applied_limits.max_characters {
                    break 'runs;
                }
                let source_complete = source_matches(self, run, run_offset, ch);
                source_mapping_mismatch |= !source_complete;
                let font_decision = run.char_shape_id.and_then(|char_shape_id| {
                    self.document
                        .doc_info
                        .char_shapes
                        .get(char_shape_id as usize)
                        .map(|shape| {
                            lookup_font_name_decision(
                                &self.document.doc_info,
                                language_slot,
                                shape.font_ids[language_slot],
                            )
                        })
                });
                let mut provenance = vec![measurement_provenance()?];
                let mut name_steps = Vec::new();
                if let Some(font) = &font_decision {
                    if let (Some(boundary), Some(source), Some(target)) = (
                        font.substitution_boundary,
                        font.requested_face.as_deref(),
                        font.normalized_face.as_deref(),
                    ) {
                        name_steps.push(DecisionStep {
                            kind: "styleSubstitution".into(),
                            input: Some(source.into()),
                            output: Some(target.into()),
                            reason: Some(boundary.source_boundary_id().into()),
                        });
                        provenance.push(style_provenance(
                            boundary,
                            source,
                            target,
                            font.language_slot,
                        )?);
                    }
                    if let (Some(source), Some(substitute)) =
                        (font.normalized_face.as_deref(), font.subst_font.as_deref())
                    {
                        name_steps.push(DecisionStep {
                            kind: "documentSubstFont".into(),
                            input: Some(source.into()),
                            output: Some(substitute.into()),
                            reason: Some("nonEmbeddedDocumentSubstitute".into()),
                        });
                        provenance.push(subst_font_provenance(
                            source,
                            substitute,
                            font.language_slot,
                        )?);
                        ledger_rule_missing = true;
                    }
                }
                if let Some(metric) = width.metric {
                    if metric.requested_name != metric.alias_resolved_name {
                        provenance.push(metric_alias_provenance(
                            metric.requested_name,
                            metric.alias_resolved_name,
                        )?);
                    }
                    provenance.push(metric_entry_provenance(&metric)?);
                }
                let requested_face = width
                    .metric
                    .map(|metric| metric.requested_name.to_string())
                    .or_else(|| {
                        run.style
                            .font_family
                            .split(',')
                            .next()
                            .map(|face| face.trim().to_string())
                    });
                let layout_metric = LayoutMetricDecision {
                    requested_face: requested_face.clone(),
                    alias_resolved_face: width
                        .metric
                        .map(|metric| metric.alias_resolved_name.to_string())
                        .or_else(|| requested_face.clone()),
                    match_kind: width
                        .metric
                        .map(|metric| metric.match_kind.as_str().to_string())
                        .unwrap_or_else(|| "none".into()),
                    metric_entry: width.metric.map(|metric| metric.entry_index),
                    character_match: width.character_match.into(),
                    width_source: width.width_source.into(),
                    base_advance_hwpunit: Some(px_to_hwpunit(width.base_width_px, self.dpi)),
                    transforms: transform_steps(run, ch, width),
                    final_advance_hwpunit: Some(px_to_hwpunit(width.final_width_px, self.dpi)),
                };
                let source_offset = run.char_start.map(|start| start + run_offset);
                let requested_document_face = font_decision
                    .as_ref()
                    .and_then(|font| font.requested_face.clone());
                records.push(TraceRecord {
                    record_id: format!("page:{page_num}:run:{run_index}:char:{run_offset}"),
                    source: SourceDecision {
                        status: if source_complete {
                            "complete"
                        } else {
                            "unavailable"
                        }
                        .into(),
                        section_index: run.section_index,
                        paragraph_index: source_paragraph_index(run),
                        nested_path: nested_path(run),
                        run_index: Some(run_index),
                        char_offset: source_offset,
                        character: ch.to_string(),
                        code_point: ch as u32,
                        char_shape_id: run.char_shape_id,
                    },
                    document: DocumentDecision {
                        language_slot: Some(language_slot),
                        inherited_language_slot: inherited,
                        face: requested_document_face.clone(),
                        alt_type: font_decision.as_ref().and_then(|font| font.alt_type),
                        embedded: font_decision.as_ref().and_then(|font| font.embedded),
                        subst_font: font_decision
                            .as_ref()
                            .and_then(|font| font.subst_font.clone()),
                    },
                    layout_name: LayoutNameDecision {
                        requested_face: requested_document_face,
                        normalized_face: font_decision
                            .as_ref()
                            .and_then(|font| font.normalized_face.clone()),
                        css_family_chain: font_decision
                            .as_ref()
                            .map(|font| font.css_family_chain.clone())
                            .unwrap_or_default(),
                        steps: name_steps,
                    },
                    paint: PaintDecision::stage3(
                        Some(run.style.font_family.clone()),
                        native_observer.map(|observe| {
                            observe(&run.style.font_family, ch, run.style.bold, run.style.italic)
                        }),
                        native_unavailable_reason,
                    ),
                    layout_metric,
                    provenance,
                    oracle: OracleDecision::not_provided(),
                });
            }
        }

        let records_omitted = characters_seen.saturating_sub(records.len());
        let truncated = records_omitted > 0;
        let mut reasons = vec![TraceReason {
            code: "backendUnsupported".into(),
            detail: Some(
                "Studio Canvas2D and CanvasKit observations require a current renderer snapshot."
                    .into(),
            ),
        }, TraceReason {
            code: "ledgerSourceDrift".into(),
            detail: Some("W1 candidate identities still join, but their recorded Rust source digests predate this Stage 2 trace-only refactor.".into()),
        }];
        if truncated {
            reasons.push(TraceReason {
                code: "characterLimitExceeded".into(),
                detail: Some(format!(
                    "{} characters omitted by maxCharacters={}",
                    records_omitted, applied_limits.max_characters
                )),
            });
        }
        if source_mapping_mismatch {
            reasons.push(TraceReason {
                code: "sourceMappingMismatch".into(),
                detail: Some("At least one render-tree character could not be joined exactly to source IR coordinates.".into()),
            });
        }
        if ledger_rule_missing {
            reasons.push(TraceReason {
                code: "ledgerRuleMissing".into(),
                detail: Some(
                    "Document-declared substFont is observed but is not a Stage 1 ledger rule."
                        .into(),
                ),
            });
        }
        let emitted = records.len();
        let mut trace = TraceEnvelope {
            schema_version: 1,
            status: if truncated { "truncated" } else { "complete" }.into(),
            scope: TraceScope {
                page_index: page_num,
                requested_limits,
                applied_limits,
            },
            counts: TraceCounts {
                runs_seen: runs.len(),
                characters_seen,
                records_emitted: emitted,
                records_omitted: Some(records_omitted),
            },
            records,
            backend_summary: BackendSummary::stage3(
                native_observer.is_some(),
                native_unavailable_reason,
            ),
            reasons,
            layout_hash: TraceHash::pending(),
            normalized_hash: TraceHash::pending(),
        };
        finalize_hashes(&mut trace)
            .map_err(|error| HwpError::RenderError(format!("font trace hash: {error}")))?;
        serde_json::to_string(&trace)
            .map_err(|error| HwpError::RenderError(format!("font trace serialization: {error}")))
    }
}
