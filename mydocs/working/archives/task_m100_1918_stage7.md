# Stage 7 완료 보고서 - Task #1918

**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**작성일**: 2026-07-05

---

## 1. 추가 문제

Stage 6까지 정적 overlay와 flow-static 반복 렌더는 줄였지만, 빠르게 키를 많이 입력하면 입력 이벤트가
각 키마다 동기 canvas 렌더를 기다리는 구조가 남아 있었다.

추가 계측에서 표 텍스트 mutation만 수행하면 20회 입력이 수 ms 수준이었지만,
`document-page-invalidated`를 함께 emit하면 `samples/복학원서.hwp`는 평균 63.2ms/key까지 증가했다.

## 2. 구현 내용

- `rhwp-studio/src/view/canvas-view.ts`
  - `text-edit` page invalidation을 즉시 렌더하지 않고 `requestAnimationFrame` 큐에 적재하도록 변경했다.
  - 같은 프레임 안에 같은 페이지로 들어온 invalidation은 `Map`에서 덮어써 최신 상태만 렌더한다.
  - non-text invalidation, full refresh, zoom/resize/reset, canvas pool release에서는 pending text-edit refresh를 취소한다.
- `rhwp-studio/src/view/page-renderer.ts`
  - text-edit fast path에서 이전 page layer summary를 재사용하는 `layerSummaryCache`를 추가했다.
  - cache key는 page index, render scale, canvas physical size, render profile, backend를 포함한다.
  - full refresh, page layer 제거, 전체 layer 제거, retry state reset, dispose에서 summary cache를 비운다.
- `rhwp-studio/tests/render-backend.test.ts`
  - CanvasView text-edit coalescing 계약을 추가했다.
  - PageRenderer layer summary cache 계약을 추가했다.

## 3. 검증

통과한 명령:

```bash
cd rhwp-studio
npm test
npm run build
```

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
node e2e/renderer-contract.test.mjs --mode=headless
```

성능 probe:

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
node /private/tmp/rhwp_task1918_rapid_input_probe.mjs --mode=headless
```

20회 연속 입력 + `document-page-invalidated` 결과:

| 샘플 | Stage 6 key 평균 | Stage 7 key 평균 | Stage 7 렌더 호출 |
|------|------------------|------------------|-------------------|
| `samples/복학원서.hwp` | 63.24ms/key | 0.12ms/key | `flow` 1회 |
| `samples/253E164F57A1BC6934-empty.hwp` | 15.72ms/key | 0.04ms/key | `flow-dynamic` 1회, `flow-static` 1회 |
| `samples/143E433F503322BD33.hwp` | 10.69ms/key | 0.09ms/key | `flow-dynamic` 1회, `flow-static` 1회 |
| `samples/통합재정통계(2011.10월).hwp` | 6.55ms/key | 0.30ms/key | `flow` 1회 |

Stage 7 probe에서는 text-edit 구간의 `getPageOverlayImages` 호출이 0회였다.
초기 렌더에서 만든 summary cache를 재사용했기 때문이다.

## 4. 판단

빠른 연속 입력 중 키 입력 이벤트가 page canvas 렌더 완료를 기다리던 병목을 제거했다.

Stage 6까지는 “한 번 렌더할 때의 비용”을 줄였고, Stage 7에서는 “빠른 연속 입력 중 중간 렌더를 합치는”
갱신 정책을 추가했다. 따라서 사용자가 관측한 키 입력 backlog 현상은 크게 줄어드는 것이 기대된다.
