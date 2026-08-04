//! Issue #2424 Stage A local-only pagination subphase probe.
//!
//! `RHWP_2424_PROFILE=1`과 함께 실행하면 HWP/HWPX 56번째 cell-flow boundary의
//! explicit full pagination 내부 timer를 stderr에 출력한다. timing assertion은 두지 않는다.

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

#[test]
#[ignore = "local performance diagnostic; run explicitly with RHWP_2424_PROFILE=1"]
fn issue_2424_profile_boundary_full_pagination_subphases() {
    let repeats = std::env::var("RHWP_2424_REPEATS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1)
        .clamp(1, 20);

    for run in 1..=repeats {
        for (format, relative) in FIXTURES {
            eprintln!("RHWP_2424_CASE run={run} format={format}");
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
            let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"));
            let mut doc = HwpDocument::from_bytes(&bytes)
                .unwrap_or_else(|error| panic!("load {relative}: {error}"));
            assert_eq!(doc.page_count(), 115, "{format}: initial page count");

            for inserted in 0..56 {
                let raw = doc
                    .insert_text_in_cell_native_deferred_pagination(
                        0,
                        0,
                        2,
                        2,
                        5,
                        130 + inserted,
                        "1",
                    )
                    .expect("sequential deferred cell insert");
                let result: Value = serde_json::from_str(&raw).expect("cell edit result JSON");
                assert_eq!(
                    result["cellFlowChanged"].as_bool(),
                    Some(inserted == 55),
                    "{format}: input {} flow signal",
                    inserted + 1,
                );
            }

            doc.flush_deferred_pagination()
                .expect("explicit full pagination flush");
            assert_eq!(doc.page_count(), 115, "{format}: flushed page count");
        }
    }
}

#[test]
#[ignore = "local resumable latency diagnostic; run explicitly"]
fn issue_2424_profile_resumable_fragment_steps() {
    for (format, relative) in FIXTURES {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"));
        let mut doc = HwpDocument::from_bytes(&bytes)
            .unwrap_or_else(|error| panic!("load {relative}: {error}"));
        for inserted in 0..56 {
            doc.insert_text_in_cell_native_deferred_pagination(0, 0, 2, 2, 5, 130 + inserted, "1")
                .expect("sequential deferred cell insert");
        }

        let begin_started = Instant::now();
        let begin: Value = serde_json::from_str(
            &doc.begin_deferred_pagination(1)
                .expect("begin resumable pagination"),
        )
        .expect("begin result json");
        let begin_elapsed = begin_started.elapsed();
        assert_eq!(begin["status"], "pending", "{format}: begin status");

        let mut step_durations = Vec::new();
        let total_started = Instant::now();
        loop {
            let step_started = Instant::now();
            let step: Value = serde_json::from_str(
                &doc.step_deferred_pagination(1)
                    .expect("step resumable pagination"),
            )
            .expect("step result json");
            step_durations.push(step_started.elapsed());
            match step["status"].as_str() {
                Some("pending") => {}
                Some("complete") => break,
                other => panic!("{format}: unexpected step status {other:?}: {step}"),
            }
        }
        let total_elapsed = total_started.elapsed();
        step_durations.sort_unstable();
        let percentile = |numerator: usize, denominator: usize| -> Duration {
            let index = step_durations
                .len()
                .saturating_sub(1)
                .saturating_mul(numerator)
                / denominator;
            step_durations[index]
        };
        eprintln!(
            "RHWP_2424_RESUMABLE_PROFILE format={format} begin_ms={:.3} steps={} total_ms={:.3} p50_ms={:.3} p95_ms={:.3} max_ms={:.3}",
            begin_elapsed.as_secs_f64() * 1000.0,
            step_durations.len(),
            total_elapsed.as_secs_f64() * 1000.0,
            percentile(50, 100).as_secs_f64() * 1000.0,
            percentile(95, 100).as_secs_f64() * 1000.0,
            step_durations
                .last()
                .copied()
                .unwrap_or_default()
                .as_secs_f64()
                * 1000.0,
        );
        assert_eq!(step_durations.len(), 115, "{format}: fragment step count");
        assert_eq!(doc.page_count(), 115, "{format}: committed page count");
    }
}
