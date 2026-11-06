//! 관계형 계약: GPU tile/중간 target의 공개 admission 한도와 이전 JSON config 호환.
//! 갱신: 2026-11-07 (KST). 실제 재사용·픽셀 관계는 Qt/ctypes probe에서 검사한다.
//! 실물 문서 좌표·한컴 일치율·driver VRAM의 정확한 상한을 고정하지 않는다.
#![cfg(all(feature = "native-skia-gl", target_os = "linux"))]

use rhwp::native_host::gpu_gl::{
    NativeGlPresenterConfig, NativeGlTarget, NativeGlTargetFormat, NativeGlTargetOrigin,
};

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
fn tile_and_all_target_limits_reject_overflow_and_preserve_a_direct_replay_configuration() {
    for config in [
        NativeGlPresenterConfig {
            max_cached_tiles: 65,
            ..Default::default()
        },
        NativeGlPresenterConfig {
            max_tile_bytes: usize::MAX,
            ..Default::default()
        },
        NativeGlPresenterConfig {
            tile_size: 127,
            ..Default::default()
        },
        NativeGlPresenterConfig {
            max_managed_targets: 0,
            ..Default::default()
        },
        NativeGlPresenterConfig {
            max_managed_targets: 68,
            ..Default::default()
        },
        NativeGlPresenterConfig {
            max_managed_target_bytes: usize::MAX,
            ..Default::default()
        },
    ] {
        assert!(config.validate().is_err());
    }
    let minimal = NativeGlPresenterConfig {
        max_cached_tiles: 0,
        max_tile_bytes: 0,
        max_managed_targets: 1,
        max_managed_target_bytes: 16 * 8 * 4,
        ..Default::default()
    };
    minimal.validate().unwrap();
    assert_eq!(minimal.target_bytes(target()).unwrap(), 512);
    assert!(NativeGlPresenterConfig {
        max_managed_target_bytes: 511,
        ..minimal
    }
    .target_bytes(target())
    .is_err());
}

#[test]
fn earlier_presenter_json_configs_keep_bounded_tile_defaults_without_a_new_required_field() {
    let old: NativeGlPresenterConfig =
        serde_json::from_str(r#"{"maxRenderers":1,"maxScratchTargets":1,"rendererCacheBytes":0}"#)
            .unwrap();
    old.validate().unwrap();
    let defaults = NativeGlPresenterConfig::default();
    assert_eq!(old.tile_size, defaults.tile_size);
    assert_eq!(old.max_cached_tiles, defaults.max_cached_tiles);
    assert_eq!(old.max_tile_bytes, defaults.max_tile_bytes);
    assert_eq!(
        old.max_managed_target_bytes,
        defaults.max_managed_target_bytes
    );
    assert_eq!(old.renderer_cache_bytes, 0);
    let direct: NativeGlPresenterConfig =
        serde_json::from_str(r#"{"maxCachedTiles":0,"maxTileBytes":0}"#).unwrap();
    direct.validate().unwrap();
    assert_eq!(direct.target_bytes(target()).unwrap(), 512);
}
