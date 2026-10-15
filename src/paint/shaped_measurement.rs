//! Report-only shaped-width observations and caller-owned membership summaries.
//!
//! These values never choose replay variants, change layout positions, infer line
//! membership, or feed pagination. Unknown context remains distinct from absence.

use serde::{Serialize, Serializer};
use serde_json::{json, Value};

use crate::paint::{GlyphRunQuality, TextShapeReport, TextSourceSpan};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapedMeasurementRunReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_op_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_text_source")]
    pub source: Option<crate::paint::TextSourceSpan>,
    pub legacy_width_px: f64,
    pub shaped_width_px: f64,
    pub delta_px: f64,
    pub delta_ratio: f64,
    pub cluster_mismatch_count: u32,
    pub fallback_font_difference: bool,
    pub vertical_metric_difference: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shaping_quality: Option<GlyphRunQuality>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapedMeasurementLineReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph_id: Option<String>,
    pub line_index: u32,
    pub legacy_line_width_px: f64,
    pub shaped_line_width_px: f64,
    pub delta_px: f64,
    pub delta_ratio: f64,
    pub contributing_run_count: u32,
    pub max_run_delta_px: f64,
    pub fallback_font_difference_count: u32,
    pub vertical_metric_difference_count: u32,
    pub cluster_mismatch_count_sum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LineBreakChangeRisk {
    InsufficientContext,
    NoChangeLikely,
    ChangePossible,
    ChangeLikely,
}

impl LineBreakChangeRisk {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InsufficientContext => "insufficientContext",
            Self::NoChangeLikely => "noChangeLikely",
            Self::ChangePossible => "changePossible",
            Self::ChangeLikely => "changeLikely",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "camelCase")]
pub enum LineBreakContextValue<T> {
    Known(T),
    KnownAbsent,
    Unknown,
}

impl<T> LineBreakContextValue<T> {
    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCellConstraintSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_width_px: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_width_px: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_width_px: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabStopSummary {
    pub count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_tab_stop_px: Option<f64>,
    pub has_leaders: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JustificationMode {
    None,
    Left,
    Right,
    Center,
    Justify,
    Distributed,
}

impl JustificationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Left => "left",
            Self::Right => "right",
            Self::Center => "center",
            Self::Justify => "justify",
            Self::Distributed => "distributed",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineBreakShadowReport {
    pub document_id: Option<String>,
    pub sample_id: Option<String>,
    pub page_index: Option<u32>,
    pub paragraph_id: Option<String>,
    pub line_index: u32,
    pub has_full_layout_context: bool,
    pub legacy_available_width_px: LineBreakContextValue<f64>,
    pub paragraph_width_px: LineBreakContextValue<f64>,
    pub container_width_px: LineBreakContextValue<f64>,
    pub table_cell_constraint: LineBreakContextValue<TableCellConstraintSummary>,
    pub tab_stop_summary: LineBreakContextValue<TabStopSummary>,
    pub justification: LineBreakContextValue<JustificationMode>,
    pub legacy_line_segmentation_available: LineBreakContextValue<bool>,
    pub legacy_line_width_px: f64,
    pub shaped_line_width_px: f64,
    pub overflow_delta_px: Option<f64>,
    pub risk: LineBreakChangeRisk,
    pub reason: Option<String>,
}

impl LineBreakShadowReport {
    pub fn has_minimum_layout_context(&self) -> bool {
        self.has_full_layout_context
            && self.legacy_available_width_px.is_known()
            && (self.paragraph_width_px.is_known() || self.container_width_px.is_known())
            && !self.table_cell_constraint.is_unknown()
            && !self.tab_stop_summary.is_unknown()
            && !self.justification.is_unknown()
            && !self.legacy_line_segmentation_available.is_unknown()
    }

    pub fn reported_risk(&self) -> LineBreakChangeRisk {
        if self.has_minimum_layout_context() {
            self.risk
        } else {
            LineBreakChangeRisk::InsufficientContext
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapedMeasurementParagraphSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph_id: Option<String>,
    pub line_count: u32,
    pub run_count: u32,
    pub max_run_delta_px: f64,
    pub max_line_delta_px: f64,
    pub max_line_delta_ratio: f64,
    pub total_abs_delta_px: f64,
    pub cluster_mismatch_count_sum: u32,
    pub fallback_font_difference_count: u32,
    pub vertical_metric_difference_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapedMeasurementPageSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_index: Option<u32>,
    pub paragraph_count: u32,
    pub line_count: u32,
    pub run_count: u32,
    pub max_run_delta_px: f64,
    pub max_line_delta_px: f64,
    pub total_abs_delta_px: f64,
    pub fallback_font_difference_count: u32,
    pub vertical_metric_difference_count: u32,
    pub cluster_mismatch_count_sum: u32,
}

impl TextShapeReport {
    /// Adds a report-only line summary from already-collected run measurements.
    ///
    /// The caller owns line membership. This method deliberately does not infer
    /// line breaks or mutate layout; it only aggregates telemetry for future
    /// layout migration analysis.
    pub fn summarize_shaped_measurement_line(
        &mut self,
        document_id: Option<String>,
        sample_id: Option<String>,
        page_index: Option<u32>,
        paragraph_id: Option<String>,
        line_index: u32,
        run_indices: &[usize],
    ) -> Option<&ShapedMeasurementLineReport> {
        let mut contributing_run_count = 0u32;
        let mut legacy_line_width_px = 0.0;
        let mut shaped_line_width_px = 0.0;
        let mut max_run_delta_px = 0.0f64;
        let mut fallback_font_difference_count = 0u32;
        let mut vertical_metric_difference_count = 0u32;
        let mut cluster_mismatch_count_sum = 0u32;

        for index in run_indices {
            let Some(run) = self.shaped_measurements.get(*index) else {
                continue;
            };
            contributing_run_count = contributing_run_count.saturating_add(1);
            legacy_line_width_px += run.legacy_width_px;
            shaped_line_width_px += run.shaped_width_px;
            max_run_delta_px = max_run_delta_px.max(run.delta_px.abs());
            if run.fallback_font_difference {
                fallback_font_difference_count = fallback_font_difference_count.saturating_add(1);
            }
            if run.vertical_metric_difference {
                vertical_metric_difference_count =
                    vertical_metric_difference_count.saturating_add(1);
            }
            cluster_mismatch_count_sum =
                cluster_mismatch_count_sum.saturating_add(run.cluster_mismatch_count);
        }

        if contributing_run_count == 0 {
            return None;
        }

        let delta_px = shaped_line_width_px - legacy_line_width_px;
        let delta_ratio = if legacy_line_width_px.abs() > f64::EPSILON {
            delta_px / legacy_line_width_px
        } else {
            0.0
        };

        self.shaped_measurement_lines
            .push(ShapedMeasurementLineReport {
                document_id,
                sample_id,
                page_index,
                paragraph_id,
                line_index,
                legacy_line_width_px,
                shaped_line_width_px,
                delta_px,
                delta_ratio,
                contributing_run_count,
                max_run_delta_px,
                fallback_font_difference_count,
                vertical_metric_difference_count,
                cluster_mismatch_count_sum,
            });
        self.shaped_measurement_lines.last()
    }

    /// Adds a report-only paragraph summary from existing line summaries.
    ///
    /// The caller owns paragraph membership. This method does not infer
    /// paragraph boundaries, reflow lines, or decide whether line breaks would
    /// change under shaped measurement.
    pub fn summarize_shaped_measurement_paragraph(
        &mut self,
        document_id: Option<String>,
        sample_id: Option<String>,
        page_index: Option<u32>,
        paragraph_id: Option<String>,
        line_indices: &[usize],
    ) -> Option<&ShapedMeasurementParagraphSummary> {
        let mut line_count = 0u32;
        let mut run_count = 0u32;
        let mut max_run_delta_px = 0.0f64;
        let mut max_line_delta_px = 0.0f64;
        let mut max_line_delta_ratio = 0.0f64;
        let mut total_abs_delta_px = 0.0f64;
        let mut cluster_mismatch_count_sum = 0u32;
        let mut fallback_font_difference_count = 0u32;
        let mut vertical_metric_difference_count = 0u32;

        for index in line_indices {
            let Some(line) = self.shaped_measurement_lines.get(*index) else {
                continue;
            };
            line_count = line_count.saturating_add(1);
            run_count = run_count.saturating_add(line.contributing_run_count);
            max_run_delta_px = max_run_delta_px.max(line.max_run_delta_px.abs());
            max_line_delta_px = max_line_delta_px.max(line.delta_px.abs());
            max_line_delta_ratio = max_line_delta_ratio.max(line.delta_ratio.abs());
            total_abs_delta_px += line.delta_px.abs();
            cluster_mismatch_count_sum =
                cluster_mismatch_count_sum.saturating_add(line.cluster_mismatch_count_sum);
            fallback_font_difference_count =
                fallback_font_difference_count.saturating_add(line.fallback_font_difference_count);
            vertical_metric_difference_count = vertical_metric_difference_count
                .saturating_add(line.vertical_metric_difference_count);
        }

        if line_count == 0 {
            return None;
        }

        self.shaped_measurement_paragraphs
            .push(ShapedMeasurementParagraphSummary {
                document_id,
                sample_id,
                page_index,
                paragraph_id,
                line_count,
                run_count,
                max_run_delta_px,
                max_line_delta_px,
                max_line_delta_ratio,
                total_abs_delta_px,
                cluster_mismatch_count_sum,
                fallback_font_difference_count,
                vertical_metric_difference_count,
            });
        self.shaped_measurement_paragraphs.last()
    }

    /// Adds a report-only page summary from existing paragraph summaries.
    ///
    /// The caller owns page membership. This method is telemetry-only and does
    /// not feed layout measurement, pagination, or CI pass/fail decisions.
    pub fn summarize_shaped_measurement_page(
        &mut self,
        document_id: Option<String>,
        sample_id: Option<String>,
        page_index: Option<u32>,
        paragraph_indices: &[usize],
    ) -> Option<&ShapedMeasurementPageSummary> {
        let mut paragraph_count = 0u32;
        let mut line_count = 0u32;
        let mut run_count = 0u32;
        let mut max_run_delta_px = 0.0f64;
        let mut max_line_delta_px = 0.0f64;
        let mut total_abs_delta_px = 0.0f64;
        let mut fallback_font_difference_count = 0u32;
        let mut vertical_metric_difference_count = 0u32;
        let mut cluster_mismatch_count_sum = 0u32;

        for index in paragraph_indices {
            let Some(paragraph) = self.shaped_measurement_paragraphs.get(*index) else {
                continue;
            };
            paragraph_count = paragraph_count.saturating_add(1);
            line_count = line_count.saturating_add(paragraph.line_count);
            run_count = run_count.saturating_add(paragraph.run_count);
            max_run_delta_px = max_run_delta_px.max(paragraph.max_run_delta_px.abs());
            max_line_delta_px = max_line_delta_px.max(paragraph.max_line_delta_px.abs());
            total_abs_delta_px += paragraph.total_abs_delta_px;
            fallback_font_difference_count = fallback_font_difference_count
                .saturating_add(paragraph.fallback_font_difference_count);
            vertical_metric_difference_count = vertical_metric_difference_count
                .saturating_add(paragraph.vertical_metric_difference_count);
            cluster_mismatch_count_sum =
                cluster_mismatch_count_sum.saturating_add(paragraph.cluster_mismatch_count_sum);
        }

        if paragraph_count == 0 {
            return None;
        }

        self.shaped_measurement_pages
            .push(ShapedMeasurementPageSummary {
                document_id,
                sample_id,
                page_index,
                paragraph_count,
                line_count,
                run_count,
                max_run_delta_px,
                max_line_delta_px,
                total_abs_delta_px,
                fallback_font_difference_count,
                vertical_metric_difference_count,
                cluster_mismatch_count_sum,
            });
        self.shaped_measurement_pages.last()
    }

    /// A separate telemetry artifact, not an extension to paint/replay schema.
    /// Non-finite caller-provided numbers serialize as null.
    pub fn shaped_measurements_json(&self) -> String {
        json!({
            "shapedMeasurements": self.shaped_measurements,
            "shapedMeasurementLines": self.shaped_measurement_lines,
            "lineBreakShadows": self.line_break_shadows,
            "shapedMeasurementParagraphs": self.shaped_measurement_paragraphs,
            "shapedMeasurementPages": self.shaped_measurement_pages,
        })
        .to_string()
    }
}

impl Serialize for LineBreakShadowReport {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Serialize the gated risk, never the optimistic caller-supplied risk
        // when minimum layout context is missing.
        let mut value = json!({
            "lineIndex": self.line_index,
            "hasFullLayoutContext": self.has_full_layout_context,
            "legacyAvailableWidthPx": self.legacy_available_width_px,
            "paragraphWidthPx": self.paragraph_width_px,
            "containerWidthPx": self.container_width_px,
            "tableCellConstraint": self.table_cell_constraint,
            "tabStopSummary": self.tab_stop_summary,
            "justification": self.justification,
            "legacyLineSegmentationAvailable": self.legacy_line_segmentation_available,
            "legacyLineWidthPx": self.legacy_line_width_px,
            "shapedLineWidthPx": self.shaped_line_width_px,
            "risk": self.reported_risk(),
        });
        for (key, field) in [
            ("documentId", self.document_id.as_ref()),
            ("sampleId", self.sample_id.as_ref()),
            ("paragraphId", self.paragraph_id.as_ref()),
            ("reason", self.reason.as_ref()),
        ] {
            if let Some(field) = field {
                value[key] = json!(field);
            }
        }
        if let Some(page_index) = self.page_index {
            value["pageIndex"] = json!(page_index);
        }
        if let Some(delta) = self.overflow_delta_px {
            value["overflowDeltaPx"] = json!(delta);
        }
        value.serialize(serializer)
    }
}

fn serialize_text_source<S: Serializer>(
    source: &Option<TextSourceSpan>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let Some(source) = source else {
        return serializer.serialize_none();
    };
    // Reuse today's stable-source representation, not the obsolete skia
    // { stableSourceKey: { scheme: ... } } wrapper.
    let mut json = String::new();
    crate::paint::json::write_text_source_span(&mut json, source);
    let value: Value = serde_json::from_str(&json).map_err(serde::ser::Error::custom)?;
    value.serialize(serializer)
}
