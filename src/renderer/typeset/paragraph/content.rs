//! 수용한 원본 글줄의 내용 끝을 배치와 같은 단 좌표에서 조회한다.

use super::metrics::FormattedParagraph;
use super::{Paragraph, ResolvedStyleSet, TypesetState};
use crate::model::control::Control;
use crate::model::paragraph::LineSeg;
use crate::renderer::{hwpunit_to_px, pagination::PageItem};

/// 단의 원본 글줄 축과 구성 메트릭이 그대로일 때의 실제 하단.
/// 순차 fit 커서에서 생략한 간격을 글줄의 물리 원점으로 대신하지 않는다.
/// 다른 단 원점·미주·재조판은 이 저장 좌표 계약으로 인증하지 않는다.
pub(super) fn stored_content_bottom(
    st: &TypesetState,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<f64> {
    if !(st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
        || st.profile.hwp3_layout()
        || st.profile.session_edited()
        || st.current_endnote_flow
        || st.vpos_ladder_dirty
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || fmt.exact_reflow_text_box
        || fmt.line_count() != para.line_segs.len()
    {
        return None;
    }
    let recovered_frame = accepted_preceding_tac_frame(st, para, paragraphs, dpi)
        .or_else(|| accepted_first_text_frame(st, paragraphs))
        .or_else(|| {
            (st.vpos_page_base.is_none() && st.vpos_lazy_base.is_none())
                .then(|| accepted_tac_column_frame(st, paragraphs, styles, dpi))
                .flatten()
        });
    let (first_owner, first_line) = match st.current_items.first()? {
        PageItem::FullParagraph { para_index } => (*para_index, 0),
        PageItem::Table { para_index, .. } if recovered_frame.is_some() => (*para_index, 0),
        PageItem::PartialParagraph {
            para_index,
            start_line,
            ..
        } => (*para_index, *start_line),
        _ => return None,
    };
    let first = paragraphs.get(first_owner)?;
    let first_seg = first.line_segs.get(first_line)?;
    // 단 첫 줄의 앞 간격을 보존한 프레임은 원점과 첫 vpos가 다르다.
    // pagination과 paint가 이미 수용한 같은 프레임을 내용 하단에도 쓴다.
    // 첫 vpos를 원점으로 요구하면 이 경로만 간격을 잃어 뒤 TAC 표가 올라간다.
    let retained_frame = (first_line == 0)
        .then(|| st.stored_column_top_spacings.get(&first_owner))
        .flatten()
        .filter(|frame| {
            frame
                .source_origin_hu
                .checked_add(crate::renderer::px_to_hwpunit(frame.before, dpi))
                == Some(first_seg.vertical_pos)
        });
    // 책갈피처럼 줄 상자를 소유하지 않는 표시가 있어도 첫 문단은
    // 같은 쪽 상대 여백 축을 쓴다. 단 초기화와 paint가 사용하는 인증을
    // 그대로 소비하며, 프레임 기록 유무를 물리 여백 유무로 대신하지 않는다.
    let page_relative_margin = first_line == 0
        && (st.vpos_page_base == Some(0) || recovered_frame.is_some_and(|(base, _)| base == 0))
        && crate::renderer::layout::stored_first_margin_is_page_relative(
            first,
            paragraphs.get(first_owner.checked_add(1)?),
            styles,
            dpi,
        );
    // 표의 확정 원점이 인증한 축도 실제 단 시작 높이를 소유한다.
    // 저장 좌표의 기준과 물리 앵커를 분리하여 시작 밴드를 두 번 더하지 않는다.
    let (source_base, anchor) = if let Some(frame) = recovered_frame {
        frame
    } else if st.vpos_page_base_stored {
        let base = st.vpos_page_base?;
        if first_seg.vertical_pos != base
            && !retained_frame.is_some_and(|frame| frame.source_origin_hu == base)
            && !page_relative_margin
        {
            return None;
        }
        (base, st.vpos_col_anchor)
    } else if first_seg.vertical_pos == 0 && st.current_start_height == 0.0 {
        (0, 0.0)
    } else if first_line == 0
        && st.vpos_col_anchor == st.current_start_height
        && first_seg.vertical_pos
            == (st.current_start_height * crate::renderer::HWPUNIT_PER_INCH / dpi).round() as i32
    {
        // 자동·명시적 단 시작 모두 원본 줄과 실제 시작 밴드가 같은 축을 인증한다.
        // 첫 표가 일반 float여도 뒤 본문을 순차 누적 높이로 되돌리지 않는다.
        (first_seg.vertical_pos, st.vpos_col_anchor)
    } else {
        return None;
    };
    if first.stored_text_partition_is_dirty()
        || first.cell_format_vpos_dirty
        || first_seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || (recovered_frame.is_none()
            && retained_frame.is_none()
            && !page_relative_margin
            && styles
                .para_styles
                .get(first.para_shape_id as usize)?
                .spacing_before
                != 0.0)
        || para
            .line_segs
            .windows(2)
            .any(|pair| pair[1].vertical_pos < pair[0].vertical_pos)
    {
        return None;
    }
    let mut bottom = None;
    for (index, seg) in para.line_segs.iter().enumerate() {
        // 새로 구성한 줄의 높이·간격은 원본 사다리에 억지로 맞추지 않는다.
        if seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || seg.vertical_pos < 0
            || seg.line_height < 0
            || fmt.line_heights[index] != hwpunit_to_px(seg.line_height, dpi)
            || fmt.line_spacings[index] != hwpunit_to_px(seg.line_spacing, dpi)
        {
            return None;
        }
        let end = seg.vertical_pos.checked_add(seg.line_height)?;
        bottom = Some(bottom.map_or(end, |previous: i32| previous.max(end)));
    }
    bottom
        .and_then(|end| end.checked_sub(source_base))
        .map(|end| anchor + hwpunit_to_px(end, dpi))
}

/// 표와 별도로 수용한 첫 본문 조각도 자기 원점과 저장 축을 소유한다.
/// 현재 단의 다른 축이나 재조판 결과로 이를 덮어쓰지 않는다.
fn accepted_first_text_frame(st: &TypesetState, paragraphs: &[Paragraph]) -> Option<(i32, f64)> {
    let (owner, end_line) = match st.current_items.first()? {
        PageItem::FullParagraph { para_index } => {
            (*para_index, paragraphs.get(*para_index)?.line_segs.len())
        }
        PageItem::PartialParagraph {
            para_index,
            end_line,
            ..
        } => (*para_index, *end_line),
        _ => return None,
    };
    let bounds = st
        .paragraph_fragment_content_bottoms
        .get(&(owner, end_line))?;
    let frame = bounds.stored_frame?;
    let base = frame.source_origin_hu;
    let anchor = frame.column_anchor_y;
    (anchor == st.vpos_col_anchor
        && st
            .vpos_page_base
            .or(st.vpos_lazy_base)
            .is_none_or(|active| active == base))
    .then_some((base, anchor))
}

/// 단 중간의 확정 TAC도 후속 글줄의 실제 좌표축을 소유할 수 있다.
/// 흐름에서 트림한 앞 간격을 물리 내용 하단에서 다시 빼지 않는다.
fn accepted_preceding_tac_frame(
    st: &TypesetState,
    current: &Paragraph,
    paragraphs: &[Paragraph],
    dpi: f64,
) -> Option<(i32, f64)> {
    let current_top = current.line_segs.first()?.vertical_pos;
    let active_origin = st.vpos_page_base.or(st.vpos_lazy_base)?;
    for item in st.current_items.iter().rev() {
        let (owner, control) = match item {
            PageItem::FullParagraph { .. } | PageItem::PartialParagraph { .. } => continue,
            PageItem::Table {
                para_index,
                control_index,
            } => (*para_index, *control_index),
            _ => return None,
        };
        let host = paragraphs.get(owner)?;
        let [line] = host.line_segs.as_slice() else {
            return None;
        };
        if host.stored_text_partition_is_dirty()
            || host.cell_format_vpos_dirty
            || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || current_top < line.vertical_pos.checked_add(line.line_height)?
            || !matches!(host.controls.get(control), Some(Control::Table(table)) if table.common.treat_as_char)
        {
            return None;
        }
        let placement = st.inline_placements.get(&(owner, control))?;
        let frame = placement.stored_column_frame?;
        let origin = frame.source_origin_hu;
        let anchor = frame.column_anchor_y;
        let expected = anchor + hwpunit_to_px(line.vertical_pos.checked_sub(origin)?, dpi);
        return (origin == active_origin
            && anchor == st.vpos_col_anchor
            && (placement.y - expected).abs() <= hwpunit_to_px(1, dpi))
        .then_some((origin, anchor));
    }
    None
}

/// 같은 단의 저장 TAC 계획이 실제로 보존한 축만 본문 내용 하단에 이어준다.
/// 표 뒤의 단순 cursor 추정이나 선언 높이만으로 초기 축을 되살리지 않는다.
fn accepted_tac_column_frame(
    st: &TypesetState,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(i32, f64)> {
    let anchor = st.current_start_height;
    let tolerance = hwpunit_to_px(1, dpi);
    let first_item = st.current_items.first()?;
    let first_owner = first_item.para_index();
    let first = paragraphs.get(first_owner)?;
    let first_line = first.line_segs.first()?;
    let base = match first_item {
        PageItem::FullParagraph { .. } => {
            if let Some(frame) = st.stored_column_top_spacings.get(&first_owner) {
                frame.source_origin_hu
            } else if first_line.vertical_pos == 0
                || crate::renderer::layout::stored_first_margin_is_page_relative(
                    first,
                    paragraphs.get(first_owner.checked_add(1)?),
                    styles,
                    dpi,
                )
            {
                0
            } else {
                return None;
            }
        }
        PageItem::Table { control_index, .. } => {
            let placement = st
                .inline_placements
                .get(&(first_owner, *control_index))
                .filter(|placement| placement.owns_vertical_frame())?;
            let before = styles
                .para_styles
                .get(first.para_shape_id as usize)?
                .spacing_before;
            if first.line_segs.len() != 1 || (placement.y - anchor - before).abs() > tolerance {
                return None;
            }
            first_line
                .vertical_pos
                .checked_sub(crate::renderer::px_to_hwpunit(before, dpi))?
        }
        _ => return None,
    };
    let mut saw_tac = false;
    let mut previous_vpos = None;
    for item in &st.current_items {
        let owner = item.para_index();
        let para = paragraphs.get(owner)?;
        if para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || para.line_segs.is_empty()
        {
            return None;
        }
        for line in &para.line_segs {
            if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || line.line_height <= 0
                || line.line_spacing < 0
                || previous_vpos.is_some_and(|previous| line.vertical_pos < previous)
            {
                return None;
            }
            previous_vpos = Some(line.vertical_pos);
        }
        match item {
            PageItem::FullParagraph { .. }
                if super::super::controls::stored_tac::text_controls_share_line(para) => {}
            PageItem::Table { control_index, .. } => {
                let [line] = para.line_segs.as_slice() else {
                    return None;
                };
                let Control::Table(table) = para.controls.get(*control_index)? else {
                    return None;
                };
                let placement = st.inline_placements.get(&(owner, *control_index))?;
                let source_top = anchor + hwpunit_to_px(line.vertical_pos.checked_sub(base)?, dpi);
                if !table.common.treat_as_char
                    || !placement.advance_end.is_some_and(|end| {
                        end.is_finite()
                            && end + tolerance >= source_top + hwpunit_to_px(line.line_height, dpi)
                    })
                    || (placement.y - source_top).abs() > tolerance
                {
                    return None;
                }
                saw_tac = true;
            }
            _ => return None,
        }
    }
    saw_tac.then_some((base, anchor))
}

/// A certified source guide owns its whole empty line band, including trailing
/// space. Fit may omit the trailing space at the page end; accepted flow must
/// still advance by the same band which the paragraph painter consumes.
pub(super) fn stored_overlay_guide_advance(
    st: &TypesetState,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    index: usize,
) -> Option<f64> {
    ((st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
        && !st.profile.session_edited()
        && !st.vpos_ladder_dirty
        && crate::renderer::float_placement::stored_overlay_guide_owns_flow(paragraphs, index))
    .then_some(fmt.total_height)
}

/// 다단의 다음 TAC가 소유한 물리 줄 원점을 앞 문단의 점유 끝과 연결한다.
/// 일반 문단의 fit 전진은 저장 사다리의 절대 좌표로 대체하지 않는다.
/// 줄 상자가 겹쳐도 최대 점유 끝과 뒤 간격이 다음 개체 원점에서 닫혀야 한다.
#[allow(clippy::too_many_arguments)]
fn stored_multicolumn_tac_successor_flow(
    st: &TypesetState,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    next: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(f64, f64)> {
    if st.col_count <= 1
        || fmt.stored_ladder_is_filler
        || fmt.computed_host_lines.is_some()
        || para.controls.iter().any(|control| {
            !super::super::controls::stored_tac::inline_text_control(control)
                && !matches!(control, Control::Table(table) if table.common.treat_as_char)
                && !crate::renderer::composer::control_is_width_neutral_marker(control)
        })
    {
        return None;
    }
    // 저장된 마지막 간격은 일반 본문의 쪽말 fit과 같은 값이 아니다.
    // 실제로 그 간격을 소유할 다음 원본 개체 줄을 함께 인증한다.
    let next_line = crate::renderer::composer::stored_first_tac_line(next)?;
    if next.cell_format_vpos_dirty
        || next.line_segs.len() != 1
        || next.column_type != crate::model::paragraph::ColumnBreakType::None
    {
        return None;
    }
    let last = para.line_segs.last()?;
    let own_after = crate::renderer::px_to_hwpunit(fmt.spacing_after, dpi);
    let next_before = styles
        .para_styles
        .get(next.para_shape_id as usize)?
        .spacing_before;
    let source_next = last
        .vertical_pos
        .checked_add(last.line_height)?
        .checked_add(last.line_spacing)?
        .checked_add(own_after)?
        .checked_add(crate::renderer::px_to_hwpunit(next_before, dpi))?;
    if source_next != next_line.vertical_pos {
        return None;
    }
    let bottom = stored_content_bottom(st, para, fmt, paragraphs, styles, dpi)?;
    let trailing = *fmt.line_spacings.last()?;
    let end = bottom + trailing + fmt.spacing_after;
    (trailing >= 0.0 && fmt.spacing_after >= 0.0 && end.is_finite() && bottom >= st.current_height)
        .then_some((bottom, end))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn stored_successor_flow(
    st: &TypesetState,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    para_idx: usize,
    dpi: f64,
) -> Option<crate::renderer::float_placement::StoredParagraphFlowProjection> {
    if st.vpos_ladder_dirty {
        return None;
    }
    if st.col_count > 1 {
        return stored_multicolumn_tac_successor_flow(
            st,
            para,
            fmt,
            paragraphs,
            paragraphs.get(para_idx + 1)?,
            styles,
            dpi,
        )
        .map(|(content_bottom, end)| {
            crate::renderer::float_placement::StoredParagraphFlowProjection {
                content_bottom,
                advance_end: end,
                paint_end: Some(end),
            }
        });
    }
    if fmt.stored_ladder_is_filler
        || fmt.computed_host_lines.is_some()
        || fmt.line_count() != para.line_segs.len()
    {
        return None;
    }
    let frame = crate::renderer::float_placement::stored_paragraph_successor_flow(
        para,
        paragraphs.get(para_idx + 1)?,
        styles,
        st.paragraph_flow_profile(),
        dpi,
    )?;
    frame.project(
        st.current_height,
        st.vpos_page_base.or(st.vpos_lazy_base),
        if st.vpos_page_base.is_some() {
            st.vpos_col_anchor
        } else {
            0.0
        },
        dpi,
    )
}

/// 일반 진입과 분할 재시도가 수용한 첫 문단의 저장 좌표 원점을 공유한다.
pub(in crate::renderer::typeset) fn initialize_first_paragraph_origin(
    st: &mut TypesetState,
    para_idx: usize,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    if st.col_count != 1 || !st.current_items.is_empty() {
        return;
    }
    st.record_vpos_column_anchor(st.current_height);
    st.record_vpos_page_origin(
        paragraphs
            .get(para_idx)
            .and_then(|p| p.line_segs.first())
            .map(|seg| {
                // 배치가 보존하는 문단 앞 여백을 측정의 쪽 원점으로 빼지 않는다.
                let margin_is_page_relative = (st.profile.hwpx_stored_layout()
                    || st.profile.hwp5_stored_pagination_layout()
                    || st.profile.hwp3_native_layout())
                    && !st.profile.session_edited()
                    && paragraphs.get(para_idx).is_some_and(|first| {
                        crate::renderer::stored_page_origin::stored_first_margin_is_page_relative(
                            first,
                            paragraphs.get(para_idx + 1),
                            styles,
                            dpi,
                        )
                    });
                if margin_is_page_relative {
                    0
                } else {
                    seg.vertical_pos
                        - paragraphs.get(para_idx).map_or(0, |first| {
                            crate::renderer::layout::synthesized_first_margin_hu(first, styles, dpi)
                        })
                }
            }),
    );
    st.record_vpos_lazy_origin(None);
    st.record_stored_column_origin(st.vpos_page_base.map(|base| (base, st.vpos_col_anchor)));
    // [#2243] 저장 여부 태깅 — dirty 역스냅 금지 판단용.
    st.record_vpos_origin_provenance(
        paragraphs
            .get(para_idx)
            .and_then(|p| p.line_segs.first())
            .is_some_and(|s| {
                s.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            }),
    );
    st.record_vpos_ladder_validity(paragraphs.get(para_idx).is_some_and(|para| {
        !para.line_segs.is_empty()
            && para.line_segs.iter().all(|seg| {
                seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            })
    }));
}

/// paint와 같은 첫 수용 항목의 저장 원점과 물리 앵커를 보존한다.
/// 이미 확립한 단 첫 여백 프레임은 실제 첫 vpos로 덮어쓰지 않는다.
pub(super) fn record_first_fragment_origin(
    st: &mut TypesetState,
    item: &crate::renderer::pagination::PageItem,
    paragraphs: &[Paragraph],
) {
    if st.current_items.is_empty() {
        let origin = st.vpos_stored_column_origin.or_else(|| {
            super::page_item_vpos_base(item, paragraphs).map(|base| (base, st.current_height))
        });
        st.record_stored_column_origin(origin);
    }
}
