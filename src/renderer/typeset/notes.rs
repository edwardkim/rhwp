//! 표 배치 뒤 각주 연결 조정.
//! fragment 큐가 소유하지 않은 직접 셀 각주를 원래 순서로 즉시 등록한다.
//! 각주 높이·쪽 배분과 큐 예약 알고리즘은 아직 상위 엔진에 남아 있다.

use super::TypesetState;
use crate::model::control::Control;
use crate::model::footnote::Footnote;
use crate::model::table::Table;
use crate::renderer::pagination::FootnoteSource;

pub(super) fn register_unqueued_table_cells(
    st: &mut TypesetState,
    table: &Table,
    para_idx: usize,
    ctrl_idx: usize,
    mut register: impl FnMut(&mut TypesetState, &Footnote, FootnoteSource),
) {
    // 진입 때 한 번만 판단한다. 개별 등록이 쪽 상태를 바꾸어도 재판단하지 않는다.
    if !st.has_fragment_queued_table_footnotes(para_idx, ctrl_idx) {
        for (cell_idx, cell) in table.cells.iter().enumerate() {
            for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                for (cc_idx, cc) in cp.controls.iter().enumerate() {
                    if let Control::Footnote(fn_ctrl) = cc {
                        register(
                            st,
                            fn_ctrl,
                            FootnoteSource::TableCell {
                                para_index: para_idx,
                                table_control_index: ctrl_idx,
                                cell_index: cell_idx,
                                cell_para_index: cp_idx,
                                cell_control_index: cc_idx,
                            },
                        );
                    }
                }
            }
        }
    }
}
