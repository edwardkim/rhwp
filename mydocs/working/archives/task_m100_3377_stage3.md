---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3377.md
last_verified: 2026-07-30
---

# Task #3377 Stage 3 보고 — 수정 구현 + red-check

## 구현 (rhwp-studio/src/view/canvas-view.ts 단일 파일)

1. `renderCanvas` 의 위치 설정 블록을 `positionPageElement(element, pageIdx)` 로 추출 —
   캔버스·오버레이 공용 단일 관문.
2. `recalcLayout()` 끝에 활성 페이지 재배치 추가:
   - 평상시: `repositionActivePages()` — `canvasPool.activePages` 의 캔버스 +
     `[data-rhwp-overlay-page]`/`[data-rhwp-grid-page]` 오버레이에 현재 좌표 재적용.
     줌 불변이므로 재렌더 없음(위치만 동기화).
   - 줌 애니메이션 중: 기존 preview 경로(`updateRenderedPageZoomPreview`) 재사용 —
     scale 동반 변환 규약 보존.

오버레이가 캔버스의 top/left/transform 을 복사하는 기존 규약(`page-renderer.ts
applyPageLayerBox`)과 정합함을 확인하고 같은 좌표를 적용한다.

## 검증

| 검증 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 결정 재현 하니스 (`task3377_repro_final.mjs`) | REPRODUCED:true, uniqueLefts ["1385px","1400px"] | **false, ["1385px"]** — 좌표계 통일 |
| both-stale 변형 (`task3377_repro_timed.mjs`) | 전 페이지가 구좌표(1400) 잔존 | **전 페이지 현행 좌표(1385=19.85+1365) 추종** |
| studio `npm test` | — | 678/678 통과 |
| `npm run build` (tsc + vite) + 확장 `build.mjs` | — | 성공 |

Rust/WASM 비접촉. 남은 검증(Stage 4): 작업지시자 실환경(Windows 확장) 첫 로딩
시각 판정 — 재빌드된 `rhwp-chrome/dist` 로드 후 SO-SUEOP.hwp 첫 화면에서 1·2쪽
정렬 확인.
