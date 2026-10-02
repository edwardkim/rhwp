//! 빈 문단 조기 반환의 읽기 전용 판단. 각 경로의 서로 다른 상태 효과는 호출자가 보존한다.

use super::super::para_has_visible_text;
use super::metrics::FormattedParagraph;
use crate::model::paragraph::{ColumnBreakType, Paragraph};
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
    Unadvanced,
}

/// 분할 표의 배치를 끝내고 바로 만나는 구역 종료 문단은 저장본의 종료 guide일 수 있다.
/// 저장 줄이 원래 쪽 안에 있고 명시적/저장 쪽 경계가 없을 때만 fit 실패를 흡수한다.
/// 반복 Enter는 앞 항목이 본문 문단이므로 이 경로에 들어오지 않는다.
pub(super) fn is_stored_table_closing_guide(
    para: &Paragraph,
    para_idx: usize,
    paragraphs: &[Paragraph],
    is_last_in_section: bool,
    dpi: f64,
    page: &EmptyTailPage<'_>,
) -> bool {
    if !is_last_in_section
        || page.col_count != 1
        || para_has_visible_text(para)
        || !para.controls.is_empty()
        || matches!(
            para.column_type,
            ColumnBreakType::Page | ColumnBreakType::Section
        )
    {
        return false;
    }
    let Some(PageItem::PartialTable { para_index, .. }) = page.current_items.last() else {
        return false;
    };
    // paragraph flow는 table coordinator가 모든 조각을 소비한 뒤에 호출된다.
    // 뒤에 다른 문단을 거친 빈 줄 묶음은 표의 종료 guide로 해석하지 않는다.
    if para_index.checked_add(1) != Some(para_idx) {
        return false;
    }
    let Some(previous) = paragraphs.get(*para_index) else {
        return false;
    };
    if crate::renderer::typeset::stored_vpos_top_collision(previous, para) {
        return false;
    }
    let [line] = para.line_segs.as_slice() else {
        return false;
    };
    if crate::renderer::typeset::is_synthetic_line_seg(line)
        || line.tag & crate::model::paragraph::LineSeg::TAG_FIRST_SEGMENT == 0
        || line.segment_width <= 0
        || line.line_height <= 0
    {
        return false;
    }
    line.vertical_pos >= crate::renderer::px_to_hwpunit(page.current_zone_y_offset, dpi)
        && line.vertical_pos.saturating_add(line.line_height)
            <= crate::renderer::px_to_hwpunit(page.body_height, dpi)
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
            // 앞 줄의 누적 간격이 조금 넘쳤어도 다음 줄 상자의 점유는 별개다.
            // 이 문단 자체가 fit할 때만 흡수하고, 아니면 정상 이월을 진행한다.
            if fit_fail_within_safety || fit_fail_only_after_footnote_reserve {
                return TailDisposition::Unadvanced;
            }
        }
    }

    TailDisposition::Continue
}
