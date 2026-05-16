# Task #931 최종 보고서

## 1. 이슈

- GitHub Issue: `#931`
- 제목: rhwp-studio 축소 시 BehindText 워터마크 overlay 줌 불일치 수정
- 대상 문서: `samples/복학원서.hwp`

## 2. 원인

`복학원서.hwp`에는 실제 HWP 객체로 BehindText 이미지 2개가 포함되어 있다.

- 좌상단 로고 이미지: `x=65.493, y=49.013, w=77.013, h=87.893`
- 중앙 워터마크 이미지: `x=137.707, y=270.24, w=495.04, h=495.733`

결함은 워터마크를 제거해야 하는 문제가 아니라, rhwp-studio의 DOM overlay가 canvas 표시 배율과 다른 좌표계로 배치되는 문제였다.

기존 구현은 canvas가 25%로 줄어도 overlay layer와 `<img>` bbox가 원본 페이지 좌표계 크기에 가깝게 남아 회색 작업 영역 뒤쪽에 큰 워터마크처럼 보였다. 또한 이미지 로딩 보정용 지연 재렌더가 필터 없는 canvas 렌더를 호출해 BehindText/InFrontOfText 분리 렌더링을 다시 깨뜨릴 위험이 있었다.

## 3. 수정 내용

### 3.1 overlay 표시 배율 정합화

수정 파일:

- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/src/view/page-renderer.ts`

핵심 변경:

- `renderScale = zoom × dpr`와 `displayScale = zoom`을 분리했다.
- `PageRenderer.renderPage()`에 `renderScale`, `displayScale`, `dpr`을 명시적으로 전달한다.
- overlay layer의 CSS 표시 크기를 `canvas.width / dpr`, `canvas.height / dpr`로 계산한다.
- overlay `<img>` bbox에 `displayScale`을 적용한다.
- BehindText/InFrontOfText overlay layer에 `overflow: hidden`을 적용해 페이지 밖 노출을 막는다.

### 3.2 지연 재렌더 filter 보존

수정 파일:

- `rhwp-studio/src/view/page-renderer.ts`

핵심 변경:

- `scheduleReRender()`가 필터 없는 `renderPageToCanvas()` 대신 `renderPageToCanvasFiltered(..., 'flow')`를 호출하게 수정했다.
- 지연 재렌더 이후에도 BehindText/InFrontOfText 이미지는 canvas가 아니라 DOM overlay sibling으로만 유지된다.

### 3.3 Stage 5 시각 피드백 보정

Stage 4 후 작업지시자 시각 검증에서 워터마크가 보이지 않는 문제가 확인됐다. 원인은 DOM overlay가 없어지는 것이 아니라 flow canvas의 흰 페이지 배경 뒤에 가려지는 것이었다.

추가 수정:

- `src/renderer/web_canvas.rs`
  - BehindText가 있는 `LayerFilter::FlowOnly` 렌더에서는 flow canvas의 페이지 배경을 투명하게 유지한다.
  - `PaintOp::PageBackground`도 해당 flow 렌더에서는 skip한다.
- `rhwp-studio/src/view/page-renderer.ts`
  - `page background(z=0) → BehindText overlay(z=1) → transparent flow canvas(z=2) → InFrontOfText overlay(z=3)` 순서로 layer를 분리한다.
  - DOM 워터마크에도 기존 canvas renderer와 같은 `opacity=0.17`을 적용한다.
- `rhwp-studio/src/view/canvas-view.ts`
  - canvas release/re-render 시 sibling overlay layer를 함께 정리한다.

## 4. 검증

### 4.1 빌드

```bash
cd rhwp-studio
npm run build
```

결과: 통과

추가 Stage 5 검증:

```bash
cargo build
docker-compose --env-file .env.docker run --rm wasm
cargo test --test issue_516
```

결과:

- `cargo build`: 통과
- WASM 빌드: 통과
- `cargo test --test issue_516`: `8 passed`

### 4.2 브라우저 조작 검증

검증 URL:

```text
http://127.0.0.1:7700/?url=/samples/%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp&filename=%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp
```

조작:

- `복학원서.hwp` 자동 로드 확인
- 상태 표시줄 줌 아웃 버튼으로 100%에서 25%까지 축소
- 25% 상태에서 canvas, behind overlay layer, 워터마크 `<img>` DOM rect 측정

25% 측정값:

| 대상 | 측정값 | 기대 |
|------|--------|------|
| canvas rect | `198 × 280.5` | 페이지 25% 표시 크기 |
| behind overlay layer rect | `198 × 280.5` | canvas와 동일 |
| 워터마크 img rect | `123.7578 × 123.9297` | 원본 bbox의 25% |
| 워터마크 left/top | `34.4267 / 67.56` | 원본 좌표의 25% |
| overlay overflow | `hidden` | 페이지 밖 clipping |
| console error/warn | 없음 | 회귀 없음 |

### 4.3 반복 실측

headless Chrome 실측:

| 줌 | canvas rect | behind overlay rect | 워터마크 rect |
|----|-------------|---------------------|---------------|
| 100% | `793 × 1122` | `793 × 1122` | `495.03 × 495.72` |
| 85% | `674 × 954` | `674 × 954` | `420.78 × 421.36` |
| 25% | `198 × 280` | `198 × 280` | `123.75 × 123.92` |

결과:

- overlay layer가 모든 검증 줌에서 canvas와 같은 표시 크기를 유지했다.
- 워터마크 bbox가 zoom 비율에 맞게 줄어들었다.
- 200ms/600ms 지연 재렌더 이후에도 값이 유지됐다.

### 4.4 Stage 5 재검증

작업지시자 시각 피드백 후 재검증:

| 줌 | background layer | behind layer | canvas | watermark |
|----|------------------|--------------|--------|-----------|
| 100% | `793 × 1122`, `z=0` | `793 × 1122`, `z=1` | `793 × 1122`, `z=2`, transparent | `495.03 × 495.72`, `opacity=0.17` |
| 25% | `198 × 280`, `z=0` | `198 × 280`, `z=1` | `198 × 280`, `z=2`, transparent | `123.75 × 123.92`, `opacity=0.17` |

스크린샷:

- `/private/tmp/rhwp-watermark-analysis/stage5_zoom_100.png`
- `/private/tmp/rhwp-watermark-analysis/stage5_zoom_25.png`

## 5. 산출물

- 수행계획서: `mydocs/plans/task_m100_931.md`
- 구현계획서: `mydocs/plans/task_m100_931_impl.md`
- Stage 1 보고서: `mydocs/working/task_m100_931_stage1.md`
- Stage 2 보고서: `mydocs/working/task_m100_931_stage2.md`
- Stage 3 보고서: `mydocs/working/task_m100_931_stage3.md`
- Stage 4 보고서: `mydocs/working/task_m100_931_stage4.md`
- Stage 5 보고서: `mydocs/working/task_m100_931_stage5.md`
- 최종 보고서: `mydocs/report/task_m100_931_report.md`

## 6. 직접 검증

요청에 따라 rhwp-studio dev server를 실행한 상태로 유지했다.

```text
http://127.0.0.1:7700/?url=/samples/%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp&filename=%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp
```

## 7. 남은 위험

- high-DPR 디스플레이에서는 `dpr` 보정 경로가 사용되므로 논리상 같은 계산을 적용하지만, 이번 자동 검증은 `devicePixelRatio=1` 중심으로 수행했다.
- InFrontOfText 샘플 문서에 대한 별도 시각 검증은 수행하지 않았다. 동일 overlay 경로를 사용하므로 코드상 동일한 배율 규칙이 적용된다.
- BehindText가 있는 페이지의 custom page background는 현재 rhwp-studio DOM background layer가 흰색으로 대체한다. `복학원서.hwp` 재현 범위에서는 문제가 없지만, 비흰색/이미지 page background와 BehindText가 함께 있는 문서는 후속 정밀화 대상이다.

## 8. 완료 승인 요청

Task #931의 구현과 검증이 완료됐다. 이슈 close는 작업지시자 승인 후 별도로 수행한다.
