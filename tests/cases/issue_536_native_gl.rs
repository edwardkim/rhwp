//! 관계형 계약: GL owner의 입력 한도와 registry borrow의 예산·세대 수명.
//! 갱신: 2026-11-07 (KST). 실제 GL 실행은 별도 Qt/ctypes probe가 검사한다.
//! 합성 계약이며 실물 페이지 좌표·한컴 일치율·하드웨어 성능 인증이 아니다.
#![cfg(not(target_arch = "wasm32"))]

use std::time::{Duration, Instant};

use rhwp::native_host::presentation::{
    NativePresentationConfig, NativePresentationRegistry, NativePresentationSnapshotBorrow,
    NativePresentationStamp,
};
use rhwp::native_host::render_snapshot::NativeRenderSnapshotLimits;
use rhwp::native_host::runtime::{
    NativeRequestId, NativeRuntime, NativeRuntimeCommand, NativeRuntimeConfig, NativeRuntimeOutput,
};
use rhwp::paint::RenderProfile;

fn receive(runtime: &mut NativeRuntime, request: NativeRequestId) -> NativeRuntimeOutput {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(completion) = runtime.try_recv().unwrap() {
            assert_eq!(completion.request_id, request);
            return completion.result.unwrap();
        }
        assert!(Instant::now() < deadline, "snapshot input timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn registry_snapshot_borrow_pins_the_budget_and_reuses_input_after_view_changes() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<NativePresentationSnapshotBorrow>();
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig::default()).unwrap();
    let request = runtime
        .try_submit(NativeRuntimeCommand::New { replace: None })
        .unwrap();
    let NativeRuntimeOutput::Info(info) = receive(&mut runtime, request) else {
        panic!("expected info")
    };
    let stamp = NativePresentationStamp {
        document_id: info.document_id.as_u64(),
        revision: info.revision,
        font_generation: info.font_generation,
        view_generation: 1,
        device_generation: 1,
        device_scale: 1.0,
    };
    let registry = NativePresentationRegistry::new(NativePresentationConfig {
        max_snapshots: 1,
        ..Default::default()
    })
    .unwrap();
    let source = registry.create_source(runtime.observer(), stamp).unwrap();
    let request = runtime
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
    } = receive(&mut runtime, request)
    else {
        panic!("expected snapshot")
    };
    let id = registry
        .retain_snapshot(source, document_id.as_u64(), snapshot.clone())
        .unwrap();
    let borrowed = registry.borrow_snapshot(id, stamp).unwrap();
    let usage = borrowed.snapshot().usage();
    registry.release_snapshot(id).unwrap();
    let stats = registry.stats().unwrap();
    assert_eq!(stats.snapshot_handles, 0);
    assert_eq!(stats.retained_snapshots, 1);
    assert_eq!(stats.snapshot_estimated_bytes, usage.estimated_bytes);
    assert!(registry
        .retain_snapshot(source, document_id.as_u64(), snapshot.clone())
        .is_err());
    drop(snapshot);
    let changed = NativePresentationStamp {
        view_generation: 2,
        device_scale: 2.0,
        ..stamp
    };
    registry.configure_source(source, changed).unwrap();
    assert!(borrowed.validate_current(stamp).is_err());
    borrowed.validate_current(changed).unwrap();
    assert_eq!(borrowed.snapshot().page(), 0);
    runtime.close();
    assert!(borrowed.validate_current(changed).is_err());
    registry.close_source(source).unwrap();
    drop(borrowed);
    assert_eq!(registry.stats().unwrap().retained_snapshots, 0);
    assert_eq!(registry.stats().unwrap().snapshot_estimated_bytes, 0);
    let deadline = Instant::now() + Duration::from_secs(20);
    while !runtime.is_finished() {
        assert!(Instant::now() < deadline, "owner shutdown timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(runtime.try_join().unwrap());
}

#[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
mod gl {
    use super::*;
    use rhwp::native_host::gpu_gl::{
        NativeGlPlacement, NativeGlPresenterConfig, NativeGlTarget, NativeGlTargetFormat,
        NativeGlTargetOrigin,
    };
    use rhwp::native_host::NativeDocumentSession;

    fn target() -> NativeGlTarget {
        NativeGlTarget {
            context_id: 1,
            device_generation: 1,
            fbo: 0,
            width: 16,
            height: 8,
            format: NativeGlTargetFormat::Rgba8,
            origin: NativeGlTargetOrigin::BottomLeft,
            sample_count: 0,
            stencil_bits: 8,
        }
    }

    #[test]
    fn gl_target_limits_reject_invalid_public_ceilings_geometry_and_multisampling() {
        for config in [
            NativeGlPresenterConfig {
                max_renderers: 9,
                ..Default::default()
            },
            NativeGlPresenterConfig {
                max_scratch_bytes: usize::MAX,
                ..Default::default()
            },
            NativeGlPresenterConfig {
                renderer_cache_bytes: 9 * 1024 * 1024,
                ..Default::default()
            },
            NativeGlPresenterConfig {
                max_pages: 0,
                ..Default::default()
            },
        ] {
            assert!(config.validate().is_err());
        }
        let config = NativeGlPresenterConfig::default();
        assert_eq!(config.target_bytes(target()).unwrap(), 16 * 8 * 4);
        for invalid in [
            NativeGlTarget {
                width: i32::MAX,
                ..target()
            },
            NativeGlTarget {
                height: 0,
                ..target()
            },
            NativeGlTarget {
                context_id: 0,
                ..target()
            },
            NativeGlTarget {
                sample_count: 4,
                ..target()
            },
            NativeGlTarget {
                stencil_bits: 32,
                ..target()
            },
        ] {
            assert!(config.target_bytes(invalid).is_err());
        }
        assert!(NativeGlPresenterConfig {
            max_target_bytes: 511,
            ..config
        }
        .target_bytes(target())
        .is_err());
        assert!(NativeGlPresenterConfig {
            max_pixels: 127,
            ..config
        }
        .target_bytes(target())
        .is_err());
    }

    #[test]
    fn gl_page_placements_reject_nonfinite_and_unrepresentable_values_before_replay() {
        let session = NativeDocumentSession::new_blank().unwrap();
        let snapshot = session.render_snapshot(0, RenderProfile::Screen).unwrap();
        let valid = NativeGlPlacement {
            snapshot: rhwp::native_host::presentation::NativePresentationId::from_raw(1),
            scale: 1.0,
            origin_x: -10.0,
            origin_y: 4.0,
            background_color: Some(0xffffff),
        };
        valid.validate(&snapshot).unwrap();
        for invalid in [
            NativeGlPlacement {
                scale: 0.0,
                ..valid
            },
            NativeGlPlacement {
                scale: f64::MIN_POSITIVE,
                ..valid
            },
            NativeGlPlacement {
                scale: 17.0,
                ..valid
            },
            NativeGlPlacement {
                origin_x: f64::NAN,
                ..valid
            },
            NativeGlPlacement {
                origin_y: f64::MAX,
                ..valid
            },
        ] {
            assert!(invalid.validate(&snapshot).is_err());
        }
    }
}
