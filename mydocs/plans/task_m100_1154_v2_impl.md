# Task M100 #1154 v2 — 구현 계획서

## 단계 구성

3 단계 (Stage 1 — backend + overlay frontend, Stage 2 — image prefetch frontend, Stage 3 — 검증 + WASM 재빌드).

### Stage 1 — overlay path 가 crop 을 honor 하도록 수정

**파일:**
- `src/document_core/queries/rendering.rs`
  - `write_overlay_image` 에서 `image.crop` 이 `Some` 일 때 JSON 에 `"crop":{"left":..,"top":..,"right":..,"bottom":..}` 출력 (layer tree 메인 JSON 직렬화 구조와 동일).
- `rhwp-studio/src/view/page-renderer.ts`
  - `OverlayImageInfo` 인터페이스에 `crop?: { left; top; right; bottom }` 필드 추가.
  - 폴백 경로 `toOverlayInfo` 도 `op.crop` 을 그대로 전달.
  - `createOverlayLayer` 본체에서 crop 있으면 wrapper div + overflow:hidden 패턴 사용.
    - source rect (px) = (crop.left / 75, crop.top / 75, span / 75, span / 75)  — HU/px = 75 가정 (`compute_image_crop_src` 와 동일).
    - dest rect (px) = bbox * displayScale.
    - scaleX = dw / sw, scaleY = dh / sh.
    - wrapper: position absolute (dx, dy), size (dw, dh), overflow:hidden, pointer-events:none.
    - 내부 `<img>`: position absolute (-sx*scaleX, -sy*scaleY), width = naturalWidth*scaleX, height = naturalHeight*scaleY (onload 시점 확정).
  - 기존 filter/mixBlendMode/opacity 처리는 그대로 `<img>` 에 적용.

**완료 조건:** 페이지 2 박스 18 캡쳐에서 윈도우 frame 단일 정상 표시.

### Stage 2 — 큰 이미지 비동기 디코드 대응

**파일:** `rhwp-studio/src/view/page-renderer.ts`

- `scheduleReRender` delays 배열 확장: `[200, 600]` → `[200, 600, 1500]`.
- 신규 `prefetchFlowImages` 메서드:
  - `wasm.getPageLayerTree(pageIdx)` JSON 에서 image 항목의 mime + base64 추출 (정규식 사용, behindText/inFrontOfText 제외 — overlay 별도 처리).
  - 각 image 를 `new Image()` + `image.decode()` 으로 prefetch — 디코드 완료 시 resolve. 미지원 브라우저 폴백으로 `onload` 도 등록.
  - 모든 image prefetch 완료 후 `wasm.renderPageToCanvasFiltered(pageIdx, canvas, scale, 'flow')` 한 번 더 호출.
- `scheduleReRender` 안에서 `queueMicrotask` 로 `prefetchFlowImages` 실행 → setTimeout 흐름과 독립적으로 안전망 동작.

**완료 조건:** 페이지 4 박스 27 캡쳐에서 박스 외곽 그림 (종이 + 핀 디자인) 표시.

### Stage 3 — WASM 재빌드 + 회귀 검증

**작업:**
1. `docker compose --env-file .env.docker run --rm wasm` 으로 pkg/ 재빌드.
2. headless 캡쳐 재실행:
   - 페이지 2 박스 18 (page.screenshot + overlay 포함) 정상.
   - 페이지 4 박스 27 (canvas toDataURL, lazy load) 정상.
3. cargo test --release --lib (1432 passed 유지).
4. cargo clippy --release --lib -- -D warnings (clean 유지).
5. npx tsc --noEmit (clean).

**완료 조건:** 모든 검증 통과.

## 회귀 보호 전략

- Stage 1 변경은 BehindText/InFrontOfText overlay 경로에만 영향. flow layer (canvas) 변경 없음. 기존 페이지 2 SVG export 결과 동일 (이미 검증).
- Stage 2 변경은 flow layer 의 재렌더 횟수 + prefetch 만 추가. 이미지 없는 페이지는 imageCount=0 으로 skip — 비용 없음.
- 두 변경 모두 추가 호출 / DOM 노드 한정 — 기존 동작에 부정적 영향 가능성 낮음.

## 위험

| 항목 | 완화 |
|---|---|
| naturalWidth/Height onload 콜백 도착 전 임시 시각 어색 | scheduleReRender 가 디코드 완료 후 다시 그리는 안전망 (`prefetchFlowImages`) 가 커버 |
| crop 좌표 환산 오류 (HU/px 비율) | compute_image_crop_src 동일 상수(75) 사용 — 백엔드와 일관 |
| prefetch JSON 정규식 매칭 오류 | wrap 필드만 필터링 (단순 패턴), 매칭 실패 시 단순 skip (앱 동작에 영향 없음) |
