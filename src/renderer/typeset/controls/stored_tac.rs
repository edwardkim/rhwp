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
