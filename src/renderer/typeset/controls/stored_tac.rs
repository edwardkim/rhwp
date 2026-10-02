//! 저장 줄을 소유한 TAC 표의 수용 판단과 배치 좌표 계산.
//! composer의 기존 줄 소속·점유 끝점을 재사용하며 새 줄 구성이나 상태 쓰기를 하지 않는다.

use super::super::paragraph::metrics::FormattedParagraph;
use crate::model::{
    control::Control, paragraph::Paragraph, provenance::LayoutCompatibilityProfile,
};
use crate::renderer::composer::{stored_tac_lines, StoredTacLine};
use crate::renderer::float_placement::InlineBoxPlacement;
use crate::renderer::height_measurer::MeasuredTable;
use crate::renderer::hwpunit_to_px;

pub(in crate::renderer::typeset) struct StoredTacPage {
    pub profile: LayoutCompatibilityProfile,
    pub current_height: f64,
    pub vpos_col_anchor: f64,
    pub vpos_page_base: Option<i32>,
    pub vpos_lazy_base: Option<i32>,
    pub side_wrap_empty: bool,
    pub stored_table_column_base: Option<i32>,
}

pub(super) struct StoredTacPlan {
    pub lines: Vec<StoredTacLine>,
    origin: f64,
    pub(super) source_origin: Option<i32>,
}

pub(in crate::renderer::typeset) struct StoredTacControlPlacement {
    pub control_index: usize,
    pub inline: InlineBoxPlacement,
    pub end: f64,
}

/// A TAC line remains an independent flow owner when a coanchored float starts
/// outside its saved band. Commit just this control, so the float still takes
/// the ordinary split path. Fit and paint consume the same saved pen and end.
pub(super) fn prepare_coanchored_first_line(
    control_index: usize,
    para: &Paragraph,
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
            Control::SectionDef(_)
            | Control::ColumnDef(_)
            | Control::Header(_)
            | Control::Footer(_) => {}
            _ => return None,
        }
    }
    if !has_float {
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
        + hwpunit_to_px(line.line_spacing, dpi) * 0.5
        + fmt.spacing_after;
    if origin < page.current_height || end > available_height {
        return None;
    }
    Some(StoredTacControlPlacement {
        control_index,
        inline: InlineBoxPlacement {
            x: 0.0,
            y: origin,
            clearance: 0.0,
            advance_end: Some(end),
        },
        end,
    })
}

/// 저장 좌표가 없는 단일 개체 줄은 현재 흐름에서 물리 점유를 확정한다.
/// 원점과 후행 간격을 같은 결과로 넘겨 저장 사다리의 차감·상한을 재적용하지 않는다.
pub(super) fn prepare_computed(
    para_idx: usize,
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    fmt: &FormattedParagraph,
    measured_tables: &[MeasuredTable],
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
    let [Control::Table(table)] = para.controls.as_slice() else {
        return None;
    };
    let [seg] = para.line_segs.as_slice() else {
        return None;
    };
    let computed = crate::renderer::para_has_no_stored_line_segs(para);
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
    if !computed && !saved_column_top {
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
        .find(|m| m.para_index == para_idx && m.control_index == 0)?;
    // 일반 표 포맷과 paint가 소비하는 저장 셀의 뒤 간격 보정을 먼저 적용한다.
    // 보정 전 측정값으로 줄 소유를 거절하면 표 뒤 흐름만 legacy 상한으로 되돌아간다.
    let fitted =
        super::super::table::fit_measured_for_host(para, table, Some(measured), dpi, || {
            page.profile
        });
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
    } else {
        flow_origin
    };
    // 다음 가시 문단의 저장 시작이 이 줄의 끝과 정확히 이어지면
    // 후행 간격 전량이 그 문단 앞에 있다. 빈 문단 경계에서는 일반 TAC
    // 조판처럼 양쪽이 간격을 나누어 갖는다.
    let full_trailing_spacing = next_para.is_some_and(|next| {
        super::super::para_has_non_whitespace_text(next)
            && next.line_segs.first().is_some_and(|next_seg| {
                seg.vertical_pos
                    .saturating_add(seg.line_height)
                    .saturating_add(seg.line_spacing)
                    == next_seg.vertical_pos
            })
    });
    // 원본 저장 줄은 다음 줄 원점까지 후행 간격을 전량 소유한다.
    // 절반만 소비하고 저장 끝으로 기준축을 역산하면 이후 모든 줄이 위로 이동한다.
    // 저장 좌표가 없는 합성 줄에서만 기존 빈 문단 간격 분배를 적용한다.
    let trailing_fraction = if saved_column_top || full_trailing_spacing {
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
        control_index: 0,
        inline: InlineBoxPlacement {
            x: 0.0,
            y: origin,
            clearance: 0.0,
            advance_end: Some(end),
        },
        end,
    })
}

/// 각주·미주는 일반 컨트롤 경로에서 예약·등록해야 하므로 통배치 단축을 쓰지 않는다.
fn table_has_notes(table: &crate::model::table::Table) -> bool {
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
    page: StoredTacPage,
    shared_spacing_before: f64,
    next_para: Option<&Paragraph>,
    next_spacing_before: f64,
    available_height: impl Fn() -> f64,
    dpi: f64,
) -> Option<StoredTacPlan> {
    // 완전한 저장 줄 계약은 표별 높이를 합산하지 않고, 같은 pen/end를 배치에도 전달한다.
    if !page.profile.session_edited()
        && (page.profile.hwp5_stored_pagination_layout() || page.profile.hwpx_stored_layout())
        && page.side_wrap_empty
    {
        // A saved single object line owns its caption and outer margins.
        // Whitespace on that same line does not create another text line;
        // the original owner and complete height below must prove this.
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
            let [Control::Table(table)] = para.controls.as_slice() else {
                return None;
            };
            let control_index = 0;
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
            let trailing = if crate::renderer::composer::native_tac_next_line_full_spacing(
                para,
                next_para,
                fmt.spacing_after,
                next_spacing_before,
                seg,
                page.profile.hwp5_stored_pagination_layout(),
                dpi,
            ) {
                hwpunit_to_px(seg.line_spacing, dpi)
            } else {
                crate::renderer::composer::tac_host_trailing_spacing(
                    para,
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
            let source_top = hwpunit_to_px(
                para.source_line_seg_vertical_pos
                    .as_ref()
                    .and_then(|positions| positions.first())
                    .copied()
                    .unwrap_or(para.line_segs[0].vertical_pos),
                dpi,
            );
            let flow_origin = if single_saved_line {
                // The saved line origin already owns spacing_before. The
                // current physical flow is its lower bound, not another copy
                // of that leading band.
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
            let saved_origin = page.vpos_col_anchor
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
            // A complete object height authenticates the local line, not the
            // page-relative origin. A single-line shortcut may only reuse a
            // source origin which agrees with the current physical flow: its
            // leading band is either already reserved or still belongs to
            // this paragraph. Other stored ladders use ordinary TAC flow.
            if single_saved_line {
                let unreserved_origin =
                    page.current_height + fmt.spacing_before - shared_spacing_before;
                let same_frame = (saved_origin - page.current_height).abs() <= dpi / 7200.0
                    || (saved_origin - unreserved_origin).abs() <= dpi / 7200.0;
                if !same_frame {
                    return None;
                }
            }
            let origin = flow_origin.max(saved_origin);
            let measured_fits = lines.iter().all(|line| {
                let Some(Control::Table(table)) = para.controls.get(line.control) else {
                    return false;
                };
                // 새로 수용한 공백 줄 캐리어도 각주 예약은 일반 경로가 담당한다.
                if !para.text.is_empty() && table_has_notes(table) {
                    return false;
                }
                measured_tables
                    .iter()
                    .find(|m| m.para_index == para_idx && m.control_index == line.control)
                    .is_some_and(|m| {
                        let caption_extent = table.caption.as_ref().map_or(0.0, |caption| {
                            crate::renderer::composer::caption_height_px(&table.caption, dpi)
                                + hwpunit_to_px(i32::from(caption.spacing), dpi)
                        });
                        (m.total_height
                            - hwpunit_to_px(table.common.height as i32, dpi)
                            - caption_extent)
                            .abs()
                            <= 0.5
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
                let source_origin = (single_saved_line
                    && page.profile.hwp5_stored_pagination_layout())
                .then(|| {
                    let seg = &para.line_segs[0];
                    // Authenticate the shared coordinate axis at the original
                    // object origin. A flow-only trailing gap is not a source
                    // translation and must not shift the following paragraphs.
                    seg.vertical_pos
                        .saturating_sub(crate::renderer::px_to_hwpunit(origin, dpi))
                });
                return Some(StoredTacPlan {
                    lines,
                    origin,
                    source_origin,
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
                advance_end: Some(end),
            },
            end,
        }
    }
}
