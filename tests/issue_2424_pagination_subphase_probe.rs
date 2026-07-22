//! Issue #2424 Stage A local-only pagination subphase probe.
//!
//! `RHWP_2424_PROFILE=1`과 함께 실행하면 HWP/HWPX 56번째 cell-flow boundary의
//! explicit full pagination 내부 timer를 stderr에 출력한다. timing assertion은 두지 않는다.

use std::fs;
use std::path::Path;

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
