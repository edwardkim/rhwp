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
basic form objects, root `TextRun` compatibility payloads, horizontal text
special visuals, and the currently supported `GlyphOutline` color-layer subset.
It still treats vertical text, effect-heavy text, several image effects,
page-background fills, and document-object families as fallback or diagnostic
work until their payload contract is strict enough for direct replay.

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
- `rhwp-studio/src/view/renderer-session.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`
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
- remaining root `TextRun` effects such as vertical text, ratio scaling, shadow,
  outline, emboss, engrave, and shade.

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

P35 connects the first font-native resource producers to normal layer export.
HWPX `binaryItemIDRef` values are resolved by exact package manifest ID and
kept separately from the original round-trip string. A single-scalar,
horizontal, unstyled text run may receive a bitmap glyph sidecar only when its
`charShapeId` and Unicode language slot select that embedded face. Multi-face
collections additionally require one unambiguous internal family-name match.
The anchored `TextRun` remains present whenever parsing, face selection, glyph
lookup, or payload limits fail. Multi-scalar, vertical, rotated, stretched,
bold, and italic runs remain on the text fallback in this phase.

The static-SVG font decoder and strict resource contract remain available for
explicit proof fixtures, but normal font lowering does not yet label an SVG
document as an exact sidecar. That requires preserving the OpenType SVG em-box
and baseline geometry in the paint payload; treating the fragment alone as
page-positioned output would overstate parity.

The additive JSON contract advances to layer schema `1.19` and resource table
`1.5`. Bitmap and SVG sidecar IDs are accompanied by the encoded image bytes,
static SVG fragments, and content-addressed keys in `resources`, so a consumer
never receives an arena-local reference without its corresponding payload.
CanvasKit validates that key, decodes bitmap headers under the same pixel
limits as ordinary images, and parses every static SVG path before selecting
the sidecar. Selection is exclusive per `equivalenceGroup`: a verified sidecar
suppresses its `TextRun`, while missing, corrupt, oversized, or unparseable
resources keep only the anchored text fallback.

CanvasKit image decoding is page-failure-contained. Encoded size and decoded
pixel limits reject pathological payloads, decode failures use a bounded
negative cache, and the native `SkImage` cache is a bounded LRU that deletes
evicted objects. `missingPicture` placeholders are editor visuals for screen
profiles and are suppressed for print-equivalent profiles across layered SVG,
Canvas2D, CanvasKit, and native Skia; OLE placeholders keep the existing static
replay. The `export-png` CLI defaults to the `high-quality` profile; callers
must request `--profile screen` explicitly to include editor-only visuals.

Equation ops now carry their bounded semantic `layoutBox` in the layer JSON.
CanvasKit replays that tree directly, so a missing or malformed equation SVG
does not require a DOM/SVG overlay and cannot abort the page. Non-finite,
over-deep, oversized, or unsupported layout trees stop at an explicit readiness
blocker instead of being reported as completed direct replay.

P39 externalizes producer-positioned space, tab, paragraph-end, and line-break
marks and replays them together with horizontal character overlap, tab leaders,
underline, strikeout, and emphasis marks. Layer schema `1.19` advertises
`text.charOverlapOp.bounded`, `text.controlMarkOp.positioned`,
`text.controlMarkOp.bounded`, `text.tabLeaderOp.bounded`, and
`text.decorationOp.bounded`. Each operation emits at most 4,096 positioned
items or source characters and carries a completeness flag so truncation is
never accepted as direct replay. Decoration metrics include script
baseline/size adjustment and PUA display positions, while tab-leader endpoints
use the same following-text clamp as Canvas2D. Combined-number character
overlap uses the compatibility renderer's digit-count scale. `legacyVisuals: "mirror"` keeps the anchored
`TextRun` from painting the same visual twice. Positioned text replay may split
a run between the prepared family, the default face, and the bounded old-Hangul
subset while retaining producer advances. At most 4,096 contiguous fallback
spans may be planned for one run, preventing adversarial alternating-glyph
input from multiplying CanvasKit font probes and draw calls without bound.
Hancom boxed-number PUA characters use a deterministic vector box and digit
fallback. A glyph that remains unresolved is an unexpected runtime diagnostic
and causes automatic mode to fall back for the whole document revision.
Vertical, rotated, malformed, incomplete, or over-limit special visual
payloads remain fail-closed in preflight and replay.

### 5. Visual And Artifact Diff Widening

Use render-diff CI to compare Canvas2D and CanvasKit output on focused
fixtures before broadening default behavior. Full-corpus or PDF artifact
comparison can be added as report-only first, then promoted only after the
noise floor is understood.

## Readiness Gate Contract

The browser keeps Canvas2D as the compatibility default. An explicit
`?renderer=auto` request runs a bounded Rust document capability preflight over
the selected render profile before CanvasKit is loaded. The
result is a compact summary and bounded blocker list; it does not serialize
`PageLayerTree` JSON or resource bytes across the WASM boundary. CanvasKit is
selected only when the scan is complete and eligible. A page/work limit,
lowering failure, hidden-overlay requirement, unsupported item, compat overlay,
or explicit text fallback makes the automatic choice fail closed to Canvas2D.
The scan first estimates work from the borrowed `PageRenderTree`, before font
parsing or layer allocation, and then verifies the lowered tree and resource
arena. Both walks are iterative and depth-bounded. Text-like payloads, encoded
image bytes, path commands, and lowered resource bytes count toward the fixed
work budget; oversized or invalid browser image payloads are not reported as
direct replay candidates. The reported page cost is the larger of the
pre-lowering estimate and the lowered-tree cost.

The same report carries a bounded, sorted `requiredFontFamilies` list for text
fallbacks that the selected replay plan will actually paint. A strict glyph
outline variant does not require its source family. Each browser surface maps
that list through the shared font catalog before lazy CanvasKit initialization;
an unavailable family adds a surface blocker and keeps the document on
Canvas2D. Paragraph and control mark view options participate in the decision
key. `showParagraphMarks` is eligible when its producer-positioned horizontal
marker ops pass the same replay plan, while `showControlCodes` remains a
document-level Canvas2D blocker because structural markers such as table and
image labels do not yet have paint ops. Unsupported marker geometry also fails
closed. Eligible families are fetched under a 32 MiB per-face bound and
registered before the first replay. A named family that still reaches replay
without a prepared typeface is a document-wide resource failure, not a silent
substitution with the default Noto face.

The decision is pinned for the whole document revision. Its key includes the
document digest, edit revision, render profile, resource generation, requested
backend, and CanvasKit mode. In automatic mode, an edit immediately advances to
a Canvas2D-pinned revision. Repeated edits coalesce behind a 300 ms quiet period,
after which one new revision runs the bounded selection again. A resource or
view-option or profile change also invalidates the decision; there is no
per-page, per-op, or per-replay-plane backend mixing. CanvasKit initialization
and resource preparation failures also pin that revision to Canvas2D. Explicit
`?renderer=auto` enables document selection, while `?renderer=canvas2d` and
`?renderer=canvaskit` bypass automatic preflight selection. Rejected request
values remain visible in diagnostics. A CanvasKit surface or replay failure
discovered after selection also moves an automatic session to Canvas2D for the
whole revision; the explicit CanvasKit diagnostic path remains available for
backend debugging.

Studio, its browser-extension and iframe-embed surfaces, and the VS Code
webview share this `RendererSession` contract while retaining Canvas2D as their
default request. VS Code adapts its direct `HwpDocument` binding to the same
bounded preflight source so automatic selection can be enabled separately
after fidelity convergence. Main pages and thumbnails use the same pinned
decision. A
layer-resource or runtime failure replaces any CanvasKit-owned canvas and
queues one whole-document Canvas2D replay; the selection diagnostics are sent
to the extension host as additive webview messages. The VS Code package exposes
only its copied font files to this catalog, explicitly disables external web
fonts for CanvasKit planning, and cleans stale lazy webview chunks on each
production build. Externally hosted CSS fonts may still serve Canvas2D, but
they never make a document eligible for CanvasKit selection in the VS Code
webview.

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
result for the page under test. Studio exposes the requested and effective
backend, selection reason, fallback reason, decision identity, preflight, and
page snapshot through `getRendererDiagnostics` on the
existing `rhwp-request` API. The public `@rhwp/editor` wrapper exposes the same
operation as `getRendererDiagnostics(page)` with `schemaVersion: 1` after the
peer advertises `renderer-diagnostics-v1`. Older protocol-v1 peers fail this
new operation explicitly instead of returning the pre-versioned shape. CanvasKit
snapshots include bounded image-cache counters and the last render duration so
cold resource preparation and warm replay can be compared without inspecting
private renderer state. Selection, preflight, resource, and replay failures are
reported in `selectionError`; `initializationError` is reserved for CanvasKit
module initialization in the selection snapshot and for Studio application
initialization at the top level. A failed CanvasKit page snapshot remains
available after an automatic whole-document fallback so the initiating failure
is not erased by the subsequent Canvas2D replay.
For compatibility, the existing v1 `request.backend.backend` enum remains
limited to `canvas2d | canvaskit`; automatic request intent and its decision are
reported only through the additive optional `selection` snapshot. Consumers
that understand only the original v1 shape therefore do not receive a new enum
member, while newer consumers can inspect `selection.request` and
`selection.requestedBackend` for `auto`.
The selected readiness corpus records both document-load/initial-render time
and one immediate warm replay. Every readiness sample declares cold, warm,
renderer-duration, and image-cache-pixel budgets; a missing measurement or
budget fails the gate, and the values remain in the JSON and Markdown
artifacts for regression analysis. The generated
`render-p35-font-native-bitmap.hwpx` sample additionally requires a
`bitmapGlyph` sidecar in the exported layer tree and at least one warm image
cache hit, so the producer and CanvasKit resource replay path cannot pass by
rendering only the text fallback.

The hard readiness set covers paragraph, table, image, positioned paragraph
marks, PUA fallback, and font-native bitmap cases. Synthetic renderer-contract
tests cover character overlap, tab leaders, and decorations; focused
document-backed visual fixtures for those three operations remain a follow-up.
The readiness set checks requested mode/surface, page canvas
ownership, expected/unexpected diagnostics, visual thresholds, declared layer
payloads, warm cache hits, decoded-image pixel limits, synchronous warm replay,
and the document load plus initial render interval. Browser scheduling and the
post-load screenshot stabilization delay are not part of the performance
budgets.

The generated font fixtures use the pinned dependency in
`scripts/requirements-font-fixtures.txt`. Regenerate them in order with
`generate_font_glyph_payload_fixture.py`,
`generate_exact_face_collection_fixture.py`, and
`generate_font_native_hwpx_fixture.py`; the checked-in outputs and Render Diff
path filters keep generator, font, and HWPX changes under the same review gate.
If automatic CanvasKit initialization or resource preparation fails after the
Studio app itself initialized, this API reports Canvas2D as the effective
backend together with the exact selection and fallback reason. A Studio app
initialization failure still reports `initialized: false` and a null effective
backend.

The manifest flag `canvaskitReadinessGate` selects a bounded paragraph, table,
and image corpus. `scripts/renderer_baseline.py --readiness-only --profiles
screen` runs only Canvas2D and CanvasKit default on the automatic surface. The
CanvasKit capture explicitly requests `renderer=auto`; it does not depend on
the public browser default. Each
selected case must satisfy all of these conditions:

1. the effective backend is CanvasKit after an explicit `auto` request selects
   a complete and eligible document, with `default` mode and
   `auto` surface preference;
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

Ordinary visual differences and surface sweeps remain report-only. The explicit
readiness command additionally gates selected visual thresholds. Every mode
still fails on capture, provenance, or replay-contract failures, and writes its
JSON/Markdown reports before reporting that failure.

Ordinary baseline captures now retain the Rust replay-plan status, reason, and
feature inventory beside page-scoped CanvasKit runtime diagnostics. Missing or
invalid direct-only plans, incomplete renders, render errors, and unexpected
runtime operations fail the baseline contract. Known direct-replay gaps remain
visible as report inventory instead of being reclassified as successful output.

The cross-backend corpus also renders the manifest's exact page index into a
dedicated intrinsic-scale capture surface. This keeps nonzero-page diagnostics,
repeated headers, and HWP/HWPX paired fixtures tied to the same page/profile
identity instead of accidentally capturing the first visible viewport canvas.
Canvas2D image resources, composed DOM images, and CanvasKit local typefaces
must settle before the selected page is replayed and captured.

The versioned corpus records a SHA-256 document digest and diagnostic axes for
every sample. Browser and native comparisons require matching sample, digest,
page, and profile identities before comparing pixels, while retaining backend
and actual surface provenance. Identity mismatches are a separate result class,
not visual noise. The default `representative` tier retains the existing 21-case
runtime envelope. `--scope full` and the manual workflow's `corpus=full` input
select the complete 120-case browser/native corpus; WebGPU/software surface
sweeps remain representative. The selected multi-profile sweep also collects
verified print-profile PDF artifacts, while selected CanvasKit readiness cases
remain the bounded visual hard gate.

## Direct PDF Export Contract

P37 adds a native-only, opt-in `PageLayerTree` PDF path using the same Skia
replay-plane, inherited-layer, clip, text-variant, and resource handling as the
PNG renderer. PDF recording applies one `72/96` transform because layer
coordinates are CSS pixels and PDF page boxes use points. Direct export uses the
print profile unless the caller explicitly selects another profile.

The existing SVG-derived PDF APIs and CLI behavior remain the compatibility
default. Direct export is exposed only with `native-skia` through the additive
`render_*_pdf_direct_native` methods and `export-pdf --backend direct`; WASM,
C/Swift, and XCFramework surfaces are unchanged. SVG-only fallback-family,
equation-family, and text-as-path options are rejected when direct mode is
selected instead of being silently ignored.

Direct mode preflights every selected page before opening the PDF writer. Native
Skia approximations that would lose visible semantics, including gradient,
pattern, shadow, multi-line/arrow, connector, or unbaked
image-adjustment payloads, return a page/op-specific error directing the caller
to the SVG backend. Image decode and Raw SVG fallback failures also abort the
document rather than recording a placeholder. Raw SVG is the explicit raster
fallback in this phase and uses the requested fallback DPI; supported text,
paths, simple shapes, clips, equations, images, and form appearances are
recorded through the PDF canvas.

Render-diff CI keeps browser Canvas versus compatibility PDF report-only. A
separate selected corpus (`biz_plan.hwp`, `kps-ai.hwp`, and
`tac-case-001.hwp`) rasterizes direct and compatibility PDF page 1 at the same
DPI and hard-fails above a 2% differing-pixel ratio or on a page-size/resource
error. Broader documents and unsupported vector subsets remain report/fallback
work rather than being declared parity-complete.

## Non-Goals

- This plan does not make CanvasKit a public default; automatic selection is an
  explicit opt-in and remains fail-closed and document-scoped.
- This plan does not add a hidden Canvas2D overlay fallback.
- This plan does not enable CanvasKit `GlyphRun` or `GlyphOutline` selection
  without proof resources and deterministic diagnostics.
- This plan does not claim native Skia or PDF export parity is complete.
