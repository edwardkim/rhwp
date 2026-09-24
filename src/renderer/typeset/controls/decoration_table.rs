//! 장식 표의 현재 쪽 컷과 다음 쪽 예약량 조회.
//! Shape 발행·표 포맷·대기열 반영·host 텍스트 배치는 수행하지 않는다.
use super::super::FormattedTable;
use crate::model::{paragraph::Paragraph, table::Table};
use crate::renderer::hwpunit_to_px;

pub(in crate::renderer::typeset) struct OverlayContinuation {
    pub first_unfit: usize,
    pub remaining_px: f64,
    pub reserve_px: f64,
    pub room: f64,
}

pub(super) fn continuation(
    para: &Paragraph,
    table: &Table,
    next_para: Option<&Paragraph>,
    ft: &FormattedTable,
    current_height: f64,
    dpi: f64,
    base_available_height: impl FnOnce() -> f64,
) -> Option<OverlayContinuation> {
    let anchor_y = current_height + hwpunit_to_px(table.common.vertical_offset as i32, dpi);
    let room = base_available_height() - anchor_y;
    if room > 0.0 && ft.effective_height > room {
        // `cumulative_heights` 는 접두합(len = 행 수 + 1)이다 —
        // `cum[i]` 는 행 0..i 의 합이므로, 처음으로 room 을 넘는
        // 인덱스 i 는 "행 i-1 이 안 들어간다"는 뜻이다.
        let first_unfit = ft
            .cumulative_heights
            .iter()
            .position(|cum| *cum > room)
            .map(|i| i.saturating_sub(1))
            .unwrap_or(ft.row_heights.len());
        if first_unfit > 0 && first_unfit < ft.row_heights.len() {
            let remaining_px = (ft.effective_height
                - ft.cumulative_heights
                    .get(first_unfit)
                    .copied()
                    .unwrap_or(0.0))
            .max(0.0);
            // [#5792] 잔여 행이 놓일 자리를 뒤따르는 흐름이 스스로
            // 만드는가? #4514 형상은 앵커 뒤 빈 필러 문단들이 표
            // 높이만큼 흐름을 만들므로(저장 사다리가 앵커 → 필러로
            // 연속 전진) 다음 쪽에 잔여 높이를 다시 예약하면 이중
            // 계상이다. 반대로 뒤 문단의 저장 vpos 가 앵커보다
            // **되감기면**(쪽 리셋) 그 문단은 새 쪽 상단에서 다시
            // 시작하는 좌표라 잔여 행의 자리가 어디에도 없다. 그때
            // 예약하지 않으면 다음 쪽 본문이 잔여 행 위에 겹쳐
            // 그려지고(2700727 3쪽 'Ⅱ. 곤충이용'·'1. 설치기준'),
            // 그 본문 표가 잔여 행의 페인트 상한을 깎아 행이 통째로
            // 사라진다(42행 중 17행 소실).
            let ladder_resets_after_anchor = next_para
                .and_then(|np| np.line_segs.first().map(|seg| seg.vertical_pos))
                .zip(para.line_segs.first().map(|seg| seg.vertical_pos))
                .is_some_and(|(next_vpos, anchor_vpos)| next_vpos < anchor_vpos);
            let reserve_px = if ladder_resets_after_anchor {
                remaining_px
            } else {
                0.0
            };
            return Some(OverlayContinuation {
                first_unfit,
                remaining_px,
                reserve_px,
                room,
            });
        }
    }
    None
}
