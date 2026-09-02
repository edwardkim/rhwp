//! [Issue #5966] debug 빌드가 표 각주 큐 불변식("fresh page must accept one
//! queued table footnote or pending tail")에서 패닉한다 — 10k 전수 중 유일 재현
//! (`samples/issue5966/1130000-202100008_franchise_review_report.hwp`, 143쪽).
//!
//! 근인은 불변식 ②(실결함): `force_next_page` 저장 지시가 붙은 표-셀 각주는
//! 큐 소진용으로 **강제로 연 새 쪽**에서도 원자 배치가 차단됐고, 마커 행이
//! 현재 fragment 밖이라 분할 필터도 기각 — 빈 새 쪽(avail 876.9px)에 note 0
//! (h=90.1px)이 들어가는데도 진행 불가였다. release 는 break 로 그 각주 등록을
//! 조용히 덮었다.
//!
//! 수정: 강제로 연 새 쪽에서는 `force_next_page` 가 이미 충족된 것으로 보고
//! 일반 fit(원자 배치) 경로로 보낸다. debug 불변식은 진짜 병리의 tripwire 로
//! 유지한다.
//!
//! 이 테스트는 debug 프로파일에서 조판이 패닉 없이 완주하고 총 143쪽(release
//! 실측과 동일)임을 고정한다. 결함 상태에서는 debug 에서 패닉으로 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5966/1130000-202100008_franchise_review_report.hwp";

#[test]
fn issue_5966_queued_table_footnote_completes_on_forced_fresh_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(
        core.page_count(),
        143,
        "release/debug 동일하게 143쪽이어야 한다"
    );
}
