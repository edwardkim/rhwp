# RHWP Layered Renderer Architecture

## 1. Purpose

This document explains the current multi-renderer architecture of rhwp from the implementation point of view.
It focuses on the following topics.

- The role split between `PageRenderTree` and `PageLayerTree`
- The current relationship between legacy SVG, layered SVG, browser Canvas2D, browser CanvasKit, and native Skia
- The meaning of CanvasKit `compat` and `default` modes
- Screenshot-diff-based parity strategy
- The touch points required when adding a new backend or a new paint op

For higher-level historical design context, see [rendering_engine_design.md](./rendering_engine_design.md).

## 2. Current rendering paths at a glance

rhwp is no longer best described as “one render tree consumed directly by every backend”.
The current system uses two representation layers.

```text
Document / Section / Paragraph / Control
  -> compose / paginate / layout
  -> PageRenderTree
  -> LayerBuilder
  -> PageLayerTree
  -> backend replay
```

The concrete paths are currently split as follows.

| Path | Input | Main files | Current role |
|---|---|---|---|
| Legacy SVG | `PageRenderTree` | `src/document_core/queries/rendering.rs`, `src/renderer/svg.rs` | Existing reference path, structural baseline |
| Layered SVG | `PageLayerTree` | `src/paint/*`, `src/renderer/svg_layer.rs` | Layered replay validation path |
| Browser Canvas2D | `PageRenderTree` | `src/wasm_api.rs`, `src/renderer/web_canvas.rs`, `rhwp-studio/src/view/page-renderer.ts` | Current web baseline renderer |
| Browser CanvasKit | `PageLayerTree` | `src/wasm_api.rs`, `rhwp-studio/src/view/canvaskit-renderer.ts` | Layered browser backend |
| Native Skia | `PageLayerTree` | `src/renderer/skia/renderer.rs` | Layered raster backend |

Two points matter here.

1. The web baseline, Canvas2D, still uses the established WASM Canvas rendering path.
2. New backends, namely layered SVG, CanvasKit, and native Skia, all consume `PageLayerTree`.

So the current structure is not “all backends already use the same path”.
It is closer to “keep the proven baseline path, while converging new backends on the layered path”.

Another important point is that the output-side contract is not fully unified yet.

- The `LayerRenderer` trait is a narrow transitional contract for stateful backends that accumulate output, such as the layered SVG bridge.
- Native Skia still keeps an explicit raster API, namely `render_png()`, because its natural output is encoded bytes rather than an internal scene buffer.

So the accurate statement today is not “every layered backend already shares one Rust trait”.
It is “the shared input IR is now `PageLayerTree`, while the output contract is still in transition depending on backend shape”.

## 3. Why `PageLayerTree` exists

`PageRenderTree` is appropriate as a layout result, but too semantic to serve as a replay IR for all backends.
It still contains document-level concepts such as `Header`, `Footer`, `Table`, `TextLine`, `TextBox`, and `Group`.
That forces each backend to reinterpret semantic containers on its own.

This causes several problems.

- Adding a new raster or vector backend requires re-implementing semantic-container traversal.
- It becomes difficult to keep composition order, clipping, and transforms aligned across SVG, Canvas, and Skia.
- Browser CanvasKit and native Skia cannot share a common replay input.

`PageLayerTree` was introduced to solve this.

- It lowers semantic containers into visual layers.
- It keeps only the information a backend needs for replay.
- It makes clip, group, and leaf paint ops explicit.
- It lets backends focus on replay instead of layout.

The entry point for this layer is `src/paint/mod.rs`, and
`LayerBuilder` in `src/paint/builder.rs` performs the `PageRenderTree -> PageLayerTree` conversion.

## 4. The role split: `PageRenderTree` vs `PageLayerTree`

| Item | `PageRenderTree` | `PageLayerTree` |
|---|---|---|
| Main purpose | Represent layout result | Serve as backend replay input |
| Included information | Document meaning plus layout result | Visual composition information |
| Node character | Semantic containers plus leaves | Group / clip / leaf op |
| Typical types | `RenderNodeType::Table`, `TextLine`, `Group` | `LayerNodeKind::Group`, `ClipRect`, `Leaf` |
| Consumers | Legacy SVG, debug/query layers | Layered SVG, native Skia, CanvasKit |
| Need for backend reinterpretation | High | Low |

In practical terms:

- `PageRenderTree` explains how the document was laid out.
- `PageLayerTree` explains in what order and in what way the result should be painted.

## 5. Core rules of the layered path

### 5.1 Layout must happen only once

Backends must not recalculate line breaking, paragraph composition, table placement, or inner-shape layout.
That responsibility belongs to the compose, paginate, and layout stages.

### 5.2 Backends only replay

Backends should only handle:

- paint-op replay
- clip application
- transform application
- raster or vector output
- browser-specific fallback or compatibility handling

### 5.3 Semantic information should be retained only as needed

Metadata such as `GroupKind`, `ClipKind`, and `CacheHint` is acceptable,
but `PageLayerTree` should not force a backend to interpret document semantics again.

### 5.4 Shape children must be preserved

Even when a shape itself becomes a leaf paint op, its image fill, text box content, or grouped child nodes must not disappear from the layer tree.

This is an important invariant of the layered path.
The recent `group-drawing-02` parity issue reaffirmed that lowering a shape leaf must not silently drop its children.

## 6. Current backend behavior

### 6.1 Legacy SVG

- Entry: `DocumentCore::render_page_svg_legacy_native()`
- Implementation: `src/renderer/svg.rs`
- Input: `PageRenderTree`

This path is not removed.
It remains as an existing export path and as a structural comparison baseline against the layered path.
Unless `RHWP_RENDER_PATH=layer-svg` is set, SVG export still uses this path by default.

### 6.2 Layered SVG

- Entry: `DocumentCore::render_page_svg_layer_native()`
- Implementation: `src/renderer/svg_layer.rs`
- Input: `PageLayerTree`

`SvgLayerRenderer` reconstructs a temporary render-tree-shaped structure from the layer tree and reuses the existing SVG leaf logic.
In other words, the input is layered, but the mature SVG output logic is still intentionally reused.

### 6.3 Browser Canvas2D

- Entry: `PageRenderer.renderPage()` calling `this.wasm.renderPageToCanvas(...)`
- Implementation: `src/renderer/web_canvas.rs`
- Input: `PageRenderTree`

This is currently the browser baseline.
CanvasKit parity tests compare screenshots against this path.

The important point is that Canvas2D has not yet moved to the layered path.
So CanvasKit parity is not a pure sibling test where two backends replay the same tree.
It is a validation that the new layered backend looks sufficiently close to the established baseline.

### 6.4 Browser CanvasKit

- Entry: `PageRenderer.renderPage()` calling `this.wasm.getPageLayerTree(...)`
- Implementation: `rhwp-studio/src/view/canvaskit-renderer.ts`
- Input: `PageLayerTree`

CanvasKit is a browser-side replay renderer.
The Rust core performs layout and layer-tree export, and TypeScript replays that data with the CanvasKit API.

This path exists to provide:

- a Skia-like 2D drawing model in the browser
- an experimental Skia-family renderer on the web
- a foundation for future native/backend extensibility

### 6.5 Native Skia

- Entry: `DocumentCore::render_page_png_native()`
- Implementation: `src/renderer/skia/renderer.rs`
- Feature flag: `native-skia`
- Input: `PageLayerTree`

Native Skia is the layered raster backend for non-wasm targets.
At the moment it is primarily a test and validation path.
There is not yet a general-purpose end-user `export-png` CLI.

### 6.6 RenderProfile defaults

`RenderProfile` is the enum that hints which quality/cache profile a layered output path should prefer.
Not every variant produces a large behavioral difference yet, but the call sites now make their defaults explicit.

| Path | Default profile |
|---|---|
| browser layer tree (`getPageLayerTree`) | `Screen` |
| layered SVG export | `Print` |
| native Skia PNG | `HighQuality` |

These defaults can be overridden with `RHWP_RENDER_PROFILE`, using one of:
`screen`, `print`, `high-quality`, or `fast-preview`.

At the moment `FastPreview` only changes page-background cache hints.
It is mostly a reserved staging point for more aggressive preview simplification later.

## 7. CanvasKit render modes

CanvasKit currently exposes two modes.

| Mode | Meaning | Default |
|---|---|---|
| `default` | Prefer native CanvasKit behavior | No |
| `compat` | Prefer visual similarity to Canvas2D | Yes |

`rhwp-studio/src/view/render-backend.ts` resolves this from query parameters and local storage.

`compat` is the default for several reasons.

- The current browser baseline is still Canvas2D.
- Text rasterization, font fallback, and glyph positioning can differ significantly between CanvasKit and Canvas2D.
- Switching the renderer should not immediately make the document look obviously different to end users.

At the moment, compat mode especially uses Canvas2D-based overlay or fallback behavior for text-like operations to absorb pure CanvasKit raster differences.
This logic is a browser-side compatibility layer.
It is not intended to rewrite the Rust layout core itself.

## 8. Parity and diff strategy

Renderer parity is not managed with a strict “exact diff must always be zero” rule.
Different engines naturally produce small differences in anti-aliasing, subpixel coverage, and font rasterization.

The current validation strategy is as follows.

### 8.1 Legacy SVG vs layered SVG

- Purpose: structural migration validation
- Rule: primarily exact-match oriented
- Artifacts: `output/layer-svg-diff/`

### 8.2 Layered SVG vs native Skia PNG

- Purpose: layered raster backend validation
- Rule: tolerant diff pixel budget
- Artifacts: `output/skia-diff/`

### 8.3 Browser Canvas2D vs CanvasKit

- Purpose: browser parity validation
- Test file: `rhwp-studio/e2e/canvaskit-render.test.mjs`
- Rule:
  - exact diffs are always recorded
  - tolerant diff ignores pixels whose per-pixel channel delta is `8` or less
  - final pass/fail uses tolerant diff ratio `<= 0.25%`
- Artifacts:
  - `output/e2e/`
  - `rhwp-studio/e2e/screenshots/`

The intent is:

- keep exact diffs for inspection and trend tracking
- use tolerant diff as the actual acceptance gate for renderer-engine-only differences
- distinguish visible structural mismatches from minor raster differences

## 9. What the tests are protecting

The current parity tests are not just cosmetic screenshot checks.
They protect important invariants of the layered architecture.

They should catch problems such as:

- dropping shape children while lowering shape leaves
- changing draw order while flattening group or clip hierarchy
- losing text fallback in CanvasKit and breaking Hangul or equations
- regressions in samples that are especially sensitive to backend differences, such as equations, crop, fields, and grouped drawings

So the screenshot tests are effectively contract tests for how closely the new layered path must track the existing baseline.

## 10. How to add a new backend

When adding a new backend, the usual order is:

1. Decide whether the backend will consume `PageLayerTree`.
2. Define the replay strategy for `LayerNodeKind::Group`, `ClipRect`, and `Leaf`.
3. Map each required `PaintOp` to backend draw calls.
4. If the backend is browser-side, update `paint/json.rs` and TypeScript layer types as well.
5. Add parity tests against an existing baseline.

The main touch points are usually these files.

| Purpose | Main files |
|---|---|
| Layer-tree generation | `src/paint/builder.rs` |
| Layer-tree JSON export | `src/paint/json.rs` |
| Layered SVG replay | `src/renderer/svg_layer.rs` |
| Native Skia replay | `src/renderer/skia/renderer.rs` |
| WASM export | `src/document_core/queries/rendering.rs`, `src/wasm_api.rs` |
| Browser CanvasKit replay | `rhwp-studio/src/view/canvaskit-renderer.ts` |
| Browser parity tests | `rhwp-studio/e2e/canvaskit-render.test.mjs` |

## 11. Checklist for adding a new paint op

Adding a new `PaintOp` usually requires all of the following.

1. Add the type in `src/paint/paint_op.rs`
2. Convert the corresponding render node in `src/paint/builder.rs`
3. Extend serialization in `src/paint/json.rs`
4. Update replay in `src/renderer/svg_layer.rs`
5. Update replay in `src/renderer/skia/renderer.rs`
6. If used in the browser, update TypeScript layer types and CanvasKit replay
7. Add a parity test sample

If any one of these is omitted, asymmetric failures such as “visible in one backend but missing in another” become very likely.

## 12. Important caveats when reading the current architecture

### 12.1 Canvas2D and CanvasKit are not fully symmetric yet

Canvas2D is still the legacy browser path.
CanvasKit is the layered browser path.
They are not two implementations of the exact same replay contract yet.

### 12.2 The layered path has not completely replaced the semantic tree

Legacy SVG and several query or debug capabilities still depend on `PageRenderTree`.
So the current stage is not “semantic tree removed”.
It is “layered replay introduced in parallel”.

### 12.3 Compat code is an intentional transition buffer

Compat mode is best understood as a transition layer that keeps the existing baseline stable while the new backend is being introduced.

However, that code should stay in the browser layer, primarily inside `rhwp-studio`, and should not leak back into the Rust layout core.

## 13. Summary

The current layered renderer architecture in rhwp can be summarized as follows.

- Layout produces `PageRenderTree`.
- The visual replay IR is `PageLayerTree`.
- Layered SVG, native Skia, and CanvasKit all share `PageLayerTree`.
- Browser Canvas2D is still kept as the baseline path.
- Therefore parity tests currently validate how closely the new layered backends match the established baseline.
- The most important rule when adding a new backend is: do not re-layout, replay the layer tree.
