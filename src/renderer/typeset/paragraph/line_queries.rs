//! 구성된 문단의 줄 범위와 텍스트·TAC 참여 여부를 조회한다.
//!
//! 입력은 기존 ComposedParagraph이며 줄을 재구성하거나 높이·페이지 상태를
//! 결정하지 않는다. 문자 범위, 텍스트 분류, 개체 참여는 서로 다른 조회다.

use crate::renderer::composer::ComposedParagraph;

/// 다음 줄의 시작 또는 마지막 줄의 기존 run/개행 길이로 끝 위치를 구한다.
pub(in crate::renderer::typeset) fn composed_line_char_end(
    comp: &ComposedParagraph,
    line_idx: usize,
) -> usize {
    if let Some(next) = comp.lines.get(line_idx + 1) {
        return next.char_start;
    }
    let Some(line) = comp.lines.get(line_idx) else {
        return 0;
    };
    line.char_start
        + line
            .runs
            .iter()
            .map(|run| run.text.chars().count())
            .sum::<usize>()
        + usize::from(line.has_line_break)
}

/// 기존 줄 범위 [start, end)에 TAC 컨트롤 위치가 포함되는지 확인한다.
pub(in crate::renderer::typeset) fn line_has_strict_tac_control(
    comp: &ComposedParagraph,
    line_idx: usize,
) -> bool {
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    let start = line.char_start;
    let end = composed_line_char_end(comp, line_idx);
    end > start
        && comp
            .tac_controls
            .iter()
            .any(|(pos, _, _)| *pos >= start && *pos < end)
}

/// 기존 텍스트 분류를 유지한다. 일반 공백도 true이며 줄 점유 높이 판정이 아니다.
pub(in crate::renderer::typeset) fn line_has_visible_text(
    comp: &ComposedParagraph,
    line_idx: usize,
) -> bool {
    comp.lines
        .get(line_idx)
        .map(|line| {
            line.runs
                .iter()
                .flat_map(|run| run.text.chars())
                .any(|c| c > '\u{001F}' && c != '\u{FFFC}')
        })
        .unwrap_or(false)
}

/// 줄의 문자 범위가 비어 있지 않은지 확인한다. 글자 가시성과는 구별한다.
pub(in crate::renderer::typeset) fn line_has_text_span(
    comp: &ComposedParagraph,
    line_idx: usize,
) -> bool {
    comp.lines
        .get(line_idx)
        .is_some_and(|line| composed_line_char_end(comp, line_idx) > line.char_start)
}
