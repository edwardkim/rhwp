//! 관계형: 기존 Metal config의 기본값 호환과 캐시 설정 한도를 검사한다.
//! 주석 갱신: 2026-11-12T02:22:57+09:00; 신규 파일.
//! 입력은 합성 blank snapshot 또는 자원 drop 계수이며 설치 글꼴에 의존하지 않는다.
//! API·할당량·수명 계약이며 실제 문서의 절대 좌표·golden·조판 일치를 고정하지 않는다.
//! Native/fresh WASM·독립 한컴 PDF의 실물 시각 측정은 이 assertion에 비해당이다.
//! Apple SDK/실제 Metal 실행은 Linux에서 미실행이며 portable 통과로 대신하지 않는다.
//! 검증 경로와 범위: bindings/Flutter/macos/README.md의 내부 GPU 타일 캐시 절.

#![cfg(all(feature = "native-skia-metal", target_os = "macos"))]

use rhwp::native_host::gpu_metal::NativeMetalPresenterConfig;
const MIB: usize = 1024 * 1024;

#[test]
fn old_config_json_uses_bounded_tile_defaults_and_can_disable_cache() {
    let default: NativeMetalPresenterConfig = serde_json::from_str("{}").unwrap();
    default.validate().unwrap();
    assert_eq!(default.max_cached_tiles, 64);
    assert_eq!(default.max_tile_bytes, 16 * MIB);
    assert_eq!(default.max_managed_targets, 66);
    assert_eq!(default.max_managed_target_bytes, 48 * MIB);
    let disabled = NativeMetalPresenterConfig {
        max_cached_tiles: 0,
        ..default
    };
    disabled.validate().unwrap();
}

#[test]
fn tile_and_total_target_limits_reject_excess_and_invalid_sizes() {
    let default = NativeMetalPresenterConfig::default();
    for invalid in [
        NativeMetalPresenterConfig {
            tile_size: 0,
            ..default
        },
        NativeMetalPresenterConfig {
            max_cached_tiles: 65,
            ..default
        },
        NativeMetalPresenterConfig {
            max_tile_bytes: 64 * MIB + 1,
            ..default
        },
        NativeMetalPresenterConfig {
            max_managed_targets: 0,
            ..default
        },
        NativeMetalPresenterConfig {
            max_managed_targets: 67,
            ..default
        },
        NativeMetalPresenterConfig {
            max_managed_target_bytes: 96 * MIB + 1,
            ..default
        },
    ] {
        assert!(invalid.validate().is_err());
    }
}
