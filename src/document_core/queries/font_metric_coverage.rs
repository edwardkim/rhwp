//! Issue #4962 W3: source usage and actual layout metric decision coverage.
//!
//! This is a read-only, native-only analysis surface. It deliberately has no CLI,
//! WASM or npm binding: corpus orchestration and publication remain separate stages.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
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
use crate::schema_registry::LEGACY_FONT_LAYOUT_HABITS_SCHEMA_VERSION;

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

const DEFAULT_MAX_WORK_UNITS: u64 = 10_000_000;
const MAX_MAX_WORK_UNITS: u64 = 2_000_000_000;
const DEFAULT_MAX_AGGREGATE_ROWS: usize = 20_000;
const MAX_MAX_AGGREGATE_ROWS: usize = 200_000;
const DEFAULT_MAX_OUTPUT_BYTES: usize = 32 * 1024 * 1024;
const MAX_MAX_OUTPUT_BYTES: usize = 128 * 1024 * 1024;
const DEFAULT_DEADLINE_MILLIS: u64 = 60_000;
const MAX_DEADLINE_MILLIS: u64 = 3_600_000;
const DEFAULT_MAX_NESTING_DEPTH: usize = 128;
const MAX_MAX_NESTING_DEPTH: usize = 4096;
const MAX_DIMENSION_STRING_BYTES: usize = 4096;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CoverageOptions {
    max_work_units: Option<u64>,
    max_aggregate_rows: Option<usize>,
    max_output_bytes: Option<usize>,
    deadline_millis: Option<u64>,
    max_nesting_depth: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
struct CoverageLimits {
    max_work_units: u64,
    max_aggregate_rows: usize,
    max_output_bytes: usize,
    deadline: Duration,
    max_nesting_depth: usize,
}

impl CoverageLimits {
    fn parse(options_json: &str) -> Result<Self, HwpError> {
        let options: CoverageOptions = if options_json.trim().is_empty() {
            CoverageOptions::default()
        } else {
            serde_json::from_str(options_json)
                .map_err(|error| coverage_error(format!("options: {error}")))?
        };
        let max_work_units = options.max_work_units.unwrap_or(DEFAULT_MAX_WORK_UNITS);
        let max_aggregate_rows = options
            .max_aggregate_rows
            .unwrap_or(DEFAULT_MAX_AGGREGATE_ROWS);
        let max_output_bytes = options.max_output_bytes.unwrap_or(DEFAULT_MAX_OUTPUT_BYTES);
        let deadline_millis = options.deadline_millis.unwrap_or(DEFAULT_DEADLINE_MILLIS);
        let max_nesting_depth = options
            .max_nesting_depth
            .unwrap_or(DEFAULT_MAX_NESTING_DEPTH);
        if !(1..=MAX_MAX_WORK_UNITS).contains(&max_work_units) {
            return Err(coverage_error(format!(
                "maxWorkUnits must be in 1..={MAX_MAX_WORK_UNITS}"
            )));
        }
        if !(1..=MAX_MAX_AGGREGATE_ROWS).contains(&max_aggregate_rows) {
            return Err(coverage_error(format!(
                "maxAggregateRows must be in 1..={MAX_MAX_AGGREGATE_ROWS}"
            )));
        }
        if !(1024..=MAX_MAX_OUTPUT_BYTES).contains(&max_output_bytes) {
            return Err(coverage_error(format!(
                "maxOutputBytes must be in 1024..={MAX_MAX_OUTPUT_BYTES}"
            )));
        }
        if !(1..=MAX_DEADLINE_MILLIS).contains(&deadline_millis) {
            return Err(coverage_error(format!(
                "deadlineMillis must be in 1..={MAX_DEADLINE_MILLIS}"
            )));
        }
        if !(1..=MAX_MAX_NESTING_DEPTH).contains(&max_nesting_depth) {
            return Err(coverage_error(format!(
                "maxNestingDepth must be in 1..={MAX_MAX_NESTING_DEPTH}"
            )));
        }
        Ok(Self {
            max_work_units,
            max_aggregate_rows,
            max_output_bytes,
            deadline: Duration::from_millis(deadline_millis),
            max_nesting_depth,
        })
    }
}

struct CoverageBudget<'a> {
    limits: CoverageLimits,
    started: Instant,
    work_units: u64,
    aggregate_rows: usize,
    cancellation: Option<&'a AtomicBool>,
}

impl<'a> CoverageBudget<'a> {
    fn new(limits: CoverageLimits, cancellation: Option<&'a AtomicBool>) -> Self {
        Self {
            limits,
            started: Instant::now(),
            work_units: 0,
            aggregate_rows: 0,
            cancellation,
        }
    }

    fn consume(&mut self, units: usize) -> Result<(), HwpError> {
        if self
            .cancellation
            .is_some_and(|cancelled| cancelled.load(Ordering::Relaxed))
        {
            return Err(coverage_cancelled_error());
        }
        if self.started.elapsed() > self.limits.deadline {
            return Err(coverage_resource_error("deadline exceeded"));
        }
        self.work_units = self
            .work_units
            .checked_add(units as u64)
            .ok_or_else(|| coverage_resource_error("work unit counter overflow"))?;
        if self.work_units > self.limits.max_work_units {
            return Err(coverage_resource_error(format!(
                "work unit budget exceeded: {} > {}",
                self.work_units, self.limits.max_work_units
            )));
        }
        Ok(())
    }

    fn add_row(&mut self) -> Result<(), HwpError> {
        self.aggregate_rows = self
            .aggregate_rows
            .checked_add(1)
            .ok_or_else(|| coverage_resource_error("aggregate row counter overflow"))?;
        if self.aggregate_rows > self.limits.max_aggregate_rows {
            return Err(coverage_resource_error(format!(
                "aggregate row budget exceeded: {} > {}",
                self.aggregate_rows, self.limits.max_aggregate_rows
            )));
        }
        Ok(())
    }

    fn check_depth(&self, depth: usize) -> Result<(), HwpError> {
        if depth > self.limits.max_nesting_depth {
            return Err(coverage_resource_error(format!(
                "nesting depth budget exceeded: {depth} > {}",
                self.limits.max_nesting_depth
            )));
        }
        Ok(())
    }

    fn check_dimension(&self, value: &str) -> Result<(), HwpError> {
        if value.len() > MAX_DIMENSION_STRING_BYTES {
            return Err(coverage_resource_error(format!(
                "dimension string exceeds {MAX_DIMENSION_STRING_BYTES} bytes"
            )));
        }
        Ok(())
    }

    fn check_output(&self, bytes: usize) -> Result<(), HwpError> {
        if bytes > self.limits.max_output_bytes {
            return Err(coverage_resource_error(format!(
                "output byte budget exceeded: {bytes} > {}",
                self.limits.max_output_bytes
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct UsageCounts {
    documents: u64,
    paragraphs: u64,
    runs: u64,
    chars: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct LegacyUsageKey {
    font: Arc<str>,
    metric_face: Option<Arc<str>>,
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
    normalized_face: Option<Arc<str>>,
    subst_font: Option<Arc<str>>,
    alt_type: Option<u8>,
    layout_family: Arc<str>,
    metric_requested_face: Option<Arc<str>>,
    metric_resolved_face: Option<Arc<str>>,
    match_kind: &'static str,
    metric_entry: Option<usize>,
    character_match: &'static str,
    width_source: &'static str,
    relation_type: Option<&'static str>,
    relation_evidence_status: Option<&'static str>,
    coverage_category: Option<&'static str>,
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

fn coverage_resource_error(message: impl Into<String>) -> HwpError {
    coverage_error(format!("[RESOURCE_LIMIT_EXCEEDED] {}", message.into()))
}

fn coverage_cancelled_error() -> HwpError {
    coverage_error("[ANALYSIS_CANCELLED] cancellation requested")
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
    font: Arc<str>,
    char_shape: &CharShape,
    language: usize,
    context: u16,
    alignment: u8,
    stored_lineseg: bool,
) -> LegacyUsageKey {
    let metric_face =
        layout_metric_face_name(&font, char_shape.bold, char_shape.italic).map(Arc::<str>::from);
    LegacyUsageKey {
        metric_face,
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
    budget: &mut CoverageBudget<'_>,
) -> Result<(), HwpError> {
    if let Some((key, chars)) = current.take() {
        if !usage.contains_key(&key) {
            budget.add_row()?;
        }
        let counts = usage.entry(key.clone()).or_default();
        counts.runs += 1;
        counts.chars += chars;
        paragraph_keys.insert(key);
    }
    Ok(())
}

fn push_run<K: Clone + Ord>(
    key: K,
    current: &mut Option<(K, u64)>,
    usage: &mut BTreeMap<K, UsageCounts>,
    paragraph_keys: &mut BTreeSet<K>,
    budget: &mut CoverageBudget<'_>,
) -> Result<(), HwpError> {
    if current.as_ref().is_some_and(|(active, _)| active == &key) {
        current.as_mut().expect("active usage run").1 += 1;
    } else {
        finish_run(current, usage, paragraph_keys, budget)?;
        *current = Some((key, 1));
    }
    Ok(())
}

fn analyze_paragraph(
    core: &DocumentCore,
    para: &Paragraph,
    context: u16,
    stats: &mut CoverageStats,
    budget: &mut CoverageBudget<'_>,
) -> Result<(), HwpError> {
    // Charge decoded bytes before materializing any collector-owned vectors. This is a
    // resource budget, not a successful-result truncation boundary.
    budget.consume(1)?;
    budget.consume(para.text.len())?;
    budget.consume(para.char_shapes.len())?;
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

    // `chars` and `refs` are both ordered by UTF-16 position. Keep one monotonic cursor
    // instead of rescanning every character for every CharShapeRef (old O(N*R) path).
    let mut char_cursor = 0usize;
    for (ref_index, shape_ref) in refs.iter().enumerate() {
        let end = refs
            .get(ref_index + 1)
            .map(|next| next.start_pos)
            .unwrap_or(u32::MAX);
        while char_cursor < chars.len() && chars[char_cursor].0 < shape_ref.start_pos {
            char_cursor += 1;
        }
        let segment_start = char_cursor;
        while char_cursor < chars.len() && chars[char_cursor].0 < end {
            char_cursor += 1;
        }
        let segment: Vec<char> = chars[segment_start..char_cursor]
            .iter()
            .filter_map(|(_, ch)| visible_layout_char(*ch).then_some(*ch))
            .collect();
        if segment.is_empty() {
            continue;
        }
        budget.consume(segment.len())?;
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

            budget.check_dimension(requested_face)?;
            let requested_face = Arc::<str>::from(requested_face);
            let normalized_face = name_decision
                .normalized_face
                .as_deref()
                .map(|face| {
                    budget.check_dimension(face)?;
                    Ok::<Arc<str>, HwpError>(Arc::from(face))
                })
                .transpose()?;
            let subst_font = name_decision
                .subst_font
                .as_deref()
                .map(|face| {
                    budget.check_dimension(face)?;
                    Ok::<Arc<str>, HwpError>(Arc::from(face))
                })
                .transpose()?;

            let legacy = legacy_key(
                requested_face,
                char_shape,
                language,
                context,
                alignment,
                stored_lineseg,
            );
            let style =
                resolved_to_text_style(&core.styles, shape_ref.char_shape_id as u32, language);
            budget.check_dimension(&style.font_family)?;
            let layout_family = Arc::<str>::from(style.font_family.as_str());
            let text: String = segment[group_start..group_end].iter().collect();
            let decisions = trace_char_width_decisions(&text, &style);
            if decisions.len() != group_end - group_start {
                return Err(coverage_error("character decision join length mismatch"));
            }
            budget.consume(decisions.len())?;

            let metric_names = decisions
                .iter()
                .find_map(|decision| decision.metric)
                .map(|metric| {
                    budget.check_dimension(metric.requested_name)?;
                    budget.check_dimension(metric.alias_resolved_name)?;
                    Ok::<(Arc<str>, Arc<str>), HwpError>((
                        Arc::from(metric.requested_name),
                        Arc::from(metric.alias_resolved_name),
                    ))
                })
                .transpose()?;

            let mut legacy_run: Option<(LegacyUsageKey, u64)> = None;
            let mut decision_run: Option<(DecisionUsageKey, u64)> = None;
            for decision in decisions {
                let metric = decision.metric;
                let (relation_type, relation_evidence_status) = metric
                    .filter(|metric| metric.requested_name != metric.alias_resolved_name)
                    .map(|metric| {
                        metric_alias_relation(metric.requested_name, metric.alias_resolved_name)
                    })
                    .map(|(relation, evidence)| (Some(relation), Some(evidence)))
                    .unwrap_or((None, None));
                let classification =
                    classify_decision(&decision, relation_type, relation_evidence_status)?;
                let category = match classification {
                    CoverageClassification::Category(category) => {
                        *stats
                            .categories
                            .get_mut(category)
                            .expect("contract category") += 1;
                        stats.coverage_characters += 1;
                        Some(category)
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
                    normalized_face: normalized_face.clone(),
                    subst_font: subst_font.clone(),
                    alt_type: name_decision.alt_type,
                    layout_family: layout_family.clone(),
                    metric_requested_face: metric
                        .and_then(|_| metric_names.as_ref().map(|names| names.0.clone())),
                    metric_resolved_face: metric
                        .and_then(|_| metric_names.as_ref().map(|names| names.1.clone())),
                    match_kind: metric
                        .map(|entry| entry.match_kind.as_str())
                        .unwrap_or("none"),
                    metric_entry: metric.map(|entry| entry.entry_index),
                    character_match: decision.character_match,
                    width_source: decision.width_source,
                    relation_type,
                    relation_evidence_status,
                    coverage_category: category,
                };
                push_run(
                    legacy.clone(),
                    &mut legacy_run,
                    &mut stats.legacy_usage,
                    &mut legacy_paragraph_keys,
                    budget,
                )?;
                push_run(
                    decision_key,
                    &mut decision_run,
                    &mut stats.decision_usage,
                    &mut decision_paragraph_keys,
                    budget,
                )?;
            }
            finish_run(
                &mut legacy_run,
                &mut stats.legacy_usage,
                &mut legacy_paragraph_keys,
                budget,
            )?;
            finish_run(
                &mut decision_run,
                &mut stats.decision_usage,
                &mut decision_paragraph_keys,
                budget,
            )?;
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
    budget: &mut CoverageBudget<'_>,
    depth: usize,
) -> Result<(), HwpError> {
    budget.check_depth(depth)?;
    budget.consume(1)?;
    if let Some(drawing) = shape.drawing() {
        if let Some(text_box) = &drawing.text_box {
            walk_paragraphs(
                core,
                &text_box.paragraphs,
                context | CTX_TEXT_BOX,
                stats,
                budget,
                depth + 1,
            )?;
        }
        if let Some(caption) = &drawing.caption {
            walk_paragraphs(
                core,
                &caption.paragraphs,
                context | CTX_CAPTION,
                stats,
                budget,
                depth + 1,
            )?;
        }
    }
    match shape {
        ShapeObject::Group(group) => {
            if let Some(caption) = &group.caption {
                walk_paragraphs(
                    core,
                    &caption.paragraphs,
                    context | CTX_CAPTION,
                    stats,
                    budget,
                    depth + 1,
                )?;
            }
            for child in &group.children {
                walk_shape(core, child, context, stats, budget, depth + 1)?;
            }
        }
        ShapeObject::Picture(picture) => {
            if let Some(caption) = &picture.caption {
                walk_paragraphs(
                    core,
                    &caption.paragraphs,
                    context | CTX_CAPTION,
                    stats,
                    budget,
                    depth + 1,
                )?;
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
    budget: &mut CoverageBudget<'_>,
    depth: usize,
) -> Result<(), HwpError> {
    budget.check_depth(depth)?;
    for para in paragraphs {
        analyze_paragraph(core, para, context, stats, budget)?;
        budget.consume(para.controls.len())?;
        for control in &para.controls {
            match control {
                Control::Table(table) => {
                    if let Some(caption) = &table.caption {
                        walk_paragraphs(
                            core,
                            &caption.paragraphs,
                            context | CTX_CAPTION,
                            stats,
                            budget,
                            depth + 1,
                        )?;
                    }
                    for cell in &table.cells {
                        walk_paragraphs(
                            core,
                            &cell.paragraphs,
                            context | CTX_TABLE_CELL,
                            stats,
                            budget,
                            depth + 1,
                        )?;
                    }
                }
                Control::Shape(shape) => {
                    walk_shape(core, shape, context, stats, budget, depth + 1)?
                }
                Control::Picture(picture) => {
                    if let Some(caption) = &picture.caption {
                        walk_paragraphs(
                            core,
                            &caption.paragraphs,
                            context | CTX_CAPTION,
                            stats,
                            budget,
                            depth + 1,
                        )?;
                    }
                }
                Control::Header(header) => {
                    walk_paragraphs(
                        core,
                        &header.paragraphs,
                        context | CTX_HEADER,
                        stats,
                        budget,
                        depth + 1,
                    )?;
                }
                Control::Footer(footer) => {
                    walk_paragraphs(
                        core,
                        &footer.paragraphs,
                        context | CTX_FOOTER,
                        stats,
                        budget,
                        depth + 1,
                    )?;
                }
                Control::Footnote(note) => {
                    walk_paragraphs(
                        core,
                        &note.paragraphs,
                        context | CTX_FOOTNOTE,
                        stats,
                        budget,
                        depth + 1,
                    )?;
                }
                Control::Endnote(note) => {
                    walk_paragraphs(
                        core,
                        &note.paragraphs,
                        context | CTX_ENDNOTE,
                        stats,
                        budget,
                        depth + 1,
                    )?;
                }
                Control::HiddenComment(comment) => walk_paragraphs(
                    core,
                    &comment.paragraphs,
                    context | CTX_HIDDEN_COMMENT,
                    stats,
                    budget,
                    depth + 1,
                )?,
                Control::Field(field) if !field.memo_paragraphs.is_empty() => {
                    walk_paragraphs(
                        core,
                        &field.memo_paragraphs,
                        context | CTX_MEMO,
                        stats,
                        budget,
                        depth + 1,
                    )?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn analyze_document(
    core: &DocumentCore,
    budget: &mut CoverageBudget<'_>,
) -> Result<CoverageStats, HwpError> {
    let mut stats = CoverageStats::new();
    for section in &core.document.sections {
        walk_paragraphs(core, &section.paragraphs, 0, &mut stats, budget, 0)?;
        for master in &section.section_def.master_pages {
            walk_paragraphs(
                core,
                &master.paragraphs,
                CTX_MASTER_PAGE,
                &mut stats,
                budget,
                0,
            )?;
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
            font: key.font.to_string(),
            metric_face: key.metric_face.as_deref().map(str::to_string),
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
            font: key.legacy.font.to_string(),
            metric_face: key.legacy.metric_face.as_deref().map(str::to_string),
            language: LANGUAGE_NAMES[key.legacy.language as usize].to_string(),
            ratio: key.legacy.ratio,
            spacing: key.legacy.spacing,
            kerning: key.legacy.kerning,
            bold: key.legacy.bold,
            italic: key.legacy.italic,
            context: context_name(key.legacy.context),
            alignment: alignment_name(key.legacy.alignment),
            stored_line_seg: key.legacy.stored_lineseg,
            normalized_face: key.normalized_face.as_deref().map(str::to_string),
            subst_font: key.subst_font.as_deref().map(str::to_string),
            alt_type: key.alt_type,
            layout_family: key.layout_family.to_string(),
            metric_requested_face: key.metric_requested_face.as_deref().map(str::to_string),
            metric_resolved_face: key.metric_resolved_face.as_deref().map(str::to_string),
            match_kind: key.match_kind.to_string(),
            metric_entry: key.metric_entry,
            character_match: key.character_match.to_string(),
            width_source: key.width_source.to_string(),
            relation_type: key.relation_type.map(str::to_string),
            relation_evidence_status: key.relation_evidence_status.map(str::to_string),
            coverage_category: key.coverage_category.map(str::to_string),
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
    pub fn get_font_metric_coverage_analysis_native(
        &self,
        options_json: &str,
    ) -> Result<String, HwpError> {
        self.get_font_metric_coverage_analysis_inner(options_json, None)
    }

    /// Same read-only analysis with cooperative cancellation for an isolated worker.
    #[doc(hidden)]
    pub fn get_font_metric_coverage_analysis_with_cancel_native(
        &self,
        options_json: &str,
        cancellation: &AtomicBool,
    ) -> Result<String, HwpError> {
        self.get_font_metric_coverage_analysis_inner(options_json, Some(cancellation))
    }

    fn get_font_metric_coverage_analysis_inner(
        &self,
        options_json: &str,
        cancellation: Option<&AtomicBool>,
    ) -> Result<String, HwpError> {
        let limits = CoverageLimits::parse(options_json)?;
        let mut budget = CoverageBudget::new(limits, cancellation);
        let stats = analyze_document(self, &mut budget)?;
        reconcile(&stats)?;
        budget.consume(stats.legacy_usage.len() + stats.decision_usage.len())?;
        let legacy_usage = legacy_records(&stats);
        let decision_usage = decision_records(&stats);
        let source_runs_seen: u64 = stats.legacy_usage.values().map(|counts| counts.runs).sum();
        let legacy_projection = json!({
            "schemaVersion": LEGACY_FONT_LAYOUT_HABITS_SCHEMA_VERSION,
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
                    "cancelled": 0,
                    "drm": 0,
                    "empty": 0,
                    "encrypted": 0,
                    "parser": 0,
                    "resource-limit": 0,
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
        let serialized = serde_json::to_vec(&report)
            .map_err(|error| coverage_error(format!("serialize aggregate: {error}")))?;
        budget.check_output(serialized.len())?;
        String::from_utf8(serialized)
            .map_err(|error| coverage_error(format!("serialize UTF-8 aggregate: {error}")))
    }
}
