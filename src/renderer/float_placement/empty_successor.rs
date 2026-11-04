//! 저장 빈 문단의 전체 전진과 실제 줄 상자 뒤 간격을 함께 생산한다.

use super::{hwpunit_to_px, Control, Paragraph, VertAlign, VertRelTo};

pub(crate) struct StoredEmptySuccessorGeometry {
    pub(crate) advance: f64,
    pub(crate) trailing: f64,
}

/// 기존 측정·배치는 같은 공통 결과의 전체 전진을 소비한다.
pub(crate) fn stored_empty_successor_advance(
    paragraphs: &[Paragraph],
    index: usize,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    dpi: f64,
) -> Option<f64> {
    stored_empty_successor_geometry(paragraphs, index, styles, dpi).map(|geometry| geometry.advance)
}

/// The next saved origin includes the empty line's actual advance and bands.
/// A coincident empty origin is accepted only when the following text stream
/// independently continues from that same origin; local zero resets do not.
pub(crate) fn stored_empty_successor_geometry(
    paragraphs: &[Paragraph],
    index: usize,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    dpi: f64,
) -> Option<StoredEmptySuccessorGeometry> {
    use crate::model::paragraph::{ColumnBreakType, LineSeg};
    let current = paragraphs.get(index)?;
    let next = paragraphs.get(index + 1)?;
    let [line] = current.line_segs.as_slice() else {
        return None;
    };
    let next_line = next.line_segs.first()?;
    let natural = |p: &Paragraph| {
        !p.stored_text_partition_is_dirty()
            && !p.cell_format_vpos_dirty
            && p.column_type == ColumnBreakType::None
            && !p.line_segs.is_empty()
            && p.line_segs.iter().all(|s| {
                s.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    && s.vertical_pos >= 0
                    && s.line_height > 0
                    && s.line_spacing >= 0
            })
    };
    if !natural(current)
        || !natural(next)
        || !current.controls.is_empty()
        || !current.text.trim().is_empty()
    {
        return None;
    }
    if !next.controls.is_empty()
        && !matches!(next.controls.as_slice(), [Control::Table(t)]
        if !t.common.treat_as_char && t.common.vert_rel_to == VertRelTo::Para
            && t.common.vert_align == VertAlign::Top && t.common.vertical_offset == 0
            && next.text.trim().is_empty())
    {
        return None;
    }
    let delta = next_line.vertical_pos.checked_sub(line.vertical_pos)?;
    if delta < 0 {
        return None;
    }
    // Repeated local zero origins do not certify a shared cell coordinate
    // frame. In particular, a later positive origin cannot be turned into
    // an enormous empty-line advance after such a coincident restart.
    // A positive-to-zero page restart and the independently authenticated
    // coincident empty/text slot below retain their separate contracts.
    if delta > 0 && line.vertical_pos == 0 {
        if let Some(previous) = index.checked_sub(1).and_then(|i| paragraphs.get(i)) {
            if previous.line_segs.last().is_some_and(|last| {
                last.vertical_pos == 0 && last.line_height > 0 && last.line_spacing >= 0
            }) {
                return None;
            }
        }
    }
    if delta == 0 {
        let last = next.line_segs.last()?;
        let following = paragraphs.get(index + 2)?;
        let following_line = following.line_segs.first()?;
        if next.text.trim().is_empty()
            || !next.controls.is_empty()
            || !natural(following)
            || !following.controls.is_empty()
            || following_line.vertical_pos
                != last
                    .vertical_pos
                    .checked_add(last.line_height)?
                    .checked_add(last.line_spacing)?
        {
            return None;
        }
    }
    let before = styles
        .para_styles
        .get(next.para_shape_id as usize)?
        .spacing_before;
    let advance = hwpunit_to_px(delta, dpi) - before;
    // 저장 차이는 현재 줄의 상단부터 시작하지만 소비자는 문단 앞 간격을
    // 적용하기 전 커서를 넘긴다. 뒤 문단의 앞 간격은 그 배치에서 소비하므로
    // 제외하고, 현재 문단의 흐름 앞 간격은 측정과 같이 한 번 포함한다.
    // 첫 문단의 저장 앞 띠는 흐름 간격이 아니므로 다시 더하지 않는다.
    let current_before = crate::renderer::cell_paragraph_spacing_before(
        current,
        index,
        styles
            .para_styles
            .get(current.para_shape_id as usize)
            .map_or(0.0, |style| style.spacing_before),
    );
    let after = styles
        .para_styles
        .get(current.para_shape_id as usize)
        .map_or(0.0, |style| style.spacing_after);
    // 원본 전진이 실제 한 글줄과 명시적인 간격으로 설명될 때만
    // 뒤 간격을 분리한다. 동시 원점·별도 예약 띠를 잉크 높이로 추정하지 않는다.
    let plain_advance = line
        .line_height
        .checked_add(line.line_spacing)
        .and_then(|height| height.checked_add(crate::renderer::px_to_hwpunit(after + before, dpi)));
    let trailing = if plain_advance == Some(delta) {
        (hwpunit_to_px(line.line_spacing, dpi) + after).max(0.0)
    } else {
        0.0
    };
    (advance >= 0.0).then_some(StoredEmptySuccessorGeometry {
        advance: current_before + advance,
        trailing,
    })
}
