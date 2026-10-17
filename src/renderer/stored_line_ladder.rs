//! 일정한 저장 줄 사다리를 글꼴·문단 객체의 실제 점유와 대조한다.

use super::style_resolver::ResolvedStyleSet;
use super::{corrected_line_metrics, hwpunit_to_px, DEFAULT_DPI};
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{TextWrap, VertAlign, VertRelTo};
use crate::model::style::LineSpacingType;
use crate::model::table::TablePageBreak;

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
        || table.page_break != TablePageBreak::None
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
