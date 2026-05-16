# Task #931 Stage 3 완료 보고서

## 1. 목적

이미지 로딩 보정용 지연 재렌더가 BehindText/InFrontOfText 분리 렌더링을 깨뜨리지 않도록 `flow` 필터를 유지하게 정정했다.

## 2. 수정 파일

- `rhwp-studio/src/view/page-renderer.ts`

## 3. 수정 내용

### 3.1 `scheduleReRender()` 의미 명확화

인자명을 `scale`에서 `renderScale`로 변경했다. 이 값은 WASM canvas 렌더링용 배율이며 `zoom × dpr`이다.

```ts
private scheduleReRender(pageIdx: number, canvas: HTMLCanvasElement, renderScale: number): void
```

### 3.2 지연 재렌더 filter 보존

기존 지연 재렌더는 필터 없는 전체 렌더를 호출했다.

```ts
this.wasm.renderPageToCanvas(pageIdx, canvas, scale);
```

수정 후에는 초기 렌더와 동일하게 본문 `flow` layer만 렌더한다.

```ts
this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow');
```

따라서 BehindText/InFrontOfText 그림은 canvas에 다시 들어가지 않고 overlay sibling으로만 유지된다.

## 4. 코드 확인

`rhwp-studio/src/view/page-renderer.ts`의 canvas 렌더 호출은 모두 filtered 경로를 사용한다.

```text
renderPage()       → renderPageToCanvasFiltered(..., 'flow')
renderPageFlow()   → renderPageToCanvasFiltered(..., 'flow')
scheduleReRender() → renderPageToCanvasFiltered(..., 'flow')
```

`WasmBridge`의 fallback 경로에는 구버전 WASM용 `renderPageToCanvas()`가 남아 있으나, 이는 `renderPageToCanvasFiltered` API가 없는 환경용 fallback이다. 현재 개발/검증 환경의 PageRenderer 직접 호출 경로에서는 unfiltered 렌더를 사용하지 않는다.

## 5. 검증

### 5.1 빌드

```bash
cd rhwp-studio
npm run build
```

결과: 통과

Vite chunk size warning은 기존 번들 크기 경고이며 이번 변경과 무관하다.

### 5.2 DOM 재실측

측정 환경:

- rhwp-studio dev server: `http://127.0.0.1:7700/`
- headless Chrome
- viewport: 1600×1000
- `devicePixelRatio=1`
- sample: `samples/복학원서.hwp`
- script: `/private/tmp/rhwp-watermark-analysis/stage1-measure.mjs`
- 각 줌 변경 후 900ms 대기: 200ms/600ms 지연 재렌더 이후 상태 포함

#### 25% 줌

| 대상 | Stage 3 측정값 | 기대값 |
|------|----------------|--------|
| canvas CSS rect | `198 × 280` | `198.4 × 280.6` |
| behind overlay layer rect | `198 × 280` | `198.4 × 280.6` |
| 워터마크 img rect | `123.75 × 123.92` | `123.8 × 123.9` |
| 워터마크 left/top | `34.4268 / 67.56` | `34.4 / 67.6` |
| overlay overflow | `hidden` | `hidden` |

Stage 2에서 정정한 overlay 크기와 bbox가 지연 재렌더 이후에도 유지됐다.

#### 85% 줌

| 대상 | Stage 3 측정값 | 기대값 |
|------|----------------|--------|
| canvas CSS rect | `674 × 954` | `674.6 × 954.1` |
| behind overlay layer rect | `674 × 954` | `674.6 × 954.1` |
| 워터마크 img rect | `420.78 × 421.36` | `420.8 × 421.4` |

85%에서도 overlay는 canvas 표시 크기와 같은 배율을 유지했다.

### 5.3 console

실측 스크립트가 외부 CDN 폰트 요청을 의도적으로 차단했기 때문에 `Failed to load resource: net::ERR_FAILED` 2건이 기록됐다. 이는 CDN 폰트 차단으로 인한 것이며 이번 렌더링 변경과 무관하다.

## 6. Stage 3 결론

Stage 3 목표를 충족했다.

- 지연 재렌더가 `flow` 필터를 유지한다.
- BehindText/InFrontOfText 분리 렌더링을 깨뜨리는 unfiltered canvas 재렌더 호출을 제거했다.
- `npm run build`가 통과했다.
- 200ms/600ms 지연 재렌더 이후에도 overlay DOM 크기는 기대값을 유지한다.

## 7. 승인 요청

Stage 4 — 브라우저 최종 검증 및 최종 보고서 작성 진행 승인 요청.
