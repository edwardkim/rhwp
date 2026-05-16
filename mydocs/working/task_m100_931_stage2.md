# Task #931 Stage 2 완료 보고서

## 1. 목적

BehindText/InFrontOfText overlay DOM이 canvas 표시 크기와 같은 줌 배율을 따르도록 정정했다.

Stage 2 범위는 overlay 좌표계 정합화이며, 지연 재렌더의 `flow` filter 보존은 Stage 3 범위로 남겨둔다.

## 2. 수정 파일

- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/src/view/page-renderer.ts`

## 3. 수정 내용

### 3.1 `renderScale`, `displayScale`, `dpr` 분리

`CanvasView.renderPage()`에서 WASM 렌더링용 `renderScale = zoom × dpr`와 overlay 표시 배율인 `zoom`, 실제 `dpr`을 분리해 `PageRenderer`로 전달했다.

```ts
this.pageRenderer.renderPage(pageIdx, canvas, renderScale, zoom, dpr);
```

### 3.2 overlay layer 표시 크기 정정

기존에는 `scale = zoom × dpr`를 `dpr`처럼 사용해 `canvas.width / scale`을 계산했다. 이 계산은 원본 페이지 크기로 되돌아가므로 줌 축소 시 overlay가 canvas보다 크게 남았다.

수정 후에는 실제 `dpr`만 사용한다.

```ts
const safeDpr = dpr > 0 && Number.isFinite(dpr) ? dpr : 1;
const cssWidth = canvas.width / safeDpr;
const cssHeight = canvas.height / safeDpr;
```

### 3.3 overlay image bbox 배율 적용

PageLayerTree의 bbox는 zoom=1 페이지 좌표계이므로 `displayScale`을 곱해 DOM에 배치한다.

```ts
el.style.left = `${img.bbox.x * displayScale}px`;
el.style.top = `${img.bbox.y * displayScale}px`;
el.style.width = `${img.bbox.width * displayScale}px`;
el.style.height = `${img.bbox.height * displayScale}px`;
```

### 3.4 페이지 경계 clipping 적용

overlay layer에 `overflow: hidden`을 적용했다. 배율/좌표가 어긋나는 회귀가 생기더라도 페이지 경계 밖 회색 작업 영역으로 노출되는 위험을 줄인다.

## 4. 검증

### 4.1 빌드

```bash
cd rhwp-studio
npm run build
```

결과: 통과

Vite의 chunk size warning은 기존 번들 크기 경고이며 이번 변경과 무관하다.

### 4.2 DOM 재실측

측정 환경:

- rhwp-studio dev server: `http://127.0.0.1:7700/`
- headless Chrome
- viewport: 1600×1000
- `devicePixelRatio=1`
- sample: `samples/복학원서.hwp`
- script: `/private/tmp/rhwp-watermark-analysis/stage1-measure.mjs`

#### 25% 줌

| 대상 | Stage 1 수정 전 | Stage 2 수정 후 | 기대값 |
|------|-----------------|-----------------|--------|
| canvas CSS rect | `198 × 280` | `198 × 280` | `198.4 × 280.6` |
| behind overlay layer rect | `792 × 1120` | `198 × 280` | `198.4 × 280.6` |
| 워터마크 img rect | `495.03 × 495.72` | `123.75 × 123.92` | `123.8 × 123.9` |
| 워터마크 left/top | `137.707 / 270.24` | `34.4268 / 67.56` | `34.4 / 67.6` |
| overlay overflow | `visible` | `hidden` | `hidden` |

Stage 2 수용 기준을 충족했다.

#### 85% 줌

| 대상 | Stage 2 수정 후 | 기대값 |
|------|-----------------|--------|
| canvas CSS rect | `674 × 954` | `674.6 × 954.1` |
| behind overlay layer rect | `674 × 954` | `674.6 × 954.1` |
| 워터마크 img rect | `420.78 × 421.36` | `420.8 × 421.4` |

85%에서도 overlay가 canvas와 같은 배율을 따른다.

### 4.3 시각 확인

25% 줌 스크린샷:

`/private/tmp/rhwp-watermark-analysis/stage1_zoom_25.png`

수정 전에는 회색 작업 영역에 원본 크기 고려대 마크가 크게 노출되었으나, 수정 후에는 워터마크가 페이지 안에서 축소되어 보이고 회색 배경에는 노출되지 않는다.

## 5. 남은 항목

`scheduleReRender()`는 아직 `renderPageToCanvas()` 전체 렌더를 사용한다. Stage 2의 overlay 배율 결함은 해소했지만, 지연 재렌더가 본문 canvas의 `flow` filter를 깨뜨릴 가능성은 남아 있다.

따라서 Stage 3에서 다음을 진행한다.

- `scheduleReRender()` 인자명을 `renderScale`로 정리
- 지연 재렌더 호출을 `renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow')`로 변경
- 지연 재렌더 후에도 BehindText/InFrontOfText 분리 렌더링이 유지되는지 확인

## 6. Stage 2 결론

Stage 2 목표를 충족했다.

- overlay layer가 canvas CSS 표시 크기와 일치한다.
- BehindText 워터마크 bbox가 zoom 배율을 따른다.
- 페이지 경계 clipping으로 회색 작업 영역 노출을 차단했다.
- `npm run build`가 통과했다.

## 7. 승인 요청

Stage 3 — 지연 재렌더 layer filter 보존 구현 진행 승인 요청.
