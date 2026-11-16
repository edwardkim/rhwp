//! 관계형: 내부 GPU 캐시의 Rc pin·LRU·scratch 소유와 축소 수명 사이의 관계를 검사합니다.
//! 주석 갱신: 2026-11-16T13:55:38+09:00; 신규 파일입니다.
//! 합성 Rc 자원으로 실제 공통 제거 정책을 실행하며 절대 px·golden·조판 일치를 고정하지 않습니다.
//! 설치 글꼴·실제 GPU·Native/fresh WASM·한컴 Print PDF의 시각 측정은 이 소유권 검사에 비해당입니다.
use std::{cell::Cell, rc::Rc};
#[path = "../support/gpu_cache_trim.rs"]
mod policy;
use policy::*;

#[test]
fn oldest_unpinned_entry_is_removed_while_all_pins_remain_alive() {
    let first = Rc::new(1);
    let pin = Rc::clone(&first);
    let mut entries = vec![first, Rc::new(2), Rc::new(3)];
    assert_eq!(*remove_oldest_idle(&mut entries).unwrap(), 2);
    assert_eq!(*remove_oldest_idle(&mut entries).unwrap(), 3);
    assert!(remove_oldest_idle(&mut entries).is_none());
    assert_eq!(entries.len(), 1);
    assert_eq!(Rc::strong_count(&pin), 2);
    drop(pin);
    assert_eq!(*remove_oldest_idle(&mut entries).unwrap(), 1);
    assert!(entries.is_empty());
}
#[test]
fn an_idle_scratch_target_keeps_its_credit_until_the_last_resource_drops() {
    struct Resource(Rc<Cell<usize>>);
    impl Drop for Resource {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let dropped = Rc::new(Cell::new(0));
    let mut scratch = Some(Rc::new(Resource(Rc::clone(&dropped))));
    let pin = Rc::clone(scratch.as_ref().unwrap());
    assert!(!remove_unique(&mut scratch));
    assert_eq!(dropped.get(), 0);
    drop(pin);
    assert!(remove_unique(&mut scratch));
    assert_eq!(dropped.get(), 1);
    assert!(!remove_unique(&mut scratch));
}
#[test]
fn partial_trim_stops_at_the_requested_budget_and_preserves_lru_order() {
    let mut entries = vec![Rc::new(32), Rc::new(16), Rc::new(64)];
    while entries.iter().map(|v| **v).sum::<usize>() > 80 {
        drop(remove_oldest_idle(&mut entries).unwrap());
    }
    assert_eq!(
        entries.iter().map(|v| **v).collect::<Vec<_>>(),
        vec![16, 64]
    );
}
#[test]
fn zero_budget_does_not_release_a_quarantined_or_working_pin() {
    let working = Rc::new(64);
    let quarantine = Rc::clone(&working);
    let mut entries = vec![working];
    assert!(remove_oldest_idle(&mut entries).is_none());
    let mut scratch = Some(Rc::clone(&quarantine));
    assert!(!remove_unique(&mut scratch));
    assert_eq!(entries.len(), 1);
    assert!(scratch.is_some());
    assert_eq!(*quarantine, 64);
}
