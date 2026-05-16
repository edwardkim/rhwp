# Task #931 구현 계획서

**선행**: [task_m100_931.md](task_m100_931.md) 수행계획서 승인.

## 단계별 진행

### Stage 1 — 재현 실측 + 기준값 확정

**목적**: `복학원서.hwp`의 BehindText overlay가 줌 전후 어떤 DOM 크기로 배치되는지 실측해 수정 전 기준값을 확정한다.

**영역**:
- rhwp-studio dev 서버 기동
- `samples/복학원서.hwp` 로드
- 100%/85%/25% 줌 상태에서 다음 값 측정:
  - canvas CSS width/height
  - `data-rhwp-overlay="behind-0"` layer width/height
  - 워터마크 `<img>`의 left/top/width/height
  - 지연 재렌더 후 canvas/overlay 상태
- PageLayerTree 기준 워터마크 bbox 재확인:
  - 원본: `x=137.707, y=270.24, w=495.04, h=495.733`
  - 25% 기대: `x≈34.4, y≈67.6, w≈123.8, h≈123.9`

**판정 기준**:
- 25%에서 overlay layer와 image bbox가 원본 크기로 남는 현상을 DOM 수치로 확정한다.
- 지연 재렌더가 `flow` 필터를 깨는지 시각/DOM 증거를 확보한다.

**산출물**:
- `mydocs/working/task_m100_931_stage1.md`

### Stage 2 — overlay 줌 정합화 구현

**목적**: BehindText/InFrontOfText overlay가 canvas 표시 크기와 동일한 화면 배율을 사용하도록 정정한다.

**정정 파일**:
- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/src/view/page-renderer.ts`

**정정 방향**:

1. `CanvasView.renderPage()`에서 `zoom`, `dpr`, `renderScale`을 명시적으로 구분한다.

```ts
const renderScale = zoom * dpr;
this.pageRenderer.renderPage(pageIdx, canvas, renderScale, zoom, dpr);
```

2. `PageRenderer.renderPage()` 시그니처를 확장한다.

```ts
renderPage(
  pageIdx: number,
  canvas: HTMLCanvasElement,
  renderScale: number,
  displayScale: number,
  dpr: number,
): void
```

3. `applyOverlays()`에서 overlay layer의 표시 크기를 canvas 표시 크기와 맞춘다.

```ts
const cssWidth = canvas.width / dpr;
const cssHeight = canvas.height / dpr;
```

4. `createOverlayLayer()`에서 이미지 bbox에 `displayScale`을 적용한다.

```ts
el.style.left = `${img.bbox.x * displayScale}px`;
el.style.top = `${img.bbox.y * displayScale}px`;
el.style.width = `${img.bbox.width * displayScale}px`;
el.style.height = `${img.bbox.height * displayScale}px`;
```

5. overlay layer에 페이지 경계 clipping을 적용한다.

```ts
layer.style.overflow = 'hidden';
```

6. 기존 `renderPageFlow()` 호출부가 있으면 동일한 렌더링 인자 체계로 정리하거나, 호출부가 없다면 혼동을 줄이는 방향으로 유지/정리한다.

**구현 원칙**:
- 워터마크 자체는 제거하지 않는다.
- `behindText`와 `inFrontOfText` 모두 동일한 배율 규칙을 적용한다.
- `pointer-events: none`과 z-index 정책은 유지한다.
- SVG/native renderer에는 영향 주지 않는다.

**산출물**:
- 정정 코드
- `mydocs/working/task_m100_931_stage2.md`

### Stage 3 — 지연 재렌더 layer filter 보존

**목적**: 이미지 로딩 보정용 지연 재렌더가 본문 canvas에 BehindText/InFrontOfText 이미지를 다시 그려 넣지 않도록 정정한다.

**정정 파일**:
- `rhwp-studio/src/view/page-renderer.ts`

**정정 방향**:

현재:

```ts
this.wasm.renderPageToCanvas(pageIdx, canvas, scale);
```

변경:

```ts
this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow');
```

**추가 점검**:
- `scheduleReRender()` 인자명을 `scale`에서 `renderScale`로 정리해 의미 혼동을 줄인다.
- 지연 재렌더 후에도 overlay layer는 DOM sibling으로 유지되어야 한다.
- 지연 재렌더가 canvas 크기를 재설정하므로, overlay layer가 재렌더 이후 canvas 표시 크기와 어긋나지 않는지 확인한다.

**산출물**:
- 정정 코드
- `mydocs/working/task_m100_931_stage3.md`

### Stage 4 — 브라우저 검증 + 최종 보고

**목적**: 실제 rhwp-studio 화면에서 결함 해소와 회귀 부재를 검증하고 최종 보고서를 작성한다.

**검증 항목**:
- `npm run build`
- 브라우저 검증:
  - 페이지 로드: `http://127.0.0.1:7700/`
  - `samples/복학원서.hwp` 로드
  - 100%/85%/25% 줌 상태 스크린샷 확인
  - 25% 줌에서 워터마크 DOM width가 약 124px인지 확인
  - 회색 작업 영역에 원본 크기 워터마크가 노출되지 않는지 확인
  - 85%/100%에서 문서 안 워터마크가 유지되는지 확인
- 브라우저 console error/warn 확인

**선택 검증**:
- 회귀 위험이 확인되면 rhwp-studio e2e에 overlay zoom 가드를 추가한다.
- TypeScript 변경에 한정되므로 cargo 전체 테스트는 기본 범위에서 제외하되, renderer core 영향이 발견되면 보조 검증으로 수행한다.

**최종 산출물**:
- `mydocs/working/task_m100_931_stage4.md`
- `mydocs/report/task_m100_931_report.md`
- `mydocs/orders/20260516.md` 상태 갱신

### Stage 5 — 시각 피드백 보정: flow canvas 배경 분리

**배경**: Stage 4 후 작업지시자 시각 검증에서 워터마크가 보이지 않는 문제가 확인됐다. DOM overlay는 존재하지만 flow canvas의 흰 페이지 배경 뒤에 가려지는 상태였다.

**정정 파일**:
- `src/renderer/web_canvas.rs`
- `rhwp-studio/src/view/page-renderer.ts`
- `rhwp-studio/src/view/canvas-view.ts`

**정정 방향**:
- BehindText가 있는 flow canvas는 페이지 배경을 투명하게 유지한다.
- rhwp-studio는 별도 page background DOM layer를 만들고 그 위에 BehindText overlay, 그 위에 transparent flow canvas를 쌓는다.
- DOM overlay 워터마크에도 기존 canvas renderer와 동일한 `opacity=0.17` 정책을 적용한다.
- canvas release 시 sibling overlay layer가 남지 않도록 정리 API를 추가한다.

**산출물**:
- 정정 코드
- `mydocs/working/task_m100_931_stage5.md`
- 최종 보고서 갱신

## 단계별 commit 전략

| Stage | commit | 영역 |
|-------|--------|------|
| 계획 | `Task #931: 수행/구현 계획서 작성` | orders + plans |
| Stage 1 | `Task #931 Stage 1: overlay zoom 결함 실측 보고` | 진단 보고서 |
| Stage 2 | `Task #931 Stage 2: overlay bbox zoom 배율 적용` | rhwp-studio view 코드 + 보고서 |
| Stage 3 | `Task #931 Stage 3: 지연 재렌더 flow filter 보존` | page-renderer 코드 + 보고서 |
| Stage 4 | `Task #931: 최종 검증 보고서 작성` | 최종 보고서 + orders |
| Stage 5 | `Task #931 Stage 5: 워터마크 layer 가림 보정` | Rust WASM renderer + rhwp-studio view 코드 + 보고서 |

## 위험 영역 + 가드

| 위험 | 설명 | 가드 |
|------|------|------|
| overlay 이중 배율 | bbox에 zoom을 곱하면서 부모에도 transform scale을 적용하면 중복 축소 가능 | 구현은 bbox 직접 배율 적용 방식으로 통일 |
| grid mode 좌표 회귀 | 50% 이하 줌에서 grid mode로 전환될 수 있음 | `canvas.style.left/top/transform` 복사 정책 유지 |
| 지연 재렌더 후 크기 어긋남 | 재렌더가 canvas width/height를 다시 설정함 | `renderScale`과 `dpr/displayScale` 관계를 DOM 실측으로 검증 |
| inFrontOfText 회귀 | 앞쪽 그림 overlay도 같은 경로 사용 | front/behind 모두 동일 기준 적용 |
| 브라우저 확장 CSP 회귀 | DOM API 기반 변경이어야 함 | `innerHTML` 사용 금지, DOM style API 유지 |

## 승인 요청

위 구현 계획 승인 후 Stage 1 실측 진단부터 진행한다.
