---
kind: reference
status: active
canonical: mydocs/tech/native_skia_region.md
last_verified: 2026-11-06
---

# Native Skia region rendering

With `native-skia`, `SkiaRegionPlan::new(page_width, page_height, region,
options)` validates a request without constructing a renderer, allocating a
bitmap, discovering fonts, or replaying a page. Page and region coordinates are
logical CSS pixels; `options.scale` maps them to the device-pixel grid.

```rust,ignore
use rhwp::renderer::skia::SkiaRegionPlan;

let plan = SkiaRegionPlan::new(tree.page_width, tree.page_height, region, options)?;
let origin = (plan.origin_x(), plan.origin_y());
let output_bytes = plan.output_bytes();
let simultaneous_bitmap_bytes = plan.peak_bitmap_bytes();
let output = renderer.render_region_pixels_with_options(&tree, region, options)?;
assert_eq!((output.origin_x, output.origin_y), origin);
assert_eq!(output.pixels.bytes.len(), output_bytes);
```

The output encloses the request: left/top use floor and right/bottom use ceil
after scaling. `intersection()` returns exclusive page-pixel copy bounds or
`None` for a wholly off-page request. Output pixels outside the page retain the
requested transparent, white, or explicit opaque background.

The renderer consumes this same plan. It keeps the full page's surface size,
device origin, transform, and clip before copying the intersection. This
preserves exact full-page crop pixels at fractional scales; translating replay
onto a small surface can change Skia antialiasing. It is not subtree culling.

Limits apply independently:

| Quantity | Limit |
| --- | --- |
| Output dimensions/pixels | `options.max_dimension` / `options.max_pixels` |
| Working page dimensions | 16,384 per axis |
| Working page pixels | 16,777,216 |
| Working N32 + output RGBA | 128 MiB |
| Scaled page extent and absolute window endpoints | 2²⁴ pixels |

A wholly off-page request has zero working bytes; output and coordinate limits
still apply. Output geometry/limits are checked before working-raster limits.
The bitmap accounting excludes Skia intermediates, renderer caches, caller-owned
buffers, and total process memory. Preflight success is not an allocation or
render-success guarantee.

`tests/cases/skia_region_plan.rs` covers the allocation-free boundaries and
agreement with real output buffers. `tests/cases/issue_536_native_viewport.rs`
checks exact crop pixels for antialiasing, groups, clips, gradients, and effects.
These synthetic contracts do not replace independent Hancom visual comparison.

## Repeated regions from an immutable page

`SkiaLayerRenderer::prepare_page_for_region(&tree, region, options)` returns the
first region and `Option<PreparedSkiaPage>`. An intersecting request rasterizes
once at the original full-page device origin; the returned snapshot owns an
immutable N32 image independently of the renderer/tree. The mutable surface is
released before returning, so later reads cannot trigger its copy-on-write.
There is no additional full-page RGBA copy. A wholly off-page first request
returns its clear output without a snapshot or replay.

`PreparedSkiaPage::render_region(region)` validates each window using the same
options and region plan, then copies its exact pixels without replay or
resampling. `retained_bytes()` reports the N32 image storage. Retained N32 plus
each new output RGBA must fit 128 MiB, including wholly off-page requests.
Existing caller-owned output buffers and Skia intermediates remain outside this
bitmap accounting. The snapshot does not establish a cross-thread/GPU contract.

## 선택적 고정 격자 타일 경로

`SkiaLayerRenderer::render_tiled_region_pixels_with_options`와
`NativeDocumentSession::render_tiled_region`은 전체 페이지 working raster 한도를
넘는 페이지에서 작은 영역을 렌더링합니다. `SkiaTilePlan::new`는 bitmap을 할당하지
않고 영역·출력·메모리를 검증하며, 필요한 타일 수를 보고합니다.

페이지를 장치 픽셀 512칸의 고정 격자로 나누고, 요청 영역과 만나는 타일마다 전체 paint
순서를 재생합니다. 각 타일은 두 픽셀 guard를 포함하며, scaled page의 정수 경계로만
잘립니다. 타일의 원점·변환·크기는 요청 창과 독립적이므로 겹치는 창은 같은 타일 픽셀을
읽습니다. 음수·페이지 밖 padding에는 기존 투명·흰색·지정 배경 정책을 적용합니다.

한 번에 expanded N32 working 타일 하나만 할당합니다. 최종 RGBA 출력과 최대
`516 × 516 × 4` bytes의 보수적인 working 한도를 합산해 128 MiB를 검사합니다.
Native session은 기존 전체 페이지 snapshot도 합산하며 필요할 때 LRU 항목을 먼저
제거합니다. 완전히 페이지 밖인 요청은 타일 할당·replay를 하지 않습니다. 이 수치는
renderer 자원·Skia 중간 surface·호출자가 보관하는 이전 출력을 제외합니다.

이 API의 타일 픽셀 계약은 별도입니다. Skia의 장치 원점 이동에 따라 AA·부동소수점
반올림이 달라질 수 있으므로 전체 페이지 crop과의 무조건적인 일치를 보장하지 않습니다.
두 픽셀 guard도 임의 filter 범위를 보장하지 않습니다. 기존 exact-crop API와 snapshot
cache는 전체 페이지 계약을 유지합니다. 캐시가 없는 타일은 전체 tree를 재생하므로
subtree culling이나 처리 속도·GPU·프로세스 RSS 개선을 완료한 것으로 해석하지 않습니다.
Native 세션의 재방문 타일 보존은 아래 공유 캐시 계약을 따릅니다.

```rust,ignore
let plan = SkiaTilePlan::new(tree.page_width, tree.page_height, region, options)?;
let pixels = renderer.render_tiled_region_pixels_with_options(&tree, region, options)?;
assert_eq!(pixels.pixels.bytes.len(), plan.output_bytes());
```

집중 회귀는 단일 타일과 전체 페이지의 일치, 분수 배율의 AA·gradient·clip 경계에서
겹치는 창의 일치, 마지막 페이지 픽셀·배경·padding, 실제 picture cache hit와 비활성
경로의 일치, 큰 페이지·고배율 Native frame을 검사합니다. region plan case는 메모리
할당 전에 큰 페이지 허용·타일 수·출력과 working 합산 한도의 거부를 검사합니다.

## Native session retention

`NativeDocumentSession::render_region` retains up to four prepared page/scale
snapshots in a least-recently-used (LRU) cache. The 8 MiB default and 64 MiB
maximum limits apply to **combined page and tile** N32 storage. The four-page cap also
bounds metadata for very small rasters. `set_region_cache_limit(bytes)` accepts
zero to disable retention. Changing the limit evicts all entries; an invalid
limit leaves the cache, recency order, and counters unchanged.

The key includes document revision, font generation, page index, and the exact
scale bits. A matching warm request skips page-tree construction and paint
replay and marks the entry most recently used. For example, `page 0 → page 1 →
page 0` performs two replays when both rasters fit. Different scales are separate
entries. Invalid requests do not promote an entry or evict another page.
Successful edits, IME preview/cancel, undo/redo, font changes, and `clear_caches`
invalidate all snapshots. Rejected/no-op font changes preserve them.

Before a cacheable miss allocates its surface, the least recently used entries
are released until the new raster fits both the combined byte limit and the
four-entry cap. The planner also checks all remaining images plus the new
working raster and output against the 128 MiB bitmap limit. An oversized page
preserves the existing bypass behavior: all retained images are released before
it renders, and its snapshot is discarded afterward. No new tile origin or
translated-surface rasterization is introduced.

Cold-key off-page requests do not replay and preserve useful cached pages when
all retained images plus output fit the bitmap limit. A warm read counts all
retained images too, including off-page reads; the session can evict first to
preserve the direct API's clear-only allowance. At the current native 64 MiB
frame and total cache caps, their sum is already at most 128 MiB. These limits
exclude earlier output buffers still owned by callers and Skia intermediates.

`region_cache_stats()` exposes current entries/retained bytes and lifetime hit,
miss, replay, eviction, invalidation, oversized-bypass, and cold-key off-page
counters. Replay counts successful page rasterizations; hits count successful
matching snapshot reads. Clearing the cache preserves lifetime counters. These
are Rust host diagnostics; this change adds no C ABI or wire command.

`tests/cases/issue_536_native_region_cache.rs` compares cold/warm AA, gradient,
clip, fractional scale, and padding pixels with independently rendered full
pages. It also covers snapshot ownership, bounds, replay counts, retention
disable/bypass, failed requests, edit/IME/font invalidation, page/scale reuse,
LRU eviction under entry/byte limits, and off-page reads. It makes no wall-clock
speed, subtree-culling, presentation,
GPU acceleration, or final Hancom fidelity claim.

## P73: Native 세션의 타일 재사용

`render_tiled_region`은 완성된 immutable N32 타일을 전체 페이지 snapshot과 같은 LRU에
보관합니다. 기본 8 MiB·최대 64 MiB는 두 종류를 합친 한도이며,
`set_region_cache_limit(0)`은 둘 다 끕니다. 전체 페이지는 최대 4개, 타일은 최대 64개로
metadata도 제한합니다. 한도 변경은 둘 다 비우고 잘못된 한도는 기존 상태를 보존합니다.

키는 문서 revision·font generation·페이지 번호·배율의 정확한 bit와 고정 격자 타일 원점입니다.
같은 타일을 다시 요청하면 paint replay 없이 보존된 image에서 RGBA를 읽습니다.
타일 image와 직접 재생은 같은 격자·guard·변환·복사 코드를 쓰며 픽셀 계약이 같습니다.
tree 생성과 layout을 생략하는 캐시는 아닙니다. 화면 구현이나 C ABI·wire 확장은 추가하지 않습니다.

출력을 할당하기 전과 새 타일을 그리기 전에 보존 page+tile, 출력, working N32의 합을
128 MiB 한도와 대조합니다. 저장 한도·종류별 개수 한도를 초과하면 해당 종류의 가장 오래된
항목 또는 전체 LRU를 먼저 제거합니다. 한 타일조차 보존 한도를 넘으면 기존 이미지를
비우고 그린 뒤 보존하지 않습니다. mutable surface는 snapshot 반환 전에 해제되며 image
clone은 같은 immutable 픽셀을 공유합니다. 총 프로세스 RSS의 상한을 뜻하지 않습니다.

편집·IME·undo/redo·font 변경·`clear_caches`는 기존 무효화 경로에서 두 종류를 함께 지웁니다.
잘못된 페이지·배율·영역은 cache lookup 전에 거부합니다. 중간 replay가 실패한 요청은
그 전에 완성한 타일만 남을 수 있으며 실패한 타일을 저장하지 않습니다.

`region_cache_stats()`의 `entries`와 기존 hit/miss/replay/eviction은 전체 페이지용입니다.
`retained_bytes`는 page+tile 합계이며 `tile_entries`·`tile_retained_bytes`는 그중 타일 양입니다.
`tile_hits`/`tile_misses`는 타일 lookup, `tile_replays`는 완료한 타일 재생,
`tile_evictions`/`tile_bypassed_oversized`는 타일 제거/보존 생략을 셉니다.
요청 중간 실패 전의 완료된 lookup/replay도 누적되며 clear로 누적값을 초기화하지 않습니다.

회귀는 기존 Native cache case 안에서 자동 실행합니다. cold/warm 및 겹치는 영역의 픽셀·재생 수,
page/tile 공유 예산과 LRU, 비활성화·큰 타일 우회, 잘못된 입력 보존, page/scale 분리 및
편집·IME·font·clear 무효화를 검사합니다. 최종 한컴·전체 코퍼스·실물 OS/GPU 검증은 사용자
후속 범위이며 아래 2026-10-30 실물 증거를 P73에서 새로 실행한 검증으로 해석하지 않습니다.

## 메인테이너 실물 검증

2026-10-30 네 실제 HWP 입력에서 scale 0.75/1/1.25의 full/prepared/host 출력,
고정 타일의 겹치는 viewport, 캐시 왕복과 무효화 계약을 검증했다. 독립 Print와의 타일
시각 비교는 scale 1(96 dpi)에서 수행했고 전쪽 최저는 99.66%였다. 두 페이지로 복제한
진단 입력의 0→1→0 요청은 base replay 3/hit 0에서 후보 replay 2/hit 1로 바뀌었고,
모든 frame은 해당 페이지의 full-page crop와 같았다. 전체 프로세스 메모리나 OS 표시 속도의
개선률은 측정하지 않았다. [입력·코드·결과](../pr/assets/seo_rii_20261030/README.md)를 참고한다.
