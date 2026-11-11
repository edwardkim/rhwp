//! 관계형: D3D 캐시 config의 이전 JSON 호환과 자원 상한을 검사한다.
//! 합성 config만 사용하는 API·admission 계약이며 실물 문서 조판 좌표를 고정하지 않는다.
//! Windows SDK 실행·D3D 출력·독립 한컴 PDF 일치의 증거로 사용하지 않는다.

#![cfg(all(feature = "native-skia-d3d", target_os = "windows"))]

use rhwp::native_host::gpu_d3d::NativeD3dPresenterConfig;

const MIB: usize = 1024 * 1024;

#[test]
fn earlier_d3d_configs_keep_bounded_defaults_and_can_disable_tile_reuse() {
    let config: NativeD3dPresenterConfig =
        serde_json::from_str(r#"{"maxPages":2,"rendererCacheBytes":0}"#).unwrap();
    config.validate().unwrap();
    assert_eq!(config.max_pages, 2);
    assert_eq!(config.max_tile_bytes, 16 * MIB);
    assert_eq!(config.max_managed_target_bytes, 48 * MIB);
    let direct: NativeD3dPresenterConfig = serde_json::from_str(
        r#"{"maxCachedTiles":0,"maxTileBytes":0,"maxManagedTargets":1,"maxManagedTargetBytes":512}"#,
    )
    .unwrap();
    direct.validate().unwrap();
    assert_eq!(direct.max_cached_tiles, 0);
    assert_eq!(direct.max_tile_bytes, 0);
    assert_eq!(direct.max_managed_target_bytes, 512);
}

#[test]
fn d3d_tile_and_all_target_ceilings_reject_unbounded_admission() {
    for config in [
        NativeD3dPresenterConfig {
            tile_size: 127,
            ..Default::default()
        },
        NativeD3dPresenterConfig {
            max_cached_tiles: 65,
            ..Default::default()
        },
        NativeD3dPresenterConfig {
            max_tile_bytes: 64 * MIB + 1,
            ..Default::default()
        },
        NativeD3dPresenterConfig {
            max_managed_targets: 0,
            ..Default::default()
        },
        NativeD3dPresenterConfig {
            max_managed_targets: 67,
            ..Default::default()
        },
        NativeD3dPresenterConfig {
            max_managed_target_bytes: usize::MAX,
            ..Default::default()
        },
    ] {
        assert!(config.validate().is_err());
    }
}
