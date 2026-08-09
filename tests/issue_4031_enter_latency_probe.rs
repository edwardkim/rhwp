//! Issue #4031 Stage 1 local-only Enter latency probe.
//!
//! 거대 셀(115쪽 분할 표) 문서에서 셀 내부 Enter(문단 분할) 1회의 동기 전 구간을
//! 분해 계측한다. `RHWP_2424_PROFILE=1`과 함께 실행하면 pagination subphase와
//! block-table timer가 stderr에 함께 출력된다. timing assertion은 두지 않는다.
//!
//! - cold Enter: pending 없는 상태의 split 1회 소유 비용(분할+reflow+vpos+full pagination)
//! - pending Enter(현행 studio 경로): deferred insert 잔여 → full flush → split
//! - pending Enter(목표 계약): flush 없이 split 1회의 `paginate_if_needed()`로 수렴

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const FIXTURES: [(&str, &str); 2] = [
    ("hwp", "samples/issue1949_giant_cell_nested_tables_perf.hwp"),
    (
        "hwpx",
        "samples/issue1949_giant_cell_nested_tables_perf.hwpx",
    ),
];

/// 2424 probe와 같은 giant-cell 좌표: section 0 / parent para 0 / control 2 / cell 2.
const SECTION: usize = 0;
const PARENT_PARA: usize = 0;
const CONTROL: usize = 2;
const CELL: usize = 2;
/// 2424 probe가 편집한 셀 문단과 오프셋(문단 길이 130 이상 보장).
const TOP_CELL_PARA: usize = 5;
const TOP_OFFSET: usize = 130;

fn load(relative: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|error| panic!("load {relative}: {error}"))
}

fn repeats() -> usize {
    std::env::var("RHWP_4031_REPEATS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
        .clamp(1, 20)
}

fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// cold Enter — pending pagination 없는 상태에서 연속 split의 소유 비용.
///
/// 셀 상단(문단 5)과 하단 부근에서 각각 3연타를 계측해 편집 위치와 무관하게
/// full pagination이 지배 항인지(=fragment 증분 재개 부재) 확인한다.
#[test]
#[ignore = "local performance diagnostic; run explicitly with RHWP_2424_PROFILE=1"]
fn issue_4031_profile_cold_enter_split() {
    for run in 1..=repeats() {
        for (format, relative) in FIXTURES {
            let mut doc = load(relative);
            assert_eq!(doc.page_count(), 115, "{format}: initial page count");
            let cell_para_count = doc
                .get_cell_paragraph_count_native(SECTION, PARENT_PARA, CONTROL, CELL)
                .expect("giant cell paragraph count");
            let bottom_para = cell_para_count.saturating_sub(8);

            for (label, base_para, base_offset) in [
                ("top", TOP_CELL_PARA, TOP_OFFSET),
                ("bottom", bottom_para, 0),
            ] {
                for stroke in 0..3 {
                    // Enter 연타: 첫 분할 뒤 캐럿은 새 문단 시작(offset 0)에 있다.
                    let (para, offset) = if stroke == 0 {
                        (base_para, base_offset)
                    } else {
                        (base_para + stroke, 0)
                    };
                    let started = Instant::now();
                    let raw = doc
                        .split_paragraph_in_cell_native(
                            SECTION,
                            PARENT_PARA,
                            CONTROL,
                            CELL,
                            para,
                            offset,
                            None,
                        )
                        .expect("cell paragraph split");
                    let elapsed = started.elapsed();
                    let result: Value = serde_json::from_str(&raw).expect("split result JSON");
                    assert_eq!(result["ok"].as_bool(), Some(true), "{format}: split ok");
                    eprintln!(
                        "RHWP_4031_COLD_ENTER run={run} format={format} pos={label} stroke={} split_ms={:.3} pages={}",
                        stroke + 1,
                        ms(elapsed),
                        doc.page_count(),
                    );
                }
            }
        }
    }
}

/// pending Enter — 현행 studio 경로 재현: 56회 deferred insert 잔여 상태에서
/// `flushDeferredPagination`(before-navigation barrier) 뒤 split.
/// 이어서 같은 초기 상태에서 flush 생략(목표 계약) split을 계측해
/// 최종 page count 정합과 flush 중복분을 수치로 고정한다.
#[test]
#[ignore = "local performance diagnostic; run explicitly with RHWP_2424_PROFILE=1"]
fn issue_4031_profile_pending_enter_flush_vs_skip() {
    for run in 1..=repeats() {
        for (format, relative) in FIXTURES {
            // 시나리오 A — 현행: flush 1회 + split의 full pagination 1회.
            let mut doc_a = load(relative);
            for inserted in 0..56 {
                doc_a
                    .insert_text_in_cell_native_deferred_pagination(
                        SECTION,
                        PARENT_PARA,
                        CONTROL,
                        CELL,
                        TOP_CELL_PARA,
                        TOP_OFFSET + inserted,
                        "1",
                    )
                    .expect("sequential deferred cell insert");
            }
            let flush_started = Instant::now();
            doc_a
                .flush_deferred_pagination()
                .expect("before-navigation full flush");
            let flush_elapsed = flush_started.elapsed();
            let split_started = Instant::now();
            doc_a
                .split_paragraph_in_cell_native(
                    SECTION,
                    PARENT_PARA,
                    CONTROL,
                    CELL,
                    TOP_CELL_PARA,
                    TOP_OFFSET,
                    None,
                )
                .expect("cell paragraph split after flush");
            let split_a_elapsed = split_started.elapsed();
            let pages_a = doc_a.page_count();

            // 시나리오 B — 목표 계약: flush 생략, split의 paginate_if_needed 1회로 수렴.
            let mut doc_b = load(relative);
            for inserted in 0..56 {
                doc_b
                    .insert_text_in_cell_native_deferred_pagination(
                        SECTION,
                        PARENT_PARA,
                        CONTROL,
                        CELL,
                        TOP_CELL_PARA,
                        TOP_OFFSET + inserted,
                        "1",
                    )
                    .expect("sequential deferred cell insert");
            }
            doc_b.cancel_deferred_pagination();
            let split_b_started = Instant::now();
            doc_b
                .split_paragraph_in_cell_native(
                    SECTION,
                    PARENT_PARA,
                    CONTROL,
                    CELL,
                    TOP_CELL_PARA,
                    TOP_OFFSET,
                    None,
                )
                .expect("cell paragraph split without flush");
            let split_b_elapsed = split_b_started.elapsed();
            let pages_b = doc_b.page_count();

            // split의 동기 pagination이 deferred 목표를 소비했으므로 잔여 flush는 즉시 완료여야 한다.
            let post_flush_started = Instant::now();
            let post_flush: Value = serde_json::from_str(
                &doc_b
                    .flush_deferred_pagination()
                    .expect("post-split flush must be trivial"),
            )
            .expect("post flush JSON");
            let post_flush_elapsed = post_flush_started.elapsed();

            assert_eq!(
                pages_a, pages_b,
                "{format}: flush-then-split and skip-flush split must converge to same page count",
            );
            eprintln!(
                "RHWP_4031_PENDING_ENTER run={run} format={format} flush_ms={:.3} split_after_flush_ms={:.3} \
                 skip_flush_split_ms={:.3} post_flush_ms={:.3} post_flush_state={} pages={pages_a}",
                ms(flush_elapsed),
                ms(split_a_elapsed),
                ms(split_b_elapsed),
                ms(post_flush_elapsed),
                post_flush["status"].as_str().unwrap_or("unknown"),
            );
        }
    }
}
