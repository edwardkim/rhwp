# Task M100 #1154 v2 — Stage 3 완료 보고서

## 수행

1. **WASM 재빌드**: `docker compose --env-file .env.docker run --rm wasm` — Stage 1 의 Rust 변경 (overlay JSON crop 필드) 반영. 결과 `pkg/rhwp_bg.wasm` 5,311 KB.
2. **헤드리스 캡쳐 재실행**:
   - 페이지 2 박스 18 (page.screenshot, overlay 포함): 정상 ✓
   - 페이지 4 박스 27 (canvas toDataURL, lazy load + 3 초 대기): 정상 ✓
3. **lint / test 재실행**.

## 검증 결과

### 시각

| 환경 | 페이지 2 박스 18 | 페이지 4 박스 27 |
|---|---|---|
| 한컴 PDF (권위) | ✓ | ✓ |
| SVG export (`rhwp export-svg`) | ✓ | ✓ |
| rhwp-studio (Step 1만, 8ee17fd4 적용 + WASM 재빌드) | 잔상 (stretched overlay) | 외곽 그림 누락 |
| rhwp-studio (Step 1 + Step 2 = Stage 1 적용) | **정상** | 외곽 그림 누락 |
| rhwp-studio (Step 1 + Step 2 + Step 3 = Stage 1 + Stage 2 적용) | **정상** | **정상** |

### 자동 검증

| 검증 | 결과 |
|---|---|
| cargo test --release --lib | 1432 passed / 0 failed / 6 ignored |
| cargo clippy --release --lib -- -D warnings | clean |
| npx tsc --noEmit (rhwp-studio) | clean |
| cargo fmt 적용 | 변경 파일만 (rendering.rs) |

## 변경 파일 (v2 누적)

| 파일 | 변경 내용 | LoC 변화 |
|---|---|---|
| `src/document_core/queries/rendering.rs` | `write_overlay_image` 에 crop 필드 출력 | +7 |
| `rhwp-studio/src/view/page-renderer.ts` | `OverlayImageInfo.crop`, `toOverlayInfo.crop`, `createOverlayLayer` wrapper, `scheduleReRender` delays, `prefetchFlowImages` 추가 | +120 / -13 |
| `pkg/*` | WASM 재빌드 (gitignore) | — |

## 한컴 PDF 권위 자료 비교 (시각)

- 페이지 2 박스 18 "Dear Rosydale City Marathon Racers" : 윈도우 chrome frame 단일, 하단 단일 라인, SVG / rhwp-studio 모두 PDF 와 시각 일치
- 페이지 4 박스 27 "Adenville City Pass Card" : 종이 디자인 + 핀 두 개 + 외곽선, SVG / rhwp-studio 모두 PDF 와 시각 일치

## 회귀 보호

- Stage 1 (overlay crop) — BehindText/InFrontOfText overlay 경로에만 영향. flow layer 비변경.
- Stage 2 (prefetch) — flow layer 의 추가 재렌더 + image prefetch. 이미지 없는 페이지는 skip.
- 둘 다 Rust 측 변경 1 줄 (JSON 추가) 외에는 TypeScript only. 기존 cargo test / clippy 통과 유지.

## 결론

페이지 2 박스 18 + 페이지 4 박스 27 모두 한컴 PDF 와 시각 일치 달성. v2 작업 완료.
