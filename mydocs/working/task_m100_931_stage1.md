# Task #931 Stage 1 완료 보고서

## 1. 목적

`samples/복학원서.hwp`에서 줌 축소 시 BehindText 워터마크 overlay가 페이지 캔버스와 같은 배율로 축소되는지 실측했다.

## 2. 실행 환경

- 브랜치: `local/task931`
- 앱: rhwp-studio Vite dev server (`http://127.0.0.1:7700/`)
- 브라우저: headless Chrome
- 뷰포트: 1600×1000, `devicePixelRatio=1`
- 샘플: `samples/복학원서.hwp`
- 측정 스크립트: `/private/tmp/rhwp-watermark-analysis/stage1-measure.mjs`

외부 CDN 폰트 요청은 실측 스크립트에서 차단했다. 이로 인해 console에 `Failed to load resource: net::ERR_FAILED` 2건이 기록되었으나, 이는 테스트 환경에서 의도적으로 차단한 CDN 폰트 요청이며 overlay 배율 실측과 무관하다.

## 3. PageLayerTree 확인

문서 1페이지의 `behindText` 이미지 2개를 확인했다.

| 항목 | wrap | bbox | 효과 |
|------|------|------|------|
| 학교 로고 | `behindText` | `x=65.493, y=49.013, w=77.013, h=87.893` | `realPic` |
| 워터마크 | `behindText` | `x=137.707, y=270.24, w=495.04, h=495.733` | `grayScale`, brightness `-50`, contrast `70`, watermark `custom` |

따라서 워터마크 자체는 문서 내용이다. 제거 대상이 아니라 페이지 좌표계 안에서 줌 배율을 따라야 한다.

## 4. DOM 실측 결과

### 100% 줌

| 대상 | 측정값 |
|------|--------|
| canvas CSS rect | `793 × 1122` |
| behind overlay layer rect | `793 × 1122` |
| 워터마크 img rect | `495.03 × 495.72` |

100%에서는 canvas와 overlay layer 크기가 같으므로 결함이 잘 드러나지 않는다.

### 85% 줌

| 대상 | 측정값 | 기대값 |
|------|--------|--------|
| canvas CSS rect | `674 × 954` | `793.7 × 0.85 ≈ 674.6` |
| behind overlay layer rect | `792.94 × 1122.34` | `674.6 × 954.1` |
| 워터마크 img rect | `495.03 × 495.72` | `495.04 × 0.85 ≈ 420.8` |

85%에서도 overlay layer와 워터마크 이미지가 원본 크기 수준으로 남는다.

### 25% 줌

| 대상 | 측정값 | 기대값 |
|------|--------|--------|
| canvas CSS rect | `198 × 280` | `793.7 × 0.25 ≈ 198.4` |
| behind overlay layer rect | `792 × 1120` | `198.4 × 280.6` |
| 워터마크 img rect | `495.03 × 495.72` | `495.04 × 0.25 ≈ 123.8` |
| 워터마크 img left/top | `137.707 / 270.24` | `34.4 / 67.6` |

결함이 확정됐다. canvas는 25% 배율로 줄지만 overlay layer와 워터마크 image bbox는 원본 페이지 좌표계를 그대로 사용한다.

## 5. 시각 증거

25% 줌 스크린샷:

`/private/tmp/rhwp-watermark-analysis/stage1_zoom_25.png`

시각적으로도 작은 페이지 아래 회색 작업 영역에 원본 크기 고려대 마크가 크게 노출된다.

## 6. 원인 확정

### 원인 1 — overlay 배율 계산 오류

`rhwp-studio/src/view/page-renderer.ts`의 `applyOverlays()`에서 `scale = zoom × dpr` 값을 `dpr`처럼 사용한다.

```ts
const dpr = scale;
const cssWidth = canvas.width / dpr;
```

현재 canvas width는 `pageWidth × zoom × dpr`이므로 위 계산은 `pageWidth`로 돌아간다. 그 결과 overlay layer는 현재 줌의 표시 크기가 아니라 원본 페이지 크기로 배치된다.

또한 `createOverlayLayer()`는 `img.bbox.x/y/width/height`에 zoom을 적용하지 않는다.

### 원인 2 — overlay clipping 부재

behind overlay layer의 computed `overflow`가 `visible`이다. 배율이 틀어진 overlay가 페이지 밖 회색 작업 영역으로 그대로 노출된다.

### 원인 3 — 지연 재렌더 filter 훼손 가능성

코드 확인상 `scheduleReRender()`는 지연 재렌더에서 `renderPageToCanvas()` 전체 렌더를 호출한다. Stage 1의 주된 시각 결함은 overlay DOM 배율 문제로 확정됐지만, 이 경로는 BehindText/InFrontOfText 분리 렌더링을 깨뜨릴 수 있으므로 Stage 3에서 함께 정정해야 한다.

## 7. Stage 2 진행 판단

구현계획서의 Stage 2 방향을 유지한다.

1. `PageRenderer.renderPage()`에 `displayScale(zoom)`과 실제 `dpr`을 분리 전달한다.
2. overlay layer 크기는 `canvas.width / dpr`, `canvas.height / dpr`로 맞춘다.
3. overlay image bbox는 `displayScale`을 곱해 배치한다.
4. overlay layer에 `overflow: hidden`을 적용한다.

## 8. Stage 1 결론

Stage 1 목표를 충족했다.

- `복학원서.hwp` 워터마크가 실제 문서의 `behindText` 이미지임을 확인했다.
- 25% 줌에서 canvas는 정상 축소되지만 overlay layer와 워터마크 bbox는 원본 크기로 남는 것을 수치로 확인했다.
- Stage 2 구현 방향은 "이미지 제거"가 아니라 "overlay 좌표계의 줌 정합화"가 맞다.

## 9. 승인 요청

Stage 2 — overlay 줌 정합화 구현 진행 승인 요청.
