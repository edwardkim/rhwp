use serde::{Deserialize, Serialize};

use crate::document_core::helpers::get_textbox_from_shape;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::ShapeObject;
use crate::renderer::composer::{
    capture_line_break_measurement, layout_paragraph_in_frame, layout_picture_band,
    stored_rows_require_external_geometry, trace_paragraph_scope, LineBreakMeasurementTrace,
};
use crate::renderer::hwpunit_to_px;
use crate::renderer::layout_frame::{
    capture_frame_carves, physical_row_ranges, FrameCarveTrace, ParagraphBox,
};
use crate::renderer::page_layout::PageLayoutInfo;

fn enabled_by_default() -> bool {
    true
}

fn default_record_limit() -> usize {
    128
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LineBreakProvenanceOptions {
    #[serde(default = "enabled_by_default")]
    geometry: bool,
    #[serde(default = "enabled_by_default")]
    measurement: bool,
    #[serde(default)]
    page_index: Option<u32>,
    #[serde(default)]
    text_x: Option<f64>,
    #[serde(default)]
    group_path: Vec<usize>,
    #[serde(default)]
    visible_frame_width_hwp: Option<i32>,
    #[serde(default)]
    geometry_mode: Option<String>,
    #[serde(default = "default_record_limit")]
    max_records: usize,
    #[serde(default = "default_record_limit")]
    max_carves: usize,
}

impl Default for LineBreakProvenanceOptions {
    fn default() -> Self {
        Self {
            geometry: true,
            measurement: true,
            page_index: None,
            text_x: None,
            group_path: Vec::new(),
            visible_frame_width_hwp: None,
            geometry_mode: None,
            max_records: default_record_limit(),
            max_carves: default_record_limit(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CellPathCoordinate {
    control_index: usize,
    cell_index: usize,
    cell_para_index: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParagraphCoordinates {
    section_idx: usize,
    paragraph_idx: Option<usize>,
    parent_para_idx: Option<usize>,
    cell_path: Vec<CellPathCoordinate>,
    group_path: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HorizontalRange {
    start: i32,
    end: i32,
    width: i32,
}

impl HorizontalRange {
    fn from_range(range: std::ops::Range<i32>) -> Self {
        Self {
            start: range.start,
            end: range.end,
            width: range.end.saturating_sub(range.start),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParagraphBoxTrace {
    coordinate_system: &'static str,
    declared: HorizontalRange,
    effective: HorizontalRange,
    origin_is_derivable: bool,
    width_px: f64,
}

fn stored_cache_is_eligible(paragraph: &Paragraph) -> bool {
    !paragraph.line_segs.is_empty()
        && paragraph.layout_only_fill_lines == 0
        && !paragraph.stored_text_partition_is_dirty()
        && paragraph
            .line_segs
            .iter()
            .all(|segment| segment.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
}

fn stored_rows_are_well_formed(paragraph: &Paragraph) -> bool {
    let line_segs = &paragraph.line_segs;
    let Some(rows) = physical_row_ranges(line_segs) else {
        return false;
    };
    line_segs
        .windows(2)
        .all(|pair| pair[0].text_start <= pair[1].text_start)
        && rows.into_iter().all(|row| {
            let first = &line_segs[row.start];
            let first_vpos = comparison_vertical_pos(paragraph, row.start, true);
            row.clone().all(|index| {
                let segment = &line_segs[index];
                comparison_vertical_pos(paragraph, index, true) == first_vpos
                    && segment.line_height == first.line_height
                    && segment.text_height == first.text_height
                    && segment.baseline_distance == first.baseline_distance
                    && segment.line_spacing == first.line_spacing
            })
        })
}

pub(crate) fn paragraph_stream_range(
    paragraph: &Paragraph,
    start_scalar: usize,
    end_scalar: usize,
) -> (u32, u32) {
    let scalar_count = paragraph.text.chars().count();
    let offset = |index: usize| {
        paragraph
            .char_offsets
            .get(index)
            .copied()
            .unwrap_or_else(|| {
                if index >= scalar_count {
                    paragraph.char_count.saturating_sub(1)
                } else {
                    paragraph
                        .text
                        .chars()
                        .take(index)
                        .map(|character| character.len_utf16() as u32)
                        .sum()
                }
            })
    };
    let start = if start_scalar == 0 {
        0
    } else {
        offset(start_scalar)
    };
    (start, offset(end_scalar).max(start))
}

pub(super) fn descend_group_path_segment<'a>(
    shape: &'a ShapeObject,
    group_path: &[usize],
    depth: &mut usize,
) -> Result<&'a ShapeObject, HwpError> {
    let mut target = shape;
    while let Some(&child_index) = group_path.get(*depth) {
        let ShapeObject::Group(group) = target else {
            break;
        };
        target = group.children.get(child_index).ok_or_else(|| {
            HwpError::RenderError(format!(
                "groupPath[{}] child {child_index} 범위 초과 (총 {}개)",
                *depth,
                group.children.len()
            ))
        })?;
        *depth += 1;
    }
    Ok(target)
}

const ROW_SNAPSHOT_SEGMENT_LIMIT: usize = 16;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SegmentFrameCalculation {
    column_start: i32,
    segment_width: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RowMetricsCalculation {
    line_height: i32,
    text_height: i32,
    baseline_distance: i32,
    line_spacing: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct VerticalFlowCalculation {
    /// Origin-independent position in this paragraph's physical-row ladder.
    delta_from_first: i64,
    /// Origin-independent advance from the preceding physical row.
    delta_from_previous: Option<i64>,
    /// Stored-domain observation. A one-sided reset makes the local vertical
    /// lane unmodelled; it is never promoted to an absolute-origin mismatch.
    reset: bool,
    /// Strict backward movement is invariant under a constant origin shift.
    rewind: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PhysicalRowCalculation {
    segment_count: usize,
    segment_window_start: usize,
    text_starts: Vec<u32>,
    segment_frames: Vec<SegmentFrameCalculation>,
    segments_truncated: bool,
    metrics: RowMetricsCalculation,
    vertical_flow: VerticalFlowCalculation,
}

#[derive(Debug, Clone, Copy)]
struct OrderedMismatch {
    kind: &'static str,
    field: &'static str,
    row_index: usize,
    segment_index: Option<usize>,
    legacy_segment_index: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RowDifferenceObservation {
    field: &'static str,
    row_index: usize,
    segment_index: Option<usize>,
    stored_row: PhysicalRowCalculation,
    fresh_row: PhysicalRowCalculation,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundaryComparison {
    comparable: bool,
    matches: Option<bool>,
    first_mismatch_kind: Option<&'static str>,
    first_mismatch_field: Option<&'static str>,
    first_mismatch_row_index: Option<usize>,
    first_mismatch_segment_index: Option<usize>,
    first_mismatch_index: Option<usize>,
    stored_mismatch_utf16_start: Option<u32>,
    fresh_mismatch_utf16_start: Option<u32>,
    stored_mismatch_row_part: Option<&'static str>,
    fresh_mismatch_row_part: Option<&'static str>,
    stored_mismatch_row: Option<PhysicalRowCalculation>,
    fresh_mismatch_row: Option<PhysicalRowCalculation>,
    metric_observation: Option<RowDifferenceObservation>,
    vertical_flow_observation: Option<RowDifferenceObservation>,
    horizontal_origin_identity_proven: bool,
    vertical_origin_identity_proven: bool,
    stored_starts_truncated: bool,
    stored_utf16_starts: Vec<u32>,
    fresh_starts_truncated: bool,
    fresh_utf16_starts: Vec<u32>,
}

fn row_part(segment: &LineSeg) -> &'static str {
    match (segment.is_first_segment(), segment.is_last_segment()) {
        (true, true) => "single",
        (true, false) => "first",
        (false, true) => "last",
        (false, false) => "middle",
    }
}

fn row_segment_index(
    rows: &[std::ops::Range<usize>],
    row_index: usize,
    segment_index: Option<usize>,
) -> Option<usize> {
    let row = rows.get(row_index)?;
    let index = segment_index.unwrap_or(0);
    (index < row.len()).then_some(row.start + index)
}

fn comparison_vertical_pos(paragraph: &Paragraph, index: usize, source: bool) -> Option<i32> {
    if source {
        if let Some(values) = &paragraph.source_line_seg_vertical_pos {
            return values.get(index).copied();
        }
    }
    paragraph
        .line_segs
        .get(index)
        .map(|segment| segment.vertical_pos)
}

fn vertical_flow_calculations(
    paragraph: &Paragraph,
    rows: &[std::ops::Range<usize>],
    source: bool,
) -> Option<Vec<VerticalFlowCalculation>> {
    let mut calculations = Vec::with_capacity(rows.len());
    let mut previous = None;
    let mut epoch_first = None;
    for (row_index, row) in rows.iter().enumerate() {
        let current = comparison_vertical_pos(paragraph, row.start, source)?;
        let reset = row_index > 0 && current == 0;
        let rewind = previous.is_some_and(|value| current < value);
        let at_epoch_start = row_index == 0 || reset || rewind;
        if at_epoch_start {
            epoch_first = Some(current);
        }
        calculations.push(VerticalFlowCalculation {
            delta_from_first: i64::from(current) - i64::from(epoch_first?),
            delta_from_previous: (!at_epoch_start)
                .then(|| previous.map(|value| i64::from(current) - i64::from(value)))
                .flatten(),
            reset,
            rewind,
        });
        previous = Some(current);
    }
    Some(calculations)
}

fn physical_row_calculation(
    paragraph: &Paragraph,
    rows: &[std::ops::Range<usize>],
    vertical_flow: &[VerticalFlowCalculation],
    row_index: usize,
    sample_segment_index: Option<usize>,
) -> Option<PhysicalRowCalculation> {
    let row = rows.get(row_index)?;
    let sampled = sample_segment_index
        .filter(|index| *index < row.len())
        .unwrap_or(0);
    let sampled_segment = paragraph.line_segs.get(row.start + sampled)?;
    let snapshot_len = row.len().min(ROW_SNAPSHOT_SEGMENT_LIMIT);
    let segment_window_start = sample_segment_index
        .filter(|index| *index >= snapshot_len)
        .map(|index| {
            index
                .saturating_sub(snapshot_len / 2)
                .min(row.len().saturating_sub(snapshot_len))
        })
        .unwrap_or(0);
    let snapshot =
        row.start + segment_window_start..row.start + segment_window_start + snapshot_len;
    let vertical_flow = *vertical_flow.get(row_index)?;
    Some(PhysicalRowCalculation {
        segment_count: row.len(),
        segment_window_start,
        text_starts: snapshot
            .clone()
            .map(|index| paragraph.line_seg_text_start(index))
            .collect(),
        segment_frames: snapshot
            .map(|index| {
                let segment = &paragraph.line_segs[index];
                SegmentFrameCalculation {
                    column_start: segment.column_start,
                    segment_width: segment.segment_width,
                }
            })
            .collect(),
        segments_truncated: row.len() > ROW_SNAPSHOT_SEGMENT_LIMIT,
        metrics: RowMetricsCalculation {
            line_height: sampled_segment.line_height,
            text_height: sampled_segment.text_height,
            baseline_distance: sampled_segment.baseline_distance,
            line_spacing: sampled_segment.line_spacing,
        },
        vertical_flow,
    })
}

fn mismatch_at(
    kind: &'static str,
    field: &'static str,
    row_index: usize,
    segment_index: Option<usize>,
    stored_rows: &[std::ops::Range<usize>],
    fresh_rows: &[std::ops::Range<usize>],
    stored_len: usize,
    fresh_len: usize,
) -> OrderedMismatch {
    let stored_index = row_segment_index(stored_rows, row_index, segment_index);
    let fresh_index = row_segment_index(fresh_rows, row_index, segment_index);
    OrderedMismatch {
        kind,
        field,
        row_index,
        segment_index,
        legacy_segment_index: stored_index
            .into_iter()
            .chain(fresh_index)
            .min()
            .unwrap_or_else(|| stored_len.min(fresh_len)),
    }
}

/// Compare physical rows in causal order rather than in flattened field order.
///
/// A later text-boundary mismatch is more useful than a row's metric residual,
/// because it establishes that the two calculations no longer describe the
/// same rows. Horizontal Frame geometry comes next. Metrics and the
/// origin-independent vertical ladder remain in each row snapshot as
/// observations: the production admission path deliberately retains valid
/// stored metric residuals, which also shift the next fresh row's local top.
fn first_ordered_mismatch(
    stored: &Paragraph,
    fresh: &Paragraph,
    stored_rows: &[std::ops::Range<usize>],
    fresh_rows: &[std::ops::Range<usize>],
    compare_column_start: bool,
) -> Option<OrderedMismatch> {
    let common_rows = stored_rows.len().min(fresh_rows.len());
    let make = |kind, field, row_index, segment_index| {
        mismatch_at(
            kind,
            field,
            row_index,
            segment_index,
            stored_rows,
            fresh_rows,
            stored.line_segs.len(),
            fresh.line_segs.len(),
        )
    };

    // 1. Physical-row topology, then logical text starts.
    for row_index in 0..common_rows {
        let stored_count = stored_rows[row_index].len();
        let fresh_count = fresh_rows[row_index].len();
        if stored_count != fresh_count {
            return Some(make(
                "topology",
                "segmentCount",
                row_index,
                Some(stored_count.min(fresh_count)),
            ));
        }
    }
    if stored_rows.len() != fresh_rows.len() {
        return Some(make("topology", "rowCount", common_rows, None));
    }
    for row_index in 0..common_rows {
        let stored_row = &stored_rows[row_index];
        let fresh_row = &fresh_rows[row_index];
        for segment_index in 0..stored_row.len() {
            if stored.line_seg_text_start(stored_row.start + segment_index)
                != fresh.line_seg_text_start(fresh_row.start + segment_index)
            {
                return Some(make(
                    "textStart",
                    "textStart",
                    row_index,
                    Some(segment_index),
                ));
            }
        }
    }

    // 2. The Frame's exact horizontal cache key.
    for row_index in 0..common_rows {
        let stored_row = &stored_rows[row_index];
        let fresh_row = &fresh_rows[row_index];
        for segment_index in 0..stored_row.len() {
            let stored_segment = &stored.line_segs[stored_row.start + segment_index];
            let fresh_segment = &fresh.line_segs[fresh_row.start + segment_index];
            if compare_column_start && stored_segment.column_start != fresh_segment.column_start {
                return Some(make(
                    "horizontalFrame",
                    "columnStart",
                    row_index,
                    Some(segment_index),
                ));
            }
            if stored_segment.segment_width != fresh_segment.segment_width {
                return Some(make(
                    "horizontalFrame",
                    "segmentWidth",
                    row_index,
                    Some(segment_index),
                ));
            }
        }
    }

    // Vertical flow is not an error predicate while absolute origin ownership
    // is unavailable. Even relative deltas inherit valid metric residuals from
    // the preceding row. The epoch-pattern argument still fail-closes the
    // enclosing comparison on an unreplayed reset/rewind boundary.
    None
}

fn first_metric_difference(
    stored: &Paragraph,
    fresh: &Paragraph,
    stored_rows: &[std::ops::Range<usize>],
    fresh_rows: &[std::ops::Range<usize>],
) -> Option<OrderedMismatch> {
    for (row_index, (stored_row, fresh_row)) in stored_rows.iter().zip(fresh_rows).enumerate() {
        for segment_index in 0..stored_row.len() {
            let stored_segment = &stored.line_segs[stored_row.start + segment_index];
            let fresh_segment = &fresh.line_segs[fresh_row.start + segment_index];
            for (field, differs) in [
                (
                    "lineHeight",
                    stored_segment.line_height != fresh_segment.line_height,
                ),
                (
                    "textHeight",
                    stored_segment.text_height != fresh_segment.text_height,
                ),
                (
                    "baselineDistance",
                    stored_segment.baseline_distance != fresh_segment.baseline_distance,
                ),
                (
                    "lineSpacing",
                    stored_segment.line_spacing != fresh_segment.line_spacing,
                ),
            ] {
                if differs {
                    return Some(mismatch_at(
                        "metrics",
                        field,
                        row_index,
                        Some(segment_index),
                        stored_rows,
                        fresh_rows,
                        stored.line_segs.len(),
                        fresh.line_segs.len(),
                    ));
                }
            }
        }
    }
    None
}

fn first_vertical_flow_difference(
    stored: &Paragraph,
    fresh: &Paragraph,
    stored_rows: &[std::ops::Range<usize>],
    fresh_rows: &[std::ops::Range<usize>],
    stored_flow: &[VerticalFlowCalculation],
    fresh_flow: &[VerticalFlowCalculation],
) -> Option<OrderedMismatch> {
    for (row_index, (stored_flow, fresh_flow)) in stored_flow.iter().zip(fresh_flow).enumerate() {
        let field = if stored_flow.delta_from_previous != fresh_flow.delta_from_previous {
            "deltaFromPrevious"
        } else if stored_flow.delta_from_first != fresh_flow.delta_from_first {
            "deltaFromFirst"
        } else {
            continue;
        };
        return Some(mismatch_at(
            "verticalFlow",
            field,
            row_index,
            None,
            stored_rows,
            fresh_rows,
            stored.line_segs.len(),
            fresh.line_segs.len(),
        ));
    }
    None
}

fn observe_row_difference(
    difference: OrderedMismatch,
    stored: &Paragraph,
    fresh: &Paragraph,
    stored_rows: &[std::ops::Range<usize>],
    fresh_rows: &[std::ops::Range<usize>],
    stored_flow: &[VerticalFlowCalculation],
    fresh_flow: &[VerticalFlowCalculation],
) -> Option<RowDifferenceObservation> {
    Some(RowDifferenceObservation {
        field: difference.field,
        row_index: difference.row_index,
        segment_index: difference.segment_index,
        stored_row: physical_row_calculation(
            stored,
            stored_rows,
            stored_flow,
            difference.row_index,
            difference.segment_index,
        )?,
        fresh_row: physical_row_calculation(
            fresh,
            fresh_rows,
            fresh_flow,
            difference.row_index,
            difference.segment_index,
        )?,
    })
}

pub(crate) fn compare_boundaries(
    stored: &Paragraph,
    fresh: &Paragraph,
    comparable: bool,
    compare_column_start: bool,
    limit: usize,
) -> BoundaryComparison {
    let stored_rows = physical_row_ranges(&stored.line_segs);
    let fresh_rows = physical_row_ranges(&fresh.line_segs);
    let structurally_comparable = comparable && stored_rows.is_some() && fresh_rows.is_some();
    let stored_rows = stored_rows.unwrap_or_default();
    let fresh_rows = fresh_rows.unwrap_or_default();
    let stored_flow = structurally_comparable
        .then(|| vertical_flow_calculations(stored, &stored_rows, true))
        .flatten();
    let fresh_flow = structurally_comparable
        .then(|| vertical_flow_calculations(fresh, &fresh_rows, false))
        .flatten();
    let vertical_epochs_comparable = stored_flow
        .as_deref()
        .zip(fresh_flow.as_deref())
        .is_some_and(|(stored, fresh)| {
            stored.len() == fresh.len()
                && stored.iter().zip(fresh).all(|(stored, fresh)| {
                    stored.reset == fresh.reset && stored.rewind == fresh.rewind
                })
        });
    let mismatch = structurally_comparable
        .then(|| {
            first_ordered_mismatch(
                stored,
                fresh,
                &stored_rows,
                &fresh_rows,
                compare_column_start,
            )
        })
        .flatten();
    let observations_are_comparable = structurally_comparable && mismatch.is_none();
    let flows = stored_flow.as_deref().zip(fresh_flow.as_deref());
    let metric_observation = flows.and_then(|(stored_flow, fresh_flow)| {
        observations_are_comparable
            .then(|| first_metric_difference(stored, fresh, &stored_rows, &fresh_rows))
            .flatten()
            .and_then(|difference| {
                observe_row_difference(
                    difference,
                    stored,
                    fresh,
                    &stored_rows,
                    &fresh_rows,
                    stored_flow,
                    fresh_flow,
                )
            })
    });
    let vertical_flow_observation = flows.and_then(|(stored_flow, fresh_flow)| {
        (observations_are_comparable && vertical_epochs_comparable)
            .then(|| {
                first_vertical_flow_difference(
                    stored,
                    fresh,
                    &stored_rows,
                    &fresh_rows,
                    stored_flow,
                    fresh_flow,
                )
            })
            .flatten()
            .and_then(|difference| {
                observe_row_difference(
                    difference,
                    stored,
                    fresh,
                    &stored_rows,
                    &fresh_rows,
                    stored_flow,
                    fresh_flow,
                )
            })
    });
    // A one-sided reset/rewind is a page/column-owner boundary that the local
    // frame did not replay. Earlier semantic mismatches remain reportable; if
    // all earlier lanes match, fail closed instead of comparing across epochs.
    let comparable = structurally_comparable && (mismatch.is_some() || vertical_epochs_comparable);
    let reported_index = mismatch.map(|value| value.legacy_segment_index);
    BoundaryComparison {
        comparable,
        matches: comparable.then_some(mismatch.is_none()),
        first_mismatch_kind: mismatch.map(|value| value.kind),
        first_mismatch_field: mismatch.map(|value| value.field),
        first_mismatch_row_index: mismatch.map(|value| value.row_index),
        first_mismatch_segment_index: mismatch.and_then(|value| value.segment_index),
        first_mismatch_index: reported_index,
        stored_mismatch_utf16_start: reported_index
            .filter(|&index| index < stored.line_segs.len())
            .map(|index| stored.line_seg_text_start(index)),
        fresh_mismatch_utf16_start: reported_index
            .filter(|&index| index < fresh.line_segs.len())
            .map(|index| fresh.line_seg_text_start(index)),
        stored_mismatch_row_part: reported_index
            .and_then(|index| stored.line_segs.get(index))
            .map(row_part),
        fresh_mismatch_row_part: reported_index
            .and_then(|index| fresh.line_segs.get(index))
            .map(row_part),
        stored_mismatch_row: mismatch.and_then(|value| {
            physical_row_calculation(
                stored,
                &stored_rows,
                stored_flow.as_deref()?,
                value.row_index,
                value.segment_index,
            )
        }),
        fresh_mismatch_row: mismatch.and_then(|value| {
            physical_row_calculation(
                fresh,
                &fresh_rows,
                fresh_flow.as_deref()?,
                value.row_index,
                value.segment_index,
            )
        }),
        metric_observation,
        vertical_flow_observation,
        horizontal_origin_identity_proven: compare_column_start,
        vertical_origin_identity_proven: false,
        stored_starts_truncated: stored.line_segs.len() > limit,
        stored_utf16_starts: stored
            .line_segs
            .iter()
            .enumerate()
            .map(|(index, _)| stored.line_seg_text_start(index))
            .take(limit)
            .collect(),
        fresh_starts_truncated: fresh.line_segs.len() > limit,
        fresh_utf16_starts: fresh
            .line_segs
            .iter()
            .enumerate()
            .map(|(index, _)| fresh.line_seg_text_start(index))
            .take(limit)
            .collect(),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TraceLane<T> {
    enabled: bool,
    owner: Option<&'static str>,
    total_records: usize,
    truncated: bool,
    records: T,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MeasurementTraceLane {
    enabled: bool,
    records: LineBreakMeasurementTrace,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LineBreakProvenanceReport {
    schema_version: u8,
    coordinates: ParagraphCoordinates,
    text_utf16_length: u32,
    paragraph_box: ParagraphBoxTrace,
    geometry_source: &'static str,
    fresh_geometry_complete: bool,
    comparison: BoundaryComparison,
    geometry: TraceLane<Vec<FrameCarveTrace>>,
    measurement: MeasurementTraceLane,
}

impl DocumentCore {
    pub(crate) fn line_break_provenance_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path: &[(usize, usize, usize)],
        options_json: &str,
    ) -> Result<String, HwpError> {
        let options = if options_json.trim().is_empty() {
            LineBreakProvenanceOptions::default()
        } else {
            serde_json::from_str(options_json).map_err(|error| {
                HwpError::RenderError(format!("line-break provenance options 오류: {error}"))
            })?
        };
        let paragraph = if options.group_path.is_empty() {
            self.resolve_control_para(section_idx, parent_para_idx, cell_path)?
        } else {
            self.resolve_group_textbox_paragraph(
                section_idx,
                parent_para_idx,
                cell_path,
                &options.group_path,
            )?
        };
        let picture_band_owner = cell_path
            .is_empty()
            .then(|| self.picture_band_owning_body_paragraph(section_idx, parent_para_idx))
            .flatten();
        let (paragraph_box, mut geometry_source, mut fresh_geometry_complete) = self
            .line_break_paragraph_box(
                section_idx,
                parent_para_idx,
                cell_path,
                paragraph,
                &options,
            )?;
        if let Some((host_index, _, _)) = picture_band_owner.as_ref() {
            let mapped_column = self
                .para_column_map
                .get(section_idx)
                .and_then(|columns| columns.get(*host_index))
                .copied();
            let section = &self.document.sections[section_idx];
            let column_def = Self::find_column_def_for_paragraph(&section.paragraphs, *host_index);
            let layout =
                PageLayoutInfo::from_page_def(&section.section_def.page_def, &column_def, self.dpi);
            fresh_geometry_complete = mapped_column
                .is_some_and(|index| layout.column_areas.get(index as usize).is_some());
            geometry_source = if fresh_geometry_complete {
                "visible-picture-band-frame"
            } else {
                "picture-band-column-fallback"
            };
        }
        let max_records = options.max_records.min(512);
        let max_carves = options.max_carves.min(512);
        let mut fresh_paragraph = paragraph.clone();
        fresh_paragraph.line_segs.clear();
        let _trace_scope = trace_paragraph_scope(Some(parent_para_idx));
        let ((fresh_rows_supported, measurement), carves) =
            capture_frame_carves(options.geometry, max_carves, Some(parent_para_idx), || {
                capture_line_break_measurement(
                    options.measurement,
                    max_records,
                    Some(parent_para_idx),
                    || {
                        let band_lines = picture_band_owner.as_ref().and_then(
                            |(host_index, paragraph_range, column_width)| {
                                let section = self.document.sections.get(section_idx)?;
                                let band = layout_picture_band(
                                    &section.paragraphs,
                                    *host_index,
                                    *column_width,
                                    &self.styles,
                                    self.dpi,
                                )?;
                                let offset = parent_para_idx.checked_sub(paragraph_range.start)?;
                                band.line_segs.get(offset).cloned()
                            },
                        );
                        if let Some(lines) = band_lines {
                            fresh_paragraph.replace_line_segs(lines);
                            true
                        } else {
                            // The query owns this diagnostic-only frame. Its
                            // zero top is intentionally local: ParagraphBox
                            // proves the horizontal coordinate system, not a
                            // page/column vpos origin. The resulting rows still
                            // traverse the production carve -> fill -> commit
                            // -> projection hooks used by stored admission.
                            let mut frame = paragraph_box.frame(0);
                            layout_paragraph_in_frame(
                                &fresh_paragraph,
                                &mut frame,
                                &self.styles,
                                self.dpi,
                            )
                            .is_some_and(|lines| {
                                fresh_paragraph.replace_line_segs(lines);
                                true
                            })
                        }
                    },
                )
            });
        fresh_geometry_complete &= fresh_rows_supported;
        let carve_count = carves.total_records;
        let carves_truncated = carves.truncated;
        let carve_records = carves.records;

        let declared = paragraph_box.declared_horizontal();
        let effective = paragraph_box.effective();
        let external_geometry_replayed = geometry_source == "visible-picture-band-frame";
        let requires_external_geometry =
            stored_rows_require_external_geometry(paragraph, &paragraph_box.frame(0));
        let frame_comparable = stored_cache_is_eligible(paragraph)
            && stored_rows_are_well_formed(paragraph)
            && fresh_geometry_complete
            && (!requires_external_geometry || external_geometry_replayed)
            && options.geometry_mode.as_deref() != Some("stored-lineseg");
        let report = LineBreakProvenanceReport {
            schema_version: 5,
            coordinates: ParagraphCoordinates {
                section_idx,
                paragraph_idx: cell_path.is_empty().then_some(parent_para_idx),
                parent_para_idx: (!cell_path.is_empty()).then_some(parent_para_idx),
                cell_path: cell_path
                    .iter()
                    .map(
                        |&(control_idx, cell_idx, cell_para_idx)| CellPathCoordinate {
                            control_index: control_idx,
                            cell_index: cell_idx,
                            cell_para_index: cell_para_idx,
                        },
                    )
                    .collect(),
                group_path: options.group_path.clone(),
            },
            text_utf16_length: paragraph.char_count.saturating_sub(1),
            paragraph_box: ParagraphBoxTrace {
                coordinate_system: if cell_path.is_empty() {
                    "column-relative"
                } else {
                    "container-content"
                },
                declared: HorizontalRange::from_range(declared),
                effective: HorizontalRange::from_range(effective),
                origin_is_derivable: paragraph_box.origin_is_derivable(),
                width_px: paragraph_box.width_px(self.dpi),
            },
            geometry_source,
            fresh_geometry_complete,
            comparison: compare_boundaries(
                paragraph,
                &fresh_paragraph,
                frame_comparable,
                frame_comparable && paragraph_box.origin_is_derivable(),
                max_records,
            ),
            geometry: TraceLane {
                enabled: options.geometry,
                owner: if !options.geometry {
                    None
                } else if carve_count == 0 {
                    Some("specialized-path-no-layout-frame")
                } else {
                    Some("layout-frame")
                },
                total_records: carve_count,
                truncated: carves_truncated,
                records: carve_records,
            },
            measurement: MeasurementTraceLane {
                enabled: options.measurement,
                records: measurement,
            },
        };
        serde_json::to_string(&report).map_err(|error| {
            HwpError::RenderError(format!("line-break provenance JSON 오류: {error}"))
        })
    }

    /// Resolve a paragraph rendered by a TextBox inside a grouped shape.
    ///
    /// Editing `cellPath` stops at the paragraph-owned Group control because group children are
    /// not paragraph controls. `group_path` is the separate render-tree child axis emitted by
    /// `getPageTextLayout`; combining the two makes the diagnostic coordinate round-trippable
    /// without changing cursor/table path semantics.
    pub(super) fn resolve_group_textbox_paragraph<'a>(
        &'a self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path: &[(usize, usize, usize)],
        group_path: &[usize],
    ) -> Result<&'a Paragraph, HwpError> {
        if cell_path.is_empty() {
            return Err(HwpError::RenderError(
                "groupPath에는 TextBox 소유 shape cellPath가 필요합니다".into(),
            ));
        }

        let mut paragraph = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {section_idx} 범위 초과")))?
            .paragraphs
            .get(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {parent_para_idx} 범위 초과")))?;
        let mut group_depth = 0usize;

        for (path_index, &(control_index, cell_index, cell_para_index)) in
            cell_path.iter().enumerate()
        {
            let control = paragraph.controls.get(control_index).ok_or_else(|| {
                HwpError::RenderError(format!(
                    "경로[{path_index}]: controls[{control_index}] 범위 초과"
                ))
            })?;
            paragraph = match control {
                Control::Table(table) => {
                    let cell = table.cells.get(cell_index).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{path_index}]: 셀 {cell_index} 범위 초과 (총 {}개)",
                            table.cells.len()
                        ))
                    })?;
                    cell.paragraphs.get(cell_para_index).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{path_index}]: 셀문단 {cell_para_index} 범위 초과 (총 {}개)",
                            cell.paragraphs.len()
                        ))
                    })?
                }
                Control::Shape(shape) => {
                    if cell_index != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{path_index}]: 글상자의 cell_index는 0이어야 합니다 ({cell_index})"
                        )));
                    }
                    let target_shape = if group_depth < group_path.len()
                        && matches!(shape.as_ref(), ShapeObject::Group(_))
                    {
                        descend_group_path_segment(shape.as_ref(), group_path, &mut group_depth)?
                    } else {
                        shape.as_ref()
                    };
                    let text_box = get_textbox_from_shape(target_shape).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{path_index}] groupPath 대상 shape가 텍스트 글상자가 아닙니다"
                        ))
                    })?;
                    text_box.paragraphs.get(cell_para_index).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{path_index}]: 글상자문단 {cell_para_index} 범위 초과 (총 {}개)",
                            text_box.paragraphs.len()
                        ))
                    })?
                }
                Control::Picture(picture) => {
                    if cell_index != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{path_index}]: 그림 캡션의 cell_index는 0이어야 합니다 ({cell_index})"
                        )));
                    }
                    let caption = picture.caption.as_ref().ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{path_index}]: controls[{control_index}] 그림에 캡션이 없습니다"
                        ))
                    })?;
                    caption.paragraphs.get(cell_para_index).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{path_index}]: 그림 캡션문단 {cell_para_index} 범위 초과 (총 {}개)",
                            caption.paragraphs.len()
                        ))
                    })?
                }
                _ => {
                    return Err(HwpError::RenderError(format!(
                        "경로[{path_index}]: controls[{control_index}]가 표/글상자/그림 캡션이 아닙니다"
                    )))
                }
            };
        }

        if group_depth != group_path.len() {
            return Err(HwpError::RenderError(format!(
                "groupPath[{group_depth}] 이후를 소비할 Group control이 cellPath에 없습니다"
            )));
        }
        Ok(paragraph)
    }

    fn line_break_paragraph_box(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path: &[(usize, usize, usize)],
        paragraph: &Paragraph,
        options: &LineBreakProvenanceOptions,
    ) -> Result<(ParagraphBox, &'static str, bool), HwpError> {
        let style = self
            .styles
            .para_styles
            .get(paragraph.para_shape_id as usize);
        if options.geometry_mode.as_deref() == Some("stored-lineseg") {
            if let Some(segment) = paragraph
                .line_segs
                .first()
                .filter(|segment| segment.segment_width > 0)
            {
                let end = segment.column_start.saturating_add(segment.segment_width);
                let scalar_frame = paragraph.line_segs.iter().all(|candidate| {
                    candidate.is_first_segment()
                        && candidate.is_last_segment()
                        && candidate.column_start == segment.column_start
                        && candidate.segment_width == segment.segment_width
                });
                return Ok((
                    ParagraphBox::content(segment.column_start..end),
                    if scalar_frame {
                        "stored-lineseg-scalar-frame"
                    } else {
                        "stored-lineseg-first-slot-only"
                    },
                    scalar_frame,
                ));
            }
        }
        if !options.group_path.is_empty() {
            if let Some(width) = options.visible_frame_width_hwp.filter(|width| *width > 0) {
                return Ok((
                    ParagraphBox::content(0..width),
                    "visible-group-line-content-frame-hwp",
                    true,
                ));
            }
            return Err(HwpError::RenderError(
                "group TextBox provenance에 visibleFrameWidthHwp가 필요합니다".into(),
            ));
        }
        if !cell_path.is_empty() {
            if let Some(width) = options.visible_frame_width_hwp.filter(|width| *width > 0) {
                return Ok((
                    ParagraphBox::content(0..width),
                    "visible-line-content-frame-hwp",
                    true,
                ));
            }
        }
        if cell_path.is_empty() {
            if let Some(page_index) = options.page_index {
                if let Ok((page, _, _)) = self.find_page(page_index) {
                    if page.section_index != section_idx {
                        return Err(HwpError::RenderError(format!(
                            "visible page {page_index} belongs to section {}, not {section_idx}",
                            page.section_index
                        )));
                    }
                    let candidates = page
                        .column_contents
                        .iter()
                        .filter(|column| {
                            column
                                .items
                                .iter()
                                .any(|item| item.para_index() == parent_para_idx)
                        })
                        .collect::<Vec<_>>();
                    let selected = options
                        .text_x
                        .and_then(|text_x| {
                            candidates.iter().copied().find(|column| {
                                let layout = column.zone_layout.as_ref().unwrap_or(&page.layout);
                                layout
                                    .column_areas
                                    .get(column.column_index as usize)
                                    .is_some_and(|area| {
                                        text_x >= area.x && text_x <= area.x + area.width
                                    })
                            })
                        })
                        .or_else(|| candidates.first().copied());
                    if let Some(column) = selected {
                        let layout = column.zone_layout.as_ref().unwrap_or(&page.layout);
                        if let Some(area) = layout.column_areas.get(column.column_index as usize) {
                            let paragraph_box =
                                ParagraphBox::body_for_style(area.width, style, self.dpi);
                            let declared = paragraph_box.declared_horizontal();
                            let uniform_frame = candidates.iter().all(|candidate| {
                                let candidate_layout =
                                    candidate.zone_layout.as_ref().unwrap_or(&page.layout);
                                candidate_layout
                                    .column_areas
                                    .get(candidate.column_index as usize)
                                    .is_some_and(|candidate_area| {
                                        ParagraphBox::body_for_style(
                                            candidate_area.width,
                                            style,
                                            self.dpi,
                                        )
                                        .declared_horizontal()
                                            == declared
                                    })
                            });
                            let has_unreplayed_wrap = candidates.iter().any(|candidate| {
                                candidate.wrap_anchors.contains_key(&parent_para_idx)
                            });
                            let complete = uniform_frame && !has_unreplayed_wrap;
                            return Ok((
                                paragraph_box,
                                if !uniform_frame {
                                    "visible-page-unequal-column-fragments"
                                } else if complete {
                                    "visible-page-column"
                                } else {
                                    "visible-page-column-with-unreplayed-wrap"
                                },
                                complete,
                            ));
                        }
                    }
                }
            }
            return Ok((
                ParagraphBox::body_for_style(self.body_wrap_width(section_idx), style, self.dpi),
                "section-initial-column-fallback",
                false,
            ));
        }

        let (cell_width, pad_left, pad_right) = self
            .resolve_innermost_cell_metrics(section_idx, parent_para_idx, cell_path)
            .ok_or_else(|| {
                HwpError::RenderError("line-break provenance 셀 frame을 해석할 수 없습니다".into())
            })?;
        let cell_width_px = hwpunit_to_px(cell_width, self.dpi);
        let padding_px = hwpunit_to_px(i32::from(pad_left), self.dpi)
            + hwpunit_to_px(i32::from(pad_right), self.dpi);
        let margins_px = style
            .map(|value| value.margin_left + value.margin_right)
            .unwrap_or(0.0);
        let final_width = (cell_width_px - padding_px - margins_px).max(0.0);
        Ok((
            ParagraphBox::content_width_px(final_width, self.dpi),
            "container-owner-frame-padding-may-shrink",
            false,
        ))
    }
}
