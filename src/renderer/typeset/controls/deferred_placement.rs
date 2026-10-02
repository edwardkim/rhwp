//! 지연 큐에서 선택된 표 하나의 재조회·포맷·배치 조정.
//! 후보 판별은 deferred, 큐 순회는 controls, 각주 연결은 notes가 담당한다.
//! 표 포맷/분할 알고리즘은 기존 엔진을 호출하며 상태는 기존 시점에 관측한다.

use super::super::{notes, TypesetEngine, TypesetState};
use super::deferred::{CoanchoredTableQuery, DeferredTableControl};
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::{
    composer::ComposedParagraph, height_measurer::MeasuredTable, style_resolver::ResolvedStyleSet,
};

pub(in crate::renderer::typeset) fn place(
    engine: &TypesetEngine,
    st: &mut TypesetState,
    deferred: DeferredTableControl,
    paragraphs: &[Paragraph],
    composed: &[ComposedParagraph],
    styles: &ResolvedStyleSet,
    measured_tables: &[MeasuredTable],
) {
    let Some(para) = paragraphs.get(deferred.para_index) else {
        return;
    };
    let Some(Control::Table(table)) = para.controls.get(deferred.control_index) else {
        return;
    };

    let host_col_w = st.deferred_table_column_width();
    let composed_para = composed.get(deferred.para_index);
    let fmt = engine.format_paragraph(para, composed_para, styles, Some(host_col_w));
    if !CoanchoredTableQuery::new(para, &fmt, engine.tac_flow_query())
        .is_deferred_coanchored_rowbreak_table(table)
    {
        return;
    }

    let is_column_top = st.flow_table_column_top();
    let ft = engine.format_table(
        para,
        deferred.para_index,
        deferred.control_index,
        table,
        measured_tables,
        styles,
        composed_para,
        paragraphs.get(deferred.para_index + 1),
        is_column_top,
    );
    let mt = measured_tables.iter().find(|mt| {
        mt.para_index == deferred.para_index && mt.control_index == deferred.control_index
    });
    let para_start_height = st.deferred_table_anchor_height();

    engine.typeset_block_table(
        st,
        deferred.para_index,
        deferred.control_index,
        para,
        table,
        &ft,
        &fmt,
        mt,
        styles,
        para_start_height,
        // [Task #1860] 예산 전용 참 para_start(원 배치 시점). 지연 배치의
        // current_height 는 선행 캡션을 이미 반영하므로 out-of-flow float
        // 예산이 이중차감된다. 렌더 위치(para_start_height)는 불변 유지하고
        // 예산 계산에만 이 값을 쓴다.
        deferred.para_start_height,
        deferred.is_first_placed,
        deferred.is_last_placed,
        paragraphs,
        composed,
    );

    notes::register_unqueued_table_cells(
        st,
        table,
        deferred.para_index,
        deferred.control_index,
        |st, footnote, source| {
            engine.register_unqueued_table_footnote(st, footnote, source);
        },
    );

    st.commit_deferred_table_anchor(deferred.para_index);
}
