//! 줄 분할 진입 전 첫 줄/저장 경계 판정. 쪽 전환과 호환성 상태 기록은 호출자가 맡는다.
use super::super::{
    is_synthetic_line_seg, page_item_para_index, para_has_visible_text,
    stored_vpos_restarts_near_body_top,
};
use super::metrics::FormattedParagraph;
use crate::model::{paragraph::Paragraph, provenance::LayoutCompatibilityProfile};
use crate::renderer::{pagination::PageItem, style_resolver::ResolvedStyleSet};

pub(in crate::renderer::typeset) struct SplitEntryPage<'a> {
    pub profile: LayoutCompatibilityProfile,
    pub col_count: u16,
    pub current_height: f64,
    pub current_items: &'a [PageItem],
    pub body_height: f64,
    pub stored_ladder_spacing_omitted: bool,
    pub hangul2024_reclaimed: f64,
}

pub(in crate::renderer::typeset) struct SplitEntryFit {
    pub first_line_h: f64,
    pub remaining: f64,
    pub hwp_first_line_before_reset_fits: bool,
    pub stored_whole_para_reset: bool,
    pub hangul2024_split_refit: bool,
}

/// 구성된 줄이 있을 때만 호출한다. 플래그 기록 전에 관측한 fit 근거를 반환한다.
#[allow(clippy::too_many_arguments)]
pub(in crate::renderer::typeset) fn inspect_entry(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    para_idx: usize,
    line_count: usize,
    available: f64,
    page: &SplitEntryPage<'_>,
    dpi: f64,
) -> SplitEntryFit {
    // 남은 공간이 없거나 첫 줄도 못 넣으면 먼저 다음 단/페이지로
    // [#2279 pi78] 다중 줄 문단의 분할 진입 첫 줄은 full advance(lh+ls)를
    // 요구한다 — 한글 COM 하단여백 18단 사다리 실측(36399374 pi78):
    // 슬랙 < lh+ls(22.5pt)면 첫 줄을 넣지 않고 통째 이월, [lh+ls, h4f)
    // 구간 1+1 분할, ≥ h4f 전체 수용 — 곡선 전체가 이 규칙과 일치.
    // 트레일링 ls 트림(#359 h4f)은 문단 마지막 줄에만 해당하므로 단일 줄
    // 문단은 종전 lh-만 유지.
    let first_line_h = if line_count > 1 && page.stored_ladder_spacing_omitted {
        // 적용 대상은 spacing-누락 서명이 검출된 기계생성 문서군뿐이다 —
        // 전역/HWPX-전체 적용은 각각 2572521 별지6(HWP5, 한글 6쪽 +1)과
        // 1733 국제고속선기준(한글-저장 HWPX, 242쪽 +2) 회귀 실측.
        fmt.line_advance(0)
    } else {
        fmt.line_heights[0]
    };
    let remaining = (available - page.current_height).max(0.0);
    // [Task #1086] 단일 단에서도 HWP가 paragraph 내부 page reset 을
    // LINE_SEG(vpos=0) 로 인코딩하는 케이스가 있다(k-water-rfp pi=66).
    // 첫 줄의 HWP 좌표가 본문 안에 있고 다음 줄이 reset 이면, 보수적
    // safety margin 으로 미리 페이지를 넘기지 말고 줄 단위 split 루프에서
    // 첫 줄만 현재 페이지에 배치하게 둔다.
    let hwp_first_line_before_reset_fits = para
        .line_segs
        .get(1)
        .map(|next| next.vertical_pos == 0)
        .unwrap_or(false)
        && para
            .line_segs
            .first()
            .map(|cur| {
                let bottom_px = crate::renderer::hwpunit_to_px(
                    cur.vertical_pos.saturating_add(cur.line_height),
                    dpi,
                );
                bottom_px <= page.body_height + 0.5
            })
            .unwrap_or(false);
    // [Task #1750] 저장 LINE_SEG 가 문단 전체를 새 쪽 상단으로 인코딩한 경우
    // (첫 줄 vpos 가 near-top 으로 리셋 + 직전 문단은 페이지 하단부) 분할하지
    // 않고 페이지를 넘긴다. 단일 단 vpos-reset 가드(line 2279)는 cv==0 만
    // 인정하므로 near-top(예: 700HU) 리셋 문서는 분할 경로로 새어 들어와
    // 첫 줄이 이전 쪽 말미에 남는다 (3024019 pi22: ls[0] vpos=700, 한글도
    // 문단 전체를 새 쪽 배치). 전체 배치가 이미 실패한 분할 직전에만 적용해
    // 일반 흐름(#418/#321 보수 기준)은 건드리지 않는다.
    let current_page_has_stale_hwpx_line_metrics = page.profile.hwpx_stored_layout()
        && page
            .current_items
            .iter()
            .filter_map(page_item_para_index)
            .any(|placed_para_idx| {
                paragraphs.get(placed_para_idx).is_some_and(|placed_para| {
                    crate::renderer::paragraph_source_line_metrics_need_reflow(
                        placed_para,
                        styles,
                        dpi,
                    )
                })
            });
    let stored_whole_para_reset = page.col_count == 1
        && para_idx > 0
        // 앞 문단의 손상 HWPX 줄을 순차 조판으로 접은 경우, 그 뒤 문단의
        // 작은 vpos는 새 쪽이 아니라 같은 손상 좌표계의 잔여값이다.
        && !current_page_has_stale_hwpx_line_metrics
        && para
            .line_segs
            .first()
            .filter(|ls| !is_synthetic_line_seg(ls))
            .map(|ls| ls.vertical_pos > 0 && ls.vertical_pos <= 2500)
            .unwrap_or(false)
        && paragraphs[para_idx - 1]
            .line_segs
            .last()
            .map(|s| s.vertical_pos.saturating_add(s.line_height) > 60_000)
            .unwrap_or(false);
    // [compat 2024] near-top 저장 리셋(stored_whole_para_reset)도 2022 쪽
    // 경계 신호다 — 앵커 줄 회수가 있고 첫 줄(빈 문단은 0)이 회수 보너스로
    // 들어가면 덮는다 (reset-trigger/되감김과 같은 규칙, 제3의 경계 지점).
    let hangul2024_split_refit = page.profile.hangul2024_layout()
        && page.hangul2024_reclaimed > 0.0
        && stored_whole_para_reset
        && page.current_height < available
        && {
            let blank = !para_has_visible_text(para) && para.controls.is_empty();
            let need = if blank { 0.0 } else { first_line_h };
            page.current_height + need <= available + page.hangul2024_reclaimed
        };
    SplitEntryFit {
        first_line_h,
        remaining,
        hwp_first_line_before_reset_fits,
        stored_whole_para_reset,
        hangul2024_split_refit,
    }
}

/// spill 플래그 기록 뒤, 저장 첫 줄 바닥을 확인하고 쪽 전환 여부만 반환한다.
pub(in crate::renderer::typeset) fn should_advance(
    para: &Paragraph,
    fit: &SplitEntryFit,
    available: f64,
    stored_vpos_rewind_break: bool,
    stored_vpos_rewind_overflow_break: bool,
    page: &SplitEntryPage<'_>,
    dpi: f64,
) -> bool {
    let SplitEntryFit {
        first_line_h,
        remaining,
        hwp_first_line_before_reset_fits,
        stored_whole_para_reset,
        hangul2024_split_refit,
    } = *fit;
    // [#6568] 줄 분할 루프의 넘침 판정에는 `li > cursor_line` 면제가 걸려 있어
    // **조각의 첫 줄은 검사조차 되지 않는다.** 그래서 예산을 넘는 줄이 무조건 그
    // 쪽에 놓인다. 156678235 pi=59 실측:
    //
    //   진입 검사  remaining 22.12 >= first_line_h 18.67   → 통과(쪽 안 넘김)
    //   루프 예산  avail_for_lines 18.12 <  line_heights[0] 18.67  → 안 들어감
    //   저장 사다리 ls[0] vpos 68896(=918.6px) + lh 1400 = 937.3px > 본문 933.6px
    //
    // 면제 자체는 무를 수 없다 — 바로 아래 `end_line <= cursor_line` 절이 "조각은
    // 최소 한 줄" 을 강제해 결과가 같아지고, 빈 조각은 쪽 진행을 멈출 위험이 있다.
    // 예산(`avail_for_lines`)을 진입 검사로 가져오는 것도 안 된다 — drift 마진을
    // 이중 차감하는 값이라 전수 게이트에서 40건이 깨진다(실측).
    //
    // 대신 **저장 사다리에 직접 묻는다**: 첫 줄의 *바닥*이 본문 하한 밖이면 한/글은
    // 그 줄을 이 쪽에 두지 않은 것이다. 이미 있는
    // `hwp_first_line_before_reset_fits`(바닥이 안에 들면 남긴다)의 정확한 거울이며,
    // 추정이 아니라 파일이 적어 놓은 값이다.
    let stored_first_line_bottom_outside_body = para
        .line_segs
        .first()
        .filter(|ls| !is_synthetic_line_seg(ls))
        .map(|ls| {
            let top_px = crate::renderer::hwpunit_to_px(ls.vertical_pos, dpi);
            let bottom_px =
                crate::renderer::hwpunit_to_px(ls.vertical_pos.saturating_add(ls.line_height), dpi);
            let body_bottom = page.body_height;
            // 저장 좌표가 **지금 조판 위치와 같은 쪽**을 가리킬 때만 증거로 쓴다.
            top_px + 0.5 >= page.current_height && top_px <= body_bottom && bottom_px > body_bottom
        })
        .unwrap_or(false);
    (page.current_height >= available
        || remaining < first_line_h
        || stored_first_line_bottom_outside_body
        || (stored_whole_para_reset && !hangul2024_split_refit)
        // [#5755] 저장 되감김 + 전체 fit 실패 = 한글이 이 문단을 통째로 다음 쪽에
        // 둔 배치 — split 로 현재 쪽에 걸치지 말고 먼저 쪽을 넘긴다.
        || stored_vpos_rewind_overflow_break
        // [#6761] 되감김 판정(`stored_vpos_rewind_break`)이 서면 전체 배치 분기는
        // 이미 건너뛴다. 그런데 분할 경로가 그 값을 안 봐서, 남은 여백에 첫 줄이
        // 들어가면 같은 쪽에 얹고 만다 — 되감김이 말한 쪽 경계가 무시된다.
        //
        // 다만 되감김 자체는 **부분 후퇴**도 포함한다(같은 쪽 안에서 표 아래 주석이
        // 앞 문단보다 위에서 시작하는 형상 — 1342000 pi=219 70880 -> pi=220 66140).
        // 쪽 경계의 되감김은 사다리가 **쪽 위쪽 띠에서 다시 시작**한다는 뜻이므로,
        // 이 분기는 `cl > 5000` 의 거울인 `nv <= 5000` 까지 요구한다
        // (1130000 pi=41 53956 -> pi=42 500 이 그 형상이다).
        || (stored_vpos_rewind_break && stored_vpos_restarts_near_body_top(para)))
        && !page.current_items.is_empty()
        && !hwp_first_line_before_reset_fits
}
