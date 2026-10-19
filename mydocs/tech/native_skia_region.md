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
