//! 보조 spacer와 저장 줄 상자가 소유한 실제 빈 줄 전진을 구분한다.

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};
use crate::model::{
    paragraph::{LineSeg, Paragraph},
    table::Cell,
};

/// 연속된 저장 사다리의 원점으로 빈 줄 자체의 점유를 확인한다.
/// 형제 셀의 유무는 이 줄 상자의 흐름 소유를 바꾸지 않는다.
fn closed_stored_empty_line_owns_advance(
    cell: &Cell,
    unit: &CellUnit,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> bool {
    if cell.dirty_flag || unit.vis_start != 0 || unit.vis_end != 1 {
        return false;
    }
    let Some(previous) = unit
        .para_idx
        .checked_sub(1)
        .and_then(|index| cell.paragraphs.get(index))
    else {
        return false;
    };
    let Some(paragraph) = cell.paragraphs.get(unit.para_idx) else {
        return false;
    };
    let next = cell.paragraphs.get(unit.para_idx + 1);
    if [Some(previous), Some(paragraph), next]
        .into_iter()
        .flatten()
        .any(|para| {
            !para.controls.is_empty()
                || para.stored_text_partition_is_dirty()
                || para.cell_format_vpos_dirty
        })
        || !paragraph.text.trim().is_empty()
        || paragraph.line_segs.len() != 1
    {
        return false;
    }
    let (Some(before), Some(line)) = (previous.line_segs.last(), paragraph.line_segs.first())
    else {
        return false;
    };
    let authentic = |segment: &LineSeg| {
        segment.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && segment.line_height > 0
            && segment.line_spacing >= 0
            && segment.vertical_pos >= 0
    };
    // 문단 경계를 넘는 저장 원점에는 앞 문단 뒤 간격과 다음 문단 앞 간격도
    // 포함된다. 해석된 스타일을 소비해 저장 단위 환산 규칙을 재구현하지 않는다.
    let closes =
        |left: &LineSeg, left_para: &Paragraph, right: &LineSeg, right_para: &Paragraph| {
            let Some(left_style) = styles.para_styles.get(left_para.para_shape_id as usize) else {
                return false;
            };
            let Some(right_style) = styles.para_styles.get(right_para.para_shape_id as usize)
            else {
                return false;
            };
            let paragraph_gap = left_style.spacing_after + right_style.spacing_before;
            let line_gap_hu = i64::from(left.vertical_pos)
                + i64::from(left.line_height)
                + i64::from(left.line_spacing)
                - i64::from(right.vertical_pos);
            let paragraph_gap_hu = paragraph_gap * crate::renderer::HWPUNIT_PER_INCH / dpi;
            paragraph_gap_hu.is_finite() && (line_gap_hu as f64 + paragraph_gap_hu).abs() <= 2.0
        };
    if !authentic(before) || !authentic(line) || !closes(before, previous, line, paragraph) {
        return false;
    }
    match next {
        Some(next) => next
            .line_segs
            .first()
            .is_some_and(|after| authentic(after) && closes(line, paragraph, after, next)),
        // 마지막 실제 저장 빈 줄도 앞 줄에서 닫히면 자기 줄 상자를 소유한다.
        // 원점만 있는 spacer나 반복된 개체 최대 높이는 이 근거로 승인하지 않는다.
        None => {
            line.vertical_pos > 0
                && line.text_height == line.line_height
                && line.baseline_distance > 0
        }
    }
}

impl LayoutEngine {
    /// 저장 줄 없는 빈 셀 문단의 현재 전진과 뒤 간격을 함께 발행한다.
    /// 단락 안에서는 전진 전체를 쓰고 물리 컷에서는 줄 상자를 남긴다.
    pub(super) fn reflow_empty_cell_line_metrics(
        &self,
        para: &Paragraph,
        styles: &ResolvedStyleSet,
        para_style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
        is_last_para: bool,
    ) -> (f64, f64) {
        let fallback = crate::renderer::hwpunit_to_px(400, self.dpi);
        let fs = para
            .char_shapes
            .first()
            .and_then(|shape| styles.char_styles.get(shape.char_shape_id as usize))
            .map_or(0.0, |style| style.font_size);
        if fs <= 0.0 {
            return (fallback, 0.0);
        }
        if is_last_para {
            return (fs, 0.0);
        }
        let Some(style) = para_style else {
            return (fs, 0.0);
        };
        let advance = crate::renderer::corrected_line_height(
            fallback,
            fs,
            style.line_spacing_type,
            style.line_spacing,
        );
        let following = if fs > fallback
            && matches!(
                style.line_spacing_type,
                crate::model::style::LineSpacingType::Percent
                    | crate::model::style::LineSpacingType::SpaceOnly
            ) {
            (advance - fs).max(0.0)
        } else {
            0.0
        };
        (advance, following)
    }

    /// 다음 원본 프레임 직전의 빈 줄도 앞 저장 슬롯이 정확히 닫으면 자기 높이를 소유한다.
    /// 프레임 경계는 호출자가 실제 컷과 같은 판별자로 확인해 전달한다.
    pub(super) fn stored_closing_empty_line_advance_hu(
        &self,
        cell: &Cell,
        table: &crate::model::table::Table,
        index: usize,
        next_starts_frame: bool,
        styles: &ResolvedStyleSet,
    ) -> Option<i32> {
        // 재조판한 표의 원본 되감김은 현재 프레임의 끝을 증명하지 못한다.
        // 다른 빈 줄의 독립적인 줄 상자 소유 판정은 그대로 사용한다.
        if !next_starts_frame
            || cell.dirty_flag
            || self
                .render_normalization
                .borrow()
                .cell_frame_reflowed(table, cell)
        {
            return None;
        }
        let previous = cell.paragraphs.get(index.checked_sub(1)?)?;
        let paragraph = cell.paragraphs.get(index)?;
        let next = cell.paragraphs.get(index + 1)?;
        if [previous, paragraph, next].iter().any(|para| {
            para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty
        }) || !paragraph.text.trim().is_empty()
            || !paragraph.controls.is_empty()
            || previous.controls.iter().any(|control| {
                !matches!(control, crate::model::control::Control::Table(table) if table.common.treat_as_char)
            })
        {
            return None;
        }
        let [line] = paragraph.line_segs.as_slice() else {
            return None;
        };
        let before = previous.line_segs.last()?;
        if [before, line].iter().any(|segment| {
            segment.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || segment.vertical_pos < 0
                || segment.line_height <= 0
        }) || line.line_spacing < 0
        {
            return None;
        }
        let left_style = styles.para_styles.get(previous.para_shape_id as usize)?;
        let right_style = styles.para_styles.get(paragraph.para_shape_id as usize)?;
        let paragraph_gap_hu = (left_style.spacing_after + right_style.spacing_before)
            * crate::renderer::HWPUNIT_PER_INCH
            / self.dpi;
        let incoming_delta = i64::from(before.vertical_pos)
            + i64::from(before.line_height)
            + i64::from(before.line_spacing)
            - i64::from(line.vertical_pos);
        (paragraph_gap_hu.is_finite() && (incoming_delta as f64 + paragraph_gap_hu).abs() <= 2.0)
            .then(|| line.line_height.checked_add(line.line_spacing))
            .flatten()
    }

    /// 실제 저장 줄 상자는 글자가 없어도 높이를 소유한다. 보조 spacer의
    /// 무높이 소비 규칙을 원본 LINE_SEG 빈 줄에 적용하면 컷만 앞서 진행하고
    /// 배치는 그 줄을 다시 그려 표가 쪽을 넘는다.
    pub(super) fn empty_unit_owns_flow_line_box(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        unit: &CellUnit,
        styles: &ResolvedStyleSet,
    ) -> bool {
        if !unit.empty_spacer || unit.height <= 0.0 || unit.vis_start >= unit.vis_end {
            return false;
        }
        let Some(para) = cell.paragraphs.get(unit.para_idx) else {
            return false;
        };
        // A genuine reflowed empty paragraph owns its composed line. Lack of
        // saved LineSegs is not evidence that its positive advance is a spacer.
        if crate::renderer::para_has_no_stored_line_segs(para) {
            return para.controls.is_empty();
        }
        // 실제 종료 줄은 뒤 문단이 없어도 앞 저장 메트릭의 닫힘으로 소유를 증명한다.
        if unit.owns_stored_closing_line
            || closed_stored_empty_line_owns_advance(cell, unit, styles, self.dpi)
            || self.closed_nested_frame_owns_following_empty_line(cell, table, unit, styles)
        {
            return true;
        }
        let Some(next) = cell.paragraphs.get(unit.para_idx + 1) else {
            // 종료 host의 빈 꼬리는 자체 줄 높이만으로 흐름 소유를 증명하지 못한다.
            // 다만 저장 줄이 새 쪽 원점(vpos 0)에서 다시 시작했다면 한/글이 그 줄을 새 쪽
            // 조각에 놓았다는 기록이다(`21761835` 3쪽 행 20: 빈 꼬리 한 줄 + 위아래 패딩 =
            // 한/글 2020 PDF 17px).
            return !cell.dirty_flag
                && unit.page_frame_reset_before
                && unit.stored_frame_break_before
                && para.controls.is_empty()
                && !para.stored_text_partition_is_dirty()
                && !para.cell_format_vpos_dirty
                && matches!(
                    para.line_segs.as_slice(),
                    [line] if line.vertical_pos == 0
                        && line.line_height > 0
                        && line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                );
        };
        if !para.controls.is_empty() {
            return false;
        }
        if !next.controls.is_empty() {
            // 뒤 개체 앞의 원본 빈 줄도 이미 발행한 저장 전진을 소유한다.
            // 일반 보조 spacer로 버리면 예약과 실제 조각에서 같은 줄이 사라진다.
            return crate::renderer::height_measurer::stored_empty_line_successor_advance_hu(
                cell,
                unit.para_idx,
            )
            .is_some();
        }
        // 원본 쪽 재시작의 빈 줄은 뒤 저장 줄과 같은 전진 상자를 소유한다.
        // 형제 셀이 없어도 이 소유를 잃고 무높이 spacer로 소비하지 않는다.
        if Self::stored_empty_restart_owns_advance(cell, unit) {
            return true;
        }
        if crate::renderer::para_has_no_stored_line_segs(next) {
            // A saved blank line followed by a reflowed paragraph has no next
            // stored origin. Preserve the existing exact incoming-slot proof.
            return cell.paragraphs[..unit.para_idx]
                .last()
                .and_then(|previous| previous.line_segs.last())
                .zip(para.line_segs.first())
                .is_some_and(|(previous, line)| {
                    let authentic = |segment: &crate::model::paragraph::LineSeg| {
                        segment.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                            == 0
                    };
                    authentic(previous)
                        && authentic(line)
                        && previous.line_height > 0
                        && line.line_height > 0
                        && (i64::from(previous.vertical_pos)
                            + i64::from(previous.line_height)
                            + i64::from(previous.line_spacing.max(0))
                            - i64::from(line.vertical_pos))
                        .abs()
                            <= 2
                });
        }
        let (Some(seg), Some(next_seg)) = (para.line_segs.last(), next.line_segs.first()) else {
            return false;
        };
        let authentic = |seg: &crate::model::paragraph::LineSeg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        };
        // 저장 줄 상자가 실제 다음 원점까지 전진해야 한다. 저장 높이만 존재하는
        // overlay/종료 spacer를 모두 예약하면 기존 거대 셀의 컷이 바뀐다.
        authentic(seg)
            && authentic(next_seg)
            && seg.line_height > 0
            && i64::from(next_seg.vertical_pos)
                >= i64::from(seg.vertical_pos) + i64::from(seg.line_height)
            && table
                .cells
                .iter()
                .filter(|other| {
                    other.row == cell.row && other.col != cell.col && other.row_span == 1
                })
                .any(|other| {
                    let mut previous = None;
                    other
                        .paragraphs
                        .iter()
                        .filter(|p| p.controls.is_empty())
                        .flat_map(|p| &p.line_segs)
                        .any(|line| {
                            let matched =
                                previous.is_some_and(|prior: &crate::model::paragraph::LineSeg| {
                                    authentic(prior)
                                        && authentic(line)
                                        && prior.vertical_pos == seg.vertical_pos
                                        && prior.line_height == seg.line_height
                                        && line.vertical_pos == next_seg.vertical_pos
                                });
                            previous = Some(line);
                            matched
                        })
                })
    }
}
