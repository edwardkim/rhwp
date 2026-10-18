//! A repeated title row starts a new outer frame at each stored table fragment.

use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::provenance::LayoutCompatibilityProfile;
use crate::model::shape::{HorzRelTo, VertAlign, VertRelTo};
use crate::model::table::{Table, TablePageBreak};

/// 저장 호스트의 실제 글줄과 앵커가 증언하는 독립 표 프레임이다.
/// 제어 문자가 글자0에 있어야 한다는 컷 소유 계약과 구분한다.
/// 여백·본문 하단 인증은 이 같은 기하 판정을 사용하며, 실제 줄보다
/// 아래에서 시작하지 않는 표나 다른 가시 개체가 섞인 호스트는 추정하지 않는다.
pub(crate) fn stored_independent_table_frame_host(para: &Paragraph, table: &Table) -> bool {
    if super::object_only_saved_table_anchor(para, table)
        || super::reflow_empty_table_host(para, table)
    {
        return true;
    }
    // 각주·미주와 구역·단 정의는 호스트에 속하며 독립된 두 번째 표 상자가 아니다.
    // 다른 가시 개체가 함께 있으면 단일 표 프레임으로 판정하지 않는다.
    let table_control_index = para
        .controls
        .iter()
        .position(|control| matches!(control, Control::Table(_)));
    let single_table_frame_host = para
        .controls
        .iter()
        .filter(|control| matches!(control, Control::Table(_)))
        .count()
        == 1
        && para.controls.iter().all(|control| {
            matches!(
                control,
                Control::Table(_)
                    | Control::Footnote(_)
                    | Control::Endnote(_)
                    | Control::SectionDef(_)
                    | Control::ColumnDef(_)
            )
        });
    // 여러 저장 글줄도 개체 앞에서 모두 끝나면 같은 바깥 프레임을 소유한다.
    // 앵커 계획과 같은 유효성 판정을 소비하므로 되감긴 원본 줄은 받아들이지 않는다.
    if single_table_frame_host
        && table_control_index.is_some_and(|index| {
            super::ParagraphFloatPlacement::text_head_control_position(para, index).is_some()
        })
        && table_control_index.is_some_and(|index| {
            super::ParagraphFloatPlacement::stored_host_lines_are_valid(para, table, index)
        })
        && para
            .line_segs
            .first()
            .is_some_and(|line| line.vertical_pos >= 0)
        && !para.cell_format_vpos_dirty
    {
        return true;
    }
    let [line] = para.line_segs.as_slice() else {
        return false;
    };
    super::para_has_non_whitespace_text(para)
        && single_table_frame_host
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
        && line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && line.vertical_pos >= 0
        && line.line_height > 0
        && table.common.vert_rel_to == VertRelTo::Para
        && matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        && super::signed_hwpunit(table.common.vertical_offset) >= line.line_height
}

/// A source host may carry a title already consumed before this table frame.
/// Its original control slot and line ladder, rather than visible text alone,
/// determine whether it owns the independent object frame.
pub(crate) fn stored_rowbreak_outer_frame_host(host: &Paragraph, table: &Table) -> bool {
    if host.text.is_empty() && matches!(host.controls.as_slice(), [Control::Table(_)]) {
        return true;
    }
    host.controls
        .iter()
        .filter(|control| matches!(control, Control::Table(_)))
        .count()
        == 1
        && host.controls.iter().all(|control| {
            matches!(
                control,
                Control::Table(_) | Control::Footnote(_) | Control::Endnote(_)
            )
        })
        && host
            .controls
            .iter()
            .position(|control| matches!(control, Control::Table(_)))
            .is_some_and(|index| {
                super::ParagraphFloatPlacement::stored_head_host_lines_are_valid(host, table, index)
            })
}

/// 가로 원점은 문단 또는 단에 속할 수 있지만 반복 제목행이 여는
/// 세로 프레임의 바깥 여백 소유는 그 가로 기준으로 바뀌지 않는다.
/// 예산·수용한 물리 행·최종 배치는 같은 저장 프레임 판정을 소비한다.
pub(crate) fn stored_repeated_header_rowbreak_outer_frame(
    profile: LayoutCompatibilityProfile,
    para: &Paragraph,
    table: &Table,
) -> bool {
    (profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
        && !profile.session_edited()
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
        && !para.line_segs.is_empty()
        && para.line_segs.iter().all(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.vertical_pos >= 0
                && line.line_height > 0
        })
        && !table.common.treat_as_char
        && super::is_para_topbottom_float(&table.common)
        && table.common.vert_rel_to == VertRelTo::Para
        && matches!(
            table.common.horz_rel_to,
            HorzRelTo::Para | HorzRelTo::Column
        )
        && table.page_break == TablePageBreak::RowBreak
        && table.repeat_header
        && !table.leading_header_rows().is_empty()
}

/// 반복 제목행의 저장 바깥 프레임은 일반 셀의 내용 컷 뒤에도 유지된다.
/// 컷은 남은 내용의 소유이며 새 단 상단의 바깥 여백 소유를 바꾸지 않는다.
/// 예약과 최종 배치가 같은 프로필과 저장 프레임 판정을 사용한다.
pub(crate) fn stored_rowbreak_fragment_opens_outer_top(
    profile: LayoutCompatibilityProfile,
    host: Option<&Paragraph>,
    table: &Table,
    is_continuation: bool,
    start_row: usize,
    start_cut: &[usize],
    starts_at_column_top: bool,
) -> bool {
    (is_continuation
        && starts_at_column_top
        && table.outer_margin_top > 0
        && host
            .is_some_and(|para| stored_repeated_header_rowbreak_outer_frame(profile, para, table)))
        || super::column_rowbreak_fragment_opens_outer_top(
            profile.hwpx_stored_layout(),
            host,
            table,
            is_continuation,
            start_row,
            start_cut,
            starts_at_column_top,
        )
}

/// An entire repeated title block is indivisible when it fits a fresh page.
/// A following original inline box still owns its recorded slot on this page.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stored_whole_header_next_page_placement(
    host: &Paragraph,
    following: &[Paragraph],
    table: &Table,
    measured_height: f64,
    frame_vpos: i32,
    actual_host_flow_y: f64,
    available_height: f64,
    dpi: f64,
) -> Option<(super::ParagraphFloatPlacement, Option<usize>)> {
    let [line] = host.line_segs.as_slice() else {
        return None;
    };
    if !host.text.is_empty()
        || !matches!(host.controls.as_slice(), [Control::Table(_)])
        || host.stored_text_partition_is_dirty()
        || host.cell_format_vpos_dirty
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.line_height <= 0
        || table.common.treat_as_char
        || !super::is_para_topbottom_float(&table.common)
        || table.common.vert_rel_to != VertRelTo::Para
        || table.common.vert_align != crate::model::shape::VertAlign::Top
        || table.page_break != TablePageBreak::RowBreak
        || !table.repeat_header
        || table.row_count <= 1
        || table.leading_header_rows().len() != usize::from(table.row_count)
        || table.caption.is_some()
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
        || !dpi.is_finite()
        || dpi <= 0.0
        || !measured_height.is_finite()
        || !actual_host_flow_y.is_finite()
        || !available_height.is_finite()
        || table.cells.iter().any(|cell| {
            cell.dirty_flag
                || cell.paragraphs.iter().any(|para| {
                    para.stored_text_partition_is_dirty()
                        || para.cell_format_vpos_dirty
                        || !para.controls.is_empty()
                        || para.line_segs.is_empty()
                        || para.line_segs.iter().any(|line| {
                            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                                || line.line_height <= 0
                                || line.vertical_pos < 0
                        })
                })
        })
    {
        return None;
    }
    let epsilon = super::hwpunit_to_px(2, dpi);
    let declared = super::hwpunit_to_px(table.common.height as i32, dpi);
    if (declared - measured_height).abs() > epsilon {
        return None;
    }
    let top = super::hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    let bottom =
        top + measured_height + super::hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    let offset = super::hwpunit_to_px(super::signed_hwpunit(table.common.vertical_offset), dpi);
    let source_host = super::hwpunit_to_px(line.vertical_pos.checked_sub(frame_vpos)?, dpi);
    if source_host <= 0.0
        || offset < 0.0
        || bottom > available_height
        || actual_host_flow_y + offset + bottom <= available_height
        || source_host + offset + bottom <= available_height
    {
        return None;
    }
    let next = following.first()?;
    let [next_line] = next.line_segs.as_slice() else {
        return None;
    };
    let [Control::Table(inline)] = next.controls.as_slice() else {
        return None;
    };
    if !inline.common.treat_as_char
        || !next.text.is_empty()
        || next.stored_text_partition_is_dirty()
        || next.cell_format_vpos_dirty
        || next_line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || next_line.line_height <= 0
        || i64::from(next_line.vertical_pos)
            != i64::from(line.vertical_pos)
                + i64::from(line.line_height)
                + i64::from(line.line_spacing)
        || i64::from(next_line.line_height)
            != i64::from(inline.common.height)
                + i64::from(inline.outer_margin_top)
                + i64::from(inline.outer_margin_bottom)
        || super::hwpunit_to_px(
            next_line
                .vertical_pos
                .checked_sub(frame_vpos)?
                .checked_add(next_line.line_height)?,
            dpi,
        ) > available_height
    {
        return None;
    }
    // A following zero-origin title belongs after the deferred physical page.
    // Keep its source ownership explicitly; the table's box is not enlarged to
    // consume that page's blank tail and cannot absorb this title below itself.
    let mut following_body = None;
    let mut previous = next_line;
    'source_tail: for (index, para) in following.iter().enumerate().skip(1) {
        if para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || !para.controls.is_empty()
            || para.line_segs.is_empty()
        {
            break;
        }
        for current in &para.line_segs {
            if current.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 || current.line_height <= 0 {
                break 'source_tail;
            }
            if current.vertical_pos < previous.vertical_pos {
                if current.vertical_pos == 0 && super::para_has_non_whitespace_text(para) {
                    following_body = Some(index);
                }
                break 'source_tail;
            }
            if super::para_has_non_whitespace_text(para)
                || i64::from(current.vertical_pos)
                    != i64::from(previous.vertical_pos)
                        + i64::from(previous.line_height)
                        + i64::from(previous.line_spacing)
            {
                break 'source_tail;
            }
            previous = current;
        }
    }
    Some((
        super::ParagraphFloatPlacement {
            flow: super::ParagraphFloatFlow::NextLine,
            anchor_y: 0.0,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: top,
            occupied_bottom: bottom,
        },
        following_body,
    ))
}
