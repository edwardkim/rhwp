//! 관계형 계약: typed GPU 실패와 독립 CPU batch frame의 예산·수명.
//! 갱신: 2026-11-07 (KST). 실제 context 전환/CPU↔GL 품질은 Qt probe가 검사한다.
//! 합성 입력이며 실제 하드웨어 device-loss·전체 VRAM·한컴 일치율 인증이 아니다.
#![cfg(not(target_arch = "wasm32"))]

#[cfg(feature = "native-skia")]
use rhwp::native_host::presentation::{NativePagePlacement, NativePresentationId};
use rhwp::native_host::presentation::{
    NativePresentationConfig, NativePresentationRegistry, NativePresentationStamp,
    NativePresentationViewport,
};
use rhwp::native_host::NativeHostError;

#[cfg(not(feature = "native-skia"))]
#[test]
fn cpu_batch_without_native_skia_is_explicitly_disabled() {
    let registry = NativePresentationRegistry::new(NativePresentationConfig::default()).unwrap();
    assert!(matches!(
        registry.render_cpu_batch(
            NativePresentationStamp {
                document_id: 1,
                revision: 0,
                font_generation: 0,
                view_generation: 1,
                device_generation: 1,
                device_scale: 1.0,
            },
            NativePresentationViewport {
                width: 1,
                height: 1
            },
            &[],
            0
        ),
        Err(NativeHostError::FeatureDisabled)
    ));
}

#[test]
fn physical_cpu_and_gl_viewports_share_checked_rgba_layout_rules() {
    let viewport = NativePresentationViewport {
        width: 16,
        height: 8,
    };
    assert_eq!(
        viewport.rgba_layout(4096, 16 * 1024 * 1024).unwrap(),
        (64, 512)
    );
    for value in [
        NativePresentationViewport {
            width: 0,
            ..viewport
        },
        NativePresentationViewport {
            width: i32::MAX,
            ..viewport
        },
        NativePresentationViewport {
            height: -1,
            ..viewport
        },
    ] {
        assert!(value.rgba_layout(4096, 16 * 1024 * 1024).is_err());
    }
    assert!(viewport.rgba_layout(4096, 127).is_err());
}

#[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
#[test]
fn only_confirmed_device_loss_is_a_device_failure_not_resource_or_stale_errors() {
    use rhwp::native_host::gpu_gl::{NativeGlError, NativeGlFailureKind};
    use rhwp::native_host::presentation::NativePresentationBorrowError;
    for kind in [
        NativeGlFailureKind::AllocationFailed,
        NativeGlFailureKind::SubmissionFailed,
        NativeGlFailureKind::BackendUnavailable,
        NativeGlFailureKind::ContextCreationFailed,
        NativeGlFailureKind::TargetUnavailable,
        NativeGlFailureKind::ContextNotCurrent,
        NativeGlFailureKind::RecoveryBudgetExhausted,
        NativeGlFailureKind::RecoveryFailed,
    ] {
        let info = NativeGlError::new(kind, "synthetic classification contract").info();
        assert!(!info.device_failure);
        assert_eq!(info.code, kind.code());
    }
    assert!(
        NativeGlError::new(NativeGlFailureKind::DeviceLost, "SDK-reported loss")
            .info()
            .device_failure
    );
    let resource = NativeGlError::from(NativeHostError::LimitExceeded("quota".into())).info();
    assert_eq!(resource.domain, "resource");
    assert!(!resource.device_failure);
    let stale = NativeGlError::from(NativePresentationBorrowError::Stale).info();
    assert_eq!(stale.code, "staleFrame");
    assert!(stale.retryable && !stale.requires_recreate);
    let invalid = NativeGlError::from(NativePresentationBorrowError::InvalidHandle).info();
    assert_eq!(invalid.code, "invalidArgument");
    assert!(!invalid.retryable);
}

#[cfg(feature = "native-skia")]
#[test]
fn cpu_batch_stages_premultiplied_pixels_and_keeps_leases_after_close() {
    use rhwp::native_host::render_snapshot::NativeRenderSnapshotLimits;
    use rhwp::native_host::runtime::{
        NativeRuntime, NativeRuntimeCommand, NativeRuntimeConfig, NativeRuntimeOutput,
    };
    use rhwp::paint::RenderProfile;
    use std::time::{Duration, Instant};
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig::default()).unwrap();
    let receive = |runtime: &mut NativeRuntime| {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(completion) = runtime.try_recv().unwrap() {
                break completion.result.unwrap();
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
    };
    runtime
        .try_submit(NativeRuntimeCommand::New { replace: None })
        .unwrap();
    let NativeRuntimeOutput::Info(info) = receive(&mut runtime) else {
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
        max_frames: 1,
        max_frame_bytes: 16,
        max_leases: 1,
        ..Default::default()
    })
    .unwrap();
    let source = registry.create_source(runtime.observer(), stamp).unwrap();
    runtime
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
    } = receive(&mut runtime)
    else {
        panic!("expected snapshot")
    };
    let snapshot = registry
        .retain_snapshot(source, document_id.as_u64(), snapshot)
        .unwrap();
    let placement = NativePagePlacement {
        snapshot,
        scale: 1.0,
        origin_x: 1.0,
        origin_y: 1.0,
        background_color: Some(0xffffff),
    };
    let viewport = NativePresentationViewport {
        width: 2,
        height: 2,
    };
    let frame = std::thread::scope(|scope| {
        scope
            .spawn(|| registry.render_cpu_batch(stamp, viewport, &[placement], 0x80402010))
            .join()
            .unwrap()
            .unwrap()
    });
    assert_eq!(frame.coordinate_space, "viewportPixels");
    assert_eq!(frame.page_count, 1);
    let lease = registry.acquire_frame(frame.frame, stamp).unwrap().lease;
    let (pointer, length) = registry.lease_data(lease).unwrap();
    let bytes = unsafe { std::slice::from_raw_parts(pointer, length) }.to_vec();
    assert_eq!(&bytes[..4], &[32, 16, 8, 128]);
    assert_eq!(&bytes[12..], &[255, 255, 255, 255]);
    registry.release_frame(frame.frame).unwrap();
    let before = registry.stats().unwrap();
    assert!(matches!(
        registry.render_cpu_batch(stamp, viewport, &[placement], 0),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert_eq!(registry.stats().unwrap(), before);
    let bad = NativePagePlacement {
        snapshot: NativePresentationId::from_raw(0),
        ..placement
    };
    assert!(registry
        .render_cpu_batch(stamp, viewport, &[bad], 0)
        .is_err());
    assert_eq!(registry.stats().unwrap(), before);
    runtime.close();
    registry.close_source(source).unwrap();
    registry.release_snapshot(snapshot).unwrap();
    let (pointer, length) = registry.lease_data(lease).unwrap();
    assert_eq!(
        unsafe { std::slice::from_raw_parts(pointer, length) },
        bytes
    );
    registry.release_lease(lease).unwrap();
    assert_eq!(registry.stats().unwrap().frame_bytes, 0);
    let deadline = Instant::now() + Duration::from_secs(20);
    while !runtime.is_finished() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(runtime.try_join().unwrap());
}
