//! TAC 표 문단의 배치 후 높이 상한과 저장 사다리 보정 조회.
//! 기존 산식과 선택 순서를 보존한다. 진단 callback 외에는 외부 효과를 갖지 않는다.
use super::super::paragraph::metrics::FormattedParagraph;
use super::tac_flow::TacFlowQuery;
use crate::model::{
    control::Control, paragraph::Paragraph, provenance::LayoutCompatibilityProfile,
};
use crate::renderer::{
    float_placement::InlineBoxPlacement, height_measurer::MeasuredTable, hwpunit_to_px,
};
use std::collections::HashMap;

pub(in crate::renderer::typeset) struct TacHeightPage<'a> {
    pub profile: LayoutCompatibilityProfile,
    pub current_height: f64,
    pub vpos_page_base: Option<i32>,
    pub vpos_col_anchor: f64,
    pub inline_placements: &'a HashMap<(usize, usize), InlineBoxPlacement>,
    pub inline_box_flow_bottom: f64,
}
pub(super) struct TacHeightCap {
    pub tac_seg_total: f64,
    pub cap: f64,
    pub stored_step_px: Option<f64>,
    pub ladder_total: f64,
    pub ladder_omits_spacing: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measure(
    para_idx: usize,
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    fmt: &FormattedParagraph,
    measured_tables: &[MeasuredTable],
    tac_count: usize,
    height_before: f64,
    profile: LayoutCompatibilityProfile,
    flow: &TacFlowQuery<'_>,
    mut trace_sibling: impl FnMut(usize, usize, f64, f64),
) -> TacHeightCap {
    let dpi = flow.dpi();
    // tac_seg_total 계산: 각 TAC 표의 max(seg.lh, 실측높이) + ls/2
    let mut tac_seg_total = 0.0;
    let mut tac_idx = 0;
    for (ci, c) in para.controls.iter().enumerate() {
        if let Control::Table(t) = c {
            if flow.is_effective_tac_table(para, t, fmt) {
                if let Some(seg) = para.line_segs.get(tac_idx) {
                    let seg_lh = hwpunit_to_px(seg.line_height, dpi);
                    let mt_h = measured_tables
                        .iter()
                        .find(|mt| mt.para_index == para_idx && mt.control_index == ci)
                        .map(|mt| mt.total_height)
                        .unwrap_or(0.0);
                    let effective_h = crate::renderer::tac_table_effective_height(seg_lh, mt_h);
                    let ls_half = hwpunit_to_px(seg.line_spacing, dpi) / 2.0;
                    tac_seg_total += effective_h + ls_half;
                }
                tac_idx += 1;
            }
        }
    }
    let has_owned_rowbreak_tac_frame = profile.hwpx_stored_layout()
        && tac_count == 1
        && fmt.line_heights.len() == 1
        && para
            .controls
            .iter()
            .enumerate()
            .any(|(control_index, control)| {
                matches!(control, Control::Table(table)
                if flow.is_effective_tac_table(para, table, fmt))
                    && crate::renderer::composer::owned_rowbreak_tac_height(para, control_index)
                        .is_some()
            });
    // [#3738] 위 합은 표만 센다. 그런데 같은 문단의 **선행 자리차지 개체가
    // 있는** TAC 그림/글상자는 Task #402 경로가 자기 line_seg 만큼 이미
    // current_height 에 더했다(위 13980 블록). cap 이 그 줄을 빼놓으면
    // 방금 더한 만큼을 도로 되감아, 표 뒤 개체가 높이 0 으로 계상된다
    // (1351000 정책연구용역 중간보고서 pi=1920: 표+글상자인데 cap=447.0 이
    // 글상자 222.1px 를 삭제 → 문단 10개가 쪽 밖으로 밀림). 가산 조건을
    // Task #402 와 글자 그대로 맞춘다.
    for (ci, c) in para.controls.iter().enumerate() {
        let is_tac_pic_or_shape = match c {
            Control::Picture(p) => p.common.treat_as_char,
            Control::Shape(s) => s.common().treat_as_char,
            _ => false,
        };
        if !is_tac_pic_or_shape {
            continue;
        }
        let prior_tac_count = para
            .controls
            .iter()
            .take(ci)
            .filter(|c| match c {
                Control::Table(t) => t.common.treat_as_char,
                Control::Picture(p) => p.common.treat_as_char,
                Control::Shape(s) => s.common().treat_as_char,
                _ => false,
            })
            .count();
        if prior_tac_count == 0 {
            continue;
        }
        if let Some(seg) = para.line_segs.get(prior_tac_count) {
            let lh = hwpunit_to_px(seg.line_height, dpi);
            let ls_extra = if seg.line_spacing > 0 {
                hwpunit_to_px(seg.line_spacing, dpi)
            } else {
                0.0
            };
            tac_seg_total += lh + ls_extra;
            trace_sibling(ci, prior_tac_count, lh, ls_extra);
        }
    }
    let cap = if tac_seg_total > 0.0 {
        let is_col_top = height_before < 1.0;
        let effective_sb = if is_col_top { 0.0 } else { fmt.spacing_before };
        let outer_top: f64 = para
            .controls
            .iter()
            .filter_map(|c| match c {
                Control::Table(t) if flow.is_effective_tac_table(para, t, fmt) => {
                    Some(hwpunit_to_px(t.outer_margin_top as i32, dpi))
                }
                _ => None,
            })
            .sum();
        let owned_row_total = effective_sb + outer_top + tac_seg_total;
        if has_owned_rowbreak_tac_frame {
            owned_row_total
        } else {
            owned_row_total.min(fmt.total_height)
        }
    } else {
        fmt.total_height
    };
    // [#2279 누적Δ] 저장 ladder 가 host paraPr spacing 을 누락한 기계생성
    // 결재문서(HWPX, 빈 host TAC 표): 저장 스텝(다음 문단 vpos−현 vpos)이
    // fmt.total_height(sb+lh+ls+sa)보다 짧으면 생성기가 sa/sb 를 좌표에
    // 반영하지 않은 것이다 — 한글 fresh 는 전량 순수 가산(36399374 재저장
    // 오라클: pi4 +500(sa)+300(sb), pi5 +300(sb), fmt.total 과 HU 단위
    // 일치). 이때 cap 을 fmt.total 로 올리고 ladder 를 dirty 로 표시해
    // 후속 스냅이 성장분을 압축 anchor 로 되감지 못하게 한다. 스텝이
    // fmt.total 과 일치(±1px)하는 정상 생성기 ladder 는 불변.
    let stored_step_px = if profile.hwpx_stored_layout() && para.text.is_empty() {
        para.line_segs
            .first()
            .filter(|s| s.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
            .zip(next_para.and_then(|np| np.line_segs.first()).filter(|s| {
                s.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            }))
            .map(|(cur, next)| {
                hwpunit_to_px(
                    (next.vertical_pos as i64 - cur.vertical_pos as i64) as i32,
                    dpi,
                )
            })
            .filter(|&s| s > 0.0)
    } else {
        None
    };
    // 저장 스텝이 fmt.total(sb+lh+ls+sa)보다 짧고 그 부족분이 host
    // paraPr spacing(sb+sa)과 정확히 일치하면, 생성기가 sb/sa 를 좌표에
    // 반영하지 않은 ladder 다 — 한글 fresh 는 전량 순수 가산한다
    // (36399374 재저장 오라클: pi4 부족분 6.7px=sa, pi5 4.0px=sb, HU
    // 단위 일치). cap 을 fmt.total 로 올리고 ladder 를 dirty 로 표시해
    // 후속 스냅이 성장분을 압축 anchor 로 되감지 못하게 한다.
    // 부족분이 sb/sa 서명과 다른 ladder(#2352 host 줄박스 계열,
    // 36392557 pi27 부족분 13.3px)는 대상이 아니다 — 그 경로의 별도
    // 가산과 이중 성장(+1쪽 회귀 실측)한다.
    // 🔴 지문은 **생성기가 쓴 값**과 대조하는 것이라, 우리가 예산용으로 더한
    // TAC 바깥 여백은 빼고 비교해야 한다(창이 ±2px 인데 그 몫이 3.7px 다).
    let ladder_total = fmt.total_height - fmt.tac_outer_margin_v_px;
    let ladder_omits_spacing = stored_step_px
        .filter(|&step| step + 1.0 < ladder_total && cap + 1.0 < ladder_total)
        .map(|step| {
            let shortfall = ladder_total - step;
            let sig = fmt.spacing_before + fmt.spacing_after;
            sig > 0.5 && (shortfall - sig).abs() < 2.0
        })
        .unwrap_or(false);

    TacHeightCap {
        tac_seg_total,
        cap,
        stored_step_px,
        ladder_total,
        ladder_omits_spacing,
    }
}

pub(super) fn snapped_base(
    para: &Paragraph,
    height_before: f64,
    ladder_omits_spacing: bool,
    page: TacHeightPage<'_>,
    dpi: f64,
) -> f64 {
    // 성장 전에 저장 anchor 로 전방 사전-스냅 — 직전 표들이 cap
    // (lh+ls/2)으로 ladder 보다 짧게 전진한 잔차(36399374 pi3 −3.1px)를
    // 표→표 연쇄(스냅 부재 구간)에서 회수한다. 전방·소폭 한정.
    if ladder_omits_spacing {
        para.line_segs
            .first()
            .zip(page.vpos_page_base)
            .map(|(seg0, base)| {
                let implied = page.vpos_col_anchor
                    + hwpunit_to_px((seg0.vertical_pos as i64 - base as i64) as i32, dpi);
                let delta = implied - height_before;
                if delta > 0.0 && delta < 24.0 {
                    implied
                } else {
                    height_before
                }
            })
            .unwrap_or(height_before)
    } else {
        height_before
    }
}

pub(super) fn effective_cap(
    cap: f64,
    ladder_total: f64,
    ladder_omits_spacing: bool,
    session_grown_tac_total: Option<f64>,
) -> f64 {
    let cap = if ladder_omits_spacing {
        ladder_total
    } else {
        cap
    };
    // [편집 세션] 셀 편집으로 자란 TAC 표는 실측 소비가 저장 줄 기반
    // cap 을 정당하게 넘는다 — cap 으로 되감으면 후행 문단이 성장분만큼
    // 안 밀려 쪽 하단을 넘긴다(셀 Enter 재현: 후행 안내 문단 잘림).
    session_grown_tac_total.map_or(cap, |grown| cap.max(grown))
}

pub(super) fn capped_bottom(
    para_idx: usize,
    snapped_base: f64,
    cap: f64,
    page: TacHeightPage<'_>,
) -> f64 {
    let side_wrap_clearance: f64 = page
        .inline_placements
        .iter()
        .filter(|((pi, _), _)| *pi == para_idx)
        .map(|(_, placement)| placement.clearance)
        .sum();
    let capped_bottom = snapped_base + cap + side_wrap_clearance;
    // 저장 host가 4px여도 회피 줄의 실측 표 높이는 되돌릴 수 없다.
    // 뒤 표까지 같은 단에서 물리 하단을 이어야 렌더와 fit의 cursor가 같다.
    if page.inline_placements.is_empty() {
        capped_bottom
    } else {
        capped_bottom.max(page.inline_box_flow_bottom)
    }
}
