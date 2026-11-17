//! 고정형 계약: 자원 상한·ID 고갈·CAS 재시도·경합(시각 측정 비해당).
//! 작성: 2026-11-18T02:25:43+09:00; 문서·글꼴·페이지 좌표를 사용하지 않는다.

use rhwp_codec_resource::atomic::{try_update_u64, try_update_usize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn usize_budget_rejection_and_arithmetic_boundaries_do_not_write() {
    let counter = AtomicUsize::new(1);
    assert_eq!(
        try_update_usize(&counter, Ordering::AcqRel, Ordering::Acquire, |used| {
            used.checked_add(2).filter(|next| *next <= 3)
        }),
        Ok(1)
    );
    assert_eq!(counter.load(Ordering::Acquire), 3);
    assert_eq!(
        try_update_usize(&counter, Ordering::AcqRel, Ordering::Acquire, |used| {
            used.checked_add(1).filter(|next| *next <= 3)
        }),
        Err(3)
    );
    assert_eq!(counter.load(Ordering::Acquire), 3);

    counter.store(usize::MAX, Ordering::Relaxed);
    assert_eq!(
        try_update_usize(&counter, Ordering::Relaxed, Ordering::Relaxed, |old| {
            old.checked_add(1)
        }),
        Err(usize::MAX)
    );
    assert_eq!(counter.load(Ordering::Relaxed), usize::MAX);
    counter.store(0, Ordering::Relaxed);
    assert_eq!(
        try_update_usize(&counter, Ordering::Relaxed, Ordering::Relaxed, |old| {
            old.checked_sub(1)
        }),
        Err(0)
    );
    assert_eq!(counter.load(Ordering::Relaxed), 0);
}

#[test]
fn u64_identifiers_return_the_prior_value_and_stop_at_exhaustion() {
    let counter = AtomicU64::new(u64::MAX - 1);
    assert_eq!(
        try_update_u64(&counter, Ordering::Relaxed, Ordering::Relaxed, |id| {
            id.checked_add(1)
        }),
        Ok(u64::MAX - 1)
    );
    assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    assert_eq!(
        try_update_u64(&counter, Ordering::Relaxed, Ordering::Relaxed, |id| {
            id.checked_add(1)
        }),
        Err(u64::MAX)
    );
    assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
}

#[test]
fn usize_retry_rechecks_the_current_budget_before_charging() {
    let counter = AtomicUsize::new(0);
    let mut first = true;
    let result = try_update_usize(&counter, Ordering::AcqRel, Ordering::Acquire, |used| {
        if first {
            first = false;
            counter.store(3, Ordering::Release);
        }
        used.checked_add(1).filter(|next| *next <= 3)
    });
    assert_eq!(result, Err(3));
    assert_eq!(counter.load(Ordering::Acquire), 3);
}

#[test]
fn u64_retry_returns_the_value_actually_replaced() {
    let counter = AtomicU64::new(0);
    let mut first = true;
    let result = try_update_u64(&counter, Ordering::AcqRel, Ordering::Acquire, |id| {
        if first {
            first = false;
            counter.store(7, Ordering::Release);
        }
        id.checked_add(1)
    });
    assert_eq!(result, Ok(7));
    assert_eq!(counter.load(Ordering::Acquire), 8);
}

#[test]
fn concurrent_budget_claims_never_exceed_the_cap_or_reuse_a_credit() {
    const CAP: usize = 256;
    let counter = Arc::new(AtomicUsize::new(0));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let counter = Arc::clone(&counter);
            std::thread::spawn(move || {
                (0..CAP)
                    .filter_map(|_| {
                        try_update_usize(&counter, Ordering::AcqRel, Ordering::Acquire, |used| {
                            used.checked_add(1).filter(|next| *next <= CAP)
                        })
                        .ok()
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    let mut credits: Vec<_> = workers
        .into_iter()
        .flat_map(|worker| worker.join().expect("budget worker panicked"))
        .collect();
    credits.sort_unstable();
    assert_eq!(credits, (0..CAP).collect::<Vec<_>>());
    assert_eq!(counter.load(Ordering::Acquire), CAP);
}

#[test]
fn concurrent_identifier_exhaustion_does_not_wrap_or_duplicate() {
    const COUNT: u64 = 64;
    let start = u64::MAX - COUNT;
    let counter = Arc::new(AtomicU64::new(start));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let counter = Arc::clone(&counter);
            std::thread::spawn(move || {
                (0..COUNT)
                    .filter_map(|_| {
                        try_update_u64(&counter, Ordering::Relaxed, Ordering::Relaxed, |id| {
                            id.checked_add(1)
                        })
                        .ok()
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    let mut ids: Vec<_> = workers
        .into_iter()
        .flat_map(|worker| worker.join().expect("ID worker panicked"))
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, (start..u64::MAX).collect::<Vec<_>>());
    assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
}
