# Zoom Anchor Symmetry Implementation Plan

**Goal:** Ctrl/Cmd+휠은 포인터 아래 문서 지점을, 버튼·키보드는 뷰포트 중심 아래 문서 지점을 유지하며 확대/축소 왕복이 같은 배율과 스크롤 위치로 돌아오게 한다.

**Architecture:** 확대/축소 요청이 정규화한 `ZoomAnchor`를 명시적으로 소유하고 `zoom-changed` 이벤트가 매 프레임 배율과 앵커를 함께 전달한다. `CanvasView`는 레이아웃 전후의 같은 쪽 내부 X/Y 비율을 순수 계산 함수로 보존하며, 쪽 맞춤은 두 UI 진입점이 하나의 계산 함수를 사용한다.

**Tech Stack:** TypeScript 7, Vite 8, Node test runner, DOM Canvas/scroll layout, browser runtime

## Global Constraints

- `ZOOM_SMOOTHING_TIME_MS = 22`와 `WHEEL_ZOOM_SENSITIVITY = 0.0042`를 유지한다.
- Ctrl/Cmd+휠은 포인터 앵커, 버튼·키보드·메뉴는 기본 `(0.5, 0.5)` 앵커를 사용한다.
- 애니메이션 중 CSS 미리보기와 정착 후 한 번의 재렌더링 구조를 유지한다.
- 최소·최대 배율과 휠 delta 정규화·제한값을 변경하지 않는다.
- toolbar, Subsecond, 입력 지연 관련 기존 dirty 변경을 수정·stage·commit하지 않는다.
- `src/view/canvas-view.ts`를 stage할 때 기존 Subsecond hunks를 제외하고 확대/축소 hunk만 stage한다.

---

### Task 1: Carry explicit anchors through ViewportManager

**Files:**
- Create: `rhwp-studio/src/view/zoom-anchor.ts`
- Modify: `rhwp-studio/src/view/viewport-manager.ts`
- Modify: `rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts`

**Interfaces:**
- Produces: `ZoomAnchor`, `CENTER_ZOOM_ANCHOR`, `normalizeZoomAnchor(anchor)`
- Produces: `smoothZoomBy(delta: number, anchor?: ZoomAnchor): void`
- Produces: `smoothZoomTo(zoom: number, anchor?: ZoomAnchor): void`
- Produces: `setZoom(zoom: number, anchor?: ZoomAnchor): void`
- Produces: `zoom-changed` arguments `(zoom: number, anchor: ZoomAnchor)`
- Produces: `setScrollLeft(x: number): void`

- [ ] **Step 1: Write failing anchor and inverse-input tests**

Add tests that inspect the event payload after a pointer-positioned wheel event and that apply equal opposite wheel deltas:

```ts
test('wheel zoom emits the pointer anchor and inverse deltas restore zoom', async (t) => {
  const frames = new FakeAnimationFrames();
  const previousRequest = globalThis.requestAnimationFrame;
  const previousCancel = globalThis.cancelAnimationFrame;
  globalThis.requestAnimationFrame = frames.request;
  globalThis.cancelAnimationFrame = frames.cancel;
  t.after(() => {
    globalThis.requestAnimationFrame = previousRequest;
    globalThis.cancelAnimationFrame = previousCancel;
  });

  const { ViewportManager } = await loadViewportManager();
  const eventBus = new FakeEventBus();
  const viewport = new ViewportManager(eventBus as never);
  (viewport as any).container = {
    getBoundingClientRect: () => ({ left: 100, top: 50, width: 800, height: 600 }),
  };

  const onWheel = (viewport as any).onWheel.bind(viewport);
  onWheel({
    ctrlKey: true,
    metaKey: false,
    clientX: 300,
    clientY: 500,
    deltaY: -8,
    deltaMode: 0,
    preventDefault() {},
  });
  let timestamp = 16;
  while (frames.pendingCount > 0 && timestamp < 1000) {
    frames.flush(timestamp);
    timestamp += 16;
  }

  const zoomedIn = viewport.getZoom();
  const firstZoomEvent = eventBus.events.find(({ event }) => event === 'zoom-changed');
  assert.deepEqual(firstZoomEvent?.args[1], { x: 0.25, y: 0.75 });

  onWheel({
    ctrlKey: true,
    metaKey: false,
    clientX: 300,
    clientY: 500,
    deltaY: 8,
    deltaMode: 0,
    preventDefault() {},
  });
  while (frames.pendingCount > 0 && timestamp < 2000) {
    frames.flush(timestamp);
    timestamp += 16;
  }

  assert.ok(zoomedIn > 1);
  assert.ok(Math.abs(viewport.getZoom() - 1) < 1e-12);
});
```

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```bash
cd rhwp-studio
node --test tests/viewport-manager-smooth-zoom.test.ts
```

Expected: FAIL because wheel coordinates are not converted to an anchor and `zoom-changed` has no second argument.

- [ ] **Step 3: Add the anchor value object**

Create `src/view/zoom-anchor.ts`:

```ts
export interface ZoomAnchor {
  x: number;
  y: number;
}

export const CENTER_ZOOM_ANCHOR: ZoomAnchor = Object.freeze({ x: 0.5, y: 0.5 });

function normalizeAxis(value: unknown): number {
  return typeof value === 'number' && Number.isFinite(value)
    ? Math.max(0, Math.min(1, value))
    : 0.5;
}

export function normalizeZoomAnchor(
  anchor?: Partial<ZoomAnchor> | null,
): ZoomAnchor {
  return {
    x: normalizeAxis(anchor?.x),
    y: normalizeAxis(anchor?.y),
  };
}
```

- [ ] **Step 4: Thread the anchor through ViewportManager**

In `src/view/viewport-manager.ts`, store the current normalized anchor and include it in every `zoom-changed` event:

```ts
private zoomAnchor: ZoomAnchor = CENTER_ZOOM_ANCHOR;

setZoom(zoom: number, anchor: ZoomAnchor = CENTER_ZOOM_ANCHOR): void {
  this.cancelZoomAnimation();
  this.zoomAnchor = normalizeZoomAnchor(anchor);
  this.zoom = this.clampZoom(zoom);
  this.zoomTarget = this.zoom;
  this.eventBus.emit('zoom-changed', this.zoom, this.zoomAnchor);
}

smoothZoomBy(delta: number, anchor: ZoomAnchor = CENTER_ZOOM_ANCHOR): void {
  this.smoothZoomTo(this.zoomTarget + delta, anchor);
}

smoothZoomTo(zoom: number, anchor: ZoomAnchor = CENTER_ZOOM_ANCHOR): void {
  this.zoomAnchor = normalizeZoomAnchor(anchor);
  this.zoomTarget = this.clampZoom(zoom);
  if (Math.abs(this.zoomTarget - this.zoom) <= ZOOM_SETTLE_EPSILON) {
    this.setZoom(this.zoomTarget, this.zoomAnchor);
    return;
  }
  this.zoomAnimating = true;
  if (this.zoomAnimationFrame === null) {
    this.zoomAnimationFrame = requestAnimationFrame(this.onZoomAnimationFrameBound);
  }
}
```

At the wheel boundary, calculate the normalized pointer position:

```ts
const rect = this.container?.getBoundingClientRect();
const anchor = rect && rect.width > 0 && rect.height > 0
  ? normalizeZoomAnchor({
      x: (e.clientX - rect.left) / rect.width,
      y: (e.clientY - rect.top) / rect.height,
    })
  : CENTER_ZOOM_ANCHOR;

this.smoothZoomTo(
  this.zoomTarget * Math.exp(-boundedDelta * WHEEL_ZOOM_SENSITIVITY),
  anchor,
);
```

Emit `this.zoomAnchor` from `onZoomAnimationFrame`, and add:

```ts
setScrollLeft(x: number): void {
  if (this.container) {
    this.container.scrollLeft = x;
    this.scrollX = this.container.scrollLeft;
  }
}
```

- [ ] **Step 5: Run the focused test and verify GREEN**

Run:

```bash
cd rhwp-studio
node --test tests/viewport-manager-smooth-zoom.test.ts
```

Expected: all viewport-manager smooth zoom tests PASS, including the pointer-anchor and inverse-delta test.

- [ ] **Step 6: Commit Task 1**

```bash
git add rhwp-studio/src/view/zoom-anchor.ts \
  rhwp-studio/src/view/viewport-manager.ts \
  rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts
git diff --cached --check
git commit -m "fix: carry explicit document zoom anchors"
```

### Task 2: Preserve the anchor through two-axis layout changes

**Files:**
- Modify: `rhwp-studio/src/view/zoom-anchor.ts`
- Modify: `rhwp-studio/src/view/canvas-view.ts`
- Create: `rhwp-studio/tests/zoom-anchor.test.ts`

**Interfaces:**
- Consumes: `ZoomAnchor` and `normalizeZoomAnchor(anchor)` from Task 1
- Produces: `ZoomPageBox`
- Produces: `calculateAnchoredScroll(oldBox, newBox, viewport, anchor)`
- Consumes: `ViewportManager.setScrollLeft(x)` and `setScrollTop(y)`

- [ ] **Step 1: Write failing two-axis geometry tests**

Create `tests/zoom-anchor.test.ts`:

```ts
import test from 'node:test';
import assert from 'node:assert/strict';
import {
  calculateAnchoredScroll,
  type ZoomPageBox,
} from '../src/view/zoom-anchor.ts';

test('center anchor stays fixed while content crosses horizontal overflow', () => {
  const oldBox: ZoomPageBox = { left: 214.25, top: 10, width: 454.5, height: 643 };
  const newBox: ZoomPageBox = { left: 20, top: 10, width: 930.5, height: 1316.5 };
  const next = calculateAnchoredScroll(
    oldBox,
    newBox,
    { width: 883, height: 683, scrollLeft: 0, scrollTop: 0 },
    { x: 0.5, y: 0.5 },
  );

  assert.ok(Math.abs(next.scrollLeft - 43.75) < 0.01);
  assert.ok(Math.abs(next.scrollTop - 347.22433903576984) < 0.01);
});

test('off-center pointer anchor is reversible', () => {
  const fit: ZoomPageBox = { left: 214.25, top: 10, width: 454.5, height: 643 };
  const enlarged: ZoomPageBox = { left: 20, top: 10, width: 930.5, height: 1316.5 };
  const viewport = { width: 883, height: 683, scrollLeft: 0, scrollTop: 0 };
  const anchor = { x: 0.25, y: 0.75 };
  const forward = calculateAnchoredScroll(fit, enlarged, viewport, anchor);
  const reverse = calculateAnchoredScroll(
    enlarged,
    fit,
    { ...viewport, ...forward },
    anchor,
  );

  assert.ok(Math.abs(reverse.scrollLeft) < 1e-9);
  assert.ok(Math.abs(reverse.scrollTop) < 1e-9);
});
```

- [ ] **Step 2: Run the geometry test and verify RED**

Run:

```bash
cd rhwp-studio
node --test tests/zoom-anchor.test.ts
```

Expected: FAIL because `ZoomPageBox` and `calculateAnchoredScroll` do not exist.

- [ ] **Step 3: Implement the pure two-axis calculation**

Extend `src/view/zoom-anchor.ts`:

```ts
export interface ZoomPageBox {
  left: number;
  top: number;
  width: number;
  height: number;
}

export interface ZoomViewportState {
  width: number;
  height: number;
  scrollLeft: number;
  scrollTop: number;
}

export function calculateAnchoredScroll(
  oldBox: ZoomPageBox,
  newBox: ZoomPageBox,
  viewport: ZoomViewportState,
  requestedAnchor: ZoomAnchor,
): Pick<ZoomViewportState, 'scrollLeft' | 'scrollTop'> {
  const anchor = normalizeZoomAnchor(requestedAnchor);
  const viewportX = viewport.width * anchor.x;
  const viewportY = viewport.height * anchor.y;
  const documentX = viewport.scrollLeft + viewportX;
  const documentY = viewport.scrollTop + viewportY;
  const ratioX = oldBox.width > 0 ? (documentX - oldBox.left) / oldBox.width : 0.5;
  const ratioY = oldBox.height > 0 ? (documentY - oldBox.top) / oldBox.height : 0.5;

  return {
    scrollLeft: newBox.left + newBox.width * ratioX - viewportX,
    scrollTop: newBox.top + newBox.height * ratioY - viewportY,
  };
}
```

- [ ] **Step 4: Apply the calculation in CanvasView**

Update the event subscription:

```ts
eventBus.on('zoom-changed', (zoom, anchor) =>
  this.onZoomChanged(
    zoom as number,
    normalizeZoomAnchor(anchor as Partial<ZoomAnchor> | undefined),
  )),
```

Add a helper that resolves the CSS-centered single-column page against the actual layout width:

```ts
private getZoomPageBox(pageIdx: number, viewportWidth: number): ZoomPageBox {
  const layoutWidth = Math.max(viewportWidth, this.virtualScroll.getTotalWidth());
  return {
    left: this.virtualScroll.getPageLeftResolved(pageIdx, layoutWidth),
    top: this.virtualScroll.getPageOffset(pageIdx),
    width: this.virtualScroll.getPageWidth(pageIdx),
    height: this.virtualScroll.getPageHeight(pageIdx),
  };
}
```

Replace the vertical-only body of `onZoomChanged` with:

```ts
private onZoomChanged(zoom: number, anchor: ZoomAnchor): void {
  if (this.pages.length === 0) return;

  const scrollTop = this.viewportManager.getScrollY();
  const scrollLeft = this.viewportManager.getScrollX();
  const { width: vpWidth, height: vpHeight } = this.viewportManager.getViewportSize();
  const anchorDocumentX = scrollLeft + vpWidth * anchor.x;
  const anchorDocumentY = scrollTop + vpHeight * anchor.y;
  const focusPage = this.virtualScroll.getPageAtPoint(anchorDocumentX, anchorDocumentY);
  const oldBox = this.getZoomPageBox(focusPage, vpWidth);

  this.recalcLayout();

  const newBox = this.getZoomPageBox(focusPage, vpWidth);
  const nextScroll = calculateAnchoredScroll(
    oldBox,
    newBox,
    {
      width: vpWidth,
      height: vpHeight,
      scrollLeft,
      scrollTop,
    },
    anchor,
  );
  this.viewportManager.setScrollLeft(nextScroll.scrollLeft);
  this.viewportManager.setScrollTop(nextScroll.scrollTop);

  this.eventBus.emit('zoom-level-display', zoom);

  if (this.viewportManager.isZoomAnimating()) {
    this.cancelPendingTextEditRefresh();
    this.cancelTextEditStaticLayerVerification();
    this.cancelPendingPrefetch();
    this.updateRenderedPageZoomPreview();
    return;
  }

  this.cancelPendingTextEditRefresh();
  this.cancelTextEditStaticLayerVerification();
  this.releaseAllRenderedPages();
  this.pageRenderer.cancelAll();
  this.updateVisiblePages();
}
```

- [ ] **Step 5: Run focused geometry and smooth-zoom tests**

Run:

```bash
cd rhwp-studio
node --test tests/zoom-anchor.test.ts tests/viewport-manager-smooth-zoom.test.ts
```

Expected: all tests PASS.

- [ ] **Step 6: Stage only zoom-related CanvasView hunks and commit**

`src/view/canvas-view.ts` already contains unrelated Subsecond edits. Inspect and interactively stage only imports and methods belonging to zoom anchoring:

```bash
git diff -- rhwp-studio/src/view/canvas-view.ts
git add rhwp-studio/src/view/zoom-anchor.ts \
  rhwp-studio/tests/zoom-anchor.test.ts
git add -p rhwp-studio/src/view/canvas-view.ts
git diff --cached --check
git diff --cached --stat
git commit -m "fix: preserve zoom anchors across layout"
```

Expected staged CanvasView hunks: `zoom-anchor` import, two-argument `zoom-changed` listener, `getZoomPageBox`, and two-axis `onZoomChanged`. Do not stage `SubsecondRevisionWatcher` hunks.

### Task 3: Make fit-page geometry match VirtualScroll

**Files:**
- Create: `rhwp-studio/src/view/zoom-fit.ts`
- Modify: `rhwp-studio/src/main.ts`
- Modify: `rhwp-studio/src/command/commands/view.ts`
- Create: `rhwp-studio/tests/zoom-fit.test.ts`

**Interfaces:**
- Produces: `calculateFitWidthZoom(containerWidth, pageWidth)`
- Produces: `calculateFitPageZoom(containerWidth, containerHeight, pageWidth, pageHeight)`
- Consumes: existing single-column vertical gap sum `20px` and horizontal frame padding `40px`

- [ ] **Step 1: Write failing fit-page tests**

Create `tests/zoom-fit.test.ts`:

```ts
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  calculateFitPageZoom,
  calculateFitWidthZoom,
} from '../src/view/zoom-fit.ts';

test('fit page uses the real ten-pixel top and bottom gaps', () => {
  const zoom = calculateFitPageZoom(883, 683, 793.8, 1122.5);
  assert.ok(Math.abs(zoom - (663 / 1122.5)) < 1e-12);
});

test('fit width keeps twenty-pixel side gutters', () => {
  assert.ok(Math.abs(calculateFitWidthZoom(883, 793.8) - (843 / 793.8)) < 1e-12);
});

test('status bar and view command share the fit helpers', () => {
  const main = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
  const commands = readFileSync(
    new URL('../src/command/commands/view.ts', import.meta.url),
    'utf8',
  );
  assert.match(main, /calculateFitPageZoom/);
  assert.match(commands, /calculateFitPageZoom/);
  assert.doesNotMatch(main, /containerHeight - 40/);
  assert.doesNotMatch(commands, /containerH - 40/);
});
```

- [ ] **Step 2: Run the fit test and verify RED**

Run:

```bash
cd rhwp-studio
node --test tests/zoom-fit.test.ts
```

Expected: FAIL because `src/view/zoom-fit.ts` does not exist.

- [ ] **Step 3: Implement the shared fit calculations**

Create `src/view/zoom-fit.ts`:

```ts
const MIN_REQUESTED_ZOOM = 0.1;
const MAX_REQUESTED_ZOOM = 4;
const HORIZONTAL_FRAME_PADDING = 40;
const VERTICAL_FRAME_PADDING = 20;

function clampRequestedZoom(zoom: number): number {
  return Math.max(MIN_REQUESTED_ZOOM, Math.min(MAX_REQUESTED_ZOOM, zoom));
}

export function calculateFitWidthZoom(
  containerWidth: number,
  pageWidth: number,
): number {
  if (pageWidth <= 0) return 1;
  return clampRequestedZoom((containerWidth - HORIZONTAL_FRAME_PADDING) / pageWidth);
}

export function calculateFitPageZoom(
  containerWidth: number,
  containerHeight: number,
  pageWidth: number,
  pageHeight: number,
): number {
  if (pageWidth <= 0 || pageHeight <= 0) return 1;
  return clampRequestedZoom(Math.min(
    (containerWidth - HORIZONTAL_FRAME_PADDING) / pageWidth,
    (containerHeight - VERTICAL_FRAME_PADDING) / pageHeight,
  ));
}
```

- [ ] **Step 4: Replace both duplicated UI calculations**

In `src/main.ts`, import both helpers and change the status-bar listeners to:

```ts
const zoom = calculateFitWidthZoom(container.clientWidth, pageInfo.width);
vm.setZoom(zoom);
```

and:

```ts
const zoom = calculateFitPageZoom(
  container.clientWidth,
  container.clientHeight,
  pageInfo.width,
  pageInfo.height,
);
vm.setZoom(zoom);
```

In `src/command/commands/view.ts`, use the same two helpers in
`view:zoom-fit-page` and `view:zoom-fit-width`.

- [ ] **Step 5: Run all zoom-focused tests**

Run:

```bash
cd rhwp-studio
node --test \
  tests/zoom-fit.test.ts \
  tests/zoom-anchor.test.ts \
  tests/viewport-manager-smooth-zoom.test.ts
```

Expected: all tests PASS.

- [ ] **Step 6: Commit Task 3**

```bash
git add rhwp-studio/src/view/zoom-fit.ts \
  rhwp-studio/src/main.ts \
  rhwp-studio/src/command/commands/view.ts \
  rhwp-studio/tests/zoom-fit.test.ts
git diff --cached --check
git commit -m "fix: align fit-page zoom geometry"
```

### Task 4: Verify runtime behavior and repository boundaries

**Files:**
- Verify: `rhwp-studio/src/view/viewport-manager.ts`
- Verify: `rhwp-studio/src/view/canvas-view.ts`
- Verify: `rhwp-studio/src/view/zoom-anchor.ts`
- Verify: `rhwp-studio/src/view/zoom-fit.ts`
- Verify: `rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts`
- Verify: `rhwp-studio/tests/zoom-anchor.test.ts`
- Verify: `rhwp-studio/tests/zoom-fit.test.ts`

**Interfaces:**
- Consumes: completed Tasks 1-3
- Produces: focused test, build, diff, and live-browser evidence

- [ ] **Step 1: Run focused tests from a fresh process**

```bash
cd rhwp-studio
node --test \
  tests/zoom-fit.test.ts \
  tests/zoom-anchor.test.ts \
  tests/viewport-manager-smooth-zoom.test.ts
```

Expected: all zoom-focused tests PASS.

- [ ] **Step 2: Run the production build**

```bash
cd rhwp-studio
npm run build
```

Expected: TypeScript and Vite build exit `0`. Existing bundle-size warnings are acceptable; new TypeScript errors are not.

- [ ] **Step 3: Check patch boundaries**

```bash
git diff --check
git status --short
git show --stat --oneline HEAD~3..HEAD
```

Expected: no whitespace errors; zoom commits contain only the planned files and the pre-existing unrelated dirty files remain uncommitted.

- [ ] **Step 4: Measure button/keyboard center anchoring live**

On `biz_plan.hwp`, click 쪽 맞춤, record page center, execute six zoom-in clicks and six zoom-out clicks, and record:

```text
fit zoom
page center X/Y relative to viewport
scrollLeft/scrollTop
scrollWidth/scrollHeight
final zoom and position deltas after the inverse sequence
```

Expected: page center drift stays at or below `1px` while crossing horizontal overflow; inverse sequence restores zoom within `1e-12` and scroll axes within `1px`.

- [ ] **Step 5: Measure off-center wheel anchoring live**

Dispatch equal opposite Ctrl-wheel events at a point such as normalized
`(0.25, 0.75)` and derive the same page-local document coordinate before,
during, and after the gesture.

Expected: pointer-anchor drift stays at or below `1px`; final zoom and scroll position return within the same tolerances.

- [ ] **Step 6: Verify rendering lifecycle**

Observe `data-rhwp-rendered-zoom` and page-render calls during one gesture.

Expected: animation frames use CSS preview without mid-animation page rerenders; the settled frame rerenders at the final zoom.

- [ ] **Step 7: Complete board tasks**

Mark Task #77 complete only after Steps 1-6 pass. Mark Task #67 complete when the final live sensitivity and anchoring trace both pass, then call `task_list`.
