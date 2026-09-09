---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3377.md
last_verified: 2026-07-30
---

# Task #3377 Stage 2 보고 — 근인 확정 + 결정 재현

## 근인 (실환경 실측으로 확정)

작업지시자가 어긋난 상태에서 실행한 판독 스니펫 결과:

```json
p1: { x:464, w:794, left:"1688px", rz:"1" }   ← stale
p2: { x:449, w:794, left:"1673px", rz:"1" }   ← 현행 레이아웃
contW:1653, contentW:"4139.7px"
```

- 줌(rz)·폭(w) 동일, transform 없음 → 줌/preview 가설 기각. **style.left만 15px 어긋남.**
- 수식 검증: `applyHorizontalPanSpace` 는 `pageLeft = (baseWidth−pageWidth)/2 + viewportWidth`,
  `totalWidth = baseWidth + 2×viewportWidth` (virtual-scroll.ts). 실측 역산 —
  baseWidth = 4139.7 − 2×1653 = 833.7 → 현행 pageLeft = 19.85 + 1653 = **1672.85 = p2** ✓,
  p1 의 1688 에서 역산한 렌더 당시 viewportWidth = **1668** → 차이 15px = **세로 스크롤바 폭**.

**인과 사슬**: 첫 로딩(스크롤바 없음, clientWidth 1668)에 1쪽 렌더 → scrollContent 높이
설정으로 스크롤바 등장 → clientWidth 1653 → ResizeObserver → `onViewportResize` →
`recalcLayout` 이 pageLefts 갱신 — 그러나 **비그리드 경로는 기렌더 캔버스를 재배치하지
않음**(`canvas-view.ts:566~574`, 위치는 `renderCanvas:420~431` 에서만 설정) → 이후 렌더되는
페이지부터 새 좌표 → 신·구 좌표계 공존 = 어긋남 고착. 42쪽 왕복 시 해소되는 것은
release→재렌더가 위치를 다시 쓰기 때문.

## headless 미재현의 이유

Linux headless 는 오버레이 스크롤바라 clientWidth 가 변하지 않아 트리거 자체가 없다.
이슈의 "dev headless 미재현" 관측도 같은 이유로 설명된다.

## 결정 재현 하니스 (red-check 용 확보)

`task3377_repro_final.mjs`: 첫 렌더 완료 후 뷰포트 15px 축소(스크롤바 등장 모사) →
미렌더 페이지 렌더 유도 → **REPRODUCED: true, uniqueLefts ["1385px","1400px"]** —
기렌더/신규렌더 페이지의 좌표 분열을 결정적으로 재현. 수정 후 이 하니스가
uniqueLefts 1종이 되어야 한다.

## Stage 3 수정 설계 (승인 대상)

**recalcLayout 을 단일 관문으로, 활성 페이지 위치 재적용을 추가한다.**

1. `renderCanvas` 의 위치 설정(420~431)을 `positionPageElement(element, pageIdx)` 로 추출.
2. `recalcLayout()` 끝에서 `canvasPool.activePages` 전체 + 페이지 오버레이
   (`[data-rhwp-overlay-page]`, grid overlay)에 위치 재적용 — 줌 불변이므로 재렌더
   불필요(위치만 동기화, transform/preview 상태는 기존 규약 유지).
3. 회귀 가드: 재현 하니스를 검증 절차에 편입(수정 전 REPRODUCED:true → 수정 후 false
   red-check). 단위 테스트로는 virtual-scroll 좌표 계약을 고정.

영향 범위: rhwp-studio 렌더 뷰 한정, Rust/WASM 비접촉. 위험: 줌 애니메이션 preview 중
recalcLayout 호출 경로와의 상호작용 — preview transform 을 덮지 않도록 재적용 시
애니메이션 중이면 preview 경로(applyZoomPreviewBox)를 재사용한다.
