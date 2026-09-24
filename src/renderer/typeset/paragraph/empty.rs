//! 빈 문단 조기 반환의 읽기 전용 판단. 각 경로의 서로 다른 상태 효과는 호출자가 보존한다.

use super::super::para_has_visible_text;
use super::metrics::FormattedParagraph;
use crate::model::paragraph::Paragraph;
use crate::renderer::pagination::PageItem;

pub(in crate::renderer::typeset) struct EmptyTailPage<'a> {
    pub col_count: u16,
    pub current_items: &'a [PageItem],
    pub current_height: f64,
    pub body_height: f64,
    pub current_zone_y_offset: f64,
    pub current_footnote_height: f64,
}

pub(super) enum TailDisposition {
    Continue,
    Hidden,
    Unadvanced,
}

pub(super) fn hide_rowbreak_guide(
    prev_is_partial_table: bool,
    para: &Paragraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
) -> bool {
    // [Task #1686] RowBreak 표 조각 뒤에 남는 빈 guide 문단 흡수.
    // pr-1674처럼 표 셀 내부 vpos reset으로 페이지가 갈린 뒤, 뒤따르는 빈 문단들이
    // 이전 좌표계의 큰 vpos(페이지 하단)를 그대로 갖고 다음 실질 앵커 표보다 아래에
    // 기록될 수 있다. 이 빈 줄들을 flow 높이로 누적하면 다음 RowBreak 표가 한컴/PDF보다
    // 늦게 시작해 page 5 내용과 총 페이지 수가 밀린다.
    if prev_is_partial_table
        && para.controls.is_empty()
        && !para_has_visible_text(para)
        && para.line_segs.len() == 1
    {
        let curr_vpos = para.line_segs.first().map(|s| s.vertical_pos);
        let next_anchor_vpos = paragraphs
            .iter()
            .skip(para_idx + 1)
            .find(|p| para_has_visible_text(p) || !p.controls.is_empty())
            .and_then(|p| p.line_segs.first().map(|s| s.vertical_pos));
        if let (Some(curr), Some(next)) = (curr_vpos, next_anchor_vpos) {
            const EMPTY_GUIDE_RESET_GAP_HU: i32 = 2000;
            if curr > next + EMPTY_GUIDE_RESET_GAP_HU {
                return true;
            }
        }
    }

    false
}

pub(super) fn hide_overflowing_empty(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    has_items: bool,
    current_height: f64,
    available: f64,
    hidden_empty_lines: u32,
) -> bool {
    let trimmed = para.text.replace(|c: char| c.is_control(), "");
    let is_empty_para = trimmed.trim().is_empty() && para.controls.is_empty();
    is_empty_para
        && has_items
        && current_height + fmt.height_for_fit > available
        && hidden_empty_lines < 2
}

pub(super) fn trailing_disposition(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    is_last_in_section: bool,
    available: f64,
    layout_drift_safety_px: f64,
    page: &EmptyTailPage<'_>,
) -> TailDisposition {
    // [Task #676] trailing empty paragraph 가드 (단단 전용):
    // 섹션 마지막 빈 paragraph 가 현재 safety 영역 내 미세 overflow 로 fit 실패 시
    // height=0 흡수 — 단독 빈 페이지 차단. 한컴2022 정합 시멘틱.
    // (통합재정통계 2010.11/2011.10: 과거 safety_margin 운용 시
    //  pi=14 의 0.8px overflow 를 흡수한 사례.)
    // hide_empty_line (Task #362) 분기와 달리 SectionDef bit 무관, 섹션 마지막 1개만 흡수.
    if is_last_in_section && page.col_count == 1 && !page.current_items.is_empty() {
        let trimmed = para.text.replace(|c: char| c.is_control(), "");
        let is_empty_para = trimmed.trim().is_empty() && para.controls.is_empty();
        if is_empty_para {
            let total_h = page.current_height + fmt.height_for_fit;
            let fit_fail_within_safety =
                total_h > available && total_h <= available + layout_drift_safety_px;
            let base_available = page.body_height - page.current_zone_y_offset;
            let fit_fail_only_after_footnote_reserve = page.current_footnote_height > 0.0
                && total_h > available
                && total_h <= base_available;
            let prior_trailing_drift = page.current_height > available
                && page.current_height <= available + layout_drift_safety_px + 0.5;
            let previous_item_is_empty_para = page
                .current_items
                .last()
                .and_then(|item| match item {
                    PageItem::FullParagraph { para_index } => Some(*para_index),
                    _ => None,
                })
                .and_then(|prev_idx| paragraphs.get(prev_idx))
                .map(|prev_para| {
                    let trimmed = prev_para.text.replace(|c: char| c.is_control(), "");
                    trimmed.trim().is_empty() && prev_para.controls.is_empty()
                })
                .unwrap_or(false);
            if prior_trailing_drift && previous_item_is_empty_para {
                return TailDisposition::Hidden;
            }
            if fit_fail_within_safety || fit_fail_only_after_footnote_reserve {
                return TailDisposition::Unadvanced;
            }
        }
    }

    TailDisposition::Continue
}
