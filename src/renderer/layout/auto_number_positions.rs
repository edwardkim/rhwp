//! 자동 번호 자리표의 원문 위치와 인라인 개체를 포함한 논리 위치를 함께 발행한다.

use super::LayoutEngine;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;

pub(crate) struct AutoNumberPlaceholderPosition {
    pub source: usize,
    pub logical: usize,
}

impl LayoutEngine {
    pub(crate) fn resolve_auto_number_placeholder_addresses(
        para: &Paragraph,
        an_type: crate::model::control::AutoNumberType,
    ) -> Vec<AutoNumberPlaceholderPosition> {
        let ctrl_positions = para.control_text_positions();
        let logical_positions = para.logical_control_positions();
        let text_chars: Vec<char> = para.text.chars().collect();
        let mut positions = Vec::new();
        let mut search_from = 0usize;

        // [#6986] 종류가 다른 `AutoNumber` 도 **자리를 소비한다.**
        //
        // 종전에는 다른 종류를 `continue` 로 건너뛰면서 `search_from` 을 전진시키지
        // 않았다. 그래서 한 문단에 `PAGE` 와 `TOTAL_PAGE` 가 같이 있으면, 두 번째
        // 종류의 폴백 탐색이 0 부터 시작해 **첫 번째 컨트롤의 자리**를 집었다.
        //
        // 법령 HWPX 의 꼬리말 표 셀이 그 형상이다 —
        // `<hp:t>- </hp:t><PAGE/><hp:t> / </hp:t><TOTAL_PAGE/><hp:t> -</hp:t>`.
        // 두 치환이 같은 자리를 쓰면 나중 것이 앞 것을 덮어, 한쪽은 총쪽수가 찍히고
        // 다른 쪽은 빈칸이 된다(`- / 187 -`). v0.8.3 에서 `TOTAL_PAGE` 치환이
        // 들어오면서 생긴 회귀다(`e69a2d286`).
        //
        // 그래서 **모든** `AutoNumber` 를 순서대로 돌며 자리를 하나씩 소비하고,
        // 그중 요청한 종류의 것만 돌려준다.
        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            let Control::AutoNumber(an) = ctrl else {
                continue;
            };
            let wanted = an.number_type == an_type;

            // 문단 끝 자동 번호는 자리표 뒤, 곧 글자 수 자리에 놓인다. `control_text_positions`
            // 는 뒤 글자가 있어야 자리표를 알아보기 때문이다. 마지막 글자가 번호 몫 8유닛을
            // 차지하면 그 글자가 자리표다. 놓치면 아래 폴백이 앞의 탭(역시 8유닛)을 집는다.
            let ctrl_pos = ctrl_positions.get(ctrl_idx).map(|&pos| {
                let last = pos.saturating_sub(1);
                if pos == text_chars.len()
                    && Self::is_auto_number_placeholder_at(para, &text_chars, last)
                {
                    last
                } else {
                    pos
                }
            });
            let direct_pos = ctrl_pos.filter(|&pos| {
                Self::is_auto_number_placeholder_at(para, &text_chars, pos)
                    || text_chars
                        .get(pos)
                        .map_or(false, |ch| Self::is_auto_number_placeholder_char(*ch))
            });

            let pos = direct_pos.or_else(|| {
                Self::find_auto_number_placeholder_char(para, &text_chars, search_from)
            });

            if let Some(pos) = pos {
                if wanted {
                    // 문단 끝 자리표의 원문 정규화도 보존하며, 제어 순서에서 정한
                    // 논리 칸 차이만 함께 전달한다. 측정은 source, 편집 트리는 logical이다.
                    let inline_slots = logical_positions[ctrl_idx] - ctrl_positions[ctrl_idx];
                    positions.push(AutoNumberPlaceholderPosition {
                        source: pos,
                        logical: pos + inline_slots,
                    });
                }
                search_from = pos.saturating_add(1);
            }
        }

        positions
    }
}
