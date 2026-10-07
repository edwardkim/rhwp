//! TAC 흐름 참여와 저장 줄 소속을 조회하는 읽기 전용 경계.
//! 기존 판별을 공유하며 표 배치·페이지 상태 쓰기는 수행하지 않는다.

use super::super::paragraph::metrics::FormattedParagraph;
use crate::model::{paragraph::Paragraph, provenance::LayoutCompatibilityProfile};
use crate::renderer::hwpunit_to_px;
use std::cell::Cell;

pub(in crate::renderer::typeset) struct TacFlowQuery<'a> {
    dpi: f64,
    profile: &'a Cell<LayoutCompatibilityProfile>,
}

impl<'a> TacFlowQuery<'a> {
    pub(in crate::renderer::typeset) fn new(
        dpi: f64,
        profile: &'a Cell<LayoutCompatibilityProfile>,
    ) -> Self {
        Self { dpi, profile }
    }

    pub(super) fn dpi(&self) -> f64 {
        self.dpi
    }

    // 생성 시 profile을 미리 읽지 않고 원래 조건의 평가 지점에서 조회한다.
    pub(super) fn session_edited(&self) -> bool {
        self.profile.get().session_edited()
    }

    /// 저장 셀 글줄이 없는 텍스트 표는 현재 줄 구성으로 높이를 정한다.
    /// host의 옛 저장 줄이 그 새 높이를 제한하는 근거가 될 수는 없다.
    pub(in crate::renderer::typeset) fn single_tac_line_has_unstored_cell_text(
        &self,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
        tac_count: usize,
    ) -> bool {
        if tac_count != 1
            || fmt.line_heights.len() != 1
            || super::super::para_has_non_whitespace_text(para)
        {
            return false;
        }
        table
            .cells
            .iter()
            .flat_map(|cell| &cell.paragraphs)
            .any(super::super::para_has_non_whitespace_text)
            && table
                .cells
                .iter()
                .flat_map(|cell| &cell.paragraphs)
                .all(crate::renderer::para_has_no_stored_line_segs)
    }

    pub(in crate::renderer::typeset) fn tac_table_line_index(
        &self,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
    ) -> Option<usize> {
        if !table.common.treat_as_char || fmt.line_heights.len() <= 1 {
            return None;
        }

        let om_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
        let om_bot = hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi);
        let table_body_h = hwpunit_to_px(table.common.height as i32, self.dpi);
        let table_line_h = table_body_h + om_top + om_bot;
        // 재구성한 개체 줄은 본체 높이를 갖고 바깥 여백은 배치에서 소비한다.
        // 실제 저장 줄의 소유 판정에는 기존 외곽 높이 계약을 유지한다.
        let matches_height = |height: f64| {
            (height - table_line_h).abs() < 1.0
                || (crate::renderer::para_has_no_stored_line_segs(para)
                    && (height - table_body_h).abs() < 1.0)
        };

        // [#2287 후속/1.hwpx p58] text_height(th) 매칭 우선 — 한컴은 문단의
        // 모든 줄에 최대 줄높이를 lh 로 저장하는 관례가 있어(1.hwpx pi=322:
        // 텍스트 줄 ls[0] lh=69085/th=1300, 표 줄 ls[1] lh=th=69085), lh 만으로
        // 는 텍스트 줄이 먼저 오매칭되어 917px TAC 표의 소비가 17.3px 로
        // 붕괴(fmt.line_heights[0] 채택)했다. th 가 표 높이와 일치하는 줄이
        // 있으면 그 줄이 표 줄의 확정 증거이고, 없으면 종전 lh 매칭 유지.
        let th_match = para.line_segs.iter().enumerate().find_map(|(idx, seg)| {
            let th = hwpunit_to_px(seg.text_height, self.dpi);
            matches_height(th).then_some(idx)
        });
        if th_match.is_some() {
            return th_match;
        }

        para.line_segs
            .iter()
            .enumerate()
            .find_map(|(idx, seg)| {
                let line_h = hwpunit_to_px(seg.line_height, self.dpi);
                if matches_height(line_h) {
                    Some(idx)
                } else {
                    None
                }
            })
            .or_else(|| {
                para.line_segs
                    .is_empty()
                    .then(|| {
                        fmt.line_heights.iter().position(|height| {
                            (height - table_line_h).abs() < 1.0
                                || (height - table_body_h).abs() < 1.0
                        })
                    })
                    .flatten()
            })
    }

    pub(in crate::renderer::typeset) fn is_effective_tac_table(
        &self,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
    ) -> bool {
        self.uses_tac_table_flow(table) || self.tac_table_line_index(para, table, fmt) == Some(0)
    }

    /// HWPX 계보 HWP는 HWP5 CTRL_HEADER를 다시 읽으면서 `table.attr` bit 0을
    /// `treatAsChar`로 채운다. 하지만 HWPX의 inline 의미는 `treatAsChar`와
    /// `flowWithText`가 모두 참일 때만 성립한다. 후자가 거짓인 표를 TAC으로
    /// 오인하면 큰 표가 통째로 배치되어 저장 직후 쪽 경계가 압축된다 (#3930).
    pub(in crate::renderer::typeset) fn uses_tac_table_flow(
        &self,
        table: &crate::model::table::Table,
    ) -> bool {
        if self.profile.get().hwpx_stored_layout() {
            table.common.treat_as_char && table.common.flow_with_text
        } else {
            table.attr & 0x01 != 0
        }
    }
}
