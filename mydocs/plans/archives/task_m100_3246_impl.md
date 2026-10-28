# 뷰포트 경계와 무관한 확대/축소 의미 구현 계획

**Goal:** 문서가 뷰포트 너비 경계를 통과해도 hybrid zoom anchor를 바꾸지 않고, 승인된 `0.00525` 민감도와 `16ms` 완화를 적용한다.

**Architecture:** `VirtualScroll`이 실제 페이지 레이아웃 양쪽에 각각 한 뷰포트 너비의 pan 공간을 추가하고 모든 페이지 X를 명시적 좌표로 제공한다. `CanvasView`는 첫 레이아웃과 resize에서 논리적 중심 스크롤을 적용하며 기존 anchored-scroll 공식은 그대로 사용한다. 눈금자와 입력 소비자는 같은 VirtualScroll 좌표를 공유한다.

**Tech Stack:** TypeScript, Node.js built-in test runner, Vite, browser runtime

## Global Constraints

- Ctrl/Cmd+휠은 포인터 아래 문서 좌표를 보존한다.
- 도구 모음과 키보드는 뷰포트 중심 아래 문서 좌표를 보존한다.
- `WHEEL_ZOOM_SENSITIVITY = 0.00525`, `ZOOM_SMOOTHING_TIME_MS = 16`을 사용한다.
- 버튼과 키보드 단계 `0.1`, zoom 범위 `0.25..4.0`, wheel delta clamp `120px`를 유지한다.
- animation 중 CSS preview만 사용하고 정착 뒤 보이는 페이지를 다시 렌더링한다.
- 관련 없는 dirty Subsecond, renderer, UI, CSS 변경을 stage하거나 수정하지 않는다.

---

### Task 1: VirtualScroll에 안정된 가로 pan 좌표 추가

**Files:**
- Modify: `rhwp-studio/src/view/virtual-scroll.ts:23-94,217-245`
- Create: `rhwp-studio/tests/virtual-scroll-horizontal-pan.test.ts`

**Interfaces:**
- Consumes: `VirtualScroll.setPageDimensions(pages, zoom, viewportWidth)`
- Produces: `VirtualScroll.getCenteredScrollLeft(viewportWidth): number`, 명시적 `getPageLeft(pageIdx)`, pan 공간을 포함한 `getTotalWidth()`

- [ ] **Step 1: underflow/overflow 중앙 배치와 anchor 왕복 실패 테스트 작성**

```ts
import test from 'node:test';
import assert from 'node:assert/strict';
import { VirtualScroll } from '../src/view/virtual-scroll.ts';
import { calculateAnchoredScroll } from '../src/view/zoom-anchor.ts';

const pages = [{ width: 800, height: 1000 }] as never;

test('horizontal pan coordinates center the page on both sides of overflow', () => {
  const viewportWidth = 900;
  const scroll = new VirtualScroll();

  scroll.setPageDimensions(pages, 0.5, viewportWidth);
  const underflowLeft = scroll.getPageLeft(0);
  const underflowCenter = scroll.getCenteredScrollLeft(viewportWidth);
  assert.ok(underflowLeft >= 0);
  assert.equal(
    underflowLeft - underflowCenter,
    (viewportWidth - scroll.getPageWidth(0)) / 2,
  );

  scroll.setPageDimensions(pages, 1.25, viewportWidth);
  const overflowLeft = scroll.getPageLeft(0);
  const overflowCenter = scroll.getCenteredScrollLeft(viewportWidth);
  assert.equal(
    overflowLeft - overflowCenter,
    (viewportWidth - scroll.getPageWidth(0)) / 2,
  );
});

test('pointer anchor remains representable across the viewport-width boundary', () => {
  const viewportWidth = 900;
  const anchor = { x: 0.35, y: 0.75 };
  const scroll = new VirtualScroll();

  scroll.setPageDimensions(pages, 0.54, viewportWidth);
  const oldBox = {
    left: scroll.getPageLeft(0),
    top: scroll.getPageOffset(0),
    width: scroll.getPageWidth(0),
    height: scroll.getPageHeight(0),
  };
  const viewport = {
    width: viewportWidth,
    height: 650,
    scrollLeft: scroll.getCenteredScrollLeft(viewportWidth),
    scrollTop: 0,
  };

  scroll.setPageDimensions(pages, 1.2, viewportWidth);
  const newBox = {
    left: scroll.getPageLeft(0),
    top: scroll.getPageOffset(0),
    width: scroll.getPageWidth(0),
    height: scroll.getPageHeight(0),
  };
  const forward = calculateAnchoredScroll(oldBox, newBox, viewport, anchor);
  assert.ok(forward.scrollLeft >= 0);
  assert.ok(forward.scrollLeft <= scroll.getTotalWidth() - viewportWidth);

  const reverse = calculateAnchoredScroll(
    newBox,
    oldBox,
    { ...viewport, ...forward },
    anchor,
  );
  assert.ok(Math.abs(reverse.scrollLeft - viewport.scrollLeft) < 1e-9);
});
```

- [ ] **Step 2: 실패 확인**

Run:

```bash
/opt/homebrew/bin/node --test tests/virtual-scroll-horizontal-pan.test.ts
```

Expected: `getCenteredScrollLeft is not a function` 또는 underflow anchor 범위 assertion 실패.

- [ ] **Step 3: pan 좌표를 최소 구현**

`setPageDimensions`의 기존 layout 호출 뒤에 다음 helper를 호출한다.

```ts
this.applyHorizontalPanSpace(viewportWidth);
```

`VirtualScroll`에 다음 메서드를 추가한다.

```ts
private applyHorizontalPanSpace(viewportWidth: number): void {
  if (viewportWidth <= 0) return;
  const baseWidth = this.totalWidth;
  this.pageLefts = this.pageLefts.map((left, pageIdx) => {
    const resolved = left >= 0
      ? left
      : (baseWidth - (this.pageWidths[pageIdx] ?? 0)) / 2;
    return resolved + viewportWidth;
  });
  this.totalWidth = baseWidth + viewportWidth * 2;
}

getCenteredScrollLeft(viewportWidth: number): number {
  return Math.max(0, (this.totalWidth - viewportWidth) / 2);
}
```

- [ ] **Step 4: VirtualScroll 테스트 통과 확인**

Run:

```bash
/opt/homebrew/bin/node --test \
  tests/virtual-scroll-horizontal-pan.test.ts \
  tests/virtual-scroll-grid-page.test.ts \
  tests/zoom-anchor.test.ts
```

Expected: 모든 테스트 PASS.

- [ ] **Step 5: Task 1 커밋**

```bash
git add \
  rhwp-studio/src/view/virtual-scroll.ts \
  rhwp-studio/tests/virtual-scroll-horizontal-pan.test.ts
git diff --cached --check
git commit -m "fix: stabilize horizontal zoom coordinates"
```

---

### Task 2: CanvasView와 눈금자를 공통 pan 좌표에 연결

**Files:**
- Modify: `rhwp-studio/src/view/zoom-anchor.ts:21-60`
- Modify: `rhwp-studio/src/view/canvas-view.ts:15-22,39-145,277-288,510-570`
- Modify: `rhwp-studio/src/view/ruler.ts:126-143,216-222`
- Modify: `rhwp-studio/tests/zoom-anchor.test.ts`

**Interfaces:**
- Consumes: Task 1의 `getCenteredScrollLeft`, `getPageLeft`, `getTotalWidth`
- Produces: `calculateAnchoredScroll(..., nextViewportSize?)`, 첫 문서 중앙 배치, resize 중심 보존, 공통 ruler X

- [ ] **Step 1: resize와 좌표 소비자 실패 테스트 작성**

`tests/zoom-anchor.test.ts`에 다음 테스트와 source contract를 추가한다.

```ts
test('anchored scroll can preserve a point across viewport resize', () => {
  const oldBox = { left: 900, top: 10, width: 500, height: 700 };
  const newBox = { left: 700, top: 10, width: 500, height: 700 };
  const next = calculateAnchoredScroll(
    oldBox,
    newBox,
    {
      width: 900,
      height: 650,
      scrollLeft: 700,
      scrollTop: 200,
    },
    { x: 0.5, y: 0.5 },
    { width: 700, height: 550 },
  );

  assert.equal(next.scrollLeft, 600);
  assert.equal(next.scrollTop, 250);
});

test('CanvasView and ruler consume the stable horizontal coordinate', () => {
  const canvasSource = readFileSync(
    new URL('../src/view/canvas-view.ts', import.meta.url),
    'utf8',
  );
  const rulerSource = readFileSync(
    new URL('../src/view/ruler.ts', import.meta.url),
    'utf8',
  );

  assert.match(canvasSource, /getCenteredScrollLeft\(/);
  assert.match(rulerSource, /getPageLeftResolved\(0, this\.virtualScroll\.getTotalWidth\(\)\)/);
  assert.doesNotMatch(rulerSource, /contentOffsetX/);
});
```

- [ ] **Step 2: 실패 확인**

Run:

```bash
/opt/homebrew/bin/node --test tests/zoom-anchor.test.ts
```

Expected: resize 결과와 source contract 실패.

- [ ] **Step 3: resize 대상 viewport를 anchored-scroll helper에 추가**

`zoom-anchor.ts`에서 선택적 다음 viewport 크기를 받는다.

```ts
export function calculateAnchoredScroll(
  oldBox: ZoomPageBox,
  newBox: ZoomPageBox,
  viewport: ZoomViewportState,
  requestedAnchor: ZoomAnchor,
  nextViewportSize: Pick<ZoomViewportState, 'width' | 'height'> = viewport,
): Pick<ZoomViewportState, 'scrollLeft' | 'scrollTop'> {
  const anchor = normalizeZoomAnchor(requestedAnchor);
  const viewportX = viewport.width * anchor.x;
  const viewportY = viewport.height * anchor.y;
  const nextViewportX = nextViewportSize.width * anchor.x;
  const nextViewportY = nextViewportSize.height * anchor.y;
  const documentX = viewport.scrollLeft + viewportX;
  const documentY = viewport.scrollTop + viewportY;
  const ratioX = oldBox.width > 0
    ? (documentX - oldBox.left) / oldBox.width
    : 0.5;
  const ratioY = oldBox.height > 0
    ? (documentY - oldBox.top) / oldBox.height
    : 0.5;

  return {
    scrollLeft: newBox.left + newBox.width * ratioX - nextViewportX,
    scrollTop: newBox.top + newBox.height * ratioY - nextViewportY,
  };
}
```

- [ ] **Step 4: CanvasView 초기 배치와 resize 보존 구현**

`CanvasView`에 마지막 layout viewport 크기를 저장한다.

```ts
private layoutViewportSize = { width: 0, height: 0 };
```

`zoom-anchor.ts` import에 center anchor를 추가한다.

```ts
import {
  calculateAnchoredScroll,
  CENTER_ZOOM_ANCHOR,
  normalizeZoomAnchor,
  type ZoomAnchor,
  type ZoomPageBox,
} from './zoom-anchor.ts';
```

`recalcLayout()`은 현재 크기를 기록한다.

```ts
const viewport = this.viewportManager.getViewportSize();
this.virtualScroll.setPageDimensions(this.pages, zoom, viewport.width);
this.scrollContent.style.height = `${this.virtualScroll.getTotalHeight()}px`;
this.scrollContent.style.width = `${this.virtualScroll.getTotalWidth()}px`;
this.layoutViewportSize = viewport;
```

첫 `loadDocument()`의 `recalcLayout()` 직후 가운데 가로 위치를 적용한다.

```ts
this.viewportManager.setScrollLeft(
  this.virtualScroll.getCenteredScrollLeft(this.layoutViewportSize.width),
);
```

`onViewportResize()`를 다음으로 교체한다.

```ts
private onViewportResize(): void {
  const nextViewport = this.viewportManager.getViewportSize();
  if (this.pages.length === 0) {
    this.layoutViewportSize = nextViewport;
    this.updateVisiblePages();
    return;
  }

  const previousViewport = this.layoutViewportSize;
  const canPreserveCenter = previousViewport.width > 0 && previousViewport.height > 0;
  const scrollLeft = this.viewportManager.getScrollX();
  const scrollTop = this.viewportManager.getScrollY();
  const focusPage = canPreserveCenter
    ? this.virtualScroll.getPageAtPoint(
      scrollLeft + previousViewport.width / 2,
      scrollTop + previousViewport.height / 2,
    )
    : 0;
  const oldBox = canPreserveCenter
    ? this.getZoomPageBox(focusPage, previousViewport.width)
    : null;

  const wasGrid = this.virtualScroll.isGridMode();
  this.recalcLayout();
  const isGrid = this.virtualScroll.isGridMode();

  if (oldBox) {
    const newBox = this.getZoomPageBox(focusPage, nextViewport.width);
    const nextScroll = calculateAnchoredScroll(
      oldBox,
      newBox,
      {
        width: previousViewport.width,
        height: previousViewport.height,
        scrollLeft,
        scrollTop,
      },
      CENTER_ZOOM_ANCHOR,
      nextViewport,
    );
    this.viewportManager.setScrollLeft(nextScroll.scrollLeft);
    this.viewportManager.setScrollTop(nextScroll.scrollTop);
  } else {
    this.viewportManager.setScrollLeft(
      this.virtualScroll.getCenteredScrollLeft(nextViewport.width),
    );
  }

  if (wasGrid || isGrid) {
    this.cancelPendingTextEditRefresh();
    this.cancelTextEditStaticLayerVerification();
    this.releaseAllRenderedPages();
    this.pageRenderer.cancelAll();
  }
  this.updateVisiblePages();
}
```

- [ ] **Step 5: 눈금자 X 계산 통합**

`Ruler.getPageScreenLeft`를 다음으로 교체한다.

```ts
private getPageScreenLeft(scrollX: number): number {
  return this.virtualScroll.getPageLeftResolved(
    0,
    this.virtualScroll.getTotalWidth(),
  ) - scrollX;
}
```

`drawHorizontal()` 호출은 다음을 사용한다.

```ts
const pageScreenLeft = this.getPageScreenLeft(scrollX);
```

- [ ] **Step 6: 집중 테스트 통과 확인**

Run:

```bash
/opt/homebrew/bin/node --test \
  tests/zoom-anchor.test.ts \
  tests/virtual-scroll-horizontal-pan.test.ts \
  tests/virtual-scroll-grid-page.test.ts
```

Expected: 모든 테스트 PASS.

- [ ] **Step 7: Task 2 커밋**

`canvas-view.ts`의 unrelated Subsecond hunk를 stage하지 않고 zoom hunk만 stage한다.

```bash
git add \
  rhwp-studio/src/view/zoom-anchor.ts \
  rhwp-studio/src/view/ruler.ts \
  rhwp-studio/tests/zoom-anchor.test.ts
git diff -- rhwp-studio/src/view/canvas-view.ts
git apply --cached <zoom-only-canvas-view.patch>
git diff --cached --check
git commit -m "fix: preserve zoom meaning across viewport bounds"
```

---

### Task 3: 민감도와 완화 시간 조정

**Files:**
- Modify: `rhwp-studio/src/view/viewport-manager.ts:11-12`
- Modify: `rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts:169-215`

**Interfaces:**
- Consumes: 기존 대칭 지수 zoom target과 animation-frame 병합
- Produces: `WHEEL_ZOOM_SENSITIVITY = 0.00525`, `ZOOM_SMOOTHING_TIME_MS = 16`

- [ ] **Step 1: 더 강한 이동량과 빠른 정착 실패 테스트 작성**

기존 8픽셀 테스트의 이름과 마지막 assertion 두 개를 다음으로 교체한다.

```ts
test('an eight-pixel trackpad gesture settles within four frames and moves over four percent', async (t) => {
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
  const viewport = new ViewportManager(new FakeEventBus() as never);
  const onWheel = (
    viewport as unknown as {
      onWheel: (event: {
        ctrlKey: boolean;
        metaKey: boolean;
        deltaY: number;
        deltaMode: number;
        preventDefault: () => void;
      }) => void;
    }
  ).onWheel.bind(viewport);

  onWheel({
    ctrlKey: true,
    metaKey: false,
    deltaY: 8,
    deltaMode: 0,
    preventDefault: () => {},
  });

  let timestamp = 16;
  let frameCount = 0;
  while (frames.pendingCount > 0 && timestamp < 1000) {
    frames.flush(timestamp);
    timestamp += 16;
    frameCount += 1;
  }

  assert.ok(viewport.getZoom() < 0.96, `expected responsive travel, got ${viewport.getZoom()}`);
  assert.ok(frameCount <= 4, `expected at most four frames, got ${frameCount}`);
});
```

- [ ] **Step 2: 기존 값에서 실패 확인**

Run:

```bash
/opt/homebrew/bin/node --test tests/viewport-manager-smooth-zoom.test.ts
```

Expected: 최종 zoom 약 `0.9670` 또는 frame count로 실패.

- [ ] **Step 3: 상수 두 개만 변경**

```ts
const ZOOM_SMOOTHING_TIME_MS = 16;
const WHEEL_ZOOM_SENSITIVITY = 0.00525;
```

- [ ] **Step 4: zoom 집중 테스트 통과 확인**

Run:

```bash
/opt/homebrew/bin/node --test \
  tests/viewport-manager-smooth-zoom.test.ts \
  tests/zoom-anchor.test.ts \
  tests/virtual-scroll-horizontal-pan.test.ts
```

Expected: 모든 테스트 PASS.

- [ ] **Step 5: Task 3 커밋**

```bash
git add \
  rhwp-studio/src/view/viewport-manager.ts \
  rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts
git diff --cached --check
git commit -m "fix: accelerate symmetric document zoom"
```

---

### Task 4: 전체와 실제 문서 검증

**Files:**
- Verify: Tasks 1-3의 모든 파일

**Interfaces:**
- Consumes: 안정된 pan 좌표, resize 보존, `0.00525` / `16ms`
- Produces: focused/full test, build, diff, live-browser 근거

- [ ] **Step 1: 전체 Node 테스트**

Run:

```bash
PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm test
```

Expected: 실패 `0`.

- [ ] **Step 2: production build와 diff 검사**

Run:

```bash
PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run build
git diff --check
```

Expected: build exit `0`, diff 오류 없음.

- [ ] **Step 3: 실제 브라우저 포인터 roundtrip**

`biz_plan.hwp`를 fit으로 맞춘 뒤 포인터 `(0.25, 0.75)`에 동일한 확대 입력을 연속으로
보내 문서가 viewport 경계를 통과하게 한다. 각 frame에서 page-local 좌표와
`data-rhwp-rendered-zoom`을 수집하고 반대 입력으로 복원한다.

Expected:

- 경계 전·중·후 page-local anchor drift가 문서 좌표 `1px` 이하
- final zoom error `1e-12` 이하
- final scroll error `1px` 이하
- animation 중 `renderCanvas` 호출 `0`
- 정착 뒤 보이는 page redraw

- [ ] **Step 4: 도구 모음 중심과 resize 검증**

여섯 번 확대/여섯 번 축소로 viewport 중심점을 확인하고, viewport 너비를 한 번 줄였다
되돌려 같은 page-local 중심 좌표를 측정한다.

Expected: 각 roundtrip의 중심점과 scroll drift `1px` 이하.

- [ ] **Step 5: 작업 경계 확인**

```bash
git status --short
git show --stat --oneline HEAD~3..HEAD
```

Expected: zoom 계획 파일과 Tasks 1-3 파일만 새 commit에 있고 unrelated dirty 파일은
그대로 uncommitted 상태다.
