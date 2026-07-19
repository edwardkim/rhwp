# Task 126 Execution Plan -- Canvas DPR Scaling

## Background

### Current Problem

rhwp-studio does not apply `devicePixelRatio (DPR)` during Canvas rendering. Text and lines appear blurry on high-resolution displays (Retina, 4K).

Current flow:
```
WASM renderPageToCanvas(page, canvas)  -> scale=1.0 fixed
canvas.width = pageWidthPx             -> 1x physical pixels
canvas.style.width = pageWidthPx * zoom -> CSS scale
-> On DPR=2 display, 1x resolution stretched to 2x -> blurry
```

### Commercial Product Analysis Results

Commercial product canvas rendering analysis (`mydocs/tech/investigations/issue-124/canvas_rendering_analysis.md`) shows Hancom web viewer, Google Docs, and a commercial web office all use the same DPR pattern:

```
canvas.width  = pageWidth  * zoom * DPR    (physical pixels)
canvas.style.width  = pageWidth * zoom + "px"  (CSS logical pixels)
ctx.scale(zoom * DPR, zoom * DPR)
```

### Current Implementation Status

- Task 123 already added `scale: f64` parameter to WASM `render_page_to_canvas`
- `web/editor.js` already implements `renderScale = zoom * dpr` pattern
- **Only rhwp-studio (TypeScript) has not applied this** -- always renders with scale=1.0

### Solution Direction

Calculate `renderScale = zoom * DPR` in rhwp-studio's `page-renderer.ts` and `canvas-view.ts`, pass to WASM, and set CSS display size to `physical pixels / DPR`.

## Core Formula

```
renderScale = zoom * DPR

canvas.width  = pageWidthPx  * renderScale    (WASM auto-sets)
canvas.height = pageHeightPx * renderScale

canvas.style.width  = canvas.width  / DPR + "px"  (CSS logical pixels)
canvas.style.height = canvas.height / DPR + "px"

ctx.scale(renderScale, renderScale)  (applied internally by WASM)
```

## Change Impact Analysis

| Component | DPR Impact | Change Needed | Rationale |
|-----------|-----------|---------------|-----------|
| page-renderer.ts | Pass scale to WASM | **Yes** | renderScale calculation and passing |
| canvas-view.ts | CSS size calculation | **Yes** | physical pixels / DPR |
| wasm_api.rs | Scale upper limit extension | **Yes** | zoom 3.0 * DPR 4.0 = 12.0 |
| margin guides | ctx scale applied | **Yes** | ctx transform reset after WASM |
| virtual-scroll.ts | CSS logical pixel based | No | DPR only affects canvas internal resolution |
| selection-renderer.ts | HTML DIV based | No | Already accurate with CSS coordinates * zoom |
| cell-selection-renderer.ts | HTML DIV based | No | Same as above |
| caret-renderer.ts | HTML DIV based | No | CSS logical coordinate based |
| input-handler.ts | Mouse coordinates / zoom | No | CSS absorbs DPR |
| main.ts | Zoom calculation | No | CSS logical size based |
| coordinate-system.ts | CSS logical coordinates | No | DPR independent |

## Implementation Phases (3 Phases)

---

### Phase 1: Rust WASM Scale Range Extension

**File**: `src/wasm_api.rs` (line 212)

**Change**: `scale.clamp(0.25, 8.0)` -> `scale.clamp(0.25, 12.0)`

To support zoom 3.0 * DPR 4.0 = 12.0 (4K monitor + max zoom).

---

### Phase 2: TypeScript DPR Scaling Application

**File 1**: `rhwp-studio/src/view/page-renderer.ts`

- Add `scale: number` parameter to `renderPage()` signature
- Apply `ctx.setTransform(scale, 0, 0, scale, 0, 0)` to `drawMarginGuides()`
  - Explicitly set since ctx transform state is uncertain after WASM rendering
- Pass same scale to `scheduleReRender()`

**File 2**: `rhwp-studio/src/view/canvas-view.ts`

- Calculate `renderScale = zoom * devicePixelRatio` inside `renderPage()`
- WASM call: `pageRenderer.renderPage(pageIdx, canvas, renderScale)`
- CSS size: `canvas.style.width = canvas.width / dpr + "px"`
- Remove existing `if (zoom !== 1.0)` branch -> always set CSS size

---

### Phase 3: Integration Testing and Verification

| Item | Method |
|------|--------|
| 571 regression tests | `docker compose run --rm test` |
| WASM build | `docker compose run --rm wasm` |
| TypeScript type check | `npx tsc --noEmit` |
| Default rendering (100% zoom) | Verify identical to existing on DPR=1 environment |
| High-res rendering | Verify text sharpness on DPR=2 environment |
| Zoom in/out | Verify sharpness at 200%, 50% |
| Mouse hit test | Click -> caret accuracy in zoom + DPR state |
| Selection area | Drag select -> highlight position accuracy |
| Caret | Caret position accuracy after zoom change |
| Width/page fit | zoom-fit-width/page normal operation |

---

## Changed Files Summary

| File | Changes | Scope |
|------|---------|-------|
| `src/wasm_api.rs` | Scale upper limit 8.0->12.0, comment update | 1 line |
| `rhwp-studio/src/view/page-renderer.ts` | scale parameter addition, setTransform for margin guides | ~15 lines |
| `rhwp-studio/src/view/canvas-view.ts` | DPR calculation, renderScale passing, CSS size = physical/DPR | ~10 lines |

## Expected Benefits

| Item | Current | After |
|------|---------|-------|
| DPR=2 sharpness | Blurry (1x resolution CSS enlarged) | Sharp (2x physical pixel rendering) |
| 200% zoom sharpness | Sharp (Task 123 scale) | Sharp + DPR support |
| WASM size | 1.83MB | 1.83MB (no change) |
| Change scope | -- | 3 files, ~25 lines |

## Architecture Comparison with Hancom

```
[Hancom Web Viewer]
  HWP -> Server (HWP filter) -> JSON commands -> JS Canvas 2D fillText
  DPR: ctx.scale(DPR) + canvas size * DPR

[rhwp (after applying)]
  HWP -> Rust WASM parser -> render tree -> Rust Canvas 2D fillText
  DPR: ctx.scale(zoom*DPR) + canvas size * zoom * DPR

  * Common: Both use browser Canvas 2D fillText, same DPR scaling
  * Difference: Hancom calls fillText from JS, rhwp calls from Rust/WASM
  * Advantage: No server needed, single WASM binary (1.83MB)
```
