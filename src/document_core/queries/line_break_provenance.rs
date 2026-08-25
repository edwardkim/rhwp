use serde::{Deserialize, Serialize};

use crate::document_core::helpers::get_textbox_from_shape;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::ShapeObject;
use crate::renderer::composer::{
    capture_line_break_measurement, layout_picture_band, reflow_line_segs, trace_paragraph_scope,
    LineBreakMeasurementTrace,
};
use crate::renderer::hwpunit_to_px;
use crate::renderer::layout_frame::{capture_frame_carves, FrameCarveTrace, ParagraphBox};
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

pub(super) fn first_partition_mismatch(stored: &Paragraph, fresh: &Paragraph) -> Option<usize> {
    stored
        .line_segs
        .iter()
        .zip(&fresh.line_segs)
        .enumerate()
        .position(|(index, (stored_segment, fresh_segment))| {
            stored.line_seg_text_start(index) != fresh.line_seg_text_start(index)
                || stored_segment.is_first_segment() != fresh_segment.is_first_segment()
                || stored_segment.is_last_segment() != fresh_segment.is_last_segment()
        })
        .or_else(|| {
            (stored.line_segs.len() != fresh.line_segs.len())
                .then_some(stored.line_segs.len().min(fresh.line_segs.len()))
        })
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

fn stored_rows_are_well_formed(line_segs: &[LineSeg]) -> bool {
    if line_segs.is_empty() {
        return false;
    }
    let mut expects_first = true;
    let mut previous_text_start = None;
    for segment in line_segs {
        if previous_text_start.is_some_and(|previous| segment.text_start < previous)
            || segment.is_first_segment() != expects_first
        {
            return false;
        }
        previous_text_start = Some(segment.text_start);
        expects_first = segment.is_last_segment();
    }
    expects_first
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundaryComparison {
    comparable: bool,
    matches: Option<bool>,
    first_mismatch_index: Option<usize>,
    stored_mismatch_utf16_start: Option<u32>,
    fresh_mismatch_utf16_start: Option<u32>,
    stored_mismatch_row_part: Option<&'static str>,
    fresh_mismatch_row_part: Option<&'static str>,
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

pub(crate) fn compare_boundaries(
    stored: &Paragraph,
    fresh: &Paragraph,
    comparable: bool,
    limit: usize,
) -> BoundaryComparison {
    let mismatch = first_partition_mismatch(stored, fresh);
    let reported = comparable.then_some(mismatch).flatten();
    BoundaryComparison {
        comparable,
        matches: comparable.then_some(mismatch.is_none()),
        first_mismatch_index: reported,
        stored_mismatch_utf16_start: reported
            .filter(|&index| index < stored.line_segs.len())
            .map(|index| stored.line_seg_text_start(index)),
        fresh_mismatch_utf16_start: reported
            .filter(|&index| index < fresh.line_segs.len())
            .map(|index| fresh.line_seg_text_start(index)),
        stored_mismatch_row_part: reported
            .and_then(|index| stored.line_segs.get(index))
            .map(row_part),
        fresh_mismatch_row_part: reported
            .and_then(|index| fresh.line_segs.get(index))
            .map(row_part),
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
        let (((), measurement), carves) =
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

        let declared = paragraph_box.declared_horizontal();
        let effective = paragraph_box.effective();
        let frame_comparable = stored_cache_is_eligible(paragraph)
            && stored_rows_are_well_formed(&paragraph.line_segs)
            && fresh_geometry_complete;
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
