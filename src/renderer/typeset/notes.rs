//! 문단/표에서 참조한 각주와 구역/문서 끝 미주의 처리 경계.
//! footnotes는 내용 측정·소유 쪽 선택, reservation은 쪽 예산 반영,
//! endnotes는 참조 순서·준비·측정·수용 Query·배치를 담당한다.
//! 표 fragment 큐의 수명은 table/continuation이 계속 소유한다.

pub(super) mod endnotes;
pub(super) mod footnotes;
mod reservation;

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
