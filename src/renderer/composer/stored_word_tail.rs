//! 현재 프레임에서 이어 붙일 수 있는 저장된 마지막 자동 줄바꿈.

use super::{line_breaking, utf16_range_to_text_range, ResolvedStyleSet};
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::layout_frame::LayoutFrame;

/// 저장된 마지막 줄이 앞줄과 같은 한글 어절의 일부인지 확인한다.
/// 명시적 개행·실제 쪽/단 경계·어울림 상자의 경계는 재조판으로 합치지 않는다.
/// 완결된 마지막 어절은 그대로 보존하며, 같은 번호 마커·스타일·가용 폭을 쓰는
/// 프레임이 더 적은 줄로 수용할 때만 낡은 자동 줄 경계를 재검토한다.
pub(super) fn can_rejoin_stored_word_tail(
    para: &Paragraph,
    frame: &LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
    supported_inline_picture: bool,
) -> bool {
    if para.line_segs.len() < 2
        || (!para.controls.is_empty() && !supported_inline_picture)
        || para.text.contains('\n')
        || para.stored_text_partition_is_dirty()
        || frame.models_exclusions()
        || para
            .line_segs
            .iter()
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
    {
        return false;
    }
    let last = para.line_segs.last().unwrap();
    let previous = &para.line_segs[para.line_segs.len() - 2];
    if last.is_first_line_of_page()
        || last.is_first_line_of_column()
        || last.vertical_pos <= previous.vertical_pos
    {
        return false;
    }
    let chars: Vec<_> = para.text.chars().collect();
    let (start, _) = utf16_range_to_text_range(
        &para.char_offsets,
        para.line_seg_text_start(para.line_segs.len() - 1),
        u32::MAX,
        chars.len(),
    );
    let is_syllable = |c: char| ('가'..='힣').contains(&c);
    // 어절 뒤의 닫는 괄호 등 줄 머리 금칙 문자는 같은 토큰의 꼬리다.
    // 새 줄에 내려온 어절 조각을 판정할 때 그 문장부호만 분리해 보되,
    // 실제 프레임 재조판에는 원문 전체를 그대로 전달한다.
    let tail_end = chars[start.min(chars.len())..]
        .iter()
        .rposition(|&ch| !line_breaking::is_line_start_forbidden(ch))
        .map_or(start, |last| start + last + 1);
    if start == 0
        || start >= chars.len()
        || tail_end <= start
        || !is_syllable(chars[start - 1])
        || !chars[start..tail_end].iter().copied().all(is_syllable)
    {
        return false;
    }
    let mut probe = frame.clone();
    let mut input = para.clone();
    input.line_segs.clear();
    line_breaking::layout_paragraph_in_frame(&input, &mut probe, styles, dpi)
        .is_some_and(|rows| rows.len() < para.line_segs.len())
}
