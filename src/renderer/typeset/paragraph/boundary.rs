//! 저장 줄과 컨트롤 의미로 문단 내부의 물리 쪽 경계 후보를 조회한다.
//!
//! 이 모듈은 원본 IR을 읽기만 한다. 후보 간 우선순위와 각주 조회 시점은
//! paragraph 조정자가, 실제 줄/각주 분할과 페이지 전환은 기존 배치 경로가 소유한다.

use super::super::{
    is_synthetic_line_seg, para_has_visible_text, para_is_treat_as_char_picture_only,
};
use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, Paragraph},
};
use crate::renderer::hwpunit_to_px;

/// Field와 hyperlink는 글자 위치의 인라인 metadata다. 저장 vpos reset의 본문 흐름을
/// 해석할 때 표·그림·각주처럼 줄이나 쪽을 점유하는 control과 구별한다.
fn controls_are_inline_text_metadata(para: &Paragraph) -> bool {
    para.controls
        .iter()
        .all(|control| matches!(control, Control::Field(_) | Control::Hyperlink(_)))
}

pub(super) fn internal_vpos_page_break_line(
    para: &Paragraph,
    line_count: usize,
    body_height_px: f64,
    dpi: f64,
    source_uses_inline_field_reset: bool,
    native_hwp5_fixed_row_zero_reset: bool,
    hwp3_converted_requires_negative_reset: bool,
) -> Option<usize> {
    if line_count < 2 || para.line_segs.len() < line_count {
        return None;
    }
    // HWP3 변환 HWP5의 두 줄 음수 cursor는 한컴이 문단의 물리 조각을 나눈
    // 증거가 아니라 local cursor 보정으로도 사용한다(pi=140 유형). 세 줄 이상인
    // multi-line fragment만 내부 페이지 경계로 승격한다.
    if hwp3_converted_requires_negative_reset && line_count < 3 {
        return None;
    }

    let first = para.line_segs.first()?;
    let text_vpos_rewind = source_uses_inline_field_reset
        && para_has_visible_text(para)
        && controls_are_inline_text_metadata(para);

    // [Issue #2006] 빈-텍스트 문단에 전면(full-page) tac 이미지가 다수 스택된 경우
    // (예: 1790387 PrEP 보고서 pi=367, tac 그림 2장 각 lh≈900px, vpos=0..0), 한글은
    // 각 전면 이미지를 쪽당 1장으로 배치한다. rhwp 는 한 쪽에 겹쳐(2×본문 높이) 두어
    // 과소 페이지가 된다(−16). 연속한 두 라인이 모두 전면급 tac 이미지면 그 경계에서
    // 강제 분할한다(캐스케이드는 잔여 재처리로). text vpos-reset 과 독립 —
    // 본 케이스는 vpos 가 0 이라 아래 first.vertical_pos>0 가드에 걸린다.
    if para_is_treat_as_char_picture_only(para) {
        let full_page_px = body_height_px * 0.8;
        let break_line = para.line_segs[..line_count]
            .windows(2)
            .enumerate()
            .find_map(|(prev_idx, pair)| {
                let (prev, cur) = (&pair[0], &pair[1]);
                if is_synthetic_line_seg(prev) || is_synthetic_line_seg(cur) {
                    return None;
                }
                (hwpunit_to_px(prev.line_height, dpi) >= full_page_px
                    && hwpunit_to_px(cur.line_height, dpi) >= full_page_px)
                    .then_some(prev_idx + 1)
            });
        if break_line.is_some() {
            return break_line;
        }
    }

    if !text_vpos_rewind {
        return None;
    }

    if first.vertical_pos <= 0 || hwpunit_to_px(first.vertical_pos, dpi) < body_height_px * 0.7 {
        return None;
    }

    para.line_segs[..line_count]
        .windows(2)
        .enumerate()
        .find_map(|(prev_idx, pair)| {
            let prev = &pair[0];
            let cur = &pair[1];
            if is_synthetic_line_seg(prev) || is_synthetic_line_seg(cur) || prev.vertical_pos <= 0 {
                return None;
            }

            // Native HWP5 normally leaves zero resets to its dedicated
            // row/footnote paths. The 27469 contract is narrower: an
            // ordinary text row with this exact saved 1200HU grid represents
            // `vpos=0` as the next physical page top. Keep the 1048HU
            // RowBreak prose and other native grids on their existing paths.
            let native_hwp5_stored_zero_reset = native_hwp5_fixed_row_zero_reset
                && cur.line_height == 1200
                && cur.baseline_distance == 1020
                && cur.line_spacing == 1440
                && cur.tag == 0x0016_0000
                && cur.vertical_pos == 0;

            let stored_page_reset = if hwp3_converted_requires_negative_reset {
                // HWP3 변환 HWP5의 `vpos=0`은 문단 내부의 실제 쪽 reset이 아니라
                // 변환기의 local cursor 초기화로도 쓰인다. 음수로 되감긴 조각만
                // 저장된 다음 페이지 조각으로 해석한다.
                cur.vertical_pos < 0
            } else {
                native_hwp5_stored_zero_reset
                    || (text_vpos_rewind
                        && (cur.vertical_pos <= 0
                            || (cur.vertical_pos < prev.vertical_pos
                                && hwpunit_to_px(
                                    prev.vertical_pos.saturating_add(prev.line_height),
                                    dpi,
                                ) >= body_height_px * 0.72
                                && hwpunit_to_px(cur.vertical_pos, dpi) <= body_height_px * 0.06)))
            };

            if stored_page_reset {
                Some(prev_idx + 1)
            } else {
                None
            }
        })
}

/// 저장 HWPX가 명시적 다음 쪽 표제 바로 앞에 남긴 마지막 한 줄을 분리한다.
///
/// 일부 HWPX는 문단의 마지막 줄만 `vpos=0`으로 다음 물리 쪽에 저장하고, 이어지는
/// 표제 문단에는 명시적 쪽나누기를 둔다. 이 tail을 현재 쪽에 합치면 한컴보다 한 쪽이
/// 줄어든다. HWP3와 일반 HWP5는 별도 저장 규칙을 가지므로 호출부에서 HWPX stored
/// layout으로 한정한다.
pub(in crate::renderer::typeset) fn hwpx_explicit_page_break_tail_line(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    line_count: usize,
    body_height_px: f64,
    dpi: f64,
) -> Option<usize> {
    if para.controls.is_empty()
        && para_has_visible_text(para)
        && line_count >= 2
        && para.line_segs.len() == line_count
        && next_para.is_some_and(|next| {
            matches!(
                next.column_type,
                ColumnBreakType::Page | ColumnBreakType::Section
            )
        })
    {
        let split_line = line_count - 1;
        let prev = para.line_segs.get(split_line - 1)?;
        let tail = para.line_segs.get(split_line)?;
        if !is_synthetic_line_seg(prev)
            && !is_synthetic_line_seg(tail)
            && prev.vertical_pos > 0
            && tail.vertical_pos == 0
            && hwpunit_to_px(prev.vertical_pos, dpi) >= body_height_px * 0.70
            && hwpunit_to_px(prev.vertical_pos.saturating_add(prev.line_height), dpi)
                <= body_height_px + 1.0
        {
            return Some(split_line);
        }
    }
    None
}
