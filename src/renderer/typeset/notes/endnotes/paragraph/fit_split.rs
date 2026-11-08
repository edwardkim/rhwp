//! 단 이동 전 예산에서 계산한 미주 줄 컷을 새 단의 실제 예산으로 갱신한다.

use crate::renderer::typeset::FormattedParagraph;

pub(super) fn refresh_after_column_change(
    fmt: &FormattedParagraph,
    previous_cut: Option<usize>,
    remaining: f64,
    whole_advance: f64,
) -> Option<usize> {
    previous_cut?;
    if whole_advance <= remaining {
        return None;
    }
    let mut used = 0.0;
    let mut cut = 0;
    for index in 0..fmt.line_heights.len() {
        let next = used + fmt.line_advance(index);
        if next > remaining {
            break;
        }
        used = next;
        cut = index + 1;
    }
    // 새 단에서도 첫 줄 자체가 너무 크면 빈 조각을 방출하지 않는다.
    // 저장 내용 컷은 이 helper에 전달하지 않으며 별도로 보존한다.
    (cut > 0 && cut < fmt.line_heights.len()).then_some(cut)
}
