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

`NativeDocumentSession::render_region` retains at most one prepared page, with an
8 MiB default limit. `set_region_cache_limit(bytes)` accepts zero to disable
retention and limits up to 64 MiB. Changing the limit evicts the current entry;
an invalid limit leaves the cache and counters unchanged.

The key includes document revision, font generation, page index, and the exact
scale bits. A matching warm request skips page-tree construction and paint
replay. Successful edits, IME preview/cancel, undo/redo, font changes, and
`clear_caches` invalidate the snapshot. Rejected/no-op font changes preserve it.
Page/scale changes replace it; an oversized page still renders and is discarded.
The previous image is released before a replacement surface is allocated.

Cold-key off-page requests do not replay and can preserve another useful cached
page when retained image plus output fit the bitmap limit. A prepared read
counts its image even off-page; the session can evict first to preserve the
direct API's clear-only allowance. At the current native 64 MiB frame and cache
caps, their sum is already at most 128 MiB.

`region_cache_stats()` exposes current entries/retained bytes and lifetime hit,
miss, replay, eviction, invalidation, oversized-bypass, and cold-key off-page
counters. Replay counts successful page rasterizations; hits count successful
matching snapshot reads. Clearing the cache preserves lifetime counters. These
are Rust host diagnostics; this change adds no C ABI or wire command.

`tests/cases/issue_536_native_region_cache.rs` compares cold/warm AA, gradient,
clip, fractional scale, and padding pixels with independently rendered full
pages. It also covers snapshot ownership, bounds, replay counts, retention
disable/bypass, failed requests, edit/IME/font invalidation, page/scale changes,
and off-page reads. It makes no wall-clock speed, subtree-culling, presentation,
GPU acceleration, or final Hancom fidelity claim.
