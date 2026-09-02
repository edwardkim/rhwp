//! [Issue #5822] 여러 쪽 표 구간에서 본문이 한 쪽 밀린다 — rhwp 40쪽 vs 한글 39쪽
//! (156634833; 등록 시점의 "총 쪽수 40 동일" 은 2-up 캐시 오염으로, 신선 COM
//! 실측은 39쪽).
//!
//! 근인: p6 말미의 TAC 차트 표(2×2, 177px)가 한글 저장 frame(page-local
//! 748.2..929.1 ≤ body 933.6)으로는 그 쪽에 앉는데, rhwp 흐름이 누적 드리프트로
//! 사다리보다 42px 앞서 달려(790.4) 흐름-기준 적합검사가 밀어냈다 — 이하 절
//! 전체가 한 쪽씩 밀리고 끝까지 회복되지 않았다.
//!
//! 수정: `saved_table_bounds_fit_at_flow_tail` 에 "흐름이 저장 frame **안**"
//! (top ≤ 흐름 ≤ bottom) 신뢰 조건 추가 — frame 이 body 에 들어가면 그 쪽 소유.
//! frame 이 흐름 뒤에 통째로 처진 stale anchor 는 여전히 제외. 자매 문서
//! 156673604 는 40쪽=한글 그대로(회귀 0).
//!
//! 픽스처는 원본 HWPX 구역0 문단 29..40(p6 전체 시퀀스) 절단 + BinData 스텁(98KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5822/tac_chart_stored_frame_fit.hwpx";

#[test]
fn issue_5822_tac_chart_stays_on_stored_frame_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    // 결함 시 차트 표가 다음 쪽으로 밀려 2쪽이 된다.
    assert_eq!(
        core.page_count(),
        1,
        "차트 표가 저장 frame 대로 같은 쪽에 앉아 1쪽이어야 한다"
    );
}
