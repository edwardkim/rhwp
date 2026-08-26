//! [#4126/#4128/#4129 회귀 가드] 결정적(비-시계) 성능 회귀 판별용 프로세스 누적 카운터.
//!
//! 통합 테스트가 "콜드 캐럿 질의 1회가 page render tree 를 몇 번 짓는가"(#4126/#4128),
//! "분할 표 컷 높이 평가가 셀 유닛을 총 몇 개 스캔하는가"(#4129)를 상한으로 고정한다.
//! 시계 기반 판별은 CI 러너 편차로 플레이크가 나므로 작업량 카운터로 판별한다.
//! 프로세스 전역 누적 카운터를 쓰는 테스트는 파일당 1개(전용 프로세스)로 두고, 측정 구간
//! 직전에 [`reset`]을 호출한다. generated integration suite처럼 여러 테스트가 한 프로세스에서
//! 병렬 실행될 때 page tree 측정은 current-thread 카운터를 사용한다.

use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

thread_local! {
    static THREAD_PAGE_TREE_BUILDS: Cell<u64> = const { Cell::new(0) };
}

/// `DocumentCore::build_page_tree` (비캐시 빌드) 호출 누적.
pub static PAGE_TREE_BUILDS: AtomicU64 = AtomicU64::new(0);

/// `mixed_nested_flow_extra_from_cut` 이 스캔한 셀 유닛 누적 (호출당 방문 유닛 수 합산).
pub static MIXED_NESTED_UNITS_SCANNED: AtomicU64 = AtomicU64::new(0);

pub fn page_tree_builds() -> u64 {
    PAGE_TREE_BUILDS.load(Ordering::Relaxed)
}

/// 현재 테스트/호출 스레드에서 발생한 `DocumentCore::build_page_tree` 호출 누적.
///
/// 다른 테스트 스레드의 빌드를 포함하지 않으므로 generated integration suite의
/// 병렬 실행에서도 측정 구간을 격리할 수 있다.
pub fn thread_page_tree_builds() -> u64 {
    THREAD_PAGE_TREE_BUILDS.with(Cell::get)
}

pub fn mixed_nested_units_scanned() -> u64 {
    MIXED_NESTED_UNITS_SCANNED.load(Ordering::Relaxed)
}

pub fn reset() {
    PAGE_TREE_BUILDS.store(0, Ordering::Relaxed);
    MIXED_NESTED_UNITS_SCANNED.store(0, Ordering::Relaxed);
    reset_thread_page_tree_builds();
}

/// 현재 스레드의 page tree build 누적만 초기화한다.
pub fn reset_thread_page_tree_builds() {
    THREAD_PAGE_TREE_BUILDS.with(|builds| builds.set(0));
}

/// 전역 및 current-thread page tree build 누적을 함께 기록한다.
pub(crate) fn record_page_tree_build() {
    PAGE_TREE_BUILDS.fetch_add(1, Ordering::Relaxed);
    THREAD_PAGE_TREE_BUILDS.with(|builds| builds.set(builds.get().saturating_add(1)));
}
