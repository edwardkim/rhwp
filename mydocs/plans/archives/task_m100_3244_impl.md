# 문서 확대/축소 반응성 조정 구현 계획

**Goal:** Ctrl/Cmd+휠 확대/축소가 현재보다 더 크게 반응하고 짧은 시간 안에 목표 배율에 정착하게 한다.

**Architecture:** 기존 `ViewportManager`의 지수 완화, animation frame 병합, 배율 제한 구조는 유지한다. 회귀 테스트로 8픽셀 트랙패드 입력의 최종 이동량을 `0.0042` 계약으로 강화한 뒤 민감도 상수만 변경한다.

**Tech Stack:** TypeScript, Node.js built-in test runner, Vite, browser runtime

## Global Constraints

- 정규화된 휠 민감도는 `0.0042`로 변경한다.
- 지수 완화 시간 상수는 `22ms`로 변경한다.
- 버튼과 키보드 확대/축소 단계는 `0.1`을 유지한다.
- animation frame 병합, CSS 미리보기, 완료 후 재렌더링, 페이지 중심점 보정을 유지한다.
- 최소·최대 확대 배율과 즉시 적용하는 `setZoom` 계약을 변경하지 않는다.
- 관련 없는 dirty worktree 변경을 수정하거나 커밋하지 않는다.

---

### Task 1: 휠 확대/축소 반응성 조정

**Files:**
- Modify: `rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts`
- Modify: `rhwp-studio/src/view/viewport-manager.ts`

**Interfaces:**
- Consumes: `ViewportManager`의 기존 private `onWheel(WheelEvent)`, `getZoom()`, animation frame 흐름
- Produces: 8픽셀 휠 입력이 6 frame 이내에 정착하고 배율을 2%보다 크게 줄이는 입력 계약

- [ ] **Step 1: 실패하는 반응성 회귀 테스트 작성**

`rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts`에 다음 테스트를 추가한다.

```ts
test('an eight-pixel trackpad gesture settles within six frames and moves over three percent', async (t) => {
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

  assert.ok(viewport.getZoom() < 0.97, `expected responsive travel, got ${viewport.getZoom()}`);
  assert.ok(frameCount <= 6, `expected at most six frames, got ${frameCount}`);
});
```

- [ ] **Step 2: 회귀 테스트가 기존 값에서 실패하는지 확인**

Run:

```bash
/opt/homebrew/bin/node --test tests/viewport-manager-smooth-zoom.test.ts
```

Expected: 강화된 이동량 계약이 현재 최종 배율 약 `0.9724`에서 실패한다.

- [ ] **Step 3: 민감도 상수만 변경**

`rhwp-studio/src/view/viewport-manager.ts`의 민감도 상수를 다음과 같이 변경한다.

```ts
const WHEEL_ZOOM_SENSITIVITY = 0.0042;
```

- [ ] **Step 4: 집중 테스트가 통과하는지 확인**

Run:

```bash
/opt/homebrew/bin/node --test tests/viewport-manager-smooth-zoom.test.ts
```

Expected: 모든 확대/축소 집중 테스트가 통과한다.

- [ ] **Step 5: 실제 문서에서 반응성과 보존 계약 확인**

실제 브라우저의 현재 6쪽 문서에서 배율을 `1`로 초기화하고 `deltaY: 1`인 Ctrl+휠 이벤트를
8회 보낸다. animation frame별 배율, 목표 정착 시간, `renderCanvas` 호출 시점, 중심 페이지의
상대 위치를 측정한다.

Expected:

- 최종 배율이 기존 약 `0.9724`보다 작은 약 `0.9670`
- 입력 시작 후 약 `100ms` 안에 정착
- 애니메이션 중 `renderCanvas` 호출 0회
- 완료 후 보이는 페이지 재렌더링
- 중심 페이지 상대 위치 drift `0.002` 이하

- [ ] **Step 6: 전체 검증 실행**

Run:

```bash
/opt/homebrew/bin/node --test tests/*.test.ts ../npm/editor/tests/*.test.mjs
npm run build
git diff --check
```

Expected: 전체 Node 테스트 실패 0개, production build exit 0, diff 오류 없음.

- [ ] **Step 7: 확대/축소 범위만 커밋**

검증이 끝나면 이번 작업의 확대/축소 파일만 stage한다.

```bash
git add \
  rhwp-studio/src/command/commands/view.ts \
  rhwp-studio/src/main.ts \
  rhwp-studio/src/view/canvas-view.ts \
  rhwp-studio/src/view/viewport-manager.ts \
  rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts
git commit -m "fix: make document zoom responsive"
```

Expected: 관련 없는 dirty 파일은 stage되지 않고 그대로 남는다.
