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
    Unadvanced,
}

pub(super) fn hide_rowbreak_guide(
    prev_is_partial_table: bool,
    para: &Paragraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    current_height: f64,
    dpi: f64,
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
        let anchor_idx = paragraphs
            .iter()
            .enumerate()
            .skip(para_idx + 1)
            .find(|(_, p)| para_has_visible_text(p) || !p.controls.is_empty())
            .map(|(idx, _)| idx);
        let next_anchor_vpos =
            anchor_idx.and_then(|idx| paragraphs[idx].line_segs.first().map(|s| s.vertical_pos));
        if let (Some(curr), Some(next), Some(anchor_idx)) =
            (curr_vpos, next_anchor_vpos, anchor_idx)
        {
            const EMPTY_GUIDE_RESET_GAP_HU: i32 = 2000;
            if curr > next + EMPTY_GUIDE_RESET_GAP_HU {
                // 되감긴 다음 앵커의 저장 위치가 지금 흐름 위치에서 이 빈 줄들의 저장 전진을
                // 모두 담을 만큼 아래면, 한컴은 새 쪽에서도 빈 줄 자리를 남겼다. 그 줄은
                // 옛 좌표의 guide 가 아니다(1480000 44쪽 pi=264: 표 끝+아래 여백 677.8px
                // + 빈 줄 26.4px = 앵커 pi=265 저장 704.2px, 한/글 2020 PDF 도 빈 줄을 둔다).
                // pr-1674 처럼 앵커가 흐름 위치보다 위거나 빈 줄들을 담지 못하면 흡수한다.
                let empty_advance_px: f64 = paragraphs[para_idx..anchor_idx]
                    .iter()
                    .filter_map(|p| p.line_segs.first())
                    .map(|seg| {
                        crate::renderer::hwpunit_to_px(
                            seg.line_height + seg.line_spacing.max(0),
                            dpi,
                        )
                    })
                    .sum();
                let anchor_px = crate::renderer::hwpunit_to_px(next, dpi);
                const STORED_ADVANCE_ROUNDING_PX: f64 = 1.0;
                let stored_room_keeps_lines =
                    anchor_px - current_height >= empty_advance_px - STORED_ADVANCE_ROUNDING_PX;
                return !stored_room_keeps_lines;
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
