//! 소유 문단 안의 컨트롤 배치 경로 조정.
//!
//! 저장 TAC 줄의 수용·계산은 stored_tac, 확정 좌표와 흐름 반영은 state가 소유한다.
//! 일반 TAC의 배치 전 판단은 tac_fit, 공유 판별은 tac_flow가 소유한다.
//! 컨트롤 순서와 첫/마지막 표 선택은 order가 소유한다.
//! float, 지연 배치와 표 분할 경로는 아직 상위 구현에 남아 있다.

pub(super) mod order;
pub(super) mod stored_tac;
pub(super) mod tac_fit;
pub(super) mod tac_flow;

use super::paragraph::metrics::FormattedParagraph;
use super::TypesetState;
use crate::model::paragraph::Paragraph;
use crate::renderer::height_measurer::MeasuredTable;

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
