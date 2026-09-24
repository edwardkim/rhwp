//! 표를 소유한 문단의 진입·컨트롤 순회·후처리 조정.
//! 개별 경로의 조회/배치 담당자를 순서대로 연결하며 새 조판 규칙은 정의하지 않는다.

use super::super::{controls, notes, TypesetEngine, TypesetState};
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::{
    composer::ComposedParagraph, float_placement::FloatLaneSet, height_measurer::MeasuredTable,
    style_resolver::ResolvedStyleSet,
};

pub(in crate::renderer::typeset) struct TableParagraphInput<'a> {
    pub para_idx: usize,
    pub para: &'a Paragraph,
    pub composed: Option<&'a ComposedParagraph>,
    pub next_para: Option<&'a Paragraph>,
    pub styles: &'a ResolvedStyleSet,
    pub measured_tables: &'a [MeasuredTable],
    pub paragraphs_all: &'a [Paragraph],
    pub composed_all: &'a [ComposedParagraph],
}

pub(in crate::renderer::typeset) fn place(
    engine: &TypesetEngine,
    st: &mut TypesetState,
    input: TableParagraphInput<'_>,
) {
    let TableParagraphInput {
        para_idx,
        para,
        composed,
        next_para,
        styles,
        measured_tables,
        paragraphs_all,
        composed_all,
    } = input;
    st.trace_table_paragraph_entry(para_idx);
    // [#2322] 자리차지 float(양수 v_off) 표가 만든 배타 영역을 표 문단 경로도
    // 소비한다 — 종전에는 텍스트 경로(typeset_paragraph)만 소비해, 후속 표
    // 문단의 블록 표 fit 이 존 위에 겹쳐 배치됐다 (19439117: 870px 서식 표
    // 존 [31..902] 위에 866px 표가 y≈36 에 통배치 → 1쪽, 한글 2쪽).
    let host_col_w = st.prepare_table_paragraph_column();
    let fmt = engine.format_paragraph(para, composed, styles, Some(host_col_w));
    if controls::try_place_stored_tac_paragraph(
        st,
        para_idx,
        para,
        &fmt,
        measured_tables,
        engine.dpi,
    ) {
        return;
    }
    let controls::tac_fit::TacFitPlan {
        tac_count,
        has_tac,
        session_grown_tac_total,
        ..
    } = controls::prepare_tac_paragraph(
        st,
        para_idx,
        para,
        &fmt,
        measured_tables,
        engine.tac_flow_query(),
    );

    st.ensure_page();

    let (height_before, page_count_before) = st.table_paragraph_flow_position();
    let para_start_height = height_before;
    let mut para_float_lanes = FloatLaneSet::new();

    let controls::order::ControlPlacementOrder {
        ctrl_order,
        first_placed_table,
        last_placed_table,
    } = controls::order::for_paragraph(para, &fmt, engine.tac_flow_query());

    // [#703 잔여] 데코레이션(글앞/글뒤) 표 단축은 표만 방출하고 흐름을 0
    // 소비했다. host 문단에 제목 등 가시 텍스트가 있으면 그 텍스트가
    // 발행되지 않아 렌더에서 통째로 사라지고(제목 미노출), 텍스트 높이가
    // 흐름에 안 실려 다음 표가 위로 붙는다(표 틀어짐). 단축 진입 시 host
    // 텍스트를 한 번 발행하도록 문단 단위로 추적한다.
    let mut decoration_host_text_pending = false;
    let mut flow_table_owns_host_text = false;
    for (order_pos, ctrl_idx) in ctrl_order.iter().copied().enumerate() {
        let ctrl = &para.controls[ctrl_idx];
        match ctrl {
            Control::Table(table) => {
                if controls::prepare_table_control(
                    st,
                    para,
                    table,
                    para_idx,
                    ctrl_idx,
                    order_pos,
                    next_para,
                    measured_tables,
                    has_tac,
                    host_col_w,
                    engine.dpi,
                    || !engine.profile.get().hwp5_stored_pagination_layout(),
                ) {
                    controls::place_decoration_table(
                        st,
                        para_idx,
                        ctrl_idx,
                        para,
                        table,
                        next_para,
                        engine.dpi,
                        |is_column_top| {
                            engine.format_table(
                                para,
                                para_idx,
                                ctrl_idx,
                                table,
                                measured_tables,
                                styles,
                                composed,
                                next_para,
                                is_column_top,
                            )
                        },
                    );
                    // [#703 잔여] host 문단의 가시 텍스트(제목 등)를 흐름 문단으로
                    // 방출한다. 종전에는 표만 방출하고 `continue` 해서 이 텍스트를
                    // 위한 PageItem 이 어디에서도 발행되지 않아 렌더에서 통째로
                    // 사라졌고(제목 미노출), 텍스트 높이가 흐름에 실리지 않아 뒤
                    // 문단·표가 제목 자리로 올라붙었다(표 겹침).
                    //
                    // 방출은 **표 배치와 흐름 전진을 모두 끝낸 뒤**여야 한다 —
                    // 이 표의 앵커(위 anchor_y)는 vert=Para 의 문단 시작 기준
                    // 좌표라, 텍스트를 먼저 전진시키면 글앞으로 표가 제목 높이만큼
                    // 아래로 밀려 본문표를 파고든다(실측 +44px 겹침).
                    // PartialParagraph = 텍스트 줄만이고 표는 Shape 가 따로
                    // 그리므로 중복 렌더는 없다(place_table_with_text 의 pre-text
                    // 발행과 같은 계약).
                    // Defer host text until every co-anchored table has
                    // computed its anchor and overlay continuation bounds.
                    decoration_host_text_pending = true;
                    continue;
                }
                // Ordinary/TAC table paths already own host text, including
                // their deferred emission and layout fallback. Do not add a
                // second full-range PartialParagraph for mixed controls.
                flow_table_owns_host_text = true;
                let break_after_current_table = controls::flow_table::place(
                    engine,
                    st,
                    controls::flow_table::FlowTableInput {
                        para_idx,
                        ctrl_idx,
                        para,
                        table,
                        fmt: &fmt,
                        measured_tables,
                        styles,
                        composed,
                        next_para,
                        tac_count,
                        first_placed_table,
                        last_placed_table,
                        para_start_height,
                        paragraphs_all,
                        composed_all,
                        ctrl_order: &ctrl_order,
                        order_pos,
                    },
                    &mut para_float_lanes,
                );

                notes::register_unqueued_table_cells(
                    st,
                    table,
                    para_idx,
                    ctrl_idx,
                    |st, footnote, source| {
                        engine.register_unqueued_table_footnote(st, footnote, source);
                    },
                );
                if break_after_current_table {
                    break;
                }
            }
            Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
                controls::place_table_host_shape(
                    st,
                    para_idx,
                    ctrl_idx,
                    ctrl,
                    para,
                    para_start_height,
                    styles,
                    engine.dpi,
                );
            }
            _ => {}
        }
    }

    // Emit host text once after every control has resolved its anchor, before TAC reconciliation.
    controls::place_decoration_host_text(
        st,
        para_idx,
        para,
        next_para,
        &fmt,
        decoration_host_text_pending,
        flow_table_owns_host_text,
        styles,
        engine.dpi,
        || {
            engine.profile.get().hwp5_stored_pagination_layout()
                || engine.profile.get().hwpx_stored_layout()
        },
    );

    // TAC 표 높이 보정 (Paginator engine.rs:123-179 동일)
    // [#2319] 저장 lineseg 없는(기계생성) 문단은 스킵 — cap 의 두 축(tac_seg_total
    // 의 seg.lh, fallback 의 fmt.total_height)이 모두 lineseg/컴포즈에 표 높이가
    // 반영돼 있음을 전제한다. lineseg 없는 텍스트-host 문단에서는 fmt 가 표를
    // 모르므로 cap 이 측정 높이(예: 858px)를 텍스트 줄합(34.7px)으로 되감아
    // 서식 문서 과소분할을 만든다 (20544835 r15 재검증 −1 계열).
    if has_tac
        && fmt.total_height > 0.0
        && !para.line_segs.is_empty()
        && st.flow_table_page_count() == page_count_before
    {
        controls::reconcile_tac_height(
            st,
            para_idx,
            para,
            next_para,
            &fmt,
            measured_tables,
            tac_count,
            height_before,
            session_grown_tac_total,
            engine.tac_flow_query(),
        );
    }
}
