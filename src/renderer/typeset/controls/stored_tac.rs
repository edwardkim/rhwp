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
}

pub(in crate::renderer::typeset) struct StoredTacControlPlacement {
    pub control_index: usize,
    pub inline: InlineBoxPlacement,
    pub end: f64,
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
    let trailing_fraction = if full_trailing_spacing { 1.0 } else { 0.5 };
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
pub(super) fn prepare(
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    measured_tables: &[MeasuredTable],
    page: StoredTacPage,
    available_height: impl Fn() -> f64,
    dpi: f64,
) -> Option<StoredTacPlan> {
    // 완전한 저장 줄 계약은 표별 높이를 합산하지 않고, 같은 pen/end를 배치에도 전달한다.
    if !page.profile.session_edited()
        && (page.profile.hwp5_stored_pagination_layout() || page.profile.hwpx_stored_layout())
        && page.side_wrap_empty
    {
        if let Some(lines) = stored_tac_lines(para) {
            let flow_origin = page.current_height
                + if page.current_height < 1.0 {
                    0.0
                } else {
                    fmt.spacing_before
                };
            // 앞 문단의 저장 사다리가 누적 높이보다 앞서 있으면 그 앵커를
            // fit와 paint에 함께 보존한다. 문단 상대 top만 더하면 앞 표로 되감긴다.
            let saved_origin = page.vpos_col_anchor
                + hwpunit_to_px(
                    para.line_segs[0]
                        .vertical_pos
                        .saturating_sub(page.vpos_page_base.or(page.vpos_lazy_base).unwrap_or(0)),
                    dpi,
                );
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
                        (m.total_height - hwpunit_to_px(table.common.height as i32, dpi)).abs()
                            <= 0.5
                    })
            });
            let fits = lines.iter().all(|line| {
                origin + hwpunit_to_px(line.occupied_end.max(line.end), dpi) + fmt.spacing_after
                    <= available_height()
            });
            if measured_fits && fits {
                return Some(StoredTacPlan { lines, origin });
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
