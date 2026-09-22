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
        let table_line_h = hwpunit_to_px(table.common.height as i32, self.dpi) + om_top + om_bot;

        // [#2287 후속/1.hwpx p58] text_height(th) 매칭 우선 — 한컴은 문단의
        // 모든 줄에 최대 줄높이를 lh 로 저장하는 관례가 있어(1.hwpx pi=322:
        // 텍스트 줄 ls[0] lh=69085/th=1300, 표 줄 ls[1] lh=th=69085), lh 만으로
        // 는 텍스트 줄이 먼저 오매칭되어 917px TAC 표의 소비가 17.3px 로
        // 붕괴(fmt.line_heights[0] 채택)했다. th 가 표 높이와 일치하는 줄이
        // 있으면 그 줄이 표 줄의 확정 증거이고, 없으면 종전 lh 매칭 유지.
        let th_match = para.line_segs.iter().enumerate().find_map(|(idx, seg)| {
            let th = hwpunit_to_px(seg.text_height, self.dpi);
            ((th - table_line_h).abs() < 1.0).then_some(idx)
        });
        if th_match.is_some() {
            return th_match;
        }

        para.line_segs.iter().enumerate().find_map(|(idx, seg)| {
            let line_h = hwpunit_to_px(seg.line_height, self.dpi);
            if (line_h - table_line_h).abs() < 1.0 {
                Some(idx)
            } else {
                None
            }
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
