---
kind: reference
status: active
canonical: mydocs/tech/native_skia_region.md
last_verified: 2026-10-26
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

## Native session retention

`NativeDocumentSession::render_region` retains up to four prepared page/scale
snapshots in a least-recently-used (LRU) cache. The 8 MiB default and 64 MiB
maximum limits apply to their **combined** N32 storage. The four-entry cap also
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
