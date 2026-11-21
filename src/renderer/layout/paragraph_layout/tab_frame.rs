use crate::model::paragraph::{tab_ext_is_placeholder, LineSeg, Paragraph};
use crate::model::style::Alignment;
use crate::renderer::composer::{ComposedLine, ComposedParagraph};
use crate::renderer::TabStop;

/// 저장 왼쪽 탭은 고정 전진이다. 줄 중간의 마지막 탭까지 자연 폭으로
/// 소비하고, 뒤 내용만 같은 양쪽 정렬 예산을 나누어 쓴다.
pub(super) fn saved_left_tab_count(
    paragraph: &Paragraph,
    composed: &ComposedParagraph,
    line: &ComposedLine,
    line_index: usize,
    first_tab: usize,
) -> Option<usize> {
    let stored = paragraph.line_segs.get(line_index)?;
    if paragraph.stored_text_partition_is_dirty()
        || paragraph.cell_format_vpos_dirty
        || paragraph.line_segs.len() != composed.lines.len()
        || stored.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
    {
        return None;
    }
    let characters = || line.runs.iter().flat_map(|run| run.text.chars());
    let (last, _) = characters()
        .enumerate()
        .filter(|(_, ch)| *ch == '\t')
        .last()?;
    if !characters().take(last).any(|ch| !ch.is_whitespace()) {
        return None;
    }
    let count = characters().filter(|&ch| ch == '\t').count();
    let extensions = composed
        .tab_extended
        .get(first_tab..first_tab.checked_add(count)?)?;
    extensions
        .iter()
        .all(|extension| (extension[0] != 0 || extension[1] != 0) && extension[2] == 0x100)
        .then_some(count)
}

/// 재계산이 필요한 저장 캐시와 유효한 인라인 전진폭을 구분한다.
pub(super) fn geometry_inline_tabs(
    extensions: &[[u16; 7]],
    first: usize,
    stored_geometry_valid: bool,
) -> Vec<[u16; 7]> {
    if stored_geometry_valid {
        extensions.iter().skip(first).copied().collect()
    } else {
        Vec::new()
    }
}

/// 왼쪽/기본 정지로 자연 줄을 구성한 뒤에는 줄 전체를 정렬 이동한다.
/// 단에 맞추는 오른쪽/가운데 정지는 해당 단의 좌표 계약을 유지한다.
pub(super) fn tab_frame_translates_with_line(
    alignment: Alignment,
    auto_tab_right: bool,
    stops: &[TabStop],
    extensions: &[[u16; 7]],
) -> bool {
    matches!(alignment, Alignment::Center | Alignment::Right)
        && !auto_tab_right
        && !stops.iter().any(|stop| matches!(stop.tab_type, 1 | 2))
        && !extensions.iter().any(|extension| {
            !tab_ext_is_placeholder(extension)
                && (matches!(extension[2] >> 8, 2 | 3) || matches!(extension[2], 1 | 2))
        })
}
