//! 표 도메인 조정. 기존 평가·상태 쓰기 순서를 보존한다.

use crate::renderer::typeset::{
    paragraph, row_geometry_table, table, BlockTableContinuationContext,
    BlockTableContinuationSource, Issue2424TypesetProfile, TypesetEngine, TypesetState,
};

use super::fragment::FragmentProfile;

impl TypesetEngine {
    pub(in crate::renderer::typeset) fn drain_block_table_continuation(
        &self,
        st: &mut TypesetState,
        mut continuation_context: BlockTableContinuationContext,
        source: BlockTableContinuationSource<'_>,
    ) {
        let para_idx = source.para_index;
        let ctrl_idx = source.control_index;
        let row_count = continuation_context.prepared.row_count;
        // [#2424 Stage C] Native caller는 step을 동기 drain한다. 각 step은 budget만큼의
        // fragment만 처리하며 cursor와 shadow page-flow state는 context에 남는다.
        while !continuation_context.is_complete() {
            self.step_block_table_continuation(&mut continuation_context, source);
        }
        if std::env::var("RHWP_2424_PROFILE").is_ok_and(|value| !value.is_empty() && value != "0") {
            eprintln!(
                "RHWP_2424_CONTINUATION_CURSOR section={} para={} control={} fragments={} steps={} budget={} final_row={} rows={}",
                continuation_context.flow_state.section_index,
                para_idx,
                ctrl_idx,
                continuation_context.cursor.fragments_emitted,
                continuation_context.steps_completed,
                continuation_context.fragment_budget,
                continuation_context.cursor.row,
                row_count,
            );
        }
        *st = continuation_context.into_flow_state();
    }

    pub(in crate::renderer::typeset) fn step_block_table_continuation(
        &self,
        continuation_context: &mut BlockTableContinuationContext,
        source: BlockTableContinuationSource<'_>,
    ) {
        let para_idx = source.para_index;
        let ctrl_idx = source.control_index;
        let para = source.paragraph;
        let table = source.table;
        let row_geometry_table = source.row_geometry_table;
        let mt = source.measured_table;
        let styles = source.styles;
        // [Issue #4326] 이 continuation이 방출하는 모든 PartialTable의 start_row/end_row/
        // start_cut/end_cut은 `row_geometry_table` 기준(투명 1×1 래퍼를 벗긴 표일 수
        // 있다)이다. 렌더러가 `end_row <= table.row_count` 로 값을 되추론하던 것을,
        // 결정 시점의 이 포인터 비교로 데이터화해 PageItem에 함께 싣는다.
        let row_cursor_is_nested = !std::ptr::eq(row_geometry_table, table);
        // [#2424 프로파일] fragment 루프 하위 단계 누적 — closure 1회 = fragment 판정 1회.
        // 스캔 두 곳 외의 잔여(배치·각주 큐·커서 전진)는 total−scan−refit 로 산출한다.
        let issue2424_step_enabled =
            crate::renderer::layout::table_layout::issue2424_profile_enabled();
        let issue2424_step_started = issue2424_step_enabled.then(std::time::Instant::now);
        let mut profile = FragmentProfile {
            enabled: issue2424_step_enabled,
            iterations: 0,
            scan: (std::time::Duration::ZERO, 0),
            refit: (std::time::Duration::ZERO, 0),
        };
        continuation_context.step(|prepared, st, continuation| {
            self.step_block_table_fragment(
                prepared,
                st,
                continuation,
                source,
                row_cursor_is_nested,
                &mut profile,
            )
        });
        if let Some(started) = issue2424_step_started {
            use crate::renderer::layout::table_layout as issue2424_tl;
            use std::sync::atomic::Ordering::Relaxed;
            let total = started.elapsed();
            let other = total
                .saturating_sub(profile.scan.0)
                .saturating_sub(profile.refit.0);
            // cum: 프로세스 누적 스냅샷 (advance_row_cut / cell_units 프리미티브).
            eprintln!(
                "RHWP_2424_STEP_PROFILE sec={} para={} iters={} total_ms={:.2} \
                 scan={:.2}/{} refit={:.2}/{} other={:.2} | cum arc_calls={} arc_ms={:.2} \
                 cu_hit={} cu_miss={} cu_miss_ms={:.2}",
                continuation_context.flow_state.section_index,
                para_idx,
                profile.iterations,
                Issue2424TypesetProfile::ms(total),
                Issue2424TypesetProfile::ms(profile.scan.0),
                profile.scan.1,
                Issue2424TypesetProfile::ms(profile.refit.0),
                profile.refit.1,
                Issue2424TypesetProfile::ms(other),
                issue2424_tl::ISSUE2424_ADVANCE_ROW_CUT_CALLS.load(Relaxed),
                issue2424_tl::ISSUE2424_ADVANCE_ROW_CUT_NANOS.load(Relaxed) as f64 / 1e6,
                issue2424_tl::ISSUE2424_CELL_UNITS_HITS.load(Relaxed),
                issue2424_tl::ISSUE2424_CELL_UNITS_MISSES.load(Relaxed),
                issue2424_tl::ISSUE2424_CELL_UNITS_MISS_NANOS.load(Relaxed) as f64 / 1e6,
            );
        }
    }
}
