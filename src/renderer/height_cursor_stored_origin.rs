//! 저장 인라인 제목이 새 쪽을 여는 경우의 공통 좌표 원점.
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;

/// 제목 도형의 작은 vpos는 쪽 원점부터의 위치이며 뒤 본문의 기준점이 아니다.
/// 기존 paint의 저장 사다리 판정을 측정과 공유해 같은 좌표계를 사용한다.
pub(crate) fn saved_inline_heading_page(
    paragraphs: &[Paragraph],
    para_index: usize,
    hwp5_stored_layout: bool,
) -> bool {
    hwp5_stored_layout && para_index > 0 && paragraphs.get(para_index).is_some_and(|para| {
        para.line_segs.len() == 1
            && (1..=2500).contains(&para.line_segs[0].vertical_pos)
            && para.controls.len() == 1
            && matches!(&para.controls[0], Control::Shape(shape) if shape.common().treat_as_char)
            && !para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
    })
        && paragraphs
            .get(para_index - 1)
            .and_then(|para| para.line_segs.last())
            .is_some_and(|seg| seg.vertical_pos.saturating_add(seg.line_height) > 60_000)
        && paragraphs
            .get(para_index + 1)
            .and_then(|para| para.line_segs.first())
            .is_some_and(|seg| {
                seg.vertical_pos > paragraphs[para_index].line_segs[0].vertical_pos
                    && seg.vertical_pos < 30_000
            })
}
