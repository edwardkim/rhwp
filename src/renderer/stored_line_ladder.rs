//! 일정한 저장 줄 사다리를 글꼴·문단 객체의 실제 점유와 대조한다.

use super::style_resolver::ResolvedStyleSet;
use super::{corrected_line_metrics, hwpunit_to_px, DEFAULT_DPI};
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{TextWrap, VertAlign, VertRelTo};
use crate::model::style::LineSpacingType;
use crate::model::table::TablePageBreak;

/// 실저장 줄이 다음 줄과 겹치면, 두 줄의 시작 차이가 앞 줄의 흐름 전진이다.
/// 소유·빈 줄 여부는 호출자가 확인하고 측정과 배치가 같은 사다리를 소비한다.
pub(crate) fn stored_overlap_advance_hu(
    para: &Paragraph,
    line_idx: usize,
    composed_lines: usize,
    runs_all_whitespace: bool,
) -> Option<i64> {
    if !runs_all_whitespace || composed_lines != para.line_segs.len() {
        return None;
    }
    let current = para.line_segs.get(line_idx)?;
    let next = para.line_segs.get(line_idx + 1)?;
    let authored = |seg: &crate::model::paragraph::LineSeg| {
        seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
    };
    let current_start = i64::from(current.vertical_pos);
    let next_start = i64::from(next.vertical_pos);
    let end = current_start + i64::from(current.line_height) + i64::from(current.line_spacing);
    (authored(current) && authored(next) && next_start > current_start && next_start < end)
        .then_some(next_start - current_start)
}

/// 빈 문단의 자리차지 표는 객체와 양쪽 바깥여백의 점유 띠를 소유한다.
/// 다른 앵커·글줄 참여·오프셋·캡션은 별도의 위치 계약으로 처리한다.
fn unaccounted_topbottom_band_hu(para: &Paragraph) -> Option<i64> {
    let [Control::Table(table)] = para.controls.as_slice() else {
        return None;
    };
    let common = &table.common;
    if para
        .text
        .chars()
        .any(|ch| !ch.is_whitespace() && ch > '\u{001f}' && ch != '\u{fffc}')
        || common.treat_as_char
        || common.text_wrap != TextWrap::TopAndBottom
        || common.vert_rel_to != VertRelTo::Para
        || common.vert_align != VertAlign::Top
        || common.vertical_offset != 0
        || common.height == 0
        || common.height > i32::MAX as u32
        || table.caption.is_some()
        || table.layout_page_break() != TablePageBreak::None
    {
        return None;
    }
    Some(
        i64::from(common.height)
            + i64::from(table.outer_margin_top)
            + i64::from(table.outer_margin_bottom),
    )
}

pub(crate) fn stored_line_ladder_is_uniform_filler(
    paragraphs: &[crate::model::paragraph::Paragraph],
    styles: &ResolvedStyleSet,
) -> bool {
    /// 사다리 하나로 단정하기 위한 최소 문단 수.
    const MIN_PARAGRAPHS: usize = 8;
    /// 저장 advance 와 글꼴 advance 가 어긋나야 하는 최소 비율의 역수 (1/4).
    const CONTRADICTION_RATIO_DIVISOR: usize = 4;

    if paragraphs.len() < MIN_PARAGRAPHS {
        return false;
    }

    let mut tuple: Option<(i32, i32, i32, i32)> = None;
    let mut prev_vpos: Option<i32> = None;
    let mut contradicting = 0usize;

    for (index, para) in paragraphs.iter().enumerate() {
        let [seg] = para.line_segs.as_slice() else {
            return false;
        };
        if seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || seg.line_height <= 0
            || seg.text_height <= 0
        {
            return false;
        }
        let current = (
            seg.line_height,
            seg.text_height,
            seg.baseline_distance,
            seg.line_spacing,
        );
        match tuple {
            None => tuple = Some(current),
            Some(first) if first != current => return false,
            _ => {}
        }
        // 사다리 걸음은 그 튜플의 advance 와 같아야 한다 — 실측 조판이면 문단마다
        // 다른 값이 나온다.
        let step = seg.line_height.saturating_add(seg.line_spacing);
        if let Some(previous) = prev_vpos {
            if seg.vertical_pos.saturating_sub(previous) != step {
                return false;
            }
        }
        prev_vpos = Some(seg.vertical_pos);

        let max_fs = para
            .char_shapes
            .iter()
            .filter_map(|shape| styles.char_styles.get(shape.char_shape_id as usize))
            .map(|style| style.font_size)
            .fold(0.0f64, f64::max);
        let (ls_type, ls_val) = styles
            .para_styles
            .get(para.para_shape_id as usize)
            .map(|style| (style.line_spacing_type, style.line_spacing))
            .unwrap_or((LineSpacingType::Percent, 160.0));
        let (font_lh, font_ls) = corrected_line_metrics(0.0, 0.0, max_fs, ls_type, ls_val);
        let stored_advance = hwpunit_to_px(step, DEFAULT_DPI);
        let font_contradiction = max_fs > 0.0 && (font_lh + font_ls - stored_advance).abs() > 0.5;
        // 같은 걸음의 다음 저장 줄이 있어야 객체 점유와 대조할 수 있다.
        // 쪽/종이 앵커·TAC·오프셋 객체는 이 문단의 줄 전진을 뜻하지 않는다.
        let object_contradiction = index + 1 < paragraphs.len()
            && unaccounted_topbottom_band_hu(para).is_some_and(|height| height > i64::from(step));
        if font_contradiction || object_contradiction {
            contradicting += 1;
        }
    }

    contradicting * CONTRADICTION_RATIO_DIVISOR >= paragraphs.len()
}

/// [#8474] 단 맨 위 문단의 저장 첫 줄이 증언하는 쪽 위 간격(px).
///
/// 한/글은 문단 위 간격을 쪽 맨 위에서도 적용한다 — 구역 첫 쪽·쪽 나누기·자연 쪽 넘김
/// 모두 첫 줄 vpos 가 위 간격과 같다(합성 실측, `samples/issue8474/`). 저장 사다리를 믿는
/// 경로에서 첫 줄이 원본 줄이고 그 vpos 가 0 보다 크고 위 간격 이하이면 그 vpos 를 쪽 위
/// 간격으로 쓴다. 그 밖(합성 줄·편집·구역 누적 좌표)은 0 이다. 글자처럼 취급 표 host 의
/// 단일 저장 줄 TAC 계획(`physical_tac`)이 이 값을 원점에 두고 paint 가 그 원점을 그대로 쓴다.
pub(crate) fn stored_column_top_spacing(
    para: &Paragraph,
    spacing_before: f64,
    stored_profile: bool,
    dpi: f64,
) -> f64 {
    if !stored_profile
        || spacing_before <= 0.0
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
    {
        return 0.0;
    }
    let Some(first) = para.line_segs.first() else {
        return 0.0;
    };
    if first.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 {
        return 0.0;
    }
    let vpos = para
        .source_line_seg_vertical_pos
        .as_ref()
        .and_then(|positions| positions.first())
        .copied()
        .unwrap_or(first.vertical_pos);
    let saved_top = hwpunit_to_px(vpos, dpi);
    if saved_top > 0.0 && saved_top <= spacing_before + 0.5 {
        saved_top
    } else {
        0.0
    }
}

/// 원본 FIRST..LAST가 같은 물리 행의 비중첩 수평 조각을 완결하는지 확인한다.
/// bit31은 구현 편의 속성이며, 이 소유 관계 자체를 별도 줄로 바꾸지 않는다.
/// 같은 높이의 개별 완결 행·쪽/단 시작·내용 없는 합성 자리표는 이 경계가 아니다.
pub(crate) fn has_declared_fragment_boundary(para: &Paragraph, index: usize) -> bool {
    let rows = &para.line_segs;
    let Some((previous, current)) = index
        .checked_sub(1)
        .and_then(|before| rows.get(before).zip(rows.get(index)))
    else {
        return false;
    };
    if previous.vertical_pos != current.vertical_pos
        || previous.column_start == current.column_start
    {
        return false;
    }
    let top = current.vertical_pos;
    let mut start = index - 1;
    while start > 0 && rows[start - 1].vertical_pos == top {
        start -= 1;
    }
    let mut end = index + 1;
    while end < rows.len() && rows[end].vertical_pos == top {
        end += 1;
    }
    let row = &rows[start..end];
    row.iter().enumerate().all(|(offset, segment)| {
        segment.line_height > 0
            && segment.segment_width > 0
            && segment.is_first_segment() == (offset == 0)
            && segment.is_last_segment() == (offset + 1 == row.len())
            && (offset == 0
                || (!segment.is_first_line_of_page() && !segment.is_first_line_of_column()))
    }) && row.windows(2).all(|pair| {
        pair[0]
            .column_start
            .checked_add(pair[0].segment_width)
            .is_some_and(|right| right <= pair[1].column_start)
    })
}
