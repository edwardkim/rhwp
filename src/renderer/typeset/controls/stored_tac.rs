//! 저장 줄을 소유한 TAC 표의 수용 판단과 배치 좌표 계산.
//! composer의 기존 줄 소속·점유 끝점을 재사용하며 새 줄 구성이나 상태 쓰기를 하지 않는다.

pub(in crate::renderer::typeset) mod owned_row;

use super::super::paragraph::metrics::FormattedParagraph;
use crate::model::{
    control::Control, paragraph::Paragraph, provenance::LayoutCompatibilityProfile,
};
use crate::renderer::composer::{stored_tac_lines, StoredTacLine};
use crate::renderer::float_placement::{InlineBoxPlacement, StoredColumnFrame};
use crate::renderer::height_measurer::MeasuredTable;
use crate::renderer::hwpunit_to_px;

/// 같은 텍스트 줄의 실제 fit 내용 끝을 사용할 수 있는 컨트롤이다.
/// 독립 float·표는 별도 흐름이므로 이 문단 줄의 끝으로 인증하지 않는다.
pub(in crate::renderer::typeset) fn text_controls_share_line(para: &Paragraph) -> bool {
    para.controls.iter().all(|control| {
        inline_text_control(control)
            || matches!(control, Control::Footnote(_) | Control::Endnote(_))
            || crate::renderer::composer::control_is_width_neutral_marker(control)
    })
}

/// 표와 독립 float를 제외하고 텍스트 줄에 참여하는 실제 컨트롤 소유다.
pub(in crate::renderer::typeset) fn inline_text_control(control: &Control) -> bool {
    match control {
        Control::Equation(equation) => equation.common.treat_as_char,
        Control::Picture(picture) => picture.common.treat_as_char,
        Control::Shape(shape) => shape.common().treat_as_char,
        Control::Form(form) => form.common.treat_as_char,
        _ => false,
    }
}

pub(in crate::renderer::typeset) struct StoredTacPage {
    pub profile: LayoutCompatibilityProfile,
    pub current_height: f64,
    pub empty_column: bool,
    pub vpos_col_anchor: f64,
    pub vpos_page_base: Option<i32>,
    pub vpos_lazy_base: Option<i32>,
    /// 실제 수용한 직전 문단의 소유·끝 줄과 뒤 간격을 제외한 내용 하단.
    pub previous_content_bottom: Option<(usize, usize, f64)>,
    /// 바로 이전 실제 쪽의 마지막 원문 소유 문단과 소비한 끝 줄.
    pub previous_page_tail: Option<(usize, usize)>,
    pub side_wrap_empty: bool,
    pub stored_table_column_base: Option<i32>,
    /// 원본 단 첫 글줄과 물리 앵커가 함께 인증한 영점 축.
    pub stored_zero_column_origin: bool,
    /// 첫 원본 TAC의 확정 배치와 저장 줄이 현재 단 원점에서 일치하는가.
    pub original_object_column_origin: bool,
}

pub(super) struct StoredTacPlan {
    pub lines: Vec<StoredTacLine>,
    origin: f64,
    pub(super) source_frame: Option<StoredColumnFrame>,
}

pub(in crate::renderer::typeset) struct StoredTacControlPlacement {
    pub control_index: usize,
    pub inline: InlineBoxPlacement,
    pub end: f64,
    pub rebase_line_origin: bool,
}

/// 어울림 회피 프레임에서 온전히 소유한 단일 원본 표 줄의 점유와 전진량.
/// 원본 줄 높이가 실제 외곽 상자를 닫는 경우만 수용한다. 앞뒤 글줄,
/// 캡션·각주 또는 다른 흐름 개체는 각자의 일반 분할 경로를 유지한다.
#[allow(clippy::too_many_arguments)]
pub(in crate::renderer::typeset) fn side_wrap_row(
    para: &Paragraph,
    control_index: usize,
    table: &crate::model::table::Table,
    measured_body_height: f64,
    fmt: &FormattedParagraph,
    next_para: Option<&Paragraph>,
    next_spacing_before: f64,
    profile: LayoutCompatibilityProfile,
    has_exclusions: bool,
    reflowed: bool,
    dpi: f64,
) -> Option<(f64, f64)> {
    if !has_exclusions
        || !profile.hwp5_stored_pagination_layout()
        || profile.session_edited()
        || reflowed
        || para.cell_format_vpos_dirty
        || para.line_segs.len() != 1
        || fmt.line_count() != 1
        || table_has_notes(table)
        || para.controls.iter().enumerate().any(|(index, control)| {
            if index == control_index {
                return false;
            }
            match control {
                Control::Picture(picture) => {
                    picture.common.treat_as_char
                        || picture.common.vert_rel_to != crate::model::shape::VertRelTo::Paper
                }
                Control::SectionDef(_)
                | Control::ColumnDef(_)
                | Control::Header(_)
                | Control::Footer(_)
                | Control::PageNumberPos(_)
                | Control::PageHide(_) => false,
                _ => true,
            }
        })
    {
        return None;
    }
    let line = crate::renderer::composer::stored_first_tac_line(para)?;
    if crate::renderer::layout::control_line_seg_index(para, control_index) != Some(0)
        || line.line_spacing < 0
        || (measured_body_height - hwpunit_to_px(table.common.height as i32, dpi)).abs()
            > dpi / 7200.0
    {
        return None;
    }
    let trailing = if crate::renderer::composer::tac_next_line_full_spacing(
        para,
        next_para,
        fmt.spacing_after,
        next_spacing_before,
        line,
        profile,
        dpi,
    ) {
        hwpunit_to_px(line.line_spacing, dpi)
    } else {
        crate::renderer::composer::tac_host_trailing_spacing(
            para,
            control_index,
            line,
            false,
            true,
            dpi,
        )
    };
    Some((
        hwpunit_to_px(line.line_height, dpi),
        trailing + fmt.spacing_after,
    ))
}

/// 같은 문단의 float가 저장 밴드 밖에서 시작해도 TAC 줄은 독립된 흐름을 소유한다.
/// 이 개체만 확정하여 float의 일반 분할 경로를 보존하고 fit과 paint는 같은 원점·끝을 쓴다.
pub(super) fn prepare_coanchored_first_line(
    control_index: usize,
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    next_spacing_before: f64,
    fmt: &FormattedParagraph,
    measured_body_height: f64,
    page: StoredTacPage,
    available_height: f64,
    dpi: f64,
) -> Option<StoredTacControlPlacement> {
    if !page.profile.hwp5_stored_pagination_layout()
        || page.profile.session_edited()
        || !page.side_wrap_empty
        || para.cell_format_vpos_dirty
    {
        return None;
    }
    let line = crate::renderer::composer::stored_first_tac_line(para)?;
    let Control::Table(table) = para.controls.get(control_index)? else {
        return None;
    };
    if !table.common.treat_as_char
        || table.caption.is_some()
        || table_has_notes(table)
        || table.cells.iter().any(|cell| {
            cell.dirty_flag
                || cell.paragraphs.iter().any(|para| {
                    para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty
                })
        })
        || crate::renderer::layout::control_line_seg_index(para, control_index) != Some(0)
        || (measured_body_height - hwpunit_to_px(table.common.height as i32, dpi)).abs()
            > dpi / 7200.0
        || line.line_spacing < 0
    {
        return None;
    }
    let mut has_float = false;
    let mut has_fixed_decoration = false;
    let mut has_nonflow_decoration = false;
    for (index, control) in para.controls.iter().enumerate() {
        if index == control_index {
            continue;
        }
        match control {
            Control::Table(sibling)
                if crate::renderer::float_placement::is_para_topbottom_float(&sibling.common)
                    && crate::renderer::float_placement::signed_hwpunit(
                        sibling.common.vertical_offset,
                    ) >= line.line_height =>
            {
                has_float = true;
            }
            Control::Table(sibling)
                if !sibling.common.treat_as_char
                    && matches!(
                        sibling.common.vert_rel_to,
                        crate::model::shape::VertRelTo::Paper
                            | crate::model::shape::VertRelTo::Page
                    ) =>
            {
                // 용지·쪽 고정 표는 자체 원점을 소유한다. 본문의 TAC 줄 번호나
                // 후행 간격을 소비하는 앞 개체로 해석하지 않는다.
                has_fixed_decoration = true;
            }
            Control::Picture(picture)
                if !picture.common.treat_as_char
                    && matches!(
                        picture.common.text_wrap,
                        crate::model::shape::TextWrap::BehindText
                            | crate::model::shape::TextWrap::InFrontOfText
                    ) =>
            {
                has_nonflow_decoration = true;
            }
            Control::Shape(shape)
                if !shape.common().treat_as_char
                    && matches!(
                        shape.common().text_wrap,
                        crate::model::shape::TextWrap::BehindText
                            | crate::model::shape::TextWrap::InFrontOfText
                    ) =>
            {
                has_nonflow_decoration = true;
            }
            Control::SectionDef(_)
            | Control::ColumnDef(_)
            | Control::Header(_)
            | Control::Footer(_) => {}
            _ => return None,
        }
    }
    let full_trailing = crate::renderer::composer::tac_next_line_full_spacing(
        para,
        next_para,
        fmt.spacing_after,
        next_spacing_before,
        line,
        page.profile,
        dpi,
    );
    // 독립 배경은 보이지만 줄을 점유하지 않는다. 단 축과 다음 저장 원점이
    // 표의 완전한 한 줄을 인증할 때만 일반 흐름의 압축 높이를 대신한다.
    let closed_decorated_line = has_nonflow_decoration
        && page.stored_zero_column_origin
        && para.line_segs.len() == 1
        && fmt.line_count() == 1
        && fmt.spacing_after == 0.0
        && next_spacing_before == 0.0
        && next_para.is_some_and(|next| {
            !next.stored_text_partition_is_dirty()
                && !next.cell_format_vpos_dirty
                && next.line_segs.first().is_some_and(|after| {
                    after.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                        && line
                            .vertical_pos
                            .checked_add(line.line_height)
                            .and_then(|end| end.checked_add(line.line_spacing))
                            == Some(after.vertical_pos)
                })
        })
        && full_trailing;
    if has_nonflow_decoration && !closed_decorated_line {
        return None;
    }
    if !has_float && !(has_fixed_decoration && full_trailing) && !closed_decorated_line {
        return None;
    }
    let origin = page.vpos_col_anchor
        + hwpunit_to_px(
            line.vertical_pos
                .saturating_sub(page.vpos_page_base.or(page.vpos_lazy_base).unwrap_or(0)),
            dpi,
        );
    let end = origin
        + hwpunit_to_px(line.line_height, dpi)
        + hwpunit_to_px(line.line_spacing, dpi) * if full_trailing { 1.0 } else { 0.5 }
        + fmt.spacing_after;
    if origin < page.current_height || end > available_height {
        return None;
    }
    Some(StoredTacControlPlacement {
        control_index,
        inline: InlineBoxPlacement {
            x: 0.0,
            y: origin + hwpunit_to_px(i32::from(table.outer_margin_top), dpi),
            clearance: 0.0,
            resolved_x: false,
            advance_end: Some(end),
            stored_horizontal_pen: None,
            stored_column_frame: None,
        },
        end,
        rebase_line_origin: false,
    })
}

/// 단일 개체 줄의 물리 점유를 같은 배치 결과로 전달한다.
/// 실제 저장 줄은 다음 원점이 닫는 상자만 수용하며, 원점·여백·뒤 간격을 한번씩 소비한다.
pub(super) fn prepare_computed(
    para_idx: usize,
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    fmt: &FormattedParagraph,
    measured_tables: &[MeasuredTable],
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    page: StoredTacPage,
    available_height: impl Fn() -> f64,
    dpi: f64,
) -> Option<StoredTacControlPlacement> {
    if !page.profile.hwpx_stored_layout()
        || page.profile.session_edited()
        || !page.side_wrap_empty
        || super::super::para_has_non_whitespace_text(para)
    {
        return None;
    }
    // 글 앞/뒤 장식은 보이지만 저장 개체 줄의 흐름을 점유하지 않는다.
    // 컨트롤 배열의 번호와 표 소유 줄을 구분하고, 원본 저장 줄이 닫는
    // 상자를 같은 fit/paint 계획으로 전달한다. 자리차지 장식과 다른 표는
    // 일반 컨트롤 경로에 남기며, 합성 줄의 기존 단축 조건은 유지한다.
    let mut owned_table = None;
    for (index, control) in para.controls.iter().enumerate() {
        match control {
            Control::Table(table) if table.common.treat_as_char && owned_table.is_none() => {
                owned_table = Some((index, table));
            }
            Control::Table(table)
                if !table.common.treat_as_char
                    && !table.common.flow_with_text
                    && table.caption.is_none()
                    && !table_has_notes(table)
                    && matches!(
                        table.common.text_wrap,
                        crate::model::shape::TextWrap::InFrontOfText
                            | crate::model::shape::TextWrap::BehindText
                    ) => {}
            Control::Shape(shape)
                if !shape.common().treat_as_char
                    && matches!(
                        shape.common().text_wrap,
                        crate::model::shape::TextWrap::InFrontOfText
                            | crate::model::shape::TextWrap::BehindText
                    ) => {}
            _ => return None,
        }
    }
    let (control_index, table) = owned_table?;
    let [seg] = para.line_segs.as_slice() else {
        return None;
    };
    let computed = crate::renderer::para_has_no_stored_line_segs(para);
    if computed && para.controls.len() != 1 {
        return None;
    }
    let source_vpos = para
        .source_line_seg_vertical_pos
        .as_ref()
        .and_then(|positions| positions.first())
        .copied()
        .unwrap_or(seg.vertical_pos);
    let saved_top = hwpunit_to_px(source_vpos, dpi);
    // 원본 줄은 단 맨 위의 양수 저장 간격을 복구할 때만 이 경로를 쓴다.
    // 구역 누적 좌표는 쪽 위 간격이 아니며, 일반 저장 표는 기존 계획이 소유한다.
    let saved_column_top = !computed
        && page.current_height < 1.0
        && saved_top > 0.0
        && saved_top <= fmt.spacing_before + 0.5;
    // 편집되지 않은 원본 한 줄과 후속 원점이 상자를 정확히 닫으면
    // 바깥 여백을 버리는 일반 단일 TAC 경로로 되돌아가지 않는다.
    let closed_stored_line = !computed
        // 후속 원점과의 연결만으로 쪽·단의 좌표축이 확립되지는 않는다.
        && page
            .vpos_page_base
            .or(page.stored_table_column_base)
            .or(page.vpos_lazy_base)
            .is_some()
        && !para.stored_text_partition_is_dirty()
        && source_vpos == seg.vertical_pos
        && fmt.spacing_before == 0.0
        && fmt.spacing_after == 0.0
        && seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && next_para.is_some_and(|next| {
            !next.stored_text_partition_is_dirty()
                && next.line_segs.first().is_some_and(|after| {
                    after.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                        && after.vertical_pos > seg.vertical_pos
                        && i64::from(after.vertical_pos)
                            == i64::from(seg.vertical_pos)
                                + i64::from(seg.line_height)
                                + i64::from(seg.line_spacing)
                })
        });
    if !computed && !saved_column_top && !closed_stored_line {
        return None;
    }
    if !table.common.treat_as_char
        || table.caption.is_some()
        || table_has_notes(table)
        || seg.line_spacing < 0
    {
        return None;
    }
    let measured = measured_tables
        .iter()
        .find(|m| m.para_index == para_idx && m.control_index == control_index)?;
    // 일반 표 포맷과 paint가 소비하는 저장 셀의 뒤 간격 보정을 먼저 적용한다.
    // 보정 전 측정값으로 줄 소유를 거절하면 표 뒤 흐름만 legacy 상한으로 되돌아간다.
    let fitted = super::super::table::fit_measured_for_host(
        para,
        table,
        Some(measured),
        dpi,
        styles,
        || page.profile,
    );
    let measured = fitted.as_ref().unwrap_or(measured);
    let top = hwpunit_to_px(table.outer_margin_top as i32, dpi);
    let bottom = hwpunit_to_px(table.outer_margin_bottom as i32, dpi);
    let height = measured.total_height + top + bottom;
    // 합성 개체 줄은 표 본체 높이 또는 바깥 여백까지 포함한 높이를 갖는다.
    // 한 줄의 공백은 별도 텍스트 줄이 아니며, 실제 점유에는 여백을 한 번 포함한다.
    // 두 높이 모두 맞지 않는 커진 표는 일반 분할기로 보낸다.
    let line_height = hwpunit_to_px(seg.line_height, dpi);
    if (height - line_height).abs() > 0.5
        && !(computed && (measured.total_height - line_height).abs() <= 0.5)
    {
        return None;
    }
    let flow_origin = page.current_height
        + if page.current_height < 1.0 {
            if saved_column_top {
                saved_top
            } else {
                0.0
            }
        } else {
            fmt.spacing_before
        };
    // 단 첫 완전한 저장 표의 원점은 paint가 처음 확립한 좌표축이다.
    // 지연 fit 커서가 압축됐어도 그 축의 빈 물리 공간을 삭제하지 않는다.
    // 새 단에 이전 구역 누적 좌표를 적용하지 않으며, 실제 흐름이 더 자랐으면 보존한다.
    let origin = if computed && page.current_height >= 1.0 {
        page.stored_table_column_base
            .map(|base| {
                flow_origin.max(
                    page.vpos_col_anchor
                        + hwpunit_to_px(seg.vertical_pos.saturating_sub(base), dpi),
                )
            })
            .unwrap_or(flow_origin)
    } else if closed_stored_line {
        flow_origin.max(
            page.vpos_col_anchor
                + hwpunit_to_px(
                    seg.vertical_pos.saturating_sub(
                        page.vpos_page_base
                            .or(page.stored_table_column_base)
                            .or(page.vpos_lazy_base)
                            .unwrap_or(0),
                    ),
                    dpi,
                ),
        )
    } else {
        flow_origin
    };
    // 다음 줄의 시작이 이 줄의 끝과 정확히 이어지면 빈 글줄도
    // 후행 간격 전량 뒤에 놓인다. 글자 유무로 간격을 반감하면
    // 확정 계획 끝에서 역산한 기준축과 이후 본문 원점이 어긋난다.
    let full_trailing_spacing = next_para.is_some_and(|next| {
        next.line_segs.first().is_some_and(|next_seg| {
            next_seg.vertical_pos > seg.vertical_pos
                && seg
                    .vertical_pos
                    .saturating_add(seg.line_height)
                    .saturating_add(seg.line_spacing)
                    == next_seg.vertical_pos
        })
    });
    // 원본 저장 줄은 다음 줄 원점까지 후행 간격을 전량 소유한다.
    // 절반만 소비하고 저장 끝으로 기준축을 역산하면 이후 모든 줄이 위로 이동한다.
    // 저장 좌표가 없는 합성 줄에서만 기존 빈 문단 간격 분배를 적용한다.
    let trailing_fraction = if saved_column_top || closed_stored_line || full_trailing_spacing {
        1.0
    } else {
        0.5
    };
    let end = origin
        + height
        + hwpunit_to_px(seg.line_spacing, dpi) * trailing_fraction
        + fmt.spacing_after;
    if end > available_height() {
        return None;
    }
    Some(StoredTacControlPlacement {
        control_index,
        inline: InlineBoxPlacement {
            x: 0.0,
            y: origin,
            clearance: 0.0,
            resolved_x: false,
            advance_end: Some(end),
            stored_horizontal_pen: None,
            stored_column_frame: None,
        },
        end,
        rebase_line_origin: !closed_stored_line,
    })
}

/// 각주·미주는 일반 컨트롤 경로에서 예약·등록해야 하므로 통배치 단축을 쓰지 않는다.
pub(super) fn table_has_notes(table: &crate::model::table::Table) -> bool {
    table.cells.iter().any(|cell| {
        cell.paragraphs.iter().any(|para| {
            para.controls.iter().any(|control| match control {
                Control::Footnote(_) | Control::Endnote(_) => true,
                Control::Table(nested) => table_has_notes(nested),
                _ => false,
            })
        })
    })
}

/// 가용 높이는 원래 all 검사 위치에서 조회한다. 진단 조회를 미리 호출하지 않는다.
#[allow(clippy::too_many_arguments)]
pub(super) fn prepare(
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    measured_tables: &[MeasuredTable],
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    page: StoredTacPage,
    shared_spacing_before: f64,
    previous_para: Option<&Paragraph>,
    previous_spacing_after: f64,
    next_para: Option<&Paragraph>,
    next_spacing_before: f64,
    explicit_page_break_before: bool,
    available_height: impl Fn() -> f64,
    dpi: f64,
) -> Option<StoredTacPlan> {
    // 완전한 저장 줄 계약은 표별 높이를 합산하지 않고, 같은 pen/end를 배치에도 전달한다.
    if !page.profile.session_edited()
        && (page.profile.hwp5_container()
            || page.profile.hwp5_stored_pagination_layout()
            || page.profile.hwpx_stored_layout())
        && page.side_wrap_empty
    {
        // 저장 단일 개체 줄은 캡션과 바깥 여백도 소유한다. 같은 줄의 공백은
        // 별도 글줄이 아니며 원본 소속과 전체 높이를 아래에서 확인한다.
        let single_saved_object_line = || {
            if !para.text.chars().all(char::is_whitespace)
                || para.stored_text_partition_is_dirty()
                || para.cell_format_vpos_dirty
            {
                return None;
            }
            let [seg] = para.line_segs.as_slice() else {
                return None;
            };
            // 머리말·꼬리말 정의는 구역 finalize가 따로 소비하며 본문 줄을
            // 점유하지 않는다. 실제 표의 control slot과 저장 줄은 보존한다.
            let mut owned_table = None;
            for (index, control) in para.controls.iter().enumerate() {
                match control {
                    Control::Table(table) if owned_table.is_none() => {
                        owned_table = Some((index, table));
                    }
                    Control::Header(_) | Control::Footer(_) => {}
                    _ => return None,
                }
            }
            let (control_index, table) = owned_table?;
            let caption_extent = if let Some(caption) = &table.caption {
                if !matches!(
                    caption.direction,
                    crate::model::shape::CaptionDirection::Top
                        | crate::model::shape::CaptionDirection::Bottom
                ) {
                    return None;
                }
                (crate::renderer::composer::caption_height_px(&table.caption, dpi) * 7200.0 / dpi)
                    .round() as i64
                    + i64::from(caption.spacing)
            } else {
                0
            };
            if !table.common.treat_as_char
                || table_has_notes(table)
                || crate::renderer::composer::control_slot_line_index(para, control_index)
                    != Some(0)
                || seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || seg.line_spacing < 0
                || i64::from(seg.line_height)
                    != i64::from(table.common.height)
                        + caption_extent
                        + i64::from(table.outer_margin_top)
                        + i64::from(table.outer_margin_bottom)
            {
                return None;
            }
            let trailing = if crate::renderer::composer::tac_next_line_full_spacing(
                para,
                next_para,
                fmt.spacing_after,
                next_spacing_before,
                seg,
                page.profile,
                dpi,
            ) {
                hwpunit_to_px(seg.line_spacing, dpi)
            } else {
                crate::renderer::composer::tac_host_trailing_spacing_at_boundary(
                    para,
                    next_para,
                    control_index,
                    seg,
                    page.profile.hwpx_stored_layout(),
                    page.profile.hwp5_stored_pagination_layout(),
                    dpi,
                )
            };
            Some(vec![StoredTacLine {
                control: control_index,
                top: 0,
                occupied_end: seg.line_height,
                end: seg
                    .line_height
                    .checked_add(crate::renderer::px_to_hwpunit(trailing, dpi))?,
            }])
        };
        let owned_lines = stored_tac_lines(para)
            .map(|lines| (lines, false))
            .or_else(|| single_saved_object_line().map(|lines| (lines, true)));
        if let Some((lines, single_saved_line)) = owned_lines {
            // 2024 단일 TAC의 앞 앵커 회수량은 일반 경로에서 후속 쪽 경계와 함께 소비한다.
            if page.profile.hangul2024_layout() && lines.len() == 1 && lines[0].top > 0 {
                return None;
            }
            let source_top = hwpunit_to_px(
                para.source_line_seg_vertical_pos
                    .as_ref()
                    .and_then(|positions| positions.first())
                    .copied()
                    .unwrap_or(para.line_segs[0].vertical_pos),
                dpi,
            );
            let flow_origin = if single_saved_line {
                // 저장 줄 원점은 앞 간격을 이미 소유한다. 현재 물리 흐름은 그 하한이며
                // 앞 간격을 다시 더하지 않는다.
                page.current_height
            } else {
                page.current_height
                    + if page.current_height < 1.0 {
                        if source_top > 0.0 && source_top <= fmt.spacing_before + 0.5 {
                            source_top
                        } else {
                            0.0
                        }
                    } else {
                        fmt.spacing_before - shared_spacing_before
                    }
            };
            // 앞 문단의 저장 사다리가 누적 높이보다 앞서 있으면 그 앵커를
            // fit와 paint에 함께 보존한다. 문단 상대 top만 더하면 앞 표로 되감긴다.
            let mut saved_origin = page.vpos_col_anchor
                + hwpunit_to_px(
                    if single_saved_line {
                        // An original object box is page-relative. A paragraph
                        // ladder's text base does not authenticate its object origin.
                        para.line_segs[0].vertical_pos
                    } else {
                        para.line_segs[0].vertical_pos.saturating_sub(
                            page.vpos_page_base.or(page.vpos_lazy_base).unwrap_or(0),
                        )
                    },
                    dpi,
                );
            let restarted_column_line = single_saved_line
                && page.empty_column
                && page.current_height == 0.0
                && page.vpos_col_anchor == 0.0
                && fmt.spacing_before == 0.0
                && fmt.spacing_after == 0.0
                && next_para.is_some_and(|next| {
                    !next.stored_text_partition_is_dirty()
                        && !next.cell_format_vpos_dirty
                        && next.controls.is_empty()
                        && next.column_type == crate::model::paragraph::ColumnBreakType::None
                        && next.line_segs.first().is_some_and(|after| {
                            !crate::renderer::typeset::is_synthetic_line_seg(after)
                                && after.vertical_pos < para.line_segs[0].vertical_pos
                                && i64::from(after.vertical_pos)
                                    == i64::from(para.line_segs[0].line_height)
                                        + i64::from(para.line_segs[0].line_spacing)
                                        + i64::from(crate::renderer::px_to_hwpunit(
                                            next_spacing_before,
                                            dpi,
                                        ))
                        })
                });
            let continued_column_line = single_saved_line
                && !explicit_page_break_before
                && page.empty_column
                && page.current_height == 0.0
                && page.vpos_col_anchor == 0.0
                && para.column_type == crate::model::paragraph::ColumnBreakType::None
                && !para.line_segs[0].is_first_line_of_page()
                && previous_para.is_some_and(|previous| {
                    if previous.stored_text_partition_is_dirty()
                        || previous.cell_format_vpos_dirty
                        || !page.previous_page_tail.is_some_and(|(owner, end)| {
                            owner.checked_add(1) == Some(para_idx)
                                && end == previous.line_segs.len()
                        })
                    {
                        return false;
                    }
                    previous.line_segs.last().is_some_and(|last| {
                        !crate::renderer::typeset::is_synthetic_line_seg(last)
                            && last.vertical_pos > 0
                            && last.line_height > 0
                            && last.line_spacing >= 0
                            && last
                                .vertical_pos
                                .checked_add(last.line_height)
                                .and_then(|end| end.checked_add(last.line_spacing))
                                .and_then(|end| {
                                    end.checked_add(crate::renderer::px_to_hwpunit(
                                        previous_spacing_after + fmt.spacing_before,
                                        dpi,
                                    ))
                                })
                                == Some(para.line_segs[0].vertical_pos)
                    })
                });
            if restarted_column_line || continued_column_line {
                // 원본에 남은 옛 host vpos는 새 쪽 개체의 출력 원점이 아니다.
                // 뒤 저장 줄이 새 개체 줄의 전체 전진과 일치하는 0축을 소유한다.
                saved_origin = flow_origin;
            }
            // A complete object height authenticates the local line, not the
            // page-relative origin. A single-line shortcut may only reuse a
            // source origin which agrees with the current physical flow: its
            // leading band is either already reserved or still belongs to
            // this paragraph. Other stored ladders use ordinary TAC flow.
            if single_saved_line {
                let unreserved_origin =
                    page.current_height + fmt.spacing_before - shared_spacing_before;
                let same_frame = (saved_origin - page.current_height).abs() <= dpi / 7200.0
                    || (saved_origin - unreserved_origin).abs() <= dpi / 7200.0
                    || (page.original_object_column_origin
                        && para.source_line_seg_vertical_pos.is_none());
                // 앞 저장 글줄의 후행 간격은 표의 진입 밴드다. 수용 커서가
                // 그 글줄의 내용 끝에 있으면 같은 좌표축에서 표 원점까지 연결한다.
                let inherited_origin = previous_para.and_then(|previous| {
                    if previous.stored_text_partition_is_dirty() || previous.cell_format_vpos_dirty
                    {
                        return None;
                    }
                    let last = previous.line_segs.last()?;
                    let next = &para.line_segs[0];
                    let valid = last.tag
                        & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                        == 0
                        && last.line_spacing >= 0
                        && last
                            .vertical_pos
                            .checked_add(last.line_height)
                            .and_then(|end| end.checked_add(last.line_spacing))
                            .and_then(|end| {
                                end.checked_add(crate::renderer::px_to_hwpunit(
                                    previous_spacing_after + fmt.spacing_before,
                                    dpi,
                                ))
                            })
                            == Some(next.vertical_pos);
                    let (owner, end_line, content_end) = page.previous_content_bottom?;
                    // 실제 수용한 완전한 글줄 조각이 소유를 인증한다. 같은 문단의
                    // 독립 표가 있다는 사실로 그 본문 소유까지 거절하지 않는다.
                    if owner.checked_add(1) != Some(para_idx)
                        || end_line != previous.line_segs.len()
                    {
                        return None;
                    }
                    // 확정된 내용 하단에 원본의 같은 줄 사다리가 소유하는
                    // 상대 간격을 한 번 연결한다. 스타일 뒤 간격의 소비량을 추측하지 않는다.
                    let source_end = last.vertical_pos.checked_add(last.line_height)?;
                    let gap = next.vertical_pos.checked_sub(source_end)?;
                    (valid && gap >= 0).then(|| content_end + hwpunit_to_px(gap, dpi))
                });
                // 저장 축과 실제 내용 하단이 갈리는 경계를 같은 진단에서 추적한다.
                if std::env::var("RHWP_DIAG_FLOW").is_ok() {
                    eprintln!(
                        "DIAG_STORED_TAC pi={para_idx} current={} saved={} same_frame={same_frame} inherited={inherited_origin:?} previous={:?} page_base={:?} lazy_base={:?} anchor={}",
                        page.current_height, saved_origin, page.previous_content_bottom,
                        page.vpos_page_base, page.vpos_lazy_base, page.vpos_col_anchor,
                    );
                }
                if !same_frame && inherited_origin.is_none() {
                    return None;
                }
                // 기존 축이 증명된 경우는 보존한다. 다른 경우에는 소유가
                // 확인된 직전 실제 줄과 저장 상대 간격으로 축을 확립한다.
                if !same_frame {
                    saved_origin = inherited_origin?;
                }
            }
            let origin = flow_origin.max(saved_origin);
            let measured_fits = lines.iter().all(|line| {
                let Some(Control::Table(table)) = para.controls.get(line.control) else {
                    return false;
                };
                // 새로 수용한 단일 빈 줄/공백 캐리어의 각주 예약은
                // 일반 경로가 담당한다. 기존 복수 제어 줄의 처리는 유지한다.
                if (lines.len() == 1 || !para.text.is_empty()) && table_has_notes(table) {
                    return false;
                }
                measured_tables
                    .iter()
                    .find(|m| m.para_index == para_idx && m.control_index == line.control)
                    .is_some_and(|m| {
                        // 측정 행은 내부 표를 소유한다. 일반 표 포맷과 같은
                        // 바깥 프레임을 복원한 전체 높이로 저장 줄을 인증한다.
                        let physical_height = if page.profile.hwp5_stored_pagination_layout()
                            || page.profile.hwpx_stored_layout()
                        {
                            crate::renderer::height_measurer::unwrapped_table_whole_height(
                                table,
                                m.total_height,
                                dpi,
                                styles,
                            )
                        } else {
                            m.total_height
                        };
                        let caption_extent = table.caption.as_ref().map_or(0.0, |caption| {
                            crate::renderer::composer::caption_height_px(&table.caption, dpi)
                                + hwpunit_to_px(i32::from(caption.spacing), dpi)
                        });
                        let residual = physical_height
                            - hwpunit_to_px(table.common.height as i32, dpi)
                            - caption_extent;
                        if std::env::var("RHWP_DIAG_FLOW").is_ok() {
                            eprintln!(
                                "DIAG_STORED_TAC_FIT pi={para_idx} control={} measured={} physical={} declared={} caption={} residual={}",
                                line.control, m.total_height, physical_height,
                                table.common.height, caption_extent, residual,
                            );
                        }
                        residual.abs() <= 0.5
                    })
            });
            // Fit the occupied line box. Its trailing gap advances the next
            // line but can lie beyond this page's final object, as for a text
            // line. Internal gaps already belong to the later occupied ends.
            let fits = lines.iter().all(|line| {
                origin + hwpunit_to_px(line.occupied_end, dpi) + fmt.spacing_after
                    <= available_height()
            });
            if measured_fits && fits {
                let source_origin = if single_saved_line {
                    if restarted_column_line {
                        Some(0)
                    } else {
                        let seg = &para.line_segs[0];
                        // 저장 축은 단의 물리 앵커를 제외한 상대 원점에서 역산한다.
                        // 시작 밴드를 기준축에도 넣으면 이후 표에 그 밴드가 중복된다.
                        // 뒤 간격은 원점 이동량이 아니므로 이 환산에 포함하지 않는다.
                        // 원본 정수 단위로 돌아갈 때 절삭하면 계산 오차가 1HU 이동을 만든다.
                        let relative_origin = ((origin - page.vpos_col_anchor)
                            * crate::renderer::HWPUNIT_PER_INCH
                            / dpi)
                            .round() as i32;
                        Some(seg.vertical_pos.saturating_sub(relative_origin))
                    }
                } else {
                    // 줄 소속을 보존한 일반 저장 TAC 계획도 기존 단 축과 실제 pen이
                    // 일치하면 그 축을 유지한다. 현재 흐름으로 밀린 계획은 인증하지 않는다.
                    page.vpos_page_base
                        .filter(|_| (origin - saved_origin).abs() <= dpi / 7200.0)
                };
                return Some(StoredTacPlan {
                    lines,
                    origin,
                    source_frame: source_origin.map(|source_origin_hu| StoredColumnFrame {
                        source_origin_hu,
                        column_anchor_y: page.vpos_col_anchor,
                    }),
                });
            }
        }
    }
    None
}

impl StoredTacPlan {
    /// 원래 순서대로 한 표씩 계산·확정한다. 마지막 소유 표에만 문단 아래 간격을 더한다.
    pub(super) fn placement(
        &self,
        line: &StoredTacLine,
        spacing_after: f64,
        dpi: f64,
    ) -> StoredTacControlPlacement {
        let end = self.origin
            + hwpunit_to_px(line.end, dpi)
            + if line.control == self.lines.last().unwrap().control {
                spacing_after
            } else {
                0.0
            };
        StoredTacControlPlacement {
            control_index: line.control,
            inline: InlineBoxPlacement {
                x: 0.0,
                y: self.origin + hwpunit_to_px(line.top, dpi),
                clearance: 0.0,
                resolved_x: false,
                advance_end: Some(end),
                stored_horizontal_pen: None,
                stored_column_frame: self.source_frame,
            },
            end,
            rebase_line_origin: false,
        }
    }
}
