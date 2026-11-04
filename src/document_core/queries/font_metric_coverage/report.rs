//! 읽기 전용 집계의 공개 projection과 기존 정렬·분모를 보존한다.

use super::{
    coverage_error, CoverageStats, HwpError, CTX_CAPTION, CTX_ENDNOTE, CTX_FOOTER, CTX_FOOTNOTE,
    CTX_HEADER, CTX_HIDDEN_COMMENT, CTX_MASTER_PAGE, CTX_MEMO, CTX_TABLE_CELL, CTX_TEXT_BOX,
};
use crate::parser::FileFormat;
use serde::Serialize;

const LANGUAGE_NAMES: [&str; 7] = ["ko", "latin", "hanja", "ja", "other", "symbol", "user"];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LegacyUsageRecord {
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
pub(super) struct DecisionUsageRecord {
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

pub(super) fn format_name(format: FileFormat) -> &'static str {
    match format {
        FileFormat::Hwp => "hwp",
        FileFormat::Hwpx => "hwpx",
        FileFormat::Hwp3 => "hwp3",
        FileFormat::Hml => "hml",
        FileFormat::Pdf => "pdf",
        FileFormat::DrmProtected => "drm",
        FileFormat::Empty => "empty",
        FileFormat::Unknown => "unknown",
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

pub(super) fn legacy_records(stats: &CoverageStats) -> Vec<LegacyUsageRecord> {
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

pub(super) fn decision_records(stats: &CoverageStats) -> Vec<DecisionUsageRecord> {
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

pub(super) fn reconcile(stats: &CoverageStats) -> Result<(), HwpError> {
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
