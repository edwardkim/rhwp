//! 문단의 현재 fragment에 수용할 줄 후보를 계산한다.
//!
//! 저장 줄/각주 fit 정책은 여기서 읽기 전용으로 평가한다. 페이지 전환과 결과 적용은
//! 호출자의 책임이며, 기존 정책의 순서·상수·예외 의미는 변경하지 않는다.

use super::super::{
    is_synthetic_line_seg, line_seg_visible_bounds_px,
    para_has_only_treat_as_char_picture_or_shape, para_near_rowbreak_table,
    saved_bounds_overlap_current_flow, saved_line_clears_footnote_area,
    saved_line_is_anchored_to_current_flow, saved_line_range_fits_body_tail,
    saved_tail_fit_chain_decision, SavedTailFitChainDecision,
};
use super::metrics::FormattedParagraph;
use crate::model::paragraph::Paragraph;
use crate::model::provenance::LayoutCompatibilityProfile;

/// 한 번의 줄 스캔 동안 불변인 페이지 관측값. 각 단/쪽 전환 뒤 새로 읽는다.
pub(in crate::renderer::typeset) struct LineScanPage {
    pub profile: LayoutCompatibilityProfile,
    pub col_count: u16,
    pub body_height: f64,
    pub current_height: f64,
    pub has_items: bool,
    pub vpos_ladder_dirty: bool,
    pub current_footnote_height: f64,
    pub footnote_safety_margin: f64,
    pub current_zone_y_offset: f64,
    pub current_bottom_fixed_exclusion: f64,
}

/// 보정 전 후보와 전체 문단 fit 재확인에 필요한 저장 꼬리 채택 증거.
pub(in crate::renderer::typeset) struct LineScanResult {
    pub end_line: usize,
    pub cumulative: f64,
    pub used_saved_tail_vpos_fit: bool,
}

/// 입력과 문단 구성 결과만 읽는다. 원본 IR과 TypesetState를 변경하지 않는다.
#[allow(clippy::too_many_arguments)]
pub(in crate::renderer::typeset) fn scan_lines(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    cursor_line: usize,
    line_count: usize,
    avail_for_lines: f64,
    forced_page_break_line: Option<usize>,
    native_hwp5_existing_footnote_reset_line: Option<usize>,
    current_page_vpos_base: Option<i32>,
    is_tac_picture_stack: bool,
    page: &LineScanPage,
    dpi: f64,
) -> LineScanResult {
    // 현재 페이지에 들어갈 줄 범위 결정
    let mut cumulative = 0.0;
    let mut end_line = cursor_line;
    let mut used_saved_tail_vpos_fit = false;
    // [#4024] 저장 사다리 신뢰(`saved_tail_vpos_fit`)는 `li..line_count` 만 보고
    // 그때까지 쌓인 `cumulative` 를 모른다. 그래서 한 번 참이 되면 예산을 넘긴
    // 채로 남은 줄을 계속 통과시켜 초과가 단조 증가한다(1480000 pi=724: 잔여
    // 18.5px 에 6줄 통과, 초과 19.7 -> 137.0px).
    //
    // 쪽 단위 실측(무작위 대형 문서 60건): 소실 쪽의 연쇄는 전부 길이 3 이상
    // (중앙 5, 최대초과 중앙 117.2px)인데, 소실 없는 쪽은 66% 가 길이 1~2
    // (중앙 2, 최대초과 중앙 49.4px)다. 한컴 정답 36쪽을 고정하는
    // `issue_554` hwp3-sample4 의 유일한 통과도 길이 1 이다.
    let mut tail_fit_chain = 0usize;
    for li in cursor_line..line_count {
        if forced_page_break_line
            .map(|break_line| li == break_line && li > cursor_line)
            .unwrap_or(false)
        {
            break;
        }
        // [Task #619] 다단 paragraph 내 vpos-reset 강제 분리.
        // line_segs[li].vertical_pos == 0 (li>0) 은 HWP 가 해당 line 을
        // 다음 단/페이지 최상단에 배치하도록 인코딩한 신호.
        // 다단 한정 적용 — 단일 단은 partial-table split 회귀 (issue #418) 차단 위해 미적용.
        //
        // [#4092] 단일 단이라도 **전면 개체 줄**은 이 신호를 존중한다. 한 문단에
        // 전면 그림이 여럿 든 형상에서, 한컴은 줄마다 reset 을 기록해 쪽마다
        // 하나씩 두는데 rhwp 는 그 신호를 버리고 한 쪽에 쌓았다(HPV 코호트
        // pi=970: 본문 895.8px 에 763~770px 짜리 줄 6개 = 4,654px, 문서 전체
        // 246쪽 ↔ 한글 235쪽).
        //
        // 조건은 문단이 아니라 **그 줄**에 건다 — 같은 문단이라도 첫 줄은
        // 199.7px 라 `is_tac_picture_stack`(모든 줄이 절반 초과)은 거짓이다.
        // #418 의 partial-table 회귀는 여기에 닿지 않는다. 이 분기는 TAC 그림/도형만
        // 남은 문단으로 한정하므로, 표·수식 등 다른 컨트롤이 섞인 문단은 배제한다.
        let tac_picture_full_page_line = para_has_only_treat_as_char_picture_or_shape(para)
            && fmt
                .line_heights
                .get(li)
                .map(|h| *h > page.body_height * 0.5)
                .unwrap_or(false);
        if (page.col_count > 1 || tac_picture_full_page_line)
            && li > cursor_line
            && para
                .line_segs
                .get(li)
                .map(|s| s.vertical_pos == 0)
                .unwrap_or(false)
        {
            break;
        }
        let content_h = fmt.line_heights[li];
        if cumulative + content_h > avail_for_lines && li > cursor_line {
            // [Task #631] HWP 권위값 더블체크
            // 누적 추정으로는 fit 실패하지만 HWP 파일 자체가 다음 줄(li+1)에
            // vpos-reset(=0) 을 인코딩한 경우, 한컴 엔진이 직접 li 까지를 현재
            // 페이지에 배치한 것이다. typeset 보수 마진(20px) 으로 인한 콘텐츠
            // 손실을 차단하기 위해 HWP 신호를 우선한다.
            // 조건: (1) 다음 줄의 vpos==0 (페이지 경계 신호)
            //       (2) 현재 줄의 hwp 좌표 vpos+lh 가 body_available 안
            let hwp_authoritative = !is_tac_picture_stack
                && para
                    .line_segs
                    .get(li + 1)
                    .map(|next| next.vertical_pos == 0)
                    .unwrap_or(false)
                && para
                    .line_segs
                    .get(li)
                    .map(|cur| {
                        let bottom_px = crate::renderer::hwpunit_to_px(
                            cur.vertical_pos.saturating_add(cur.line_height),
                            dpi,
                        );
                        bottom_px <= page.body_height
                    })
                    .unwrap_or(false);
            let overflow = cumulative + content_h - avail_for_lines;
            // [Task #1733] 국제고속선기준 잔여 over-pagination 완화:
            // 누적 높이 drift 로는 다음 쪽으로 밀리지만, 저장 LINE_SEG 가 남은
            // tail 줄들을 모두 현재 쪽 본문 하단 안에 두고 있고 중간 vpos reset 이
            // 없으면 한컴 저장 flow 를 신뢰한다. 문단 전체/표/다단에는 적용하지 않고
            // partial paragraph split 지점에서만 작동시켜 하단 겹침 회귀를 줄인다.
            let saved_tail_vpos_fit = forced_page_break_line.is_none()
                && page.col_count == 1
                && para.controls.is_empty()
                && page.has_items
                && !para_near_rowbreak_table(paragraphs, para_idx)
                // [#6031] sb-누락 ladder(dirty) 의 꼬리 좌표는 배치 좌표가
                // 아니다 — 렌더 흐름은 sb 를 가산해 이미 그 아래에 있고,
                // 이 좌표로 붙든 줄은 본문 하단 밖에 그려진다(3249937 p3
                // '바.' +25.1pt). 한글 fresh 도 이 줄을 다음 쪽에 둔다.
                && !page.vpos_ladder_dirty
                && saved_line_range_fits_body_tail(
                    para,
                    li,
                    line_count,
                    page.body_height,
                    dpi,
                );
            // HWPX의 vpos=0 reset은 일반 writer-local cursor가 아니라
            // `internal_vpos_page_break_line`이 확인한 물리 fragment 경계일 수
            // 있다. 현재 fragment의 첫 줄이 flow 앵커와 일치하면 reset 전
            // 전체는 같은 쪽 owner다. 본문 bottom을 넘었다는 계산만으로
            // 중간 tail-only 쪽을 만들지 않는다.
            let hwpx_reset_fragment_owner = forced_page_break_line.is_some_and(|break_line| {
                cursor_line < break_line
                    && li < break_line
                    && hwpx_saved_reset_fragment_matches_current_flow(
                        page,
                        para,
                        cursor_line,
                        break_line,
                        current_page_vpos_base.unwrap_or(0),
                        dpi,
                    )
            });
            // native HWP5가 문단 중간 reset 직전의 연속 줄들을 기존 각주
            // 바로 위에 저장한 경우에는, 40px safety margin 때문에 그 줄을
            // 조기 이월하지 않는다. 일반 body height가 아니라 실제
            // FootnoteArea top을 넘지 않는지, 현재 flow tail과 저장 top이
            // 일치하는지, reset 전 범위인지까지 모두 확인한다. 표·그림·각주
            // control과 reset 뒤의 줄은 종전 보수 budget을 유지한다.
            let native_hwp5_reset_tail_fits_actual_footnote_boundary = page
                .profile
                .hwp5_stored_pagination_layout()
                && page.col_count == 1
                && page.current_footnote_height > 0.0
                && para.controls.is_empty()
                && page.has_items
                && native_hwp5_existing_footnote_reset_line
                    .is_some_and(|break_line| cursor_line < break_line && li < break_line)
                && para
                    .line_segs
                    .get(cursor_line)
                    .and_then(|seg| {
                        line_seg_visible_bounds_px(seg, current_page_vpos_base.unwrap_or(0), dpi)
                    })
                    .is_some_and(|bounds| {
                        saved_bounds_overlap_current_flow(bounds, page.current_height)
                    })
                && saved_line_range_fits_body_tail(
                    para,
                    cursor_line,
                    li + 1,
                    (page.body_height
                        - page.current_footnote_height
                        - page.current_zone_y_offset
                        - page.current_bottom_fixed_exclusion)
                        .max(0.0),
                    dpi,
                );
            // [#4054] 각주 안전마진(40px = 3000 HWPUNIT)은 각주 영역과 본문 꼬리가
            // 겹칠 위험을 상수로 막는다. 그런데 저장 LineSeg 가 이 줄을 **각주 영역
            // 위**에 두고 있고 흐름 커서가 그 좌표보다 위에 있으면, 겹침은 rhwp·한글
            // 양쪽 기준 모두에서 실측으로 배제된다. 그 경우에만 마진 몫의 초과를
            // 통과시킨다.
            //
            // 10k 실측(판정 가능한 이른 분할 135지점): 이 마진이 최대 원인이었다 —
            // 73지점·133줄·42문서. 한글은 그 자리에 줄을 두는데 rhwp 만 다음 쪽으로
            // 밀어내며, 쪽수 지표는 뒤쪽의 저장 좌표 신뢰가 흡수해 침묵한다.
            let saved_line_clears_footnote_area = saved_line_clears_footnote_area(
                page.current_footnote_height,
                page.col_count == 1,
                overflow,
                page.footnote_safety_margin,
                para.line_segs.get(li).and_then(|seg| {
                    line_seg_visible_bounds_px(seg, current_page_vpos_base.unwrap_or(0), dpi)
                }),
                page.body_height,
                page.current_height,
            );
            match saved_tail_fit_chain_decision(
                tail_fit_chain,
                saved_tail_vpos_fit,
                hwp_authoritative,
                native_hwp5_reset_tail_fits_actual_footnote_boundary,
            ) {
                SavedTailFitChainDecision::Break => break,
                SavedTailFitChainDecision::Advance => tail_fit_chain += 1,
                SavedTailFitChainDecision::NoChange => {}
            }
            if !hwp_authoritative
                && !saved_tail_vpos_fit
                && !hwpx_reset_fragment_owner
                && !native_hwp5_reset_tail_fits_actual_footnote_boundary
                && !saved_line_clears_footnote_area
            {
                break;
            }
            used_saved_tail_vpos_fit |= saved_tail_vpos_fit
                || hwpx_reset_fragment_owner
                || native_hwp5_reset_tail_fits_actual_footnote_boundary;
        }
        cumulative += fmt.line_advance(li);
        end_line = li + 1;
    }

    LineScanResult {
        end_line,
        cumulative,
        used_saved_tail_vpos_fit,
    }
}

/// HWPX의 문단 내부 `vpos=0` reset은, reset 직전 fragment가 현재 flow 앵커에서
/// 시작할 때에만 다음 물리 쪽의 시작을 뜻한다. 이 경우 reset 전 줄들은 저장된
/// 현재 쪽 fragment의 owner이므로, 일반 줄 높이 예산만으로 중간 쪽으로 분리하지
/// 않는다. 표·개체·다단·local cursor rewind는 이 계약 밖에 둔다.
pub(in crate::renderer::typeset) fn hwpx_saved_reset_fragment_matches_current_flow(
    page: &LineScanPage,
    para: &Paragraph,
    start_line: usize,
    break_line: usize,
    current_page_vpos_base: i32,
    dpi: f64,
) -> bool {
    if !page.profile.hwpx_stored_layout()
        || page.col_count != 1
        || !para.controls.is_empty()
        || start_line >= break_line
        || break_line >= para.line_segs.len()
    {
        return false;
    }

    let Some(start_bounds) = para
        .line_segs
        .get(start_line)
        .filter(|seg| !is_synthetic_line_seg(seg))
        .and_then(|seg| line_seg_visible_bounds_px(seg, current_page_vpos_base, dpi))
    else {
        return false;
    };
    if !saved_line_is_anchored_to_current_flow(start_bounds, page.current_height) {
        return false;
    }

    let mut previous_vpos = None;
    for seg in &para.line_segs[start_line..break_line] {
        if is_synthetic_line_seg(seg)
            || seg.vertical_pos <= 0
            || previous_vpos.is_some_and(|previous| seg.vertical_pos <= previous)
        {
            return false;
        }
        previous_vpos = Some(seg.vertical_pos);
    }

    para.line_segs
        .get(break_line)
        .is_some_and(|seg| !is_synthetic_line_seg(seg) && seg.vertical_pos == 0)
}
