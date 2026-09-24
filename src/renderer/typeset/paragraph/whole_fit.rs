//! 전체 문단 fit의 저장 근거·되감김 판단. 호환성 spill 기록과 배치는 조정자가 수행한다.

use super::super::{
    line_seg_visible_bounds_px, page_item_para_index, para_controls_only_tac_topbottom_objects,
    para_controls_only_topbottom_floats, para_has_visible_text, paragraph_page_end_fit_height,
    paragraph_text_looks_like_list_continuation_tail, preceding_stored_vpos,
    saved_bounds_fit_at_flow_tail, saved_flow_marks_page_last, single_line_visible_bounds_px,
    stored_rewind_boundary_matches_current_flow, stored_vpos_restarts_near_body_top,
    stored_vpos_rewinds, VisibleFloatExclusion, STORED_VPOS_REWIND_MIN_FILL,
};
use super::metrics::FormattedParagraph;
use crate::model::{paragraph::Paragraph, provenance::LayoutCompatibilityProfile};
use crate::renderer::{hwpunit_to_px, pagination::PageItem};

pub(in crate::renderer::typeset) struct WholeFitPage<'a> {
    pub profile: LayoutCompatibilityProfile,
    pub omit_fresh_recalc_doc: bool,
    pub col_count: u16,
    pub current_items: &'a [PageItem],
    pub current_height: f64,
    pub body_height: f64,
    pub visible_float_exclusions: &'a [VisibleFloatExclusion],
    pub hangul2024_reclaimed: f64,
}

/// 저장 줄이 없는 제목 뒤에 번호 목록이 오면, 한/글은 제목과 첫 항목을 같은 쪽에서
/// 시작시킨다. 페이지 꼬리의 `height_for_fit`은 제목의 마지막 line spacing을 빼므로,
/// 그 값만으로는 제목이 layout에서 본문 밖으로 잘려 목록과 분리될 수 있다.
fn no_lineseg_heading_precedes_ordered_list(para: &Paragraph, next: Option<&Paragraph>) -> bool {
    if !para.line_segs.is_empty() || !para.controls.is_empty() || !para_has_visible_text(para) {
        return false;
    }
    let Some(next) = next else {
        return false;
    };
    let trimmed = next.text.trim_start();
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0 && trimmed.as_bytes().get(digits) == Some(&b'.')
}

pub(super) struct WholeFitEvidence {
    pub saved_single_line_bottom_fits: bool,
    pub saved_list_tail_body_vpos_fits: bool,
    pub page_end_fit_height: f64,
    pub stored_vpos_rewind_break: bool,
    pub stored_vpos_rewind_overflow_break: bool,
    pub hangul2024_rewind_override: bool,
}

/// 기존 단락 평가를 유지하며 필요한 경우에만 진단을 포함한 가용 높이를 조회한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn inspect(
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    strict_after_empty_host_float: bool,
    forced_page_break_line: Option<usize>,
    current_page_vpos_base: Option<i32>,
    available: f64,
    page: &WholeFitPage<'_>,
    dpi: f64,
    available_height: impl Fn() -> f64,
) -> WholeFitEvidence {
    // [#2279 OMIT-fit] spacing-누락 문서군에서 **저장 리셋 직전의 페이지말
    // 빈 문단**은 다음 쪽 상단 귀속이다 — 한글 fresh 는 누락 spacing 을
    // 재가산해 이 빈 문단을 다음 쪽으로 넘긴다(36392557 pi14: 저장 bottom
    // 910.6px 는 본문 안이지만 한글 PDF 는 p3 상단 36px 로 실측). 저장
    // page-last 신뢰와 h4f 트림을 함께 철회한다. 리셋이 뒤따르지 않는
    // 빈 문단(156652332 pi22 누적 구간)과 본문/개체 문단(156577742 footer
    // 표·그림)의 저장 증거·트림은 유지 — 전면 철회는 +1 회귀 실측.
    let omit_untrusted_empty = page.omit_fresh_recalc_doc
        && para.controls.is_empty()
        && !para_has_visible_text(para)
        && para
            .line_segs
            .last()
            .filter(|cs| {
                cs.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            })
            .zip(paragraphs.get(para_idx + 1).and_then(|next| {
                next.line_segs.first().filter(|ns| {
                    ns.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                })
            }))
            .is_some_and(|(cs, ns)| ns.vertical_pos < cs.vertical_pos);
    let saved_single_line_bottom_fits = !strict_after_empty_host_float
        && forced_page_break_line.is_none()
        && !omit_untrusted_empty
        && page.col_count == 1
        && fmt.line_heights.len() == 1
        // [#2137] 비-TAC 자리차지(TopAndBottom) float 만 가진 앵커도 저장
        // page-last 증거가 있으면 신뢰 — 개체는 하단 여백 스필(한컴 정합).
        && (para.controls.is_empty()
            || para_controls_only_topbottom_floats(para)
            || para_controls_only_tac_topbottom_objects(para))
        && !page.current_items.is_empty()
        // [Task #1749] 저장 flow 가 이 줄을 페이지 마지막으로 인코딩한 경우에만
        // bounds 신뢰 — 누적좌표 문서의 쪽 경계 overfill 차단.
        // [#2093] spacing_after 게이트(#1733) 제거: 신뢰 판정은 저장 줄의 시각
        // 경계(vpos~vpos+lh)로 하며, 한글은 쪽 마지막 줄의 아래 간격을 쪽 하단에서
        // 소비하지 않으므로 sa 는 배제 사유가 아니다 (1192000 해양수산 17→16쪽).
        && saved_flow_marks_page_last(paragraphs, para_idx)
        && current_page_vpos_base
            .and_then(|base| single_line_visible_bounds_px(para, base, dpi))
            .is_some_and(|bounds| {
                // [#2137] tac TopAndBottom 소형 개체 줄은 한컴이 하단 여백으로
                // 스필해 현재 쪽에 유지한다 (156637323 pi=19: 저장 vpos+lh
                // 956.6 > 본문 933.6 인데 한글 1쪽). 저장 page-last 증거가
                // 있을 때만 발동하므로 스필 허용폭은 하단 여백 급(40px)로 한정.
                let spill = if para_controls_only_tac_topbottom_objects(para) {
                    40.0
                } else {
                    0.0
                };
                let (top, bottom) = bounds;
                saved_bounds_fit_at_flow_tail(
                    (top, bottom),
                    page.current_height,
                    available_height(),
                    spill,
                )
            });
    let saved_list_tail_body_vpos_fits = !strict_after_empty_host_float
        && forced_page_break_line.is_none()
        && !omit_untrusted_empty
        && page.col_count == 1
        && fmt.line_heights.len() == 1
        && fmt.spacing_after <= 0.5
        && para.controls.is_empty()
        && !page.current_items.is_empty()
        && paragraph_text_looks_like_list_continuation_tail(para)
        && saved_flow_marks_page_last(paragraphs, para_idx)
        && para
            .line_segs
            .first()
            .and_then(|seg| line_seg_visible_bounds_px(seg, 0, dpi))
            .is_some_and(|bounds| {
                saved_bounds_fit_at_flow_tail(bounds, page.current_height, page.body_height, 0.0)
            });

    // 위 omit_untrusted_empty(저장 리셋 직전 빈 문단)는 트림 혜택도 잃는다
    // — 전량(lh+ls) 부족 시 다음 쪽 상단으로 넘긴다(36392557 pi14 36px).
    // 텍스트 문단은 종전 h4f 트림 유지(36392757 pi19: 전량 요구 시 +1 실측).
    // 저장 줄 없는 제목이 번호 목록 바로 앞에 있으면, 마지막 line spacing까지
    // 현재 쪽 예산에 넣어 제목과 첫 항목의 소속을 함께 판정한다. 일반 무저장
    // 본문까지 전량 요구하면 정상적으로 현재 쪽에 끝나는 문단을 다음 쪽으로 밀어
    // 페이지 수가 늘어나므로 이 구조에만 한정한다.
    let page_end_fit_height = paragraph_page_end_fit_height(
        fmt.total_height,
        fmt.height_for_fit,
        omit_untrusted_empty
            || strict_after_empty_host_float
            || no_lineseg_heading_precedes_ordered_list(para, paragraphs.get(para_idx + 1)),
    );
    // [#6855] "이 쪽이 찼는가"를 `current_height` 로만 물으면 **자리차지 밴드가
    // 차지한 쪽을 빈 쪽으로 읽는다.** 1613000-202200037 182쪽은 29×3 표가
    // `184.3..960.6` 을 이미 그려 놓았는데 흐름 계상은 118.0 에 머문다 — 자리차지
    // 표는 흐름에 host 줄만 계상하기 때문이다(그 규칙 자체는 바꾸지 않는다).
    // 그 118.0 으로 재니 아래 `#3837` 되감김 관문이 열리지 않아, 한/글이 쪽을 끊은
    // 자리에서 계속 담고 `pi=3`(`과목 2: 인적 요소`)을 **용지 45.5px 아래**에
    // 그렸다. 문턱(`MIN_FILL`)은 그대로 두고 **재는 양만** 실제 점유로 바꾼다.
    let page_occupied_height = page
        .visible_float_exclusions
        .iter()
        .map(|zone| zone.bottom)
        .fold(page.current_height, f64::max);
    // [#3837] 저장 vpos 가 되돌아가면 한글은 거기서 쪽을 끊었다.
    let stored_vpos_rewind_base = page.col_count == 1
        && !page.current_items.is_empty()
        // 같은 문단이 이미 이 쪽에 놓였으면 걸지 않는다 — 되돌아감은 문단 시작 신호라
        // 이미 시작한 뒤 걸면 문단을 쪼갠다.
        && !page
            .current_items
            .iter()
            .any(|it| page_item_para_index(it) == Some(para_idx))
        && stored_vpos_rewinds(preceding_stored_vpos(paragraphs, para_idx), para);
    // [#6761] 채움률 관문만으로는 "덜 찼는데 한글이 끊은 쪽"을 놓친다. 되감김이
    // **쪽 위쪽 띠에서 다시 시작**하고 되감김 직전 자리가 지금 흐름 위치와 같으면,
    // 사다리가 적은 그 쪽 경계가 지금 이 자리다 — 채움률과 무관하게 인정한다.
    let stored_rewind_at_matching_flow_position = stored_vpos_restarts_near_body_top(para)
        && stored_rewind_boundary_matches_current_flow(
            paragraphs,
            para_idx,
            page_occupied_height,
            dpi,
        );
    let stored_vpos_rewind_break = stored_vpos_rewind_base
        && (page_occupied_height >= available * STORED_VPOS_REWIND_MIN_FILL
            || stored_rewind_at_matching_flow_position);
    // [#5755] 되돌아간 문단이 통째로는 안 들어가는 경우 — 어차피 전체 배치는 실패라
    // 종전엔 split 경로로 흘러가 저장 좌표(새 쪽의 쪽-지역 좌표)를 현재 쪽 꼬리
    // 적합 근거로 오독, 본문 밖·용지 밖까지 그렸다(156677324 pi=9: 996>934px).
    // 한글은 이 문단을 통째로 다음 쪽에 둔다(2쪽 925.1≤933.6 정확 재현). 실제
    // 넘침이 있을 때만 발동하므로 MIN_FILL 완화의 연쇄(+3쪽) 부작용과 무관하다.
    // ⚠ [#6855] 여기까지 `page_occupied_height` 로 넓히면 안 된다 — 코퍼스 실측에서
    // `1480000-201600147` 의 **글자 겹침이 33 → 37** 로 는다. 이 술어는 종전대로
    // 흐름 계상으로 잰다.
    let stored_vpos_rewind_overflow_break =
        stored_vpos_rewind_base && page.current_height + page_end_fit_height > available;
    // [compat 2024] 앵커 줄 회수분이 있거나 앞선 경계를 이미 덮은 상태에서
    // 이 문단의 첫 줄이 (회수 보너스 포함) 들어가면 저장 되감김(=2022 조판의
    // 쪽 경계)을 덮는다. 회수도 선행 덮음도 없으면 종전 동작 그대로.
    let hangul2024_rewind_override = stored_vpos_rewind_break
        && page.profile.hangul2024_layout()
        && page.hangul2024_reclaimed > 0.0
        && {
            // 빈 문단 need=0 / 실문단 첫 줄 (위 reset-trigger 와 같은 규칙).
            let need: f64 = if !para_has_visible_text(para) && para.controls.is_empty() {
                0.0
            } else {
                para.line_segs
                    .first()
                    .map(|s| hwpunit_to_px(s.line_height.saturating_add(s.line_spacing), dpi))
                    .unwrap_or(page_end_fit_height)
                    .min(page_end_fit_height)
            };
            page.current_height + need <= available + page.hangul2024_reclaimed
        };
    let stored_vpos_rewind_break = stored_vpos_rewind_break && !hangul2024_rewind_override;
    WholeFitEvidence {
        saved_single_line_bottom_fits,
        saved_list_tail_body_vpos_fits,
        page_end_fit_height,
        stored_vpos_rewind_break,
        stored_vpos_rewind_overflow_break,
        hangul2024_rewind_override,
    }
}
