//! 관계형: 완료된 GPU 작업의 pin 반환과 불명확한 작업의 자원 보존을 검사한다.
//! 주석 갱신: 2026-11-12T02:22:57+09:00; 신규 파일.
//! 입력은 합성 blank snapshot 또는 자원 drop 계수이며 설치 글꼴에 의존하지 않는다.
//! API·할당량·수명 계약이며 실제 문서의 절대 좌표·golden·조판 일치를 고정하지 않는다.
//! Native/fresh WASM·독립 한컴 PDF의 실물 시각 측정은 이 assertion에 비해당이다.
//! Apple SDK/실제 Metal 실행은 Linux에서 미실행이며 portable 통과로 대신하지 않는다.
//! 검증 경로와 범위: bindings/Flutter/macos/README.md의 내부 GPU 타일 캐시 절.

#![cfg(not(target_arch = "wasm32"))]
mod frame_resources {
    include!("../support/gpu_frame_resources.rs");
}

use frame_resources::FrameResources;
use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

struct Resource(Rc<Cell<usize>>);
impl Drop for Resource {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn resource(dropped: &Rc<Cell<usize>>) -> Rc<Resource> {
    Rc::new(Resource(Rc::clone(dropped)))
}

#[test]
fn completed_frame_releases_unique_work_and_tiles_but_keeps_scratch() {
    let dropped = Rc::new(Cell::new(0));
    let mut frame = FrameResources::default();
    frame.scratch = Some(resource(&dropped));
    let work = resource(&dropped);
    let tile = resource(&dropped);
    for _ in 0..64 {
        frame.pin_working(&work);
        frame.pin_tile(&tile);
    }
    assert_eq!(frame.working_count(), 1);
    assert_eq!(frame.tile_count(), 1);
    drop((work, tile));
    assert_eq!(dropped.get(), 0);
    assert!(frame.complete_frame());
    assert_eq!(dropped.get(), 2);
    assert_eq!(frame.working_count(), 0);
    assert_eq!(frame.tile_count(), 0);
    assert!(frame.scratch.is_some());
    assert!(frame.clear_completed());
    assert_eq!(dropped.get(), 3);
}

#[test]
fn quarantine_refuses_cleanup_and_retains_every_resource_after_owner_drop() {
    let dropped = Rc::new(Cell::new(0));
    let mut frame = FrameResources::default();
    frame.scratch = Some(resource(&dropped));
    frame.pin_working(&resource(&dropped));
    frame.pin_tile(&resource(&dropped));
    let scratch = Rc::downgrade(frame.scratch.as_ref().unwrap());
    frame.quarantine();
    assert!(!frame.complete_working());
    assert!(!frame.complete_frame());
    assert!(!frame.clear_completed());
    assert_eq!(frame.working_count(), 1);
    assert_eq!(frame.tile_count(), 1);
    drop(frame);
    assert_eq!(dropped.get(), 0);
    assert!(scratch.upgrade().is_some());
}

#[test]
fn panic_unwinding_drops_local_handles_without_destroying_pinned_targets() {
    let dropped = Rc::new(Cell::new(0));
    let mut frame: FrameResources<Resource, Resource> = FrameResources::default();
    let failed = catch_unwind(AssertUnwindSafe(|| {
        let working = resource(&dropped);
        frame.pin_working(&working);
        let tile = resource(&dropped);
        frame.pin_tile(&tile);
        panic!("simulated producer panic after commands were recorded");
    }));
    assert!(failed.is_err());
    assert_eq!(dropped.get(), 0);
    frame.quarantine();
    drop(frame);
    assert_eq!(dropped.get(), 0);
}

#[test]
fn completed_frames_do_not_accumulate_pins_or_replace_the_retained_scratch() {
    let dropped = Rc::new(Cell::new(0));
    let mut frame = FrameResources::default();
    let scratch = resource(&dropped);
    frame.scratch = Some(Rc::clone(&scratch));
    for index in 0..256 {
        frame.pin_working(&resource(&dropped));
        frame.pin_tile(&resource(&dropped));
        assert!(frame.complete_frame());
        assert_eq!(dropped.get(), (index + 1) * 2);
        assert!(Rc::ptr_eq(frame.scratch.as_ref().unwrap(), &scratch));
    }
    drop(scratch);
    drop(frame);
    assert_eq!(dropped.get(), 513);
}

// P128 cache trim shares the owner resource harness and executes the actual Rc policy.
mod cache_trim {
    use std::{cell::Cell, rc::Rc};
    include!("../support/gpu_cache_trim.rs");

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
}
