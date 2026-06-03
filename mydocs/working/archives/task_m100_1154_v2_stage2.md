# Task M100 #1154 v2 — Stage 2 완료 보고서

## 구현 내용

### scheduleReRender delays 확장 + prefetchFlowImages 안전망

`rhwp-studio/src/view/page-renderer.ts`:

#### 1. delays 배열 확장

```typescript
const delays = [200, 600, 1500];   // 기존 [200, 600]
```

큰 PNG/JPEG (수십~수백 KB) 의 디코드가 1 초 이상 걸리는 경우 커버.

#### 2. prefetchFlowImages 신규 메서드

```typescript
private async prefetchFlowImages(pageIdx: number): Promise<void> {
  // wasm.getPageLayerTree(pageIdx) JSON 에서 image 항목 추출
  // mime + base64 만 사용 (정규식, behindText/inFrontOfText 제외)
  // 각 image 마다 new Image() + image.decode() 으로 prefetch
  // 모든 image 디코드 완료 후 resolve
}
```

`scheduleReRender` 안에서 `queueMicrotask(() => this.prefetchFlowImages(...).then(() => 재렌더))` 으로 setTimeout 흐름과 독립 실행. 디코드 완료 시 한 번 더 강제 `renderPageToCanvasFiltered` 호출.

decode() 미지원 브라우저 (구형) 대응으로 `onload` / `onerror` 도 등록 (둘 다 resolve).

## 검증

| 검증 | 결과 |
|---|---|
| 페이지 4 박스 27 시각 (canvas toDataURL, lazy load) | 종이 디자인 + 핀 두 개 + 외곽선 모두 정상 표시 ✓ |
| 직접 렌더 (renderPageToCanvas) 두 번 호출 + 대기 패턴 | 첫 호출 누락 → 두 번째 호출 정상 (디코드 완료) ✓ — 안전망 동작 원리 확인 |
| 페이지 4 layer JSON image bbox | image #3 = (597.1, 268.1, 403.7, 432.3), bbox 변경 없음 (clip 적용 흔적 없음 — 페이지 4 와 commit 8ee17fd4 의 PageRenderTree clip 은 무관) |
| 페이지 2 박스 18 시각 (Stage 1 회귀 검증) | 정상 유지 ✓ |
| 다른 페이지 (1, 3, 5~8) lazy load 시각 | 회귀 없음 (단순 확인, BehindText overlay 없는 페이지는 prefetch 도 동작 없음) |
| cargo test --release --lib | 1432 passed / 0 failed |
| cargo clippy --release --lib -- -D warnings | clean |
| npx tsc --noEmit | clean |

## 부수 효과

- `scheduleReRender` 호출 횟수 증가: 페이지당 최대 3 번 (기존 2 번) + prefetch 콜백 1 번. 이미지 없는 페이지는 `imageCount=0` 으로 skip 되어 비용 없음.
- prefetch 중복 디코드 비용은 무시 가능 (브라우저 캐시).

## 다음

Stage 3 — WASM 재빌드 + 통합 회귀 검증.
