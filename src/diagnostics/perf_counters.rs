//! [#4126/#4128/#4129 회귀 가드] 결정적(비-시계) 성능 회귀 판별용 프로세스 누적 카운터.
//!
//! 통합 테스트가 "콜드 캐럿 질의 1회가 page render tree 를 몇 번 짓는가"(#4126/#4128),
//! "분할 표 컷 높이 평가가 셀 유닛을 총 몇 개 스캔하는가"(#4129)를 상한으로 고정한다.
//! 시계 기반 판별은 CI 러너 편차로 플레이크가 나므로 작업량 카운터로 판별한다.
//! 프로세스 전역 누적이므로 테스트는 파일당 1개(전용 프로세스)로 두고, 측정 구간
//! 직전에 [`reset`]을 호출한다.

use std::sync::atomic::{AtomicU64, Ordering};

/// `DocumentCore::build_page_tree` (비캐시 빌드) 호출 누적.
pub static PAGE_TREE_BUILDS: AtomicU64 = AtomicU64::new(0);

/// `mixed_nested_flow_extra_from_cut` 이 스캔한 셀 유닛 누적 (호출당 방문 유닛 수 합산).
pub static MIXED_NESTED_UNITS_SCANNED: AtomicU64 = AtomicU64::new(0);

pub fn page_tree_builds() -> u64 {
    PAGE_TREE_BUILDS.load(Ordering::Relaxed)
}

pub fn mixed_nested_units_scanned() -> u64 {
    MIXED_NESTED_UNITS_SCANNED.load(Ordering::Relaxed)
}

pub fn reset() {
    PAGE_TREE_BUILDS.store(0, Ordering::Relaxed);
    MIXED_NESTED_UNITS_SCANNED.store(0, Ordering::Relaxed);
}
