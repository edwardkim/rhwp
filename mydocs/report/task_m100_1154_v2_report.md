# Task M100 #1154 v2 — 최종 결과 보고서

## 개요

commit `8ee17fd4` (v1, "fix(render): 동일 bin_id Pic 컨트롤 잔상 제거") 가 머지된 후, rhwp-studio 화면에서 잔상이 여전히 보이고 추가로 페이지 4 박스 외곽 그림이 누락된다는 작업지시자 보고에 따라 추가 조사 + 두 가지 후속 fix 를 수행.

원인 1 (동일 bin_id Pic clip) 은 v1 에서 처리 완료. v2 는 **원인 2 (overlay `<img>` 가 crop 무시) + 원인 3 (큰 이미지 비동기 디코드 미커버)** 처리.

## 원인 정리 (#1154 의 시각 증상은 세 가지 독립 원인 동시 발생)

| # | 원인 | 노출 경로 | 처리 |
|---|---|---|---|
| 1 | 동일 bin_id Pic 컨트롤 수직 인접 시 세로 스케일 미스매치 → SVG 리샘플링 잔상 | SVG / Canvas / CanvasKit 공통 | v1 (commit 8ee17fd4) PageRenderTree::clip_overlapping_same_bin_images |
| 2 | rhwp-studio 의 BehindText/InFrontOfText overlay `<img>` 가 `crop` 필드를 완전히 무시 → 원본 전체 이미지가 bbox 안에 stretch | rhwp-studio overlay only | **v2 Stage 1** — overlay JSON 에 crop 필드 출력 + `createOverlayLayer` wrapper div + overflow:hidden + scaled img 패턴 |
| 3 | 큰 PNG/JPEG 의 비동기 디코드 (1 초 이상) 가 `scheduleReRender` (200/600ms) 시점을 초과 → 첫 렌더에 이미지 누락 | rhwp-studio flow layer only | **v2 Stage 2** — `scheduleReRender` delays 확장 + `prefetchFlowImages` 안전망 (image.decode() prefetch 후 강제 재렌더) |

## 변경 파일

| 파일 | 변경 내용 |
|---|---|
| `src/document_core/queries/rendering.rs` | `get_page_overlay_images_native` 의 `write_overlay_image` 가 `image.crop` 이 Some 일 때 JSON 에 `"crop":{"left":..,"top":..,"right":..,"bottom":..}` 출력 |
| `rhwp-studio/src/view/page-renderer.ts` | `OverlayImageInfo.crop` 필드 추가 / `toOverlayInfo` (폴백) crop 전달 / `createOverlayLayer` 가 crop 있을 때 wrapper div + overflow:hidden 으로 source rect 매핑 / `scheduleReRender` delays `[200, 600] → [200, 600, 1500]` 확장 / `prefetchFlowImages` 신규 메서드 |
| `pkg/*` | WASM 재빌드 (Stage 3) — gitignore |
| `mydocs/plans/task_m100_1154_v2.md` | 수행 계획서 |
| `mydocs/plans/task_m100_1154_v2_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_1154_v2_stage1.md` / `stage2.md` / `stage3.md` | 단계별 완료 보고서 |
| `mydocs/report/task_m100_1154_v2_report.md` | 본 보고서 |

## 검증 결과

### 시각 (한컴 PDF 권위 자료 기준)

| 페이지 / 박스 | 한컴 PDF | SVG export | rhwp-studio (v2 적용) |
|---|---|---|---|
| 페이지 2 박스 18 — 윈도우 chrome frame | ✓ | ✓ | ✓ |
| 페이지 4 박스 27 — 종이 + 핀 디자인 | ✓ | ✓ | ✓ |
| 페이지 4 박스 28 — Luckwood Snow Festival | ✓ | ✓ | ✓ |

### 자동 검증

| 항목 | 결과 |
|---|---|
| cargo test --release --lib | 1432 passed / 0 failed / 6 ignored |
| cargo clippy --release --lib -- -D warnings | clean |
| npx tsc --noEmit (rhwp-studio) | clean |
| cargo fmt | 변경 파일 rendering.rs 적용 |
| Docker WASM 빌드 | 정상 (53s + 1m38s wasm-opt) |

## 회귀 보호

- v2 Stage 1 변경은 **BehindText/InFrontOfText overlay 경로에만 영향**. flow layer (canvas) 변경 없음. crop 없는 overlay 이미지는 기존 동작 그대로 유지.
- v2 Stage 2 변경은 **flow layer 의 추가 재렌더 + image prefetch 만 추가**. 이미지가 없는 페이지는 `imageCount=0` 으로 skip 되어 비용 없음. prefetch 중복 디코드는 브라우저 캐시로 무시 가능.
- 둘 다 Rust 측 변경은 JSON 출력 1 줄 추가 외에는 TypeScript only. 기존 cargo test / clippy 통과 유지.

## 작업지시자 검증 요청

다음 환경에서 시각 확인 부탁드립니다:

1. `samples/exam_eng.hwp` 2 페이지 — 박스 18 윈도우 chrome frame 정상 (잔상 없음)
2. `samples/exam_eng.hwp` 4 페이지 — 박스 27 / 28 종이 + 핀 디자인 정상
3. 다른 BehindText overlay 가 있는 문서에서 회귀 없음 (예: 워터마크 페이지)

## 결론

#1154 시각 증상의 세 가지 독립 원인을 모두 처리. v1 (commit 8ee17fd4) + v2 (본 작업) 통합으로 한컴 PDF 권위 자료와 시각 일치 달성.
