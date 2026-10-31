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

fn try_place_reflowed_table_rows_with_storage<S: crate::storage::Storage>(
    engine: &TypesetEngine,
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    measured_tables: &[MeasuredTable],
    storage: &S,
) -> Result<bool, S::Error> {
    if !crate::renderer::inline_flow::supports_table_text_rows_with_storage(para, storage)?
        && !crate::renderer::inline_flow::supports_page_square_table_rows(para)
    {
        return Ok(false);
    }
    let build = |st: &TypesetState, start: f64, preceding: bool| {
        crate::renderer::typeset::inline_flow::plan::build_plan_with_storage(
            st.inline_flow_input(start, preceding),
            para,
            para_idx,
            styles,
            measured_tables,
            engine.dpi,
            storage,
        )
    };
    let Some(mut plan) = build(st, st.current_height, true)? else {
        return Ok(false);
    };
    // Fragmented tables and footnote reservation retain their existing owner.
    // A fitting prefix must stay in this column, even if later rows need a cut.
    if plan.occupied_end() > st.available_height() {
        // 단독 행 나눔 표도 전체가 들어가면 공백과 같은 물리 줄을 공유한다.
        // 들어가지 않는 조각은 상태를 바꾸기 전에 기존 분할 소유자에게 돌려준다.
        if crate::renderer::inline_flow::standalone_table_fragment_owner_with_storage(
            para, storage,
        )? {
            return Ok(false);
        }
        if st.current_items.is_empty()
            || plan
                .rows
                .first()
                .is_none_or(|row| row.y + row.height <= st.available_height())
        {
            return Ok(false);
        }
        let Some(candidate) = build(st, 0.0, false)? else {
            return Ok(false);
        };
        if candidate.occupied_end() > st.base_available_height() {
            return Ok(false);
        }
        st.advance_column_or_new_page_with_storage(storage)?;
        let Some(candidate) = build(st, st.current_height, true)? else {
            return Ok(false);
        };
        if candidate.occupied_end() > st.available_height() {
            return Ok(false);
        }
        plan = candidate;
    }
    st.commit_inline_flow_with_storage(para_idx, plan, storage)?;
    for (control, item) in para.controls.iter().enumerate() {
        if let Control::Table(table) = item {
            st.register_page_anchored_square_table_with_storage(
                para_idx, control, para, table, storage,
            )?;
        }
    }
    Ok(true)
}

pub(in crate::renderer::typeset) fn place(
    engine: &TypesetEngine,
    st: &mut TypesetState,
    input: TableParagraphInput<'_>,
) {
    match place_with_storage(engine, st, input, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(in crate::renderer::typeset) fn place_with_storage<S: crate::storage::Storage>(
    engine: &TypesetEngine,
    st: &mut TypesetState,
    input: TableParagraphInput<'_>,
    storage: &S,
) -> Result<(), S::Error> {
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
    if st.place_mixed_host_line(para_idx, para, styles, measured_tables, engine.dpi) {
        return Ok(());
    }
    if try_place_reflowed_table_rows_with_storage(
        engine,
        st,
        para_idx,
        para,
        styles,
        measured_tables,
        storage,
    )? {
        return Ok(());
    }
    let mut fmt = engine.format_paragraph_for_flow_with_storage(
        para,
        composed,
        styles,
        Some(host_col_w),
        styles.hwp3_variant,
        false,
        storage,
    )?;
    // 문서의 다른 개체를 편집해도 이 호스트의 저장 줄과 현재 표 프레임은 유효할 수 있다.
    // 계획이 현재 측정 높이와 앞뒤 저장 원점을 검증하므로 문서 전체 편집 플래그로 거절하지 않는다.
    let stored_body_wide_host = st
        .profile
        .hwp5_container()
        .then(|| st.stored_body_wide_host_frame())
        .flatten()
        .and_then(|(body, page_height)| {
            crate::renderer::inline_flow::plan_stored_body_wide_host(
                para,
                next_para?,
                styles,
                &body,
                page_height,
                engine.dpi,
                para_idx,
                measured_tables,
            )
        });

    if let Some(plan) = crate::renderer::inline_flow::plan_square_table_host(
        para,
        para_idx,
        styles,
        measured_tables,
        host_col_w,
        engine.dpi,
    ) {
        if let Some(origin) = super::square_host_frame::stored_host_origin(
            st,
            para_idx,
            paragraphs_all,
            composed_all,
            engine.dpi,
        ) {
            // fit 전에 같은 원점을 선택한다. 이월 뒤에는 새 프레임의 흐름을 사용한다.
            st.align_flow_to(origin);
        }
        fmt.use_square_host_plan(plan, engine.dpi);
    }
    engine.observe_stored_table_origin(st, para_idx, paragraphs_all, composed_all, styles);
    if controls::try_place_stored_tac_paragraph(
        st,
        para_idx,
        para,
        &fmt,
        measured_tables,
        engine,
        paragraphs_all,
        styles,
        composed,
    ) {
        return Ok(());
    }
    let controls::tac_fit::TacFitPlan {
        tac_count,
        has_tac,
        measured_tac_floor,
        ..
    } = controls::prepare_tac_paragraph(
        st,
        para_idx,
        para,
        &fmt,
        measured_tables,
        engine.tac_flow_query(),
    );

    st.ensure_page_with_storage(storage)?;
    // 저장 줄 없는 글앞/글뒤 표 host(서식 머리의 제목과 작은 표)는 자기 줄과 거기 앵커된
    // 표가 함께 남은 자리에 들어가야 한다. 데코 경로는 표·host 줄을 fit 없이 방출하므로
    // 쪽 끝에서 host 만 남으면 표가 꼬리 글자와 겹치거나 작은 표의 행이 쪽 경계에서
    // 쪼개진다. 한/글은 이 블록을 통째로 다음 쪽에 둔다. 표가 쪽 하나보다 크면 넘겨도
    // 들어가지 않으므로 그 자리에 둔다.
    // 묶음은 host 줄 안에서 시작하는 표만이다 — 문단에서 멀리 떨어진 위치 지정 표는
    // host 줄과 한 블록이 아니어서 그 높이로 host 를 새 쪽으로 밀면 한/글에 없는 쪽이
    // 생긴다.
    if crate::renderer::para_is_unstored_overlay_table_host(para)
        && st.table_control_page().has_items
    {
        let need = para
            .controls
            .iter()
            .filter_map(|c| match c {
                Control::Table(t)
                    if matches!(t.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                        && crate::renderer::hwpunit_to_px(
                            t.common.vertical_offset as i32,
                            engine.dpi,
                        ) < fmt.height_for_fit =>
                {
                    Some(crate::renderer::hwpunit_to_px(
                        t.common.vertical_offset as i32 + t.common.height as i32,
                        engine.dpi,
                    ))
                }
                _ => None,
            })
            .fold(fmt.height_for_fit, f64::max);
        if st.decoration_table_flow_height() + need > st.available_height()
            && need <= st.base_available_height()
        {
            st.advance_column_or_new_page_with_storage(storage)?;
        }
    }
    // 같은 서식 머리의 자리차지 작은 표(host 줄 바로 아래에서 시작)도 host 줄과
    // 한 블록이다. 쪽 끝에서 블록이 안 들어가면 표만 다음 쪽으로 넘어가 host 줄(제목)이
    // 어느 쪽에도 발행되지 않거나, 작은 표의 행이 쪽 경계에서 쪼개진다. 한/글은 제목과
    // 작은 표를 통째로 다음 쪽 위에 둔다. 묶음이 쪽 하나보다 크면 넘겨도 들어가지 않으므로
    // 그 자리에 둔다.
    // 묶음은 표가 host 줄 높이 아래(오프셋 ≥ 줄 fit 높이)에서 시작해 줄과 표가 세로로
    // 쌓이는 형상만이다. 표가 host 줄과 겹쳐 시작하면 표 경로가 이미 통째로 넘기므로,
    // 여기서 앞서 쪽을 넘기면 다음 서식 머리 표의 행이 쪽 끝에서 쪼개진다.
    // host 가 제목 글자를 가질 때만 묶는다. 빈 host 줄은 앞 쪽 끝에 남고 작은 표만 다음 쪽
    // 맨 위(본문 상단 + 바깥여백)로 간다 — 빈 줄까지 넘기면 표가 오프셋만큼 내려가 뒤
    // 본문 표가 쪽을 넘는다.
    if !has_tac
        && para.text.chars().any(|c| !c.is_whitespace())
        && crate::renderer::para_has_no_stored_line_segs(para)
        && st.table_control_page().has_items
    {
        let px = |hu: i32| crate::renderer::hwpunit_to_px(hu, engine.dpi);
        let block_tables: Vec<_> =
            storage.collect(para.controls.iter().filter_map(|c| match c {
                Control::Table(t) => Some(t),
                _ => None,
            }))?;
        let header_block_need = if block_tables.is_empty() {
            None
        } else {
            block_tables.iter().try_fold(fmt.height_for_fit, |need, t| {
                let v_off = px(t.common.vertical_offset as i32);
                (!t.common.treat_as_char
                    && matches!(
                        t.common.text_wrap,
                        crate::model::shape::TextWrap::TopAndBottom
                    )
                    && matches!(t.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                    && (fmt.height_for_fit..fmt.total_height).contains(&v_off))
                .then(|| {
                    need.max(
                        v_off
                            + px(t.outer_margin_top as i32)
                            + px(t.common.height as i32)
                            + px(t.outer_margin_bottom as i32),
                    )
                })
            })
        };
        if let Some(need) = header_block_need {
            if st.decoration_table_flow_height() + need > st.available_height()
                && need <= st.base_available_height()
            {
                st.advance_column_or_new_page_with_storage(storage)?;
            }
        }
    }

    // A candidate rejected in the previous column does not describe the new
    // column. Recompose at its origin so fit and paint still consume one plan.
    if try_place_reflowed_table_rows_with_storage(
        engine,
        st,
        para_idx,
        para,
        styles,
        measured_tables,
        storage,
    )? {
        return Ok(());
    }

    // pre-fit가 단을 넘겼으면 그 단의 원점·예산으로 확정 줄 배치를 다시 계산한다.
    // 앞 단에서 거절된 후보를 이유로 새 단에서도 legacy 높이 상한을 쓰지 않는다.
    engine.observe_stored_table_origin(st, para_idx, paragraphs_all, composed_all, styles);
    if controls::try_place_stored_tac_paragraph(
        st,
        para_idx,
        para,
        &fmt,
        measured_tables,
        engine,
        paragraphs_all,
        styles,
        composed,
    ) {
        return Ok(());
    }

    let (height_before, page_count_before) = st.table_paragraph_flow_position();
    let para_start_height = height_before;
    let mut para_float_lanes = FloatLaneSet::new();

    let controls::order::ControlPlacementOrder {
        ctrl_order,
        first_placed_table,
        last_placed_table,
    } = controls::order::for_paragraph_with_storage(
        para,
        &fmt,
        engine.tac_flow_query(),
        st.base_available_height(),
        storage,
    )?;

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
                if controls::prepare_table_control_with_storage(
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
                    storage,
                )? {
                    controls::place_decoration_table_with_storage(
                        st,
                        para_idx,
                        ctrl_idx,
                        para,
                        table,
                        next_para,
                        paragraphs_all,
                        engine.dpi,
                        None,
                        |is_column_top| {
                            engine.format_table_with_storage(
                                para,
                                para_idx,
                                ctrl_idx,
                                table,
                                measured_tables,
                                styles,
                                composed,
                                next_para,
                                is_column_top,
                                storage,
                            )
                        },
                        engine,
                        styles,
                        storage,
                    )?;
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
                    // 모든 공동 앵커 표가 원점과 이월 경계를 계산한 뒤
                    // 호스트 글줄을 한 번 방출한다.
                    decoration_host_text_pending = true;
                    continue;
                }
                // 일반/TAC 경로는 지연 방출과 배치 폴백까지 호스트 글줄을 소유한다.
                // 혼합 컨트롤에 전체 PartialParagraph를 다시 추가하지 않는다.
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
                        engine.register_unqueued_table_footnote(
                            st,
                            footnote,
                            source,
                            paragraphs_all,
                            styles,
                        );
                    },
                );
                if break_after_current_table {
                    break;
                }
            }
            Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
                controls::place_table_host_shape_with_storage(
                    st,
                    para_idx,
                    ctrl_idx,
                    ctrl,
                    para,
                    para_start_height,
                    styles,
                    engine.dpi,
                    storage,
                )?;
            }
            _ => {}
        }
    }

    // 모든 컨트롤 원점이 정해진 뒤, TAC 높이 조정 전에 호스트 글줄을 한 번 방출한다.
    controls::place_decoration_host_text_with_storage(
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
        storage,
    )?;

    if let Some(plan) = stored_body_wide_host {
        // 최초 host의 모든 글줄이 같은 단에 실제 수용됐을 때만 원점과 끝을 확정한다.
        st.commit_stored_body_wide_host_with_storage(para_idx, para, plan, storage)?;
    }

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
            styles,
            measured_tables,
            tac_count,
            height_before,
            measured_tac_floor,
            engine.tac_flow_query(),
        );
    }

    Ok(())
}
