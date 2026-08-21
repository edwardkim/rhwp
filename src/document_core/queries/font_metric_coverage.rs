//! Issue #4962 W3: source usage and actual layout metric decision coverage.
//!
//! This is a read-only, native-only analysis surface. It deliberately has no CLI,
//! WASM or npm binding: corpus orchestration and publication remain separate stages.

use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;
use crate::model::shape::ShapeObject;
use crate::model::style::{Alignment, CharShape};
use crate::parser::FileFormat;
use crate::renderer::font_decision::sha256_canonical;
use crate::renderer::font_metrics_data::layout_metric_face_name;
use crate::renderer::layout::{
    resolved_to_text_style, trace_char_width_decisions, CharWidthDecision,
};
use crate::renderer::style_resolver::lookup_font_name_decision;

use super::font_decision::{metric_alias_relation, run_language_slots};

const CTX_TABLE_CELL: u16 = 1 << 0;
const CTX_TEXT_BOX: u16 = 1 << 1;
const CTX_HEADER: u16 = 1 << 2;
const CTX_FOOTER: u16 = 1 << 3;
const CTX_FOOTNOTE: u16 = 1 << 4;
const CTX_ENDNOTE: u16 = 1 << 5;
const CTX_MASTER_PAGE: u16 = 1 << 6;
const CTX_CAPTION: u16 = 1 << 7;
const CTX_MEMO: u16 = 1 << 8;
const CTX_HIDDEN_COMMENT: u16 = 1 << 9;

const CATEGORY_IDS: [&str; 7] = [
    "measured-overlay",
    "identity-alias-hit",
    "metric-surrogate",
    "exact-hit",
    "char-miss",
    "face-miss",
    "heuristic",
];

const LANGUAGE_NAMES: [&str; 7] = ["ko", "latin", "hanja", "ja", "other", "symbol", "user"];

#[derive(Debug, Clone, Default)]
struct UsageCounts {
    documents: u64,
    paragraphs: u64,
    runs: u64,
    chars: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct LegacyUsageKey {
    font: String,
    metric_face: Option<String>,
    language: u8,
    ratio: u8,
    spacing: i8,
    kerning: bool,
    bold: bool,
    italic: bool,
    context: u16,
    alignment: u8,
    stored_lineseg: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct DecisionUsageKey {
    legacy: LegacyUsageKey,
    normalized_face: Option<String>,
    subst_font: Option<String>,
    alt_type: Option<u8>,
    layout_family: String,
    metric_requested_face: Option<String>,
    metric_resolved_face: Option<String>,
    match_kind: String,
    metric_entry: Option<usize>,
    character_match: String,
    width_source: String,
    relation_type: Option<String>,
    relation_evidence_status: Option<String>,
    coverage_category: Option<String>,
}

#[derive(Debug, Default)]
struct CoverageStats {
    legacy_usage: BTreeMap<LegacyUsageKey, UsageCounts>,
    decision_usage: BTreeMap<DecisionUsageKey, UsageCounts>,
    categories: BTreeMap<String, u64>,
    paragraphs_seen: u64,
    layout_characters: u64,
    coverage_characters: u64,
    not_applicable_characters: u64,
    excluded_characters: u64,
    joined: u64,
}

impl CoverageStats {
    fn new() -> Self {
        Self {
            categories: CATEGORY_IDS
                .into_iter()
                .map(|category| (category.to_string(), 0))
                .collect(),
            ..Self::default()
        }
    }

    fn exclude(&mut self, count: u64) {
        self.layout_characters += count;
        self.excluded_characters += count;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum CoverageClassification {
    Category(&'static str),
    NotApplicable,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LegacyUsageRecord {
    font: String,
    metric_face: Option<String>,
    language: String,
    ratio: u8,
    spacing: i8,
    kerning: bool,
    bold: bool,
    italic: bool,
    context: String,
    alignment: String,
    stored_line_seg: bool,
    document_count: u64,
    paragraph_count: u64,
    run_count: u64,
    char_count: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DecisionUsageRecord {
    font: String,
    metric_face: Option<String>,
    language: String,
    ratio: u8,
    spacing: i8,
    kerning: bool,
    bold: bool,
    italic: bool,
    context: String,
    alignment: String,
    stored_line_seg: bool,
    normalized_face: Option<String>,
    subst_font: Option<String>,
    alt_type: Option<u8>,
    layout_family: String,
    metric_requested_face: Option<String>,
    metric_resolved_face: Option<String>,
    match_kind: String,
    metric_entry: Option<usize>,
    character_match: String,
    width_source: String,
    relation_type: Option<String>,
    relation_evidence_status: Option<String>,
    coverage_category: Option<String>,
    source_join_status: &'static str,
    document_count: u64,
    paragraph_count: u64,
    run_count: u64,
    char_count: u64,
}

fn coverage_error(message: impl Into<String>) -> HwpError {
    HwpError::RenderError(format!("font metric coverage: {}", message.into()))
}

fn format_name(format: FileFormat) -> &'static str {
    match format {
        FileFormat::Hwp => "hwp",
        FileFormat::Hwpx => "hwpx",
        FileFormat::Hwp3 => "hwp3",
        FileFormat::Hml => "hml",
        FileFormat::DrmProtected => "drm",
        FileFormat::Empty => "empty",
        FileFormat::Unknown => "unknown",
    }
}

fn visible_layout_char(ch: char) -> bool {
    !matches!(
        ch,
        '\0' | '\r' | '\n' | '\u{0001}'..='\u{0008}' | '\u{000B}'..='\u{001F}'
    )
}

fn alignment_code(alignment: Alignment) -> u8 {
    match alignment {
        Alignment::Justify => 0,
        Alignment::Left => 1,
        Alignment::Right => 2,
        Alignment::Center => 3,
        Alignment::Distribute => 4,
        Alignment::Split => 5,
    }
}

fn alignment_name(code: u8) -> String {
    match code {
        0 => "justify",
        1 => "left",
        2 => "right",
        3 => "center",
        4 => "distribute",
        5 => "split",
        _ => "unknown",
    }
    .to_string()
}

fn context_name(mask: u16) -> String {
    if mask == 0 {
        return "body".to_string();
    }
    [
        (CTX_TABLE_CELL, "table-cell"),
        (CTX_TEXT_BOX, "text-box"),
        (CTX_HEADER, "header"),
        (CTX_FOOTER, "footer"),
        (CTX_FOOTNOTE, "footnote"),
        (CTX_ENDNOTE, "endnote"),
        (CTX_MASTER_PAGE, "master-page"),
        (CTX_CAPTION, "caption"),
        (CTX_MEMO, "memo"),
        (CTX_HIDDEN_COMMENT, "hidden-comment"),
    ]
    .into_iter()
    .filter_map(|(bit, name)| (mask & bit != 0).then_some(name))
    .collect::<Vec<_>>()
    .join("+")
}

fn classify_decision(
    decision: &CharWidthDecision<'_>,
    relation_type: Option<&str>,
    relation_evidence_status: Option<&str>,
) -> Result<CoverageClassification, HwpError> {
    let metric_entry = decision.metric.map(|metric| metric.entry_index);
    match decision.width_source {
        "metricMiss" | "metricCharacterMiss" => Err(coverage_error(format!(
            "internal-only widthSource reached aggregate: {}",
            decision.width_source
        ))),
        "clusterContinuation"
        | "figureSpace"
        | "hwpPuaFiller"
        | "inlineObjectPlaceholder"
        | "tabAdvance" => {
            if decision.character_match != "notApplicable" || metric_entry.is_some() {
                return Err(coverage_error(format!(
                    "non-applicable widthSource has metric state: {}",
                    decision.width_source
                )));
            }
            Ok(CoverageClassification::NotApplicable)
        }
        "kopubTable"
        | "metricHalfwidthPunctuationOverlay"
        | "metricNarrowPunctuationOverlay"
        | "metricSpaceOverlay" => {
            if decision.character_match != "hit" {
                return Err(coverage_error(format!(
                    "measured overlay must be a character hit: {}",
                    decision.width_source
                )));
            }
            Ok(CoverageClassification::Category("measured-overlay"))
        }
        "embeddedMetric" | "metricHalfSpace" => {
            if decision.character_match != "hit" || metric_entry.is_none() {
                return Err(coverage_error(format!(
                    "metric widthSource requires a metric character hit: {}",
                    decision.width_source
                )));
            }
            match (relation_type, relation_evidence_status) {
                (Some("identity-alias"), Some("verified-by-bytes")) => {
                    Ok(CoverageClassification::Category("identity-alias-hit"))
                }
                (Some("identity-alias"), _) => Err(coverage_error(
                    "identity-alias requires verified-by-bytes evidence",
                )),
                (Some("metric-surrogate"), _) => {
                    Ok(CoverageClassification::Category("metric-surrogate"))
                }
                _ => Ok(CoverageClassification::Category("exact-hit")),
            }
        }
        "heuristicFullwidth" | "heuristicHalfwidth" | "heuristicNarrow" => {
            if metric_entry.is_some() {
                if decision.character_match != "miss" {
                    return Err(coverage_error(format!(
                        "metric fallback requires characterMatch miss: {}",
                        decision.width_source
                    )));
                }
                Ok(CoverageClassification::Category("char-miss"))
            } else if decision.character_match == "notApplicable" {
                Ok(CoverageClassification::Category("face-miss"))
            } else {
                Err(coverage_error(format!(
                    "face miss has contradictory metric state: {}",
                    decision.width_source
                )))
            }
        }
        "areaDotFallback" => {
            if metric_entry.is_some() || decision.character_match != "notApplicable" {
                return Err(coverage_error(
                    "areaDotFallback has contradictory metric state",
                ));
            }
            Ok(CoverageClassification::Category("heuristic"))
        }
        other => Err(coverage_error(format!("unclassified widthSource: {other}"))),
    }
}

fn legacy_key(
    font: String,
    char_shape: &CharShape,
    language: usize,
    context: u16,
    alignment: u8,
    stored_lineseg: bool,
) -> LegacyUsageKey {
    LegacyUsageKey {
        metric_face: layout_metric_face_name(&font, char_shape.bold, char_shape.italic),
        font,
        language: language as u8,
        ratio: char_shape.ratios[language],
        spacing: char_shape.spacings[language],
        kerning: char_shape.kerning,
        bold: char_shape.bold,
        italic: char_shape.italic,
        context,
        alignment,
        stored_lineseg,
    }
}

fn finish_run<K: Clone + Ord>(
    current: &mut Option<(K, u64)>,
    usage: &mut BTreeMap<K, UsageCounts>,
    paragraph_keys: &mut BTreeSet<K>,
) {
    if let Some((key, chars)) = current.take() {
        let counts = usage.entry(key.clone()).or_default();
        counts.runs += 1;
        counts.chars += chars;
        paragraph_keys.insert(key);
    }
}

fn push_run<K: Clone + Ord>(
    key: K,
    current: &mut Option<(K, u64)>,
    usage: &mut BTreeMap<K, UsageCounts>,
    paragraph_keys: &mut BTreeSet<K>,
) {
    if current.as_ref().is_some_and(|(active, _)| active == &key) {
        current.as_mut().expect("active usage run").1 += 1;
    } else {
        finish_run(current, usage, paragraph_keys);
        *current = Some((key, 1));
    }
}

fn analyze_paragraph(
    core: &DocumentCore,
    para: &Paragraph,
    context: u16,
    stats: &mut CoverageStats,
) -> Result<(), HwpError> {
    stats.paragraphs_seen += 1;
    let stored_lineseg = !para.line_segs.is_empty()
        && !para
            .line_segs
            .iter()
            .all(|line| line.is_missing_lineseg_placeholder());
    let alignment = core
        .document
        .doc_info
        .para_shapes
        .get(para.para_shape_id as usize)
        .map(|shape| alignment_code(shape.alignment))
        .unwrap_or(255);

    let mut fallback_offset = 0u32;
    let chars: Vec<(u32, char)> = para
        .text
        .chars()
        .enumerate()
        .map(|(index, ch)| {
            let offset = para
                .char_offsets
                .get(index)
                .copied()
                .unwrap_or(fallback_offset);
            fallback_offset = fallback_offset.saturating_add(ch.len_utf16() as u32);
            (offset, ch)
        })
        .collect();
    if chars.is_empty() {
        return Ok(());
    }

    let mut refs = para.char_shapes.clone();
    refs.sort_by_key(|item| item.start_pos);
    if refs.is_empty() {
        refs.push(Default::default());
    }
    let mut legacy_paragraph_keys = BTreeSet::new();
    let mut decision_paragraph_keys = BTreeSet::new();

    for (ref_index, shape_ref) in refs.iter().enumerate() {
        let end = refs
            .get(ref_index + 1)
            .map(|next| next.start_pos)
            .unwrap_or(u32::MAX);
        let segment: Vec<char> = chars
            .iter()
            .filter(|(offset, ch)| {
                *offset >= shape_ref.start_pos && *offset < end && visible_layout_char(*ch)
            })
            .map(|(_, ch)| *ch)
            .collect();
        if segment.is_empty() {
            continue;
        }
        let Some(char_shape) = core
            .document
            .doc_info
            .char_shapes
            .get(shape_ref.char_shape_id as usize)
        else {
            stats.exclude(segment.len() as u64);
            continue;
        };

        let language_slots = run_language_slots(&segment, Some(0));
        let mut group_start = 0usize;
        while group_start < segment.len() {
            let language = language_slots[group_start].0.min(6);
            let mut group_end = group_start + 1;
            while group_end < segment.len() && language_slots[group_end].0.min(6) == language {
                group_end += 1;
            }

            let name_decision = lookup_font_name_decision(
                &core.document.doc_info,
                language,
                char_shape.font_ids[language],
            );
            let Some(requested_face) = name_decision
                .requested_face
                .as_deref()
                .map(str::trim)
                .filter(|face| !face.is_empty())
            else {
                stats.exclude((group_end - group_start) as u64);
                group_start = group_end;
                continue;
            };

            let legacy = legacy_key(
                requested_face.to_string(),
                char_shape,
                language,
                context,
                alignment,
                stored_lineseg,
            );
            let style =
                resolved_to_text_style(&core.styles, shape_ref.char_shape_id as u32, language);
            let text: String = segment[group_start..group_end].iter().collect();
            let decisions = trace_char_width_decisions(&text, &style);
            if decisions.len() != group_end - group_start {
                return Err(coverage_error("character decision join length mismatch"));
            }

            let mut legacy_run: Option<(LegacyUsageKey, u64)> = None;
            let mut decision_run: Option<(DecisionUsageKey, u64)> = None;
            for decision in decisions {
                let metric = decision.metric;
                let (relation_type, relation_evidence_status) = metric
                    .filter(|metric| metric.requested_name != metric.alias_resolved_name)
                    .map(|metric| {
                        metric_alias_relation(metric.requested_name, metric.alias_resolved_name)
                    })
                    .map(|(relation, evidence)| {
                        (Some(relation.to_string()), Some(evidence.to_string()))
                    })
                    .unwrap_or((None, None));
                let classification = classify_decision(
                    &decision,
                    relation_type.as_deref(),
                    relation_evidence_status.as_deref(),
                )?;
                let category = match classification {
                    CoverageClassification::Category(category) => {
                        *stats
                            .categories
                            .get_mut(category)
                            .expect("contract category") += 1;
                        stats.coverage_characters += 1;
                        Some(category.to_string())
                    }
                    CoverageClassification::NotApplicable => {
                        stats.not_applicable_characters += 1;
                        None
                    }
                };
                stats.layout_characters += 1;
                stats.joined += 1;

                let decision_key = DecisionUsageKey {
                    legacy: legacy.clone(),
                    normalized_face: name_decision.normalized_face.clone(),
                    subst_font: name_decision.subst_font.clone(),
                    alt_type: name_decision.alt_type,
                    layout_family: style.font_family.clone(),
                    metric_requested_face: metric.map(|entry| entry.requested_name.to_string()),
                    metric_resolved_face: metric.map(|entry| entry.alias_resolved_name.to_string()),
                    match_kind: metric
                        .map(|entry| entry.match_kind.as_str().to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    metric_entry: metric.map(|entry| entry.entry_index),
                    character_match: decision.character_match.to_string(),
                    width_source: decision.width_source.to_string(),
                    relation_type,
                    relation_evidence_status,
                    coverage_category: category,
                };
                push_run(
                    legacy.clone(),
                    &mut legacy_run,
                    &mut stats.legacy_usage,
                    &mut legacy_paragraph_keys,
                );
                push_run(
                    decision_key,
                    &mut decision_run,
                    &mut stats.decision_usage,
                    &mut decision_paragraph_keys,
                );
            }
            finish_run(
                &mut legacy_run,
                &mut stats.legacy_usage,
                &mut legacy_paragraph_keys,
            );
            finish_run(
                &mut decision_run,
                &mut stats.decision_usage,
                &mut decision_paragraph_keys,
            );
            group_start = group_end;
        }
    }

    for key in legacy_paragraph_keys {
        stats.legacy_usage.entry(key).or_default().paragraphs += 1;
    }
    for key in decision_paragraph_keys {
        stats.decision_usage.entry(key).or_default().paragraphs += 1;
    }
    Ok(())
}

fn walk_shape(
    core: &DocumentCore,
    shape: &ShapeObject,
    context: u16,
    stats: &mut CoverageStats,
) -> Result<(), HwpError> {
    if let Some(drawing) = shape.drawing() {
        if let Some(text_box) = &drawing.text_box {
            walk_paragraphs(core, &text_box.paragraphs, context | CTX_TEXT_BOX, stats)?;
        }
        if let Some(caption) = &drawing.caption {
            walk_paragraphs(core, &caption.paragraphs, context | CTX_CAPTION, stats)?;
        }
    }
    match shape {
        ShapeObject::Group(group) => {
            if let Some(caption) = &group.caption {
                walk_paragraphs(core, &caption.paragraphs, context | CTX_CAPTION, stats)?;
            }
            for child in &group.children {
                walk_shape(core, child, context, stats)?;
            }
        }
        ShapeObject::Picture(picture) => {
            if let Some(caption) = &picture.caption {
                walk_paragraphs(core, &caption.paragraphs, context | CTX_CAPTION, stats)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn walk_paragraphs(
    core: &DocumentCore,
    paragraphs: &[Paragraph],
    context: u16,
    stats: &mut CoverageStats,
) -> Result<(), HwpError> {
    for para in paragraphs {
        analyze_paragraph(core, para, context, stats)?;
        for control in &para.controls {
            match control {
                Control::Table(table) => {
                    if let Some(caption) = &table.caption {
                        walk_paragraphs(core, &caption.paragraphs, context | CTX_CAPTION, stats)?;
                    }
                    for cell in &table.cells {
                        walk_paragraphs(core, &cell.paragraphs, context | CTX_TABLE_CELL, stats)?;
                    }
                }
                Control::Shape(shape) => walk_shape(core, shape, context, stats)?,
                Control::Picture(picture) => {
                    if let Some(caption) = &picture.caption {
                        walk_paragraphs(core, &caption.paragraphs, context | CTX_CAPTION, stats)?;
                    }
                }
                Control::Header(header) => {
                    walk_paragraphs(core, &header.paragraphs, context | CTX_HEADER, stats)?;
                }
                Control::Footer(footer) => {
                    walk_paragraphs(core, &footer.paragraphs, context | CTX_FOOTER, stats)?;
                }
                Control::Footnote(note) => {
                    walk_paragraphs(core, &note.paragraphs, context | CTX_FOOTNOTE, stats)?;
                }
                Control::Endnote(note) => {
                    walk_paragraphs(core, &note.paragraphs, context | CTX_ENDNOTE, stats)?;
                }
                Control::HiddenComment(comment) => walk_paragraphs(
                    core,
                    &comment.paragraphs,
                    context | CTX_HIDDEN_COMMENT,
                    stats,
                )?,
                Control::Field(field) if !field.memo_paragraphs.is_empty() => {
                    walk_paragraphs(core, &field.memo_paragraphs, context | CTX_MEMO, stats)?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn analyze_document(core: &DocumentCore) -> Result<CoverageStats, HwpError> {
    let mut stats = CoverageStats::new();
    for section in &core.document.sections {
        walk_paragraphs(core, &section.paragraphs, 0, &mut stats)?;
        for master in &section.section_def.master_pages {
            walk_paragraphs(core, &master.paragraphs, CTX_MASTER_PAGE, &mut stats)?;
        }
    }
    for counts in stats.legacy_usage.values_mut() {
        counts.documents = 1;
    }
    for counts in stats.decision_usage.values_mut() {
        counts.documents = 1;
    }
    Ok(stats)
}

fn legacy_records(stats: &CoverageStats) -> Vec<LegacyUsageRecord> {
    let mut records: Vec<LegacyUsageRecord> = stats
        .legacy_usage
        .iter()
        .map(|(key, counts)| LegacyUsageRecord {
            font: key.font.clone(),
            metric_face: key.metric_face.clone(),
            language: LANGUAGE_NAMES[key.language as usize].to_string(),
            ratio: key.ratio,
            spacing: key.spacing,
            kerning: key.kerning,
            bold: key.bold,
            italic: key.italic,
            context: context_name(key.context),
            alignment: alignment_name(key.alignment),
            stored_line_seg: key.stored_lineseg,
            document_count: counts.documents,
            paragraph_count: counts.paragraphs,
            run_count: counts.runs,
            char_count: counts.chars,
        })
        .collect();
    // Keep the v2 POC projection order: character count descending, then face.
    // Stable sorting preserves the original BTree key order for complete ties.
    records.sort_by(|left, right| {
        right
            .char_count
            .cmp(&left.char_count)
            .then_with(|| left.font.cmp(&right.font))
    });
    records
}

fn decision_records(stats: &CoverageStats) -> Vec<DecisionUsageRecord> {
    stats
        .decision_usage
        .iter()
        .map(|(key, counts)| DecisionUsageRecord {
            font: key.legacy.font.clone(),
            metric_face: key.legacy.metric_face.clone(),
            language: LANGUAGE_NAMES[key.legacy.language as usize].to_string(),
            ratio: key.legacy.ratio,
            spacing: key.legacy.spacing,
            kerning: key.legacy.kerning,
            bold: key.legacy.bold,
            italic: key.legacy.italic,
            context: context_name(key.legacy.context),
            alignment: alignment_name(key.legacy.alignment),
            stored_line_seg: key.legacy.stored_lineseg,
            normalized_face: key.normalized_face.clone(),
            subst_font: key.subst_font.clone(),
            alt_type: key.alt_type,
            layout_family: key.layout_family.clone(),
            metric_requested_face: key.metric_requested_face.clone(),
            metric_resolved_face: key.metric_resolved_face.clone(),
            match_kind: key.match_kind.clone(),
            metric_entry: key.metric_entry,
            character_match: key.character_match.clone(),
            width_source: key.width_source.clone(),
            relation_type: key.relation_type.clone(),
            relation_evidence_status: key.relation_evidence_status.clone(),
            coverage_category: key.coverage_category.clone(),
            source_join_status: "joined",
            document_count: counts.documents,
            paragraph_count: counts.paragraphs,
            run_count: counts.runs,
            char_count: counts.chars,
        })
        .collect()
}

fn reconcile(stats: &CoverageStats) -> Result<(), HwpError> {
    let category_sum: u64 = stats.categories.values().sum();
    if category_sum != stats.coverage_characters {
        return Err(coverage_error(
            "category sum does not equal coverage denominator",
        ));
    }
    if stats.layout_characters
        != stats.coverage_characters + stats.not_applicable_characters + stats.excluded_characters
    {
        return Err(coverage_error("layout denominator does not reconcile"));
    }
    if stats.layout_characters != stats.joined + stats.excluded_characters {
        return Err(coverage_error("source join denominator does not reconcile"));
    }
    let legacy_chars: u64 = stats.legacy_usage.values().map(|counts| counts.chars).sum();
    let decision_chars: u64 = stats
        .decision_usage
        .values()
        .map(|counts| counts.chars)
        .sum();
    if legacy_chars != stats.joined || decision_chars != stats.joined {
        return Err(coverage_error(
            "usage projection does not equal joined characters",
        ));
    }
    Ok(())
}

impl DocumentCore {
    /// Read-only W3 aggregate for developer-side corpus instrumentation.
    ///
    /// The JSON contains only aggregated usage keys; it never persists raw characters,
    /// document paths, file names or per-character records.
    #[doc(hidden)]
    pub fn get_font_metric_coverage_analysis_native(&self) -> Result<String, HwpError> {
        let stats = analyze_document(self)?;
        reconcile(&stats)?;
        let legacy_usage = legacy_records(&stats);
        let decision_usage = decision_records(&stats);
        let source_runs_seen: u64 = stats.legacy_usage.values().map(|counts| counts.runs).sum();
        let legacy_projection = json!({
            "schemaVersion": "poc-font-layout-habits-v2",
            "format": format_name(self.source_format),
            "paragraphs": stats.paragraphs_seen,
            "chars": stats.joined,
            "usage": legacy_usage,
        });
        let legacy_projection_hash = sha256_canonical(legacy_projection, true)
            .map_err(|error| coverage_error(format!("hash legacy projection: {error}")))?;

        let mut report = json!({
            "schemaVersion": 1,
            "kind": "font-metric-coverage-aggregate",
            "status": "complete",
            "format": format_name(self.source_format),
            "counts": {
                "paragraphsSeen": stats.paragraphs_seen,
                "sourceRunsSeen": source_runs_seen,
                "layoutCharacters": stats.layout_characters,
                "coverageCharacters": stats.coverage_characters,
                "notApplicableCharacters": stats.not_applicable_characters,
                "excludedCharacters": stats.excluded_characters,
                "truncatedCharacters": 0,
                "legacyUsageRows": legacy_usage.len(),
                "decisionUsageRows": decision_usage.len(),
            },
            "categories": stats.categories,
            "joins": {
                "joined": stats.joined,
                "layoutOnly": 0,
                "excluded": stats.excluded_characters,
            },
            "documents": {
                "attempted": 1,
                "success": 1,
                "failures": {
                    "drm": 0,
                    "empty": 0,
                    "encrypted": 0,
                    "parser": 0,
                    "unsupported": 0,
                },
            },
            "backends": {
                "requested": 0,
                "complete": 0,
                "failed": 0,
                "notObserved": 0,
                "unsupported": 0,
            },
            "legacyProjectionHash": {
                "algorithm": "sha256",
                "value": legacy_projection_hash,
            },
            "aggregateHash": {
                "algorithm": "sha256",
                "value": "",
            },
            "legacyUsage": legacy_usage,
            "decisionUsage": decision_usage,
        });
        let aggregate_hash = sha256_canonical(report.clone(), true)
            .map_err(|error| coverage_error(format!("hash aggregate: {error}")))?;
        report["aggregateHash"]["value"] = Value::String(aggregate_hash);
        serde_json::to_string(&report)
            .map_err(|error| coverage_error(format!("serialize aggregate: {error}")))
    }
}
