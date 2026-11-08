---
kind: reference
status: active
canonical: mydocs/tech/native_skia_host_surface.md
last_verified: 2026-11-07
---

# Host-owned Skia Canvas / Surface replay

`native-skia` exposes `SkiaLayerRenderer::render_to_canvas_strict` and
`render_to_surface_strict`. A native host can replay `PageLayerTree` into an
existing Skia target without converting the whole page to CPU RGBA and uploading
it again. `render_to_surface_strict` borrows the host's `skia_safe::Surface`;
`render_to_canvas_strict` also accepts its borrowed canvas.

```rust
use rhwp::paint::PageLayerTree;
use rhwp::renderer::skia::{SkiaLayerRenderer, SkiaSurfaceRenderOptions};
use skia_safe::Surface;

fn paint_page(
    renderer: &SkiaLayerRenderer,
    surface: &mut Surface,
    page: &PageLayerTree,
) -> rhwp::renderer::layer_renderer::LayerRenderResult<()> {
    renderer.render_to_surface_strict(surface, page, SkiaSurfaceRenderOptions {
        scale: 1.25,
        origin_x: 12.0,
        origin_y: 8.0,
        background_color: Some(0x00ff_ffff), // opaque COLORREF white
    })
}
```

Placement composes with the host's current affine matrix and clip. The origin is
in the current local coordinate system; scale maps page CSS pixels into that
system. An additional page rectangle clip prevents background/content from
painting neighboring pages, even if the tree disables its internal clips.
`background_color: None` preserves existing pixels. `Some` clears only the
intersection of the host and page clips to the requested opaque COLORREF color.
The maximum scale of the resulting device matrix controls resolution-dependent
SVG raster fallbacks, including inherited device-pixel scaling.

호스트가 페이지를 회전하거나 기울여도 SVG 이미지의 해상도 계산은 최종
행렬의 최대 배율을 사용합니다. SVG 문서 이미지의 기존 1–4배 raster 배율
제한은 유지합니다. 4배 확대 후 90도 회전한 SVG 이미지가 회전 전 픽셀을
정확히 회전한 결과와 같은지 회귀 테스트로 확인합니다.

The API restores matrix, clip, and save count on success and error. Invalid
geometry, a non-finite/singular/perspective device matrix, or an invalid text
contract is rejected before background or content changes target pixels. Page
dimensions/placement must remain finite and nondegenerate in Skia's `f32`
coordinates. Replay errors are not pixel transactions: an image/SVG resource
failure can leave the optional background and earlier paint in place. A host
that needs atomic frames should render into its own offscreen target and publish
only on success.

The same ordered page paint replay used by CPU raster and PDF is reused. The
`strict` methods intentionally use the PDF resource policy: a missing/undecodable
image or failed SVG raster fallback returns an error. The CPU RGBA API instead
can paint a placeholder for these cases. Text-variant validation and selection,
layer ordering, internal clips, and renderer caches keep their existing policy;
the API does not promise byte-identical CPU/GPU antialiasing or color management.

The host owns context creation, context/thread lifetime, target allocation,
backend synchronization, flush/submit, and presentation. These methods do not
allocate a destination surface, take a full-page snapshot/readback, or create a
window. Individual resources may still use CPU bitmap/SVG fallbacks. The root
`native-skia` feature keeps its existing CPU-only backend selection. A downstream
host using the same `skia-safe` version can enable its chosen Ganesh backend via
Cargo feature unification and provide a GPU-backed surface.

The integration cases in
[`tests/cases/skia_host_surface.rs`](../../tests/cases/skia_host_surface.rs) cover
borrowed CPU surfaces, inherited transforms/clips, background/page clipping,
fractional-scale pixel parity with the CPU RGBA path, preflight rejection without
pixel changes, and partial-paint/state restoration after a strict resource error.
Those checks alone do not validate a GPU context, hardware acceleration,
Metal/Windows backends, device loss, or an application's frame presentation.

## Optional headless Vulkan host probe

[`tools/skia-surface-probe`](../../tools/skia-surface-probe) is a standalone opt-in
Cargo workspace. It depends on the repository's exact `skia-safe` lock version
with `ganesh,vulkan` enabled, loads Vulkan using `ash`, and creates a small
offscreen Skia Vulkan render target. Its `rhwp` dependency enables `native-skia`
without default features, so the host does not require the browser ABI or PDF
import feature. It calls the public borrowed-surface API,
checks known interior colors and compositor state, and verifies that invalid
arguments leave the GPU target unchanged. Only the probe host flushes and reads
pixels back for verification. Missing Vulkan 1.1 support/context creation is an
error, not a passing skip.

The probe pins `skia-safe` to `0.156.0`, matching the root lock file. Its
standalone lock file must be updated together with this pin when the root Skia
dependency changes: enabling GPU features on a different Skia package does not
enable them on the renderer's package, and borrowed `Surface` types from two
versions cannot be passed between them. Flush/submit errors are propagated
before pixel verification, including errors from the preservation check.

The locked bindgen parser and formatter must also agree: bindgen `0.73.2`
uses Syn `3`, so its `prettyplease` formatter is locked to `0.3.0`, as in the
root workspace. Retaining `prettyplease 0.2` would introduce Syn `2` at that
boundary and fail compilation before the GPU probe can run.

On success it emits a single JSON object with `schemaVersion: 1`. In addition
to the device name and `softwareDevice`, the report records the packed Vulkan
API/driver version and vendor/device identifiers. `deviceInitializationMicros`
and `measurements.timingsMicros` are monotonic host wall times for device/context
initialization, target allocation, replay recording, flush/submit with a CPU
wait, and the two verification readbacks. The flush duration can include work
queued before replay and is not a GPU timestamp. The probe renders one small
synthetic frame; these measurements are diagnostics, not FPS, warm-cache
performance, document latency, or a presentation benchmark.

`measurements.hostTransfers` counts the probe's two explicit `read_pixels`
calls and their destination bytes. It does not instrument hidden driver
transfers or promise that future host adapters perform no readback.
`measurements.skiaResourceCache` exposes Skia's resource count, retained bytes,
and configured limit after verification; those values are not total VRAM,
process memory, or a cross-backend cache budget. `presentationVerified` and
`hardwareAccelerationVerified` remain false: successful offscreen checks do
not establish window presentation, external texture sharing, or hardware
qualification. Context creation/submission/readback failures return a nonzero
exit status rather than a successful skip.

Use an explicit local Cargo target selected under the workspace build rules.
The probe's GPU feature variant can download a different Skia binary cache and
rebuild dependencies; inspect disk space before running it. Generating its lock
file and building it are independent of the root workspace's default builds.

```sh
cargo generate-lockfile --manifest-path tools/skia-surface-probe/Cargo.toml
CARGO_TARGET_DIR="${rhwp_gpu_probe_target_dir:?select a local GPU probe target first}" \
  cargo run --locked --manifest-path tools/skia-surface-probe/Cargo.toml
```

To exercise Mesa lavapipe explicitly on a host that installs it, set
`VK_ICD_FILENAMES` to that host's `lvp_icd.*.json` path before the command. The probe
prints the selected device and whether Vulkan reports it as a CPU device.
Lavapipe validates Skia's Vulkan API path using software rendering; it does not
establish hardware acceleration. Hardware/Metal/Windows validation remains a
separate host/platform task.

The host setup follows the upstream
[`graphite_offscreen_vulkan` example](https://github.com/rust-skia/rust-skia/blob/0.156.0/skia-safe/examples/graphite_offscreen_vulkan.rs)
for instance/device/queue/resolver lifetime, and uses Ganesh
[`make_vulkan`](https://github.com/rust-skia/rust-skia/blob/0.156.0/skia-safe/src/gpu/ganesh/vk/vk_direct_context.rs)
and [`render_target`](https://github.com/rust-skia/rust-skia/blob/0.156.0/skia-safe/src/gpu/ganesh/surface_ganesh.rs)
for the actual borrowed GPU surface.
