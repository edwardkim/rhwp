//! 같은 문단에 매달린 표의 지연 배치 후보를 조회하는 경계.
//! 판별·후보 순서만 소유하며 큐·페이지·각주 상태를 변경하지 않는다.

use super::super::paragraph::metrics::FormattedParagraph;
use super::super::{para_has_visible_text, signed_hwpunit};
use super::tac_flow::TacFlowQuery;
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::pagination::PageItem;

#[derive(Debug, Clone)]
pub(in crate::renderer::typeset) struct DeferredTableControl {
    pub(in crate::renderer::typeset) para_index: usize,
    pub(in crate::renderer::typeset) control_index: usize,
    pub(in crate::renderer::typeset) is_first_placed: bool,
    pub(in crate::renderer::typeset) is_last_placed: bool,
    /// [Task #1860] 이 float 이 속한 문단의 **참 시작 흐름 높이**(선행 형제 control 이
    /// current_height 를 전진시키기 전). 지연 배치 시점의 current_height 는 이미 선행
    /// inline(캡션)을 반영해 out-of-flow float 예산을 과소평가하므로, 분할 예산 기준용
    /// para_start 를 원 배치 시점 값으로 보존한다.
    pub(in crate::renderer::typeset) para_start_height: f64,
}

#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) enum DeferredTableFlushPoint {
    BeforeTableParagraph(usize),
    AfterTableParagraph(usize),
    SectionEnd,
}

impl DeferredTableFlushPoint {
    /// 문단 사이 가시 본문과 원래 문단 순서로 큐 유지 여부만 조회한다.
    pub(in crate::renderer::typeset) fn keeps_pending(
        self,
        deferred: &DeferredTableControl,
        paragraphs: &[Paragraph],
    ) -> bool {
        match self {
            Self::BeforeTableParagraph(idx) => {
                idx <= deferred.para_index
                    || paragraphs[deferred.para_index + 1..idx]
                        .iter()
                        .any(para_has_visible_text)
            }
            Self::AfterTableParagraph(idx) => idx <= deferred.para_index,
            Self::SectionEnd => false,
        }
    }
}

pub(in crate::renderer::typeset) struct CoanchoredTableQuery<'a> {
    para: &'a Paragraph,
    fmt: &'a FormattedParagraph,
    flow: TacFlowQuery<'a>,
}

impl<'a> CoanchoredTableQuery<'a> {
    pub(in crate::renderer::typeset) fn new(
        para: &'a Paragraph,
        fmt: &'a FormattedParagraph,
        flow: TacFlowQuery<'a>,
    ) -> Self {
        Self { para, fmt, flow }
    }

    pub(in crate::renderer::typeset) fn is_deferred_coanchored_rowbreak_table(
        &self,
        table: &crate::model::table::Table,
    ) -> bool {
        let para = self.para;
        let fmt = self.fmt;
        use crate::model::shape::{TextWrap, VertRelTo};

        !para_has_visible_text(para)
            && !self.flow.is_effective_tac_table(para, table, fmt)
            && !table.common.treat_as_char
            && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
            && matches!(table.common.vert_rel_to, VertRelTo::Para)
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && signed_hwpunit(table.common.vertical_offset) > 0
    }

    fn is_coanchored_rowbreak_split_trigger_table(
        &self,
        table: &crate::model::table::Table,
    ) -> bool {
        let para = self.para;
        let fmt = self.fmt;
        use crate::model::shape::{TextWrap, VertRelTo};

        !para_has_visible_text(para)
            && !self.flow.is_effective_tac_table(para, table, fmt)
            && !table.common.treat_as_char
            && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
            && matches!(table.common.vert_rel_to, VertRelTo::Para)
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
    }

    pub(in crate::renderer::typeset) fn should_defer_remaining_coanchored_rowbreak_tables(
        &self,
        table: &crate::model::table::Table,
        page_count: usize,
        current_items: &[PageItem],
        pages_before_block_table: usize,
    ) -> bool {
        if !self.is_coanchored_rowbreak_split_trigger_table(table) {
            return false;
        }
        if page_count <= pages_before_block_table {
            return false;
        }
        matches!(
            current_items.last(),
            Some(PageItem::PartialTable {
                is_continuation: true,
                ..
            })
        )
    }

    /// 원래 순서의 남은 표와 최초 문단 시작 높이를 보존한다. 큐에는 아직 넣지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn remaining_controls(
        &self,
        para_idx: usize,
        ctrl_order: &[usize],
        order_pos: usize,
        first_placed_table: Option<usize>,
        last_placed_table: Option<usize>,
        para_start_height: f64,
    ) -> Vec<DeferredTableControl> {
        let para = self.para;
        ctrl_order
            .iter()
            .skip(order_pos + 1)
            .copied()
            .filter_map(|next_ctrl_idx| {
                let Control::Table(next_table) = para.controls.get(next_ctrl_idx)? else {
                    return None;
                };
                self.is_deferred_coanchored_rowbreak_table(next_table)
                    .then(|| {
                        DeferredTableControl {
                            para_index: para_idx,
                            control_index: next_ctrl_idx,
                            is_first_placed: first_placed_table == Some(next_ctrl_idx),
                            is_last_placed: last_placed_table == Some(next_ctrl_idx),
                            // [Task #1860] 원 배치 시점의 참 para_start.
                            para_start_height,
                        }
                    })
            })
            .collect()
    }
}
