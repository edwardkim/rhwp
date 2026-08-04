# 세로 휠 축 고정 구현 계획

**Goal:** 아래 방향 트랙패드 스크롤의 작은 수평 성분을 제거하고 대칭 휠 줌 감도를 높인다.

**Architecture:** `ViewportManager`가 휠의 단위를 픽셀로 통합한 뒤 입력의 우세 축을 판별한다. 세로 우세 일반 휠은 `scrollTop`에만 적용하고, 가로 우세 입력과 기존 포인터 앵커 줌 경로는 유지한다.

**Tech Stack:** TypeScript, DOM WheelEvent, Node test runner

## Global Constraints

- 세로 우세 조건은 `|deltaY| >= |deltaX|`이다.
- 줌 감도는 양방향 공통 `0.00625`이다.
- 관련 없는 Subsecond 및 UI 작업 파일은 스테이징하지 않는다.

---

### Task 1: 휠 축 고정과 줌 감도

**Files:**
- Modify: `rhwp-studio/src/view/viewport-manager.ts`
- Test: `rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts`

**Interfaces:**
- Consumes: DOM `WheelEvent.deltaX`, `deltaY`, `deltaMode`
- Produces: 세로 우세 입력의 `scrollTop` 전용 이동과 대칭 `0.00625` 줌 계수

- [ ] **Step 1: 실패 테스트 작성**

```ts
test('vertical-dominant wheel input locks horizontal pan', async () => {
  const viewport = new ViewportManager(new FakeEventBus() as never);
  const container = { scrollTop: 100 };
  (viewport as unknown as { container: typeof container }).container = container;
  let prevented = false;
  callOnWheel(viewport, {
    ctrlKey: false,
    metaKey: false,
    deltaX: 3,
    deltaY: 20,
    deltaMode: 0,
    preventDefault: () => { prevented = true; },
  });
  assert.equal(prevented, true);
  assert.equal(container.scrollTop, 120);
});
```

가로 우세 입력은 `preventDefault`를 호출하지 않는 테스트와 8픽셀 줌 결과가
`Math.exp(-8 * 0.00625)`인 테스트도 함께 추가한다.

- [ ] **Step 2: 실패 확인**

Run: `/opt/homebrew/bin/node --test tests/viewport-manager-smooth-zoom.test.ts`

Expected: 세로 입력이 가로채지지 않고 기존 감도 결과가 달라 FAIL.

- [ ] **Step 3: 최소 구현**

```ts
const WHEEL_ZOOM_SENSITIVITY = 0.00625;

private wheelDeltaPixels(delta: number, deltaMode: number): number {
  return deltaMode === 1
    ? delta * 16
    : deltaMode === 2
      ? delta * Math.max(this.viewportHeight, 1)
      : delta;
}
```

`onWheel`에서 일반 입력의 X/Y를 위 함수로 정규화하고, 세로 우세일 때
`preventDefault()` 후 `container.scrollTop += deltaY`만 수행한다. 줌 경로도
같은 정규화 함수를 사용한다.

- [ ] **Step 4: 집중 테스트와 전체 검증**

Run:

```bash
/opt/homebrew/bin/node --test tests/viewport-manager-smooth-zoom.test.ts tests/zoom-anchor.test.ts tests/virtual-scroll-horizontal-pan.test.ts
PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm test
PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run build
```

Expected: 모든 테스트와 빌드 PASS.

- [ ] **Step 5: 실제 문서 검증**

세로 우세 휠 왕복에서 `scrollLeft`가 변하지 않는지, 가로 우세 입력이
가로 팬을 유지하는지, 8픽셀 줌 왕복 오차가 부동소수점 허용 범위인지
실제 브라우저 브라우저에서 확인한다.

- [ ] **Step 6: 커밋**

```bash
git add rhwp-studio/src/view/viewport-manager.ts rhwp-studio/tests/viewport-manager-smooth-zoom.test.ts
git commit -m "fix: lock vertical wheel scrolling"
```
