//! 관계형: Native Metal의 프로세스 admission과 live/quarantine credit 수명을 검사한다.
//! 합성 reservation·drop·동시성만 사용하며 실제 Metal/Apple SDK를 실행하지 않는다.
//! 새 source를 직접 포함하고 GPU 완료 증거와 caller의 release 확인을 구분한다.

#![cfg(not(target_arch = "wasm32"))]

mod admission {
    include!("../support/metal_context_admission.rs");
}

use admission::{Admission, Credit, Stats, MAX_PROCESS_PRESENTERS};
use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc, Barrier};

fn stats(in_use: usize, quarantined: usize) -> Stats {
    Stats {
        in_use,
        quarantined,
        limit: MAX_PROCESS_PRESENTERS,
    }
}

#[test]
fn construction_failures_refund_only_unexposed_reservations() {
    let admission = Admission::new();
    for _ in 0..256 {
        let reservation = admission.reserve().unwrap();
        assert_eq!(admission.stats(), stats(1, 0));
        drop(reservation); // Constructor returned Err before exposing an owner.
        assert_eq!(admission.stats(), stats(0, 0));
    }
}

#[test]
fn panic_before_live_rolls_back_but_exposed_owner_panic_quarantines() {
    let admission = Admission::new();
    let failed = catch_unwind(AssertUnwindSafe(|| {
        let _reservation = admission.reserve().unwrap();
        panic!("constructor failure before live owner exposure");
    }));
    assert!(failed.is_err());
    assert_eq!(admission.stats(), stats(0, 0));
    let failed = catch_unwind(AssertUnwindSafe(|| {
        let mut owner = admission.reserve().unwrap();
        owner.mark_live();
        panic!("owner info/insertion/render panic after creation");
    }));
    assert!(failed.is_err());
    assert_eq!(admission.stats(), stats(1, 1));
}

#[test]
fn successful_native_release_refunds_and_repeated_release_is_inert() {
    let admission = Admission::new();
    for _ in 0..256 {
        let mut owner = admission.reserve().unwrap();
        owner.mark_live();
        owner.mark_live();
        assert_eq!(admission.stats(), stats(1, 0));
        assert!(owner.release_confirmed());
        assert!(!owner.release_confirmed());
        owner.quarantine(); // Owner::drop after normal release must not charge.
        drop(owner);
        assert_eq!(admission.stats(), stats(0, 0));
    }
}

#[test]
fn abandon_is_idempotent_and_cannot_be_revived_or_refunded() {
    let admission = Admission::new();
    let mut owner = admission.reserve().unwrap();
    owner.mark_live();
    owner.quarantine();
    owner.quarantine();
    owner.mark_live();
    assert!(!owner.release_confirmed());
    assert_eq!(admission.stats(), stats(1, 1));
    drop(owner);
    assert_eq!(admission.stats(), stats(1, 1));
}

#[test]
fn repeated_abandon_consumes_four_process_slots_across_new_threads() {
    let admission = Arc::new(Admission::new());
    for count in 1..=MAX_PROCESS_PRESENTERS {
        let shared = Arc::clone(&admission);
        std::thread::spawn(move || {
            let mut owner = shared.reserve().unwrap();
            owner.mark_live();
            owner.quarantine();
        })
        .join()
        .unwrap();
        assert_eq!(admission.stats(), stats(count, count));
    }
    let shared = Arc::clone(&admission);
    assert!(std::thread::spawn(move || shared.reserve().is_none())
        .join()
        .unwrap());
    assert_eq!(admission.stats(), stats(4, 4));
}

thread_local! {
    static TLS_OWNER: RefCell<Option<Credit<'static>>> = const { RefCell::new(None) };
}

#[test]
fn thread_local_owner_drop_preserves_quarantine_admission() {
    // A process ledger outlives its TLS owners, just like PROCESS_ADMISSION.
    let admission: &'static Admission = Box::leak(Box::new(Admission::new()));
    std::thread::spawn(move || {
        let mut owner = admission.reserve().unwrap();
        owner.mark_live();
        TLS_OWNER.with(|slot| *slot.borrow_mut() = Some(owner));
    })
    .join()
    .unwrap();
    assert_eq!(admission.stats(), stats(1, 1));
}

#[test]
fn parallel_creators_never_cross_the_process_limit_and_can_release() {
    const CREATORS: usize = 32;
    let admission = Arc::new(Admission::new());
    let barrier = Arc::new(Barrier::new(CREATORS + 1));
    let (sender, receiver) = mpsc::channel();
    let mut threads = Vec::new();
    for _ in 0..CREATORS {
        let shared = Arc::clone(&admission);
        let barrier = Arc::clone(&barrier);
        let sender = sender.clone();
        threads.push(std::thread::spawn(move || {
            let mut owner = shared.reserve();
            if let Some(owner) = &mut owner {
                owner.mark_live();
            }
            sender.send(owner.is_some()).unwrap();
            barrier.wait();
            if let Some(owner) = &mut owner {
                assert!(owner.release_confirmed());
            }
        }));
    }
    drop(sender);
    assert_eq!(
        receiver
            .iter()
            .take(CREATORS)
            .filter(|accepted| *accepted)
            .count(),
        MAX_PROCESS_PRESENTERS
    );
    assert_eq!(admission.stats(), stats(4, 0));
    barrier.wait();
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(admission.stats(), stats(0, 0));
}

#[test]
fn live_and_quarantined_owners_share_one_consistent_budget() {
    let admission = Admission::new();
    let mut owners: Vec<_> = (0..MAX_PROCESS_PRESENTERS)
        .map(|_| {
            let mut credit = admission.reserve().unwrap();
            credit.mark_live();
            credit
        })
        .collect();
    owners[0].quarantine();
    owners[1].quarantine();
    assert_eq!(admission.stats(), stats(4, 2));
    assert!(admission.reserve().is_none());
    assert!(owners[2].release_confirmed());
    assert_eq!(admission.stats(), stats(3, 2));
    let mut replacement = admission.reserve().unwrap();
    replacement.mark_live();
    assert_eq!(admission.stats(), stats(4, 2));
    assert!(replacement.release_confirmed());
    drop(owners);
    assert_eq!(admission.stats(), stats(3, 3));
}
