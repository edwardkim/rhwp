use serde::{Deserialize, Serialize};

use crate::document_core::helpers::get_textbox_from_shape;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::ShapeObject;
use crate::renderer::composer::{
    capture_line_break_measurement, layout_picture_band, reflow_line_segs,
    stored_rows_reproduce_frame_expectation, trace_paragraph_scope, LineBreakMeasurementTrace,
};
use crate::renderer::hwpunit_to_px;
use crate::renderer::layout_frame::{capture_frame_carves, FrameCarveTrace, ParagraphBox};
use crate::renderer::page_layout::PageLayoutInfo;

fn enabled_by_default() -> bool {
    true
}

fn default_row_limit() -> usize {
    128
}

fn default_token_limit() -> usize {
    256
}

const SEGMENT_RECORD_LIMIT: usize = 512;

fn default_fit_limit() -> usize {
    512
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
    #[serde(default = "default_row_limit")]
    max_rows: usize,
    #[serde(default = "default_row_limit")]
    max_carves: usize,
    #[serde(default = "default_token_limit")]
    max_tokens: usize,
    #[serde(default = "default_fit_limit")]
    max_fit_decisions: usize,
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
            max_rows: default_row_limit(),
            max_carves: default_row_limit(),
            max_tokens: default_token_limit(),
            max_fit_decisions: default_fit_limit(),
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParagraphStyleTrace {
    para_shape_id: u16,
    margin_left_px: f64,
    margin_right_px: f64,
    indent_px: f64,
    english_break_unit: u8,
    korean_break_unit: u8,
    condense_min_space: u8,
    default_tab_width_px: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LineSegTrace {
    physical_row_index: usize,
    text_start_utf16: u32,
    vertical_pos: i32,
    line_height: i32,
    baseline_distance: i32,
    line_spacing: i32,
    column_start: i32,
    segment_width: i32,
    tag: u32,
    pub(super) source_cache_authentic: bool,
}

pub(super) fn line_seg_trace(
    line_segs: &[LineSeg],
    source_prefix_len: usize,
    max_rows: usize,
    max_segments: usize,
) -> (Vec<LineSegTrace>, usize, usize) {
    let carries_row_boundaries = line_segs
        .iter()
        .any(|segment| segment.is_first_segment() || segment.is_last_segment());
    let mut physical_row_index = 0usize;
    let mut records = Vec::with_capacity(max_segments.min(line_segs.len()));
    for (segment_index, segment) in line_segs.iter().enumerate() {
        if carries_row_boundaries && segment_index > 0 && segment.is_first_segment() {
            physical_row_index += 1;
        } else if !carries_row_boundaries {
            physical_row_index = segment_index;
        }
        if physical_row_index < max_rows && records.len() < max_segments {
            records.push(LineSegTrace {
                physical_row_index,
                text_start_utf16: segment.text_start,
                vertical_pos: segment.vertical_pos,
                line_height: segment.line_height,
                baseline_distance: segment.baseline_distance,
                line_spacing: segment.line_spacing,
                column_start: segment.column_start,
                segment_width: segment.segment_width,
                tag: segment.tag,
                source_cache_authentic: segment_index < source_prefix_len
                    && segment.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0,
            });
        }
    }
    let row_count = (!line_segs.is_empty())
        .then_some(physical_row_index + 1)
        .unwrap_or(0);
    (records, row_count, line_segs.len())
}

pub(super) fn stored_rows_are_well_formed(line_segs: &[LineSeg]) -> bool {
    if line_segs.is_empty() {
        return false;
    }
    let mut expects_first = true;
    let mut previous_text_start = None;
    for segment in line_segs {
        if previous_text_start.is_some_and(|previous| segment.text_start < previous) {
            return false;
        }
        if segment.is_first_segment() != expects_first {
            return false;
        }
        previous_text_start = Some(segment.text_start);
        expects_first = segment.is_last_segment();
    }
    expects_first
}

pub(super) fn first_partition_mismatch(stored: &[LineSeg], fresh: &[LineSeg]) -> Option<usize> {
    stored
        .iter()
        .zip(fresh)
        .position(|(stored, fresh)| {
            stored.text_start != fresh.text_start
                || stored.is_first_segment() != fresh.is_first_segment()
                || stored.is_last_segment() != fresh.is_last_segment()
        })
        .or_else(|| (stored.len() != fresh.len()).then_some(stored.len().min(fresh.len())))
}

pub(super) fn classify_stored_cache(paragraph: &Paragraph) -> (&'static str, bool, bool, usize) {
    if paragraph.line_segs.is_empty() {
        return ("missing", false, false, 0);
    }
    let source_prefix_len = paragraph
        .line_segs
        .len()
        .saturating_sub(paragraph.layout_only_fill_lines);
    let authentic_count = paragraph.line_segs[..source_prefix_len]
        .iter()
        .filter(|segment| segment.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
        .count();
    if authentic_count == 0 {
        ("implementation-synthetic", false, false, source_prefix_len)
    } else if authentic_count < paragraph.line_segs.len() {
        (
            "mixed-source-and-synthetic",
            false,
            false,
            source_prefix_len,
        )
    } else if paragraph.stored_text_partition_is_dirty() {
        ("stale-text-partition", true, false, source_prefix_len)
    } else {
        ("eligible-current-cache", true, true, source_prefix_len)
    }
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BoundaryComparison {
    comparable: bool,
    matches: Option<bool>,
    first_mismatch_index: Option<usize>,
    stored_starts_truncated: bool,
    stored_utf16_starts: Vec<u32>,
    fresh_starts_truncated: bool,
    fresh_utf16_starts: Vec<u32>,
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
    stored_cache_kind: &'static str,
    stored_cache_authentic: bool,
    stored_cache_eligible: bool,
    stored_rows_well_formed: bool,
    production_admission: Option<bool>,
    paragraph_box: ParagraphBoxTrace,
    geometry_source: &'static str,
    column_index: Option<u16>,
    fresh_geometry_complete: bool,
    style: ParagraphStyleTrace,
    stored_row_count: usize,
    stored_rows_truncated: bool,
    stored_segment_count: usize,
    stored_segments_truncated: bool,
    stored: Vec<LineSegTrace>,
    fresh_row_count: usize,
    fresh_rows_truncated: bool,
    fresh_segment_count: usize,
    fresh_segments_truncated: bool,
    fresh: Vec<LineSegTrace>,
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
        let (
            stored_cache_kind,
            stored_cache_authentic,
            stored_cache_eligible,
            stored_source_prefix_len,
        ) = classify_stored_cache(paragraph);
        let picture_band_owner = cell_path
            .is_empty()
            .then(|| self.picture_band_owning_body_paragraph(section_idx, parent_para_idx))
            .flatten();
        let (paragraph_box, mut geometry_source, mut column_index, mut fresh_geometry_complete) =
            self.line_break_paragraph_box(
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
            column_index =
                mapped_column.filter(|index| layout.column_areas.get(*index as usize).is_some());
            fresh_geometry_complete = column_index.is_some();
            geometry_source = if fresh_geometry_complete {
                "visible-picture-band-frame"
            } else {
                "picture-band-column-fallback"
            };
        }
        let para_style = self
            .styles
            .para_styles
            .get(paragraph.para_shape_id as usize);
        let style = ParagraphStyleTrace {
            para_shape_id: paragraph.para_shape_id,
            margin_left_px: para_style.map(|value| value.margin_left).unwrap_or(0.0),
            margin_right_px: para_style.map(|value| value.margin_right).unwrap_or(0.0),
            indent_px: para_style.map(|value| value.indent).unwrap_or(0.0),
            english_break_unit: para_style
                .map(|value| value.english_break_unit)
                .unwrap_or(0),
            korean_break_unit: para_style.map(|value| value.korean_break_unit).unwrap_or(0),
            condense_min_space: para_style
                .map(|value| value.condense_min_space)
                .unwrap_or(0),
            default_tab_width_px: para_style
                .map(|value| value.default_tab_width)
                .unwrap_or(0.0),
        };

        let max_rows = options.max_rows.min(512);
        let max_segments = SEGMENT_RECORD_LIMIT;
        let max_carves = options.max_carves.min(512);
        let max_tokens = options.max_tokens.min(2_048);
        let max_fit_decisions = options.max_fit_decisions.min(4_096);
        let mut fresh_paragraph = paragraph.clone();
        fresh_paragraph.line_segs.clear();
        let _trace_scope = trace_paragraph_scope(Some(parent_para_idx));
        let (((), mut measurement), mut carves) =
            capture_frame_carves(options.geometry, max_carves, Some(parent_para_idx), || {
                capture_line_break_measurement(
                    options.measurement,
                    max_tokens,
                    max_fit_decisions,
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
                            fresh_paragraph.line_segs = lines;
                        } else {
                            reflow_line_segs(
                                &mut fresh_paragraph,
                                paragraph_box.clone(),
                                &self.styles,
                                self.dpi,
                            );
                        }
                    },
                )
            });
        let carve_count = carves.total_records;
        let carves_truncated = carves.truncated;
        let carve_records = carves.records;

        let stored_start_count = paragraph.line_segs.len();
        let fresh_start_count = fresh_paragraph.line_segs.len();
        let first_mismatch_index =
            first_partition_mismatch(&paragraph.line_segs, &fresh_paragraph.line_segs);
        let stored_utf16_starts = paragraph
            .line_segs
            .iter()
            .map(|segment| segment.text_start)
            .take(max_segments)
            .collect::<Vec<_>>();
        let fresh_utf16_starts = fresh_paragraph
            .line_segs
            .iter()
            .map(|segment| segment.text_start)
            .take(max_segments)
            .collect::<Vec<_>>();
        let stored_rows_well_formed = stored_rows_are_well_formed(&paragraph.line_segs);
        let frame_comparable =
            stored_cache_eligible && stored_rows_well_formed && fresh_geometry_complete;
        // Production admission is a separate question from comparison. A rejected cache is
        // precisely the case the inspector must still be able to describe as matches=false.
        // Picture bands use exclusion geometry that a scalar ParagraphBox cannot reproduce, so
        // their admission result remains unknown while their captured fresh partition is still
        // comparable.
        // Cell renderers admit caches against the cell-inner box before paragraph margins;
        // this inspector receives the visible TextLine box after those margins. Never label a
        // decision on the latter as the production admission decision.
        let production_admission = if frame_comparable
            && picture_band_owner.is_none()
            && cell_path.is_empty()
            && options.geometry_mode.as_deref() != Some("stored-lineseg")
        {
            let mut admission_frame = paragraph_box.frame(
                paragraph
                    .line_segs
                    .first()
                    .map(|segment| segment.vertical_pos)
                    .unwrap_or(0),
            );
            Some(stored_rows_reproduce_frame_expectation(
                paragraph,
                &mut admission_frame,
                &self.styles,
                self.dpi,
            ))
        } else {
            None
        };
        let declared = paragraph_box.declared_horizontal();
        let effective = paragraph_box.effective();
        let (stored_rows, stored_row_count, stored_segment_count) = line_seg_trace(
            &paragraph.line_segs,
            stored_source_prefix_len,
            max_rows,
            max_segments,
        );
        let (fresh_rows, fresh_row_count, fresh_segment_count) =
            line_seg_trace(&fresh_paragraph.line_segs, 0, max_rows, max_segments);
        let report = LineBreakProvenanceReport {
            schema_version: 3,
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
            stored_cache_kind,
            stored_cache_authentic,
            stored_cache_eligible,
            stored_rows_well_formed,
            production_admission,
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
            column_index,
            fresh_geometry_complete,
            style,
            stored_row_count,
            stored_rows_truncated: stored_row_count > max_rows,
            stored_segment_count,
            stored_segments_truncated: stored_segment_count > stored_rows.len(),
            stored: stored_rows,
            fresh_row_count,
            fresh_rows_truncated: fresh_row_count > max_rows,
            fresh_segment_count,
            fresh_segments_truncated: fresh_segment_count > fresh_rows.len(),
            fresh: fresh_rows,
            comparison: BoundaryComparison {
                comparable: frame_comparable,
                matches: frame_comparable.then_some(first_mismatch_index.is_none()),
                first_mismatch_index: frame_comparable.then_some(first_mismatch_index).flatten(),
                stored_starts_truncated: stored_start_count > stored_utf16_starts.len(),
                stored_utf16_starts,
                fresh_starts_truncated: fresh_start_count > fresh_utf16_starts.len(),
                fresh_utf16_starts,
            },
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
    fn resolve_group_textbox_paragraph<'a>(
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
    ) -> Result<(ParagraphBox, &'static str, Option<u16>, bool), HwpError> {
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
                    None,
                    scalar_frame,
                ));
            }
        }
        if !options.group_path.is_empty() {
            if let Some(width) = options.visible_frame_width_hwp.filter(|width| *width > 0) {
                return Ok((
                    ParagraphBox::content(0..width),
                    "visible-group-line-content-frame-hwp",
                    None,
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
                    None,
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
                                Some(column.column_index),
                                complete,
                            ));
                        }
                    }
                }
            }
            return Ok((
                ParagraphBox::body_for_style(self.body_wrap_width(section_idx), style, self.dpi),
                "section-initial-column-fallback",
                None,
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
            None,
            false,
        ))
    }
}
