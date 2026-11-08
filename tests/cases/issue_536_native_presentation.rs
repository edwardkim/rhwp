//! 관계형: worker snapshot 전송·quota와 cross-thread CPU frame lease 수명을 검사한다.
//! 주석 갱신: 2026-11-07T16:41:01+09:00; 직전 파일 커밋: 2026-11-07T06:28:43+09:00 (1d5cc242bad823fe68890aaa2dce9415a07d9687).
//! 입력은 파일 내부 합성 문서·PageLayerTree 또는 저장소의 허가된 글꼴 fixture다.
//! API·소유·자원/합성 paint 계약이며 실물 문서 페이지의 조판 위치를 고정하지 않는다.
//! 이 assertion의 독립 한컴 PDF 대비 Native/fresh WASM raw 측정값은 비해당이다.
//! 별도 실물 시각·플랫폼 실행 근거: mydocs/pr/assets/seo-rii-20261107/validation-scope.md.
//! 실물 페이지 좌표·한컴 시각 일치율·GPU 완료를 고정하는 검사가 아니다.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;
use std::time::{Duration, Instant};

use rhwp::native_host::presentation::{
    NativePresentationConfig, NativePresentationId, NativePresentationRegistry,
    NativePresentationStamp,
};
use rhwp::native_host::render_snapshot::{NativeRenderSnapshot, NativeRenderSnapshotLimits};
use rhwp::native_host::runtime::{
    NativeRequestId, NativeRuntime, NativeRuntimeCommand, NativeRuntimeConfig, NativeRuntimeInfo,
    NativeRuntimeOutput, NativeRuntimeTarget,
};
use rhwp::native_host::NativeHostError;
use rhwp::paint::RenderProfile;

fn receive(runtime: &mut NativeRuntime, id: NativeRequestId) -> NativeRuntimeOutput {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(completion) = runtime.try_recv().unwrap() {
            assert_eq!(completion.request_id, id);
            return completion.result.unwrap();
        }
        assert!(Instant::now() < deadline, "worker completion timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn open(config: NativeRuntimeConfig) -> (NativeRuntime, NativeRuntimeInfo) {
    let mut runtime = NativeRuntime::spawn(config).unwrap();
    let id = runtime
        .try_submit(NativeRuntimeCommand::New { replace: None })
        .unwrap();
    let NativeRuntimeOutput::Info(info) = receive(&mut runtime, id) else {
        panic!("expected document info");
    };
    (runtime, info)
}

fn snapshot(runtime: &mut NativeRuntime, info: &NativeRuntimeInfo) -> Arc<NativeRenderSnapshot> {
    let id = runtime
        .try_submit(NativeRuntimeCommand::ExportRenderSnapshot {
            target: info.target(),
            font_generation: info.font_generation,
            page: 0,
            profile: RenderProfile::Screen,
            limits: NativeRenderSnapshotLimits::default(),
        })
        .unwrap();
    let NativeRuntimeOutput::RenderSnapshot {
        document_id,
        snapshot,
    } = receive(runtime, id)
    else {
        panic!("expected immutable render snapshot");
    };
    assert_eq!(document_id, info.document_id);
    snapshot
}

fn stamp(info: &NativeRuntimeInfo) -> NativePresentationStamp {
    NativePresentationStamp {
        document_id: info.document_id.as_u64(),
        revision: info.revision,
        font_generation: info.font_generation,
        view_generation: 1,
        device_generation: 1,
        device_scale: 1.0,
    }
}

fn finish(runtime: &mut NativeRuntime) {
    runtime.close();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !runtime.is_finished() {
        assert!(Instant::now() < deadline, "worker shutdown timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(runtime.try_join().unwrap());
}

#[test]
fn small_worker_snapshots_fit_a_small_quota_without_reserving_the_public_ceiling() {
    let (mut runtime, info) = open(NativeRuntimeConfig {
        max_outstanding: 4,
        max_pending_bytes: 64 * 1024,
    });
    let first = snapshot(&mut runtime, &info);
    assert!(first.usage().estimated_bytes <= 64 * 1024);
    let second = snapshot(&mut runtime, &info);
    assert!(Arc::ptr_eq(first.tree(), second.tree()));
    finish(&mut runtime);
    assert!(first.tree().page_width > 0.0);
}

#[test]
fn unread_snapshot_outputs_charge_actual_usage_and_release_after_receipt() {
    let (mut probe, info) = open(NativeRuntimeConfig::default());
    let bytes = snapshot(&mut probe, &info).usage().estimated_bytes;
    finish(&mut probe);
    assert!(bytes > 100);
    let (mut runtime, info) = open(NativeRuntimeConfig {
        max_outstanding: 4,
        max_pending_bytes: bytes + 100,
    });
    let request = || NativeRuntimeCommand::ExportRenderSnapshot {
        target: info.target(),
        font_generation: info.font_generation,
        page: 0,
        profile: RenderProfile::Screen,
        limits: NativeRenderSnapshotLimits::default(),
    };
    let first = runtime.try_submit(request()).unwrap();
    let second = runtime.try_submit(request()).unwrap();
    // A later New changes the observable document stamp only after both exports
    // have finished. Waiting for it does not receive/free either earlier credit.
    let barrier = runtime
        .try_submit(NativeRuntimeCommand::New {
            replace: Some(info.target()),
        })
        .unwrap();
    let observer = runtime.observer();
    let target = NativeRuntimeTarget {
        document_id: barrier,
        revision: 0,
    };
    let deadline = Instant::now() + Duration::from_secs(20);
    while !observer.is_current(target, 0) {
        assert!(Instant::now() < deadline, "FIFO quota barrier timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(matches!(
        receive(&mut runtime, first),
        NativeRuntimeOutput::Skipped(_)
    ));
    let completion = runtime
        .try_recv()
        .unwrap()
        .expect("second export completed before barrier");
    assert_eq!(completion.request_id, second);
    assert_eq!(completion.result.unwrap_err().code, "limitExceeded");
    let NativeRuntimeOutput::Info(current) = receive(&mut runtime, barrier) else {
        panic!("expected replacement info");
    };
    snapshot(&mut runtime, &current);
    finish(&mut runtime);
}

#[test]
fn snapshot_handles_are_bounded_and_observe_source_close_without_owning_the_runtime() {
    let registry = NativePresentationRegistry::new(NativePresentationConfig {
        max_snapshots: 1,
        ..Default::default()
    })
    .unwrap();
    let (mut runtime, info) = open(NativeRuntimeConfig::default());
    let observer = runtime.observer();
    let source = registry
        .create_source(observer.clone(), stamp(&info))
        .unwrap();
    let input = snapshot(&mut runtime, &info);
    let id = registry
        .retain_snapshot(source, info.document_id.as_u64(), Arc::clone(&input))
        .unwrap();
    let before = registry.stats().unwrap();
    assert!(matches!(
        registry.retain_snapshot(source, info.document_id.as_u64(), input),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert_eq!(registry.stats().unwrap(), before);
    assert!(registry.snapshot_info(id).unwrap().current);
    let original = stamp(&info);
    assert!(registry.snapshot_check(id, original).unwrap());
    for changed in [
        NativePresentationStamp {
            document_id: original.document_id + 1,
            ..original
        },
        NativePresentationStamp {
            revision: original.revision + 1,
            ..original
        },
        NativePresentationStamp {
            font_generation: original.font_generation + 1,
            ..original
        },
        NativePresentationStamp {
            view_generation: original.view_generation + 1,
            ..original
        },
        NativePresentationStamp {
            device_generation: original.device_generation + 1,
            ..original
        },
        NativePresentationStamp {
            device_scale: 2.0,
            ..original
        },
    ] {
        assert!(!registry.snapshot_check(id, changed).unwrap());
    }
    assert!(std::thread::scope(|scope| scope
        .spawn(|| registry.snapshot_check(id, original).unwrap())
        .join()
        .unwrap()));
    let changed_view = NativePresentationStamp {
        view_generation: 2,
        device_scale: 1.5,
        ..original
    };
    registry.configure_source(source, changed_view).unwrap();
    assert!(!registry.snapshot_check(id, original).unwrap());
    assert!(registry.snapshot_check(id, changed_view).unwrap());
    assert!(registry.snapshot_info(id).unwrap().current);
    assert_eq!(registry.stats().unwrap(), before);
    finish(&mut runtime);
    drop(runtime);
    assert!(!observer.is_current(info.target(), info.font_generation));
    assert!(!registry.snapshot_info(id).unwrap().current);
    assert!(!registry.snapshot_check(id, changed_view).unwrap());
    registry.close_source(source).unwrap();
    registry.release_snapshot(id).unwrap();
    let empty = registry.stats().unwrap();
    assert_eq!(empty.retained_snapshots, 0);
    assert_eq!(empty.snapshot_estimated_bytes, 0);
    assert!(registry.release_snapshot(id).is_err());
    assert!(registry.snapshot_check(id, changed_view).is_err());
    assert_eq!(registry.stats().unwrap(), empty);
}

#[test]
fn snapshot_byte_quota_and_invalid_source_inputs_do_not_consume_handles_or_credits() {
    for config in [
        NativePresentationConfig {
            max_frames: 0,
            ..Default::default()
        },
        NativePresentationConfig {
            max_frame_bytes: usize::MAX,
            ..Default::default()
        },
        NativePresentationConfig {
            max_in_flight_replays: 5,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            NativePresentationRegistry::new(config),
            Err(NativeHostError::InvalidArgument(_))
        ));
    }
    let registry = NativePresentationRegistry::new(NativePresentationConfig {
        max_snapshot_bytes: 1,
        ..Default::default()
    })
    .unwrap();
    let (mut runtime, info) = open(NativeRuntimeConfig::default());
    let mut invalid_stamp = stamp(&info);
    invalid_stamp.device_scale = f64::NAN;
    let initial = registry.stats().unwrap();
    assert!(registry
        .create_source(runtime.observer(), invalid_stamp)
        .is_err());
    assert_eq!(registry.stats().unwrap(), initial);
    let source = registry
        .create_source(runtime.observer(), stamp(&info))
        .unwrap();
    let before = registry.stats().unwrap();
    assert!(matches!(
        registry.retain_snapshot(
            source,
            info.document_id.as_u64(),
            snapshot(&mut runtime, &info)
        ),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert_eq!(registry.stats().unwrap(), before);
    registry.close_source(source).unwrap();
    finish(&mut runtime);
}

#[cfg(not(feature = "native-skia"))]
#[test]
fn cpu_presentation_without_skia_is_explicitly_disabled() {
    let registry = NativePresentationRegistry::new(NativePresentationConfig::default()).unwrap();
    let stamp = NativePresentationStamp {
        document_id: 1,
        revision: 0,
        font_generation: 0,
        view_generation: 1,
        device_generation: 1,
        device_scale: 1.0,
    };
    assert!(matches!(
        registry.render_cpu(
            NativePresentationId::from_raw(1),
            stamp,
            rhwp::renderer::raster_pixels::RasterRegion {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0
            },
            1.0
        ),
        Err(NativeHostError::FeatureDisabled)
    ));
}

#[cfg(feature = "native-skia")]
mod native {
    use super::*;
    use rhwp::renderer::raster_pixels::RasterRegion;

    fn region(width: f64) -> RasterRegion {
        RasterRegion {
            x: 0.0,
            y: 0.0,
            width,
            height: 1.0,
        }
    }

    #[test]
    fn leases_pin_actual_bytes_after_producer_release_and_enforce_quota_until_last_consumer() {
        let registry = Arc::new(
            NativePresentationRegistry::new(NativePresentationConfig {
                max_frame_bytes: 4,
                max_leases: 2,
                ..Default::default()
            })
            .unwrap(),
        );
        let (mut runtime, info) = open(NativeRuntimeConfig::default());
        let stamp = stamp(&info);
        let source = registry.create_source(runtime.observer(), stamp).unwrap();
        let snapshot = registry
            .retain_snapshot(
                source,
                info.document_id.as_u64(),
                snapshot(&mut runtime, &info),
            )
            .unwrap();
        let render_registry = Arc::clone(&registry);
        let frame = std::thread::spawn(move || {
            render_registry
                .render_cpu(snapshot, stamp, region(1.0), 1.0)
                .unwrap()
        })
        .join()
        .unwrap();
        let first = registry.acquire_frame(frame.frame, stamp).unwrap().lease;
        let second = registry.acquire_frame(frame.frame, stamp).unwrap().lease;
        let full = registry.stats().unwrap();
        assert!(registry.acquire_frame(frame.frame, stamp).is_err());
        assert_eq!(registry.stats().unwrap(), full);
        let read_registry = Arc::clone(&registry);
        let bytes = std::thread::spawn(move || {
            let (pointer, length) = read_registry.lease_data(first).unwrap();
            // This test holds the lease until after joining the reader.
            unsafe { std::slice::from_raw_parts(pointer, length) }.to_vec()
        })
        .join()
        .unwrap();
        assert_eq!(bytes.len(), 4);
        registry.release_frame(frame.frame).unwrap();
        assert_eq!(registry.stats().unwrap().frame_handles, 0);
        assert_eq!(registry.stats().unwrap().frame_bytes, 4);
        assert!(matches!(
            registry.render_cpu(snapshot, stamp, region(1.0), 1.0),
            Err(NativeHostError::LimitExceeded(_))
        ));
        registry.release_lease(first).unwrap();
        assert_eq!(registry.stats().unwrap().frame_bytes, 4);
        let (pointer, length) = registry.lease_data(second).unwrap();
        assert_eq!(
            unsafe { std::slice::from_raw_parts(pointer, length) },
            bytes
        );
        registry.release_lease(second).unwrap();
        assert_eq!(registry.stats().unwrap().frame_bytes, 0);
        assert_eq!(registry.stats().unwrap().retained_frames, 0);
        let empty = registry.stats().unwrap();
        assert!(registry.release_lease(second).is_err());
        assert!(registry.release_frame(frame.frame).is_err());
        assert!(registry.lease_data(second).is_err());
        assert_eq!(registry.stats().unwrap(), empty);
        let another = registry
            .render_cpu(snapshot, stamp, region(1.0), 1.0)
            .unwrap();
        assert_ne!(another.frame, frame.frame);
        registry.release_frame(another.frame).unwrap();
        registry.release_snapshot(snapshot).unwrap();
        registry.close_source(source).unwrap();
        finish(&mut runtime);
    }

    #[test]
    fn view_device_and_closed_sources_reject_new_frames_but_keep_existing_lease_memory() {
        let registry =
            NativePresentationRegistry::new(NativePresentationConfig::default()).unwrap();
        let (mut runtime, info) = open(NativeRuntimeConfig::default());
        let original = stamp(&info);
        let source = registry
            .create_source(runtime.observer(), original)
            .unwrap();
        let snapshot = registry
            .retain_snapshot(
                source,
                info.document_id.as_u64(),
                snapshot(&mut runtime, &info),
            )
            .unwrap();
        let frame = registry
            .render_cpu(snapshot, original, region(1.0), 1.0)
            .unwrap();
        let lease = registry.acquire_frame(frame.frame, original).unwrap().lease;
        let (pointer, length) = registry.lease_data(lease).unwrap();
        let expected = unsafe { std::slice::from_raw_parts(pointer, length) }.to_vec();
        let mut current = original;
        current.view_generation += 1;
        current.device_generation += 1;
        current.device_scale = 2.0;
        registry.configure_source(source, current).unwrap();
        assert!(!registry.frame_info(frame.frame).unwrap().current);
        assert!(registry.acquire_frame(frame.frame, original).is_err());
        assert!(registry.acquire_frame(frame.frame, current).is_err());
        assert!(registry
            .render_cpu(snapshot, original, region(1.0), 1.0)
            .is_err());
        let before = registry.stats().unwrap();
        assert!(registry.configure_source(source, original).is_err());
        assert_eq!(registry.stats().unwrap(), before);
        let current_frame = registry
            .render_cpu(snapshot, current, region(1.0), 1.0)
            .unwrap();
        let replaced = runtime
            .try_submit(NativeRuntimeCommand::New {
                replace: Some(info.target()),
            })
            .unwrap();
        assert!(matches!(
            receive(&mut runtime, replaced),
            NativeRuntimeOutput::Info(_)
        ));
        // A runtime document change invalidates publication even before the UI
        // has reconfigured its source with a new view/device stamp.
        assert!(!registry.frame_info(current_frame.frame).unwrap().current);
        assert!(registry
            .acquire_frame(current_frame.frame, current)
            .is_err());
        assert!(registry
            .render_cpu(snapshot, current, region(1.0), 1.0)
            .is_err());
        registry.release_frame(current_frame.frame).unwrap();
        finish(&mut runtime);
        registry.close_source(source).unwrap();
        assert!(registry
            .render_cpu(snapshot, current, region(1.0), 1.0)
            .is_err());
        assert!(!registry.snapshot_info(snapshot).unwrap().current);
        assert!(!registry.lease_info(lease).unwrap().frame.current);
        registry.release_frame(frame.frame).unwrap();
        let (pointer, length) = registry.lease_data(lease).unwrap();
        assert_eq!(
            unsafe { std::slice::from_raw_parts(pointer, length) },
            expected
        );
        registry.release_lease(lease).unwrap();
        registry.release_snapshot(snapshot).unwrap();
        assert_eq!(registry.stats().unwrap().frame_bytes, 0);
    }

    #[test]
    fn rejected_geometry_identity_and_byte_reservations_leave_registry_state_unchanged() {
        let registry = NativePresentationRegistry::new(NativePresentationConfig {
            max_frame_bytes: 4,
            ..Default::default()
        })
        .unwrap();
        let (mut runtime, info) = open(NativeRuntimeConfig::default());
        let stamp = stamp(&info);
        let source = registry.create_source(runtime.observer(), stamp).unwrap();
        let snapshot = registry
            .retain_snapshot(
                source,
                info.document_id.as_u64(),
                snapshot(&mut runtime, &info),
            )
            .unwrap();
        let before = registry.stats().unwrap();
        assert!(registry
            .render_cpu(snapshot, stamp, region(2.0), 1.0)
            .is_err());
        assert!(registry
            .render_cpu(snapshot, stamp, region(1.0), f64::NAN)
            .is_err());
        let mut foreign = stamp;
        foreign.document_id += 1;
        assert!(registry
            .render_cpu(snapshot, foreign, region(1.0), 1.0)
            .is_err());
        assert!(registry.release_frame(snapshot).is_err());
        assert_eq!(registry.stats().unwrap(), before);
        let frame = registry
            .render_cpu(snapshot, stamp, region(1.0), 1.0)
            .unwrap();
        assert_eq!(registry.stats().unwrap().frame_bytes, 4);
        registry.release_frame(frame.frame).unwrap();
        registry.release_snapshot(snapshot).unwrap();
        registry.close_source(source).unwrap();
        finish(&mut runtime);
    }
}
