# CanvasKit Parity Implementation Plan

This document records the implementation plan for closing CanvasKit parity gaps
without treating Canvas2D as a hidden runtime fallback. It is intentionally a
plan, not a claim that every paint family already has complete direct replay.

## Goal

CanvasKit should replay the same user-visible `PageLayerTree` behavior that the
current Canvas2D path can render in the web canvas view. Canvas2D remains the
compatibility reference for behavior, paint order, and HWP-compatible layout,
but CanvasKit direct replay must not depend on Canvas2D drawing, DOM image
objects, or SVG DOM parsing to cover unsupported operations.

The target contract is:

1. Keep `PageLayerTree` as the frontend/backend boundary.
2. Prefer direct replay over approximation.
3. Ensure unsupported operations stay visible through deterministic diagnostics,
   explicit fallback policy, or strict payload rejection.
4. Keep browser-only preprocessing out of CanvasKit unless the data has first
   become a native-ready payload, resource, or pure helper.

## Current Baseline

The current implementation already has a guarded CanvasKit replay path with
explicit `default` and `compat` policy modes. It dispatches the core layer node
kinds, clips, basic page backgrounds, vector primitives, simple raster images,
basic form objects, root `TextRun` compatibility payloads, and the currently
supported `GlyphOutline` color-layer subset. It still treats several text,
image-effect, page-background fill, and document-object families as fallback or
diagnostic work until their payload contract is strict enough for direct replay.

`TextRun compatibility` remains the replay baseline for normal text. `GlyphRun`
and `GlyphOutline` are additive sidecars, not a replacement authority by
themselves. The browser CanvasKit runtime currently keeps `GlyphOutline` direct
replay behind `glyph-outline-payload-status.ts`; Rust-side replay planning and
future strict selection work should keep reporting why a sidecar was selected
or rejected.

Schema-v1 text variants are exported as ordinary `glyphRun` and `glyphOutline`
paint ops with variant metadata plus `text.variantGroups`. Those sidecar ops
must be treated as part of the same leaf-local selection set as the anchored
fallback `TextRun`. Cache keys, replay-plane detection, and backend diagnostics
should include sidecars whenever they can affect output.

`ResourceArena` is the resource identity boundary for future widening. When
new image, font, bitmap glyph, SVG glyph, or PDF/vector resources become
replay-critical, they should move through that resource table instead of
through backend-local browser objects.

## Guardrails

- CanvasKit source must not import `canvas2d-layer-renderer` or depend on
  browser Canvas2D APIs such as `CanvasRenderingContext2D`, `Path2D`,
  `OffscreenCanvas`, `ImageBitmap`, DOM image elements, `DOMParser`, or object
  URLs.
- `renderOp` must explicitly mention every `LayerPaintOp` variant exported by
  `rhwp-studio/src/core/types.ts`.
- `renderNode` must explicitly handle `group`, `clipRect`, and `leaf`.
- Fallback groups for text and special visual operations must remain explicit
  until a phase changes the policy and adds proof fixtures.
- `GlyphOutline` direct replay must remain guarded by
  `glyph-outline-payload-status.ts` before it reaches CanvasKit drawing code.
- The renderer contract guard and render-diff CI should catch drift before a
  PR changes public rendering behavior.

## Implementation Touchpoints

These paths are the first files to check when the CanvasKit parity contract
changes:

- `src/paint/text_v2.rs`
- `src/renderer/canvaskit_policy.rs`
- `rhwp-studio/src/core/types.ts`
- `rhwp-studio/src/view/canvaskit-renderer.ts`
- `rhwp-studio/src/view/canvaskit/diagnostics.ts`
- `rhwp-studio/src/view/canvaskit/`
- `rhwp-studio/src/view/glyph-outline-payload-status.ts`
- `rhwp-studio/e2e/renderer-contract.test.mjs`
- `.github/workflows/render-diff.yml`

The contract test keeps this list alive so a future rename or split has to
update the plan at the same time.

## Work Batches

### 1. Contract And Plan Guards

Pin the current dispatch surface and document the next parity boundaries before
widening runtime behavior. This batch should be docs and static contract checks
only. It should not change the public canvas default or hide unsupported work
behind an overlay.

### 2. Paint Family Parity Closures

Close the remaining paint-op families one at a time. Each family should include
a Canvas2D behavior audit, a direct CanvasKit implementation or deterministic
unsupported diagnostic, and at least one focused fixture.

Likely families:

- path command and line style branches;
- gradients, pattern fills, and image fills;
- raster image effects and crop preprocessing;
- equation and form-object bounds;
- placeholder and raw-SVG preview payloads;
- root `TextRun` effects such as rotation, vertical text, tab leaders, control
  marks, decorations, shadow, outline, and emphasis.

### 3. Strict Text Variant Replay

Keep `GlyphRun` and `GlyphOutline` behind explicit payload-status and selection
diagnostics until the payload family has a proof fixture. Do not let CanvasKit
select glyph ids against an arbitrary local font by family name. Do not allow
color, bitmap, SVG, and stroke payload families to mix in one strict outline
payload.

This batch should widen strict variant replay only when the fallback behavior
and reject reasons are exact.

### 4. Resource And Cache Proofs

Move replay-relevant bytes through `ResourceArena` before treating them as
strict payloads. Cache keys should include resource identity, output options,
sidecar payloads, and replay-plane choices that can change pixels.

This is the right place for image resource identity, static SVG resource
identity, exact font blob proof, and cache invalidation fixtures.

### 5. Visual And Artifact Diff Widening

Use render-diff CI to compare Canvas2D and CanvasKit output on focused
fixtures before broadening default behavior. Full-corpus or PDF artifact
comparison can be added as report-only first, then promoted only after the
noise floor is understood.

## Readiness Gate Contract

Canvas2D remains the public default. CanvasKit can be selected only by an
explicit URL request such as `?renderer=canvaskit`; a stored backend value must
not enable it on a later visit. Mode requests may come from URL or storage, but
their source and any rejected value remain visible in renderer diagnostics.

`CanvasKitRenderDiagnostics.passesRuntimeReadinessGate` means only that the
selected page completed a CanvasKit surface flush without a render error or
unexpected unsupported operation. Surface fallback remains explicit
telemetry because headless and constrained devices may legitimately use the
software surface. `surfaceBackend` records whether the default or software
factory actually succeeded. If CanvasKit replaces the DOM canvas during its
internal software fallback, the replacement is transferred to the page canvas
pool instead of leaving diagnostics and lifecycle ownership on the detached
canvas. Runtime readiness is not a claim of
complete visual parity. Known capability gaps remain in
`lastExpectedUnsupportedOps`; new diagnostic strings are unexpected unless
they are added to the exact allowlist with a fixture and review.

Diagnostics are snapshotted by page so viewport prefetch cannot replace the
result for the page under test. Studio exposes the request, effective backend,
fallback reason, and page snapshot through `getRendererDiagnostics` on the
existing `rhwp-request` API.
If renderer initialization fails at any stage, this API reports
`initialized: false`, a null effective backend, and the initialization error
instead of implying that Canvas2D is active.

The manifest flag `canvaskitReadinessGate` selects a bounded paragraph, table,
and image corpus. `scripts/renderer_baseline.py --readiness-only --profiles
screen` runs only Canvas2D and CanvasKit default on the automatic surface. Each
selected case must satisfy all of these conditions:

1. the effective backend is CanvasKit after explicit URL requests for the
   CanvasKit backend and `default` mode, with `auto` surface preference;
2. page-scoped CanvasKit diagnostics are available and pass the runtime gate;
3. the visible page canvas is still owned by the page canvas pool after any
   CanvasKit software fallback;
4. unexpected unsupported operations and render errors are empty;
5. the Canvas2D-vs-CanvasKit comparison passes that sample's tolerant or
   raster-aware ink/non-ink visual threshold; and
6. both captures contain the sample's minimum ink count, so two blank outputs
   cannot pass by matching each other.

The ink comparison uses deterministic maximum-cardinality matching within the
configured pixel radius. A greedy scan-order match is not sufficient because
it can reject a valid one-to-one assignment. The matcher enforces an edge
budget before allocating its graph. Threshold keys and ranges are validated
before capture, readiness samples require a positive ink floor, readiness runs
cannot be narrowed with `--filter`,
and CI pins the Chromium revision used for hard pixel comparisons. The browser
version and pinned Chromium build ID are included in the generated report.

Ordinary baseline and surface sweeps remain report-only. Only the explicit
readiness command fails CI, and its JSON/Markdown reports are written before
the process reports failure, including browser launch, document load, and
screenshot capture failures.

## Non-Goals

- This plan does not switch the public canvas default.
- This plan does not add a hidden Canvas2D overlay fallback.
- This plan does not enable CanvasKit `GlyphRun` or `GlyphOutline` selection
  without proof resources and deterministic diagnostics.
- This plan does not claim native Skia or PDF export parity is complete.
