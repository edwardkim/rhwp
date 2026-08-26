//! Issue #2097 — 쪽 하단 압축 수용(행/블록/컷 3축)의 쪽수 핀.
//!
//! 병인: 한글은 쪽 끝자락에서 행/블록/컷 첫 유닛이 잔여를 소폭 초과하면 압축해
//! 끼워 넣는데(COM·PDF 실측), rhwp 는 무조건 이월해 표 마지막 행 sliver 쪽을
//! 만든다. 수정: BOTTOM_SQUEEZE_* 게이트(초과 ≤13px, 잔여 ≤100px, 콘텐츠 여유
//! ≥12px, 측정-선언 정합, 중첩 표 제외, RowBreak 한정)의 압축 수용.
//! 정답 쪽수는 한글 2022 COM PageCount 실측 (visual sweep 픽셀 정합 동반).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

/// (샘플, 한글 실측 쪽수)
const PINS: &[(&str, u32)] = &[
    // 블록(rows 10-11: 잔여 87.8 에 98.2 압축) + 행(r14: 초과 6.9) 수용 (3→2쪽)
    ("samples/task2097/1741000_project_application.hwp", 2),
    // 압축 컷 수용 (p1 잔여 32.5 에 첫 줄 36.8 압축, 3→2쪽)
    ("samples/task2097/21298295_byeolpyo5_disaster.hwp", 2),
    // 밴드 컷(#2236) + 압축 수용 결합 게이트 — rowspan 블록 중간 행 만충 정합
    ("samples/task2146/21761835_jeonjik_exemption_table.hwp", 6),
];

#[test]
fn issue_2097_bottom_squeeze_page_pins() {
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
