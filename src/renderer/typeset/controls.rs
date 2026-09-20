//! 소유 문단 안의 컨트롤 배치 경로 조정.
//!
//! 저장 TAC 줄의 수용·계산은 stored_tac, 확정 좌표와 흐름 반영은 state가 소유한다.
//! 일반 TAC의 배치 전 판단은 tac_fit, 공유 판별은 tac_flow가 소유한다.
//! 컨트롤 순서와 첫/마지막 표 선택은 order가 소유한다.
//! 같은 문단의 형제 표 이월 판별·후보 선택·flush 시점 조회는 deferred가 소유한다.
//! 지연 큐 순회는 이 모듈이, 큐·vpos 상태 반영은 state가 소유한다.
//! 빈 호스트 float lane 조회는 empty_float가, 확정 예약은 state가 소유한다.
//! 표 문단의 그림·도형·수식 흐름 조회는 shape_flow가 소유하고 이 모듈이 배치를 조정한다.
//! 배치 후 TAC 높이 보정은 tac_reconcile 조회와 state 확정을 이 모듈에서 조정한다.
//! 나머지 float, 개별 지연 표의 측정·배치와 표 분할 경로는 상위 구현에 남아 있다.

pub(super) mod deferred;
pub(super) mod empty_float;
pub(super) mod order;
pub(super) mod shape_flow;
pub(super) mod stored_tac;
pub(super) mod tac_fit;
pub(super) mod tac_flow;
pub(super) mod tac_reconcile;

use super::paragraph::metrics::FormattedParagraph;
use super::{FormattedTable, TypesetState};
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::renderer::composer::ComposedParagraph;
use crate::renderer::float_placement::FloatLaneSet;
use crate::renderer::height_measurer::MeasuredTable;
use crate::renderer::hwpunit_to_px;
use crate::renderer::style_resolver::ResolvedStyleSet;

/// TAC 높이 조회 → 누락 표시/진단/무효화 → 앵커·상한 확정 순서를 보존한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn reconcile_tac_height(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    fmt: &FormattedParagraph,
    measured_tables: &[MeasuredTable],
    tac_count: usize,
    height_before: f64,
    session_grown_tac_total: Option<f64>,
    flow: tac_flow::TacFlowQuery<'_>,
) {
    let dpi = flow.dpi();
    let tac_reconcile::TacHeightCap {
        tac_seg_total,
        cap,
        stored_step_px,
        ladder_total,
        ladder_omits_spacing,
    } = tac_reconcile::measure(
        para_idx,
        para,
        next_para,
        fmt,
        measured_tables,
        tac_count,
        height_before,
        st.tac_height_page().profile,
        &flow,
        |ci, prior_tac_count, lh, ls_extra| {
            if std::env::var("RHWP_DIAG_TACSIB").is_ok() {
                eprintln!(
                    "DIAG_TACSIB pi={} ci={} line_idx={} add={:.1} first_seg={:.1}",
                    para_idx,
                    ci,
                    prior_tac_count,
                    lh + ls_extra,
                    para.line_segs
                        .first()
                        .map(|s0| hwpunit_to_px(
                            s0.line_height.saturating_add(s0.line_spacing),
                            dpi
                        ))
                        .unwrap_or(0.0),
                );
            }
        },
    );
    if ladder_omits_spacing {
        st.commit_tac_spacing_omission(|| {
            if std::env::var("RHWP_DIAG_LADSP").is_ok() {
                eprintln!(
                    "DIAG_LADSP pi={} OMIT step={:.1} cap={:.1} fmt_total={:.1} sb={:.1} sa={:.1}",
                    para_idx,
                    stored_step_px.unwrap_or(0.0),
                    cap,
                    fmt.total_height,
                    fmt.spacing_before,
                    fmt.spacing_after
                );
            }
        });
    }
    let snapped_base = tac_reconcile::snapped_base(
        para,
        height_before,
        ladder_omits_spacing,
        st.tac_height_page(),
        dpi,
    );
    let cap = tac_reconcile::effective_cap(
        cap,
        ladder_total,
        ladder_omits_spacing,
        session_grown_tac_total,
    );
    if std::env::var("RHWP_DIAG_TACCAP").is_ok() {
        eprintln!(
            "DIAG_TACCAP pi={} tac_seg_total={:.1} cap={:.1} fmt_total={:.1} sb={:.1} cur_h={:.1} snapped_base={:.1} clamp={}",
            para_idx,
            tac_seg_total,
            cap,
            fmt.total_height,
            fmt.spacing_before,
            st.tac_height_page().current_height,
            snapped_base,
            st.tac_height_page().current_height - snapped_base > cap
        );
    }

    let capped_bottom =
        tac_reconcile::capped_bottom(para_idx, snapped_base, cap, st.tac_height_page());
    st.commit_tac_capped_bottom(capped_bottom);
}

/// 표 문단의 비표 개체 배치 순서만 조정한다. 뒤쪽 문단 높이 보정은 호출자에 남는다.
#[allow(clippy::too_many_arguments)]
pub(super) fn place_table_host_shape(
    st: &mut TypesetState,
    para_idx: usize,
    ctrl_idx: usize,
    ctrl: &Control,
    para: &Paragraph,
    para_start_height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    // [#6146] 저장 리셋 경계에서 떠나는 쪽의 흐름 말미에 이미 흘려
    // 놓은 자리차지 밴드는 다시 배치하지 않는다.
    if st.has_spilled_page_tail_float(para_idx, ctrl_idx) {
        return;
    }
    let flow = shape_flow::prepare(ctrl, para, ctrl_idx, dpi);
    if flow.needs_advance(st.table_host_shape_page(), || st.available_height()) {
        st.advance_column_or_new_page();
    }
    st.commit_table_host_shape(para_idx, ctrl_idx, flow);
    st.register_side_wrap_picture(para_idx, ctrl_idx, para, Some(para_start_height), styles);
}

/// 조회가 후보를 수용한 경우에만 표 항목·lane·흐름 상태를 함께 반영한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn try_place_empty_para_float_table(
    st: &mut TypesetState,
    para_idx: usize,
    ctrl_idx: usize,
    para: &Paragraph,
    table: &crate::model::table::Table,
    ft: &FormattedTable,
    composed: Option<&ComposedParagraph>,
    next_para: Option<&Paragraph>,
    styles: &ResolvedStyleSet,
    para_start_height: f64,
    lanes: &mut FloatLaneSet,
    dpi: f64,
) -> bool {
    let Some(placement) = empty_float::prepare(
        para_idx,
        ctrl_idx,
        para,
        table,
        ft,
        composed,
        next_para,
        styles,
        para_start_height,
        lanes,
        st.empty_float_page(),
        || st.empty_float_available_height(ft.table_footnote_height, ft.table_footnote_count),
        dpi,
    ) else {
        return false;
    };
    st.commit_empty_float_table(para_idx, ctrl_idx, placement, lanes);
    true
}

/// 큐 처리 순서만 조정한다. 표 하나의 측정·배치는 기존 엔진 경로가 담당한다.
pub(super) fn flush_deferred_tables(
    st: &mut TypesetState,
    paragraphs: &[Paragraph],
    flush_point: deferred::DeferredTableFlushPoint,
    mut place: impl FnMut(&mut TypesetState, deferred::DeferredTableControl),
) {
    if !st.has_deferred_table_controls() {
        return;
    }
    let pending = st.take_deferred_table_controls();
    let mut remaining = Vec::new();
    for deferred in pending {
        let keep_pending = flush_point.keeps_pending(&deferred, paragraphs);
        if keep_pending {
            remaining.push(deferred);
            continue;
        }
        place(st, deferred);
    }
    st.restore_deferred_table_controls(remaining);
}

/// Query 결과를 확정한 뒤에만 기존 페이지 전이를 수행한다.
pub(super) fn prepare_tac_paragraph(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    measured_tables: &[MeasuredTable],
    flow: tac_flow::TacFlowQuery<'_>,
) -> tac_fit::TacFitPlan {
    let plan = tac_fit::prepare(
        para_idx,
        para,
        fmt,
        measured_tables,
        st.tac_fit_page(),
        || st.available_height(),
        flow,
    );
    if plan.advance_before_place {
        st.advance_column_or_new_page();
    }
    plan
}

/// 기존 저장 줄 경로가 문단 전체를 수용한 경우에만 일반 컨트롤 경로를 생략한다.
pub(super) fn try_place_stored_tac_paragraph(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    measured_tables: &[MeasuredTable],
    dpi: f64,
) -> bool {
    let Some(plan) = stored_tac::prepare(
        para_idx,
        para,
        fmt,
        measured_tables,
        st.stored_tac_page(),
        || st.available_height(),
        dpi,
    ) else {
        return false;
    };
    for line in &plan.lines {
        let placement = plan.placement(line, fmt.spacing_after, dpi);
        st.commit_stored_tac_control(para_idx, placement);
    }
    true
}
