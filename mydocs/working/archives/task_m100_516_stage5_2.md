# Task #516 Stage 5.2 완료 보고서 — 다층 레이어 (옵션 C HTML Hybrid) 도입

## 변경 요약

| 영역 | 파일 | 변경 |
|------|------|------|
| Rust — Web Canvas wrap 필터 | `src/renderer/web_canvas.rs` | `LayerFilter` enum + `WebCanvasRenderer.layer_filter` + `should_render_image` 헬퍼 + PaintOp::Image dispatcher 분기 |
| Rust — wasm API | `src/wasm_api.rs` | `renderPageToCanvasFiltered(page, canvas, scale, layer_kind)` 신규 메서드 (`"all"`/`"flow"`/`"behind"`/`"front"`) |
| Rust — PageLayerTree JSON 의 mime + 변환 | `src/paint/json.rs` | PaintOp::Image 직렬화에 `mime` 필드 추가 + PCX/BMP → PNG 변환 적용 (overlay `<img>` data URL 호환) |
| Rust — PageControlLayout wrap 노출 | `src/document_core/queries/rendering.rs` | image control 직렬화에 `wrap` 필드 추가 (옵션 3-C 의 hit-test 분기용) |
| TS — wasm-bridge | `rhwp-studio/src/core/wasm-bridge.ts` | `renderPageToCanvasFiltered` + `getPageLayerTree` 메서드 |
| TS — page-renderer | `rhwp-studio/src/view/page-renderer.ts` | `renderPage` 가 'flow' 필터로 본문 Canvas 렌더 + `applyOverlays` (BehindText/InFrontOfText 그림 `<img>` overlay) + `getOverlayImages` + `createOverlayLayer` (CSS filter + mix-blend-mode multiply) + `OverlayImageInfo` 타입 + `collectOverlayImages` (트리 순회) |
| TS — input-handler-picture (옵션 3-C) | `rhwp-studio/src/engine/input-handler-picture.ts` | `findPictureAtClick` 의 두 단계 hit-test (1차 비-BehindText 우선 / 2차 BehindText 는 텍스트 hit 없을 때만) |

## 다층 레이어 동작 흐름 (도입 후)

```
canvas-view.renderPage()
  → page-renderer.renderPage(canvas, scale)
       ├── wasm.renderPageToCanvasFiltered(canvas, scale, "flow")  ← 본문 layer
       │   (BehindText / InFrontOfText 그림 제외)
       ├── drawMarginGuides()
       └── applyOverlays(canvas, scale)
            ├── getPageLayerTree(pageIdx) → JSON parse
            ├── collectOverlayImages(root, behind, front)
            │   (트리 재귀 순회 — wrapper.root 진입 후 ops/children/child)
            ├── BehindText overlay <div> (z-index 0, pointer-events:none)
            │   └── <img> (CSS filter + mix-blend-mode multiply)
            └── InFrontOfText overlay <div> (z-index 2, pointer-events:none)
```

## 결함별 처리 결과

| 결함 (Stage 4 시각 판정 결과) | 본 Stage 5.2 처리 |
|------------------------------|-------------------|
| 1. 엠블럼 흰색 배경 투명 | ✅ multiply blend 로 자동 투명 효과 |
| 2. BehindText 위 텍스트 클릭 | ✅ pointer-events:none + 옵션 3-C 두 단계 hit-test |
| 3. 워터마크 효과 (multiply blend) | ⚠️ multiply 까지만 적용. 회색조/투명도 정합 시각은 분리 task ([#535](https://github.com/edwardkim/rhwp/issues/535)) |

## 작업지시자 시각 판정 결과 (이번 사이클)

| 검증 항목 | 결과 |
|----------|------|
| 학교 로고 (PCX → PNG) 출력 | ✅ 정상 |
| 엠블럼 흐릿 처리 | ⚠️ 회색조까지만, 한컴 시각과 차이 — 작업지시자 결정으로 분리 task |
| hit-test 옵션 3-C (텍스트 우선) | ✅ 정상 |
| 부수 — 엠블럼 객체 선택 + 드래그 | ✅ 정상 (보존, 작업지시자 "이건 좋은 듯") |

## 검증 게이트

| 게이트 | 결과 |
|--------|------|
| `cargo build --lib` | ✅ Finished |
| `cargo test --lib` | ✅ **1110 passed** (회귀 0) |
| `cargo test --test issue_516` | ✅ **8 passed** (헬퍼 3 + JSON watermark 2 + JSON wrap 1 + 진단 2) |
| `cargo test --test issue_418/501/514` | ✅ 1 + 1 + 3 (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** |
| `npx tsc --noEmit` (rhwp-studio) | ✅ 0 errors |
| WASM 빌드 | ✅ ~4.45 MB |
| **작업지시자 시각 판정** (1, 3) | ✅ 통과 |

## 다층 레이어 핵심 결정 — Discussion #529

본 task 의 다층 레이어 도입은 후보 C (HTML Hybrid) 적용. 단계적 마이그레이션 경로 보존:

- **M100 (현재)**: 후보 C (HTML overlay + 단일 본문 Canvas)
- **M150**: 후보 C + OffscreenCanvas + WebWorker (메인 thread 부하 분산)
- **M200**: 후보 B (WebGPU) — DTP 정체성 본격화 (Discussion #529 Appendix A)

## 분리 task

본 task 의 결함 2 (워터마크 시각) 는 별도 처리:

- **이슈 [#535](https://github.com/edwardkim/rhwp/issues/535)** — 워터마크 효과 회색조/투명도 시각 정합
- 작업지시자 정합: **rhwp 자체 시각 해석** (한컴 출력은 권위 미입증, `feedback_pdf_not_authoritative`)
- 권장: D-1 (multiply + opacity 강제 + IR b/c 보존)

## 다음 단계

본 task 의 최종 보고서 작성 + merge + push + 이슈 #516 close.
