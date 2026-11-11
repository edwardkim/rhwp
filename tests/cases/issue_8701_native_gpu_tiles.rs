//! 관계형: 공통 GPU 타일 키·LRU pin·count/byte admission 사이의 관계를 검사한다.
//! 주석 갱신: 2026-11-12T02:22:57+09:00; 신규 파일.
//! 입력은 합성 blank snapshot 또는 자원 drop 계수이며 설치 글꼴에 의존하지 않는다.
//! API·할당량·수명 계약이며 실제 문서의 절대 좌표·golden·조판 일치를 고정하지 않는다.
//! Native/fresh WASM·독립 한컴 PDF의 실물 시각 측정은 이 assertion에 비해당이다.
//! Apple SDK/실제 Metal 실행은 Linux에서 미실행이며 portable 통과로 대신하지 않는다.
//! 검증 경로와 범위: bindings/Flutter/macos/README.md의 내부 GPU 타일 캐시 절.

#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use rhwp::native_host::{
    presentation::NativePresentationStamp, render_snapshot::NativeRenderSnapshot,
    NativeDocumentSession,
};
use rhwp::paint;
use rhwp::paint::RenderProfile;
use std::{cell::RefCell, rc::Rc};

// Include the real private policy without widening the library API. The adapter
// only supplies the same existing snapshot/stamp/error types as the production parent.
#[allow(dead_code)]
#[path = "../support/native_gpu_tiles.rs"]
mod native_host;
use native_host::gpu_tiles::*;

fn config() -> TileConfig {
    TileConfig {
        max_dimension: 4096,
        max_pixels: 4 * 1024 * 1024,
        max_target_bytes: 16 * 1024 * 1024,
        tile_size: 128,
        max_cached_tiles: 64,
        max_tile_bytes: 16 * 1024 * 1024,
    }
}

fn stamp() -> NativePresentationStamp {
    NativePresentationStamp {
        document_id: 1,
        revision: 0,
        font_generation: 0,
        view_generation: 1,
        device_generation: 1,
        device_scale: 1.0,
    }
}

fn target() -> TileTarget {
    TileTarget {
        context_id: 1,
        width: 32,
        height: 24,
    }
}

fn placement() -> TilePlacement {
    TilePlacement {
        scale: 1.0,
        origin_x: 0.0,
        origin_y: 0.0,
        background_color: Some(0xffffff),
    }
}

fn blank_snapshot(profile: RenderProfile) -> NativeRenderSnapshot {
    NativeDocumentSession::new_blank()
        .unwrap()
        .render_snapshot(0, profile)
        .unwrap()
}

fn plan(
    snapshot: &NativeRenderSnapshot,
    stamp: NativePresentationStamp,
    target: TileTarget,
    placement: TilePlacement,
) -> TilePlan {
    TilePlan::new(config(), stamp, target, placement, snapshot, 0xffffffff).unwrap()
}

#[test]
fn integer_scroll_reuses_tiles_but_phase_scale_source_and_device_changes_miss() {
    let snapshot = blank_snapshot(RenderProfile::Screen);
    let original = plan(&snapshot, stamp(), target(), placement());
    assert_eq!(original.keys.len(), 1);
    let scroll = TilePlacement {
        origin_x: -16.0,
        origin_y: -8.0,
        ..placement()
    };
    let next_view = NativePresentationStamp {
        view_generation: 2,
        ..stamp()
    };
    let scrolled = plan(&snapshot, next_view, target(), scroll);
    assert!(original.keys == scrolled.keys);
    assert_eq!((scrolled.integer_x, scrolled.integer_y), (-16, -8));
    for changed in [
        TilePlacement {
            origin_x: 0.25,
            ..placement()
        },
        TilePlacement {
            origin_y: -0.25,
            ..placement()
        },
        TilePlacement {
            scale: 1.25,
            ..placement()
        },
        TilePlacement {
            background_color: Some(0xff0000),
            ..placement()
        },
    ] {
        assert!(original.keys != plan(&snapshot, stamp(), target(), changed).keys);
    }
    for changed in [
        NativePresentationStamp {
            document_id: 2,
            ..stamp()
        },
        NativePresentationStamp {
            revision: 1,
            ..stamp()
        },
        NativePresentationStamp {
            font_generation: 1,
            ..stamp()
        },
        NativePresentationStamp {
            device_generation: 2,
            ..stamp()
        },
        NativePresentationStamp {
            device_scale: 2.0,
            ..stamp()
        },
    ] {
        assert!(original.keys != plan(&snapshot, changed, target(), placement()).keys);
    }
    assert!(
        original.keys
            != plan(
                &snapshot,
                stamp(),
                TileTarget {
                    context_id: 2,
                    ..target()
                },
                placement()
            )
            .keys
    );
    let print = blank_snapshot(RenderProfile::Print);
    assert!(original.keys != plan(&print, stamp(), target(), placement()).keys);
}

#[test]
fn plan_falls_back_before_allocation_when_alpha_or_any_bound_cannot_fit() {
    let snapshot = blank_snapshot(RenderProfile::Screen);
    let build = |config, placement, clear| {
        TilePlan::new(config, stamp(), target(), placement, &snapshot, clear)
    };
    assert!(build(config(), placement(), 0x80ffffff).is_none());
    assert!(build(
        config(),
        TilePlacement {
            background_color: None,
            ..placement()
        },
        0xffffffff
    )
    .is_none());
    for limited in [
        TileConfig {
            max_cached_tiles: 0,
            ..config()
        },
        TileConfig {
            max_tile_bytes: 1,
            ..config()
        },
        TileConfig {
            max_dimension: 1,
            ..config()
        },
        TileConfig {
            max_pixels: 1,
            ..config()
        },
        TileConfig {
            max_target_bytes: 1,
            ..config()
        },
        TileConfig {
            tile_size: 0,
            ..config()
        },
    ] {
        assert!(build(limited, placement(), 0xffffffff).is_none());
    }
    assert!(build(
        config(),
        TilePlacement {
            origin_x: 4096.0,
            ..placement()
        },
        0xffffffff
    )
    .is_none());
}

fn tile(key: TileKey, budget: &TargetBudget) -> Rc<CachedTile> {
    let region = key.region;
    Rc::new(CachedTile {
        key,
        target: Rc::new(RefCell::new(ManagedTarget::new(
            skia_safe::surfaces::raster_n32_premul((region.width, region.height)).unwrap(),
            region.width,
            region.height,
            budget.reserve(region.bytes()).unwrap(),
        ))),
    })
}

#[test]
fn cache_lru_skips_frame_pins_and_counts_bytes_until_last_owner_drops() {
    let snapshot = blank_snapshot(RenderProfile::Screen);
    let keys = plan(
        &snapshot,
        stamp(),
        TileTarget {
            width: 256,
            ..target()
        },
        placement(),
    )
    .keys;
    assert_eq!(keys.len(), 2);
    let bytes = keys[0].region.bytes();
    let budget = TargetBudget::new(2, bytes * 2);
    let mut cache = TileCache::default();
    cache.insert(tile(keys[0].clone(), &budget));
    cache.insert(tile(keys[1].clone(), &budget));
    assert!(!cache.has_room(2, bytes * 2, 1, bytes));
    assert!(!cache.has_room(3, bytes * 2, 1, bytes));
    let pin = cache.get(&keys[0]).unwrap();
    assert!(cache.evict_idle()); // The older unpinned tile is removed.
    assert!(cache.get(&keys[1]).is_none());
    assert!(!cache.evict_idle());
    assert_eq!(budget.stats().targets, 1);
    cache.clear();
    assert_eq!(cache.stats().cached_tiles, 0);
    assert_eq!(budget.stats().estimated_bytes, bytes);
    assert!(budget.reserve(bytes + 1).is_err());
    drop(pin);
    assert_eq!(budget.stats().targets, 0);
    assert_eq!(budget.stats().estimated_bytes, 0);
    assert_eq!(cache.stats().hits, 1);
    assert_eq!(cache.stats().misses, 1);
    assert_eq!(cache.stats().evictions, 2);
}

#[test]
fn successful_lookup_moves_an_unpinned_tile_to_the_lru_tail() {
    let snapshot = blank_snapshot(RenderProfile::Screen);
    let keys = plan(
        &snapshot,
        stamp(),
        TileTarget {
            width: 256,
            ..target()
        },
        placement(),
    )
    .keys;
    let budget = TargetBudget::new(2, 2 * 128 * 128 * 4);
    let mut cache = TileCache::default();
    cache.insert(tile(keys[0].clone(), &budget));
    cache.insert(tile(keys[1].clone(), &budget));
    drop(cache.get(&keys[0]).unwrap());
    assert!(cache.evict_idle());
    assert!(cache.get(&keys[0]).is_some());
    assert!(cache.get(&keys[1]).is_none());
}

#[test]
fn target_reservations_are_atomic_and_refund_only_their_own_credit() {
    let budget = TargetBudget::new(2, 100);
    let scratch = budget.reserve(60).unwrap();
    let canonical = budget.reserve(40).unwrap();
    assert!(!budget.can_reserve(usize::MAX, usize::MAX));
    assert!(budget.reserve(1).is_err());
    assert_eq!(budget.stats().targets, 2);
    assert_eq!(budget.stats().estimated_bytes, 100);
    drop(canonical);
    assert!(!budget.can_reserve(1, 41));
    let tile = budget.reserve(40).unwrap();
    drop(scratch);
    assert_eq!(budget.stats().estimated_bytes, 40);
    drop(tile);
    assert_eq!(budget.stats().estimated_bytes, 0);
    assert_eq!(budget.stats().peak_targets, 2);
    assert_eq!(budget.stats().peak_estimated_bytes, 100);
    assert_eq!(budget.stats().rejected_admissions, 1);
}
