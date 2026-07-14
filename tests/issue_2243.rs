//! Issue #2243 — 결재문서(기계생성 HWPX) 계열 sliver 쪽 해소의 쪽수 핀.
//!
//! 병인: 표 경로의 vpos 앵커 미확립/일괄 말소로 후속 문단 스냅이 드리프트를
//! 역산한 lazy base 에 고착 → 페이지당 +16~22px 팬텀 → razor 경계에서 sliver
//! 쪽(+1~+2). 수정: 표-경로 vpos 앵커 확립 + 저장 사다리 조건부 유지 + 빈 앵커
//! float 표 host 직후 lazy 이중 계상 가드 + 저장-앵커 safety 마진 면제.
//! 정답 쪽수는 한글 2022 COM PageCount 실측 (visual sweep 픽셀 정합 동반).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

/// (샘플, 한글 실측 쪽수)
const PINS: &[(&str, u32)] = &[
    // vpos 앵커 확립 + 사다리 유지 (7→5쪽 회귀 복구, visual avg 93.92)
    ("samples/task2243/36395325_gyeoljae_consulting.hwpx", 5),
    // 빈 앵커 float 표 host 직후 lazy 이중 계상 가드 (4→3쪽)
    ("samples/task2243/36382819_gyeoljae_pm_traffic.hwpx", 3),
    // 지속 결함 복구 (7→5쪽)
    ("samples/task2243/36386907_gyeoljae_sewoon.hwpx", 5),
    // 저장-앵커 safety 마진 면제 razor (2→1쪽, stored 881.3+fit 49.6=930.9≤933.6)
    ("samples/task2243/156631374_taxi_press.hwpx", 1),
];

#[test]
fn issue_2243_gyeoljae_sliver_page_pins() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    for (sample, expected) in PINS {
        let bytes = fs::read(Path::new(repo_root).join(sample))
            .unwrap_or_else(|e| panic!("read {sample}: {e}"));
        let core =
            DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"));
        assert_eq!(
            core.page_count(),
            *expected,
            "{sample}: 한글 COM 실측 쪽수와 불일치"
        );
    }
}
