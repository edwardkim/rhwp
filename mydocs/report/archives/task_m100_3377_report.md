---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_3377.md
last_verified: 2026-07-30
---

# Task #3377 최종 보고 — 확장 첫 로딩 1·2쪽 세로축 어긋남 해소

- Issue: [#3377](https://github.com/edwardkim/rhwp/issues/3377) (M100) / 브랜치 `local/task3377`
- 기간: 2026-07-30 (수행계획 → Stage 1~3 → 실기동 판정, 당일 완결)
- 단계 기록: `mydocs/working/task_m100_3377_stage{1,2,3}.md`

## 근인

첫 로딩 중 콘텐츠 높이 설정으로 세로 스크롤바가 등장하면 컨테이너 clientWidth 가
15px 줄고, `recalcLayout` 이 페이지 좌표계(`applyHorizontalPanSpace` 의
`pageLeft = (baseWidth−pageWidth)/2 + viewportWidth`)를 갱신한다. 그러나 페이지
위치는 `renderCanvas` 안에서만 설정되어 **이미 렌더된 페이지는 구좌표에 잔존** —
이후 렌더되는 페이지와 신·구 좌표계가 공존하며 15px 어긋남이 고착됐다. 42쪽 왕복
시 해소되는 것은 release→재렌더가 위치를 다시 쓰기 때문.

- 실환경 실측(작업지시자 스니펫): p1 left=1688(당시 뷰포트 1668) vs p2 left=1673
  (현행 1653) — 차 15px = 스크롤바 폭, 수식 역산 전항 일치.
- dev/headless 미재현 이유: 오버레이 스크롤바라 clientWidth 불변 — 트리거 부재.

## 수정

`rhwp-studio/src/view/canvas-view.ts` 단일 파일:

1. 위치 설정을 `positionPageElement()` 단일 관문으로 추출(캔버스·오버레이 공용,
   `applyPageLayerBox` 복사 규약과 정합).
2. `recalcLayout()` 끝에서 `repositionActivePages()` — 활성 캔버스 +
   `[data-rhwp-overlay-page]`/`[data-rhwp-grid-page]` 오버레이에 현재 좌표 재적용.
   줌 불변이므로 재렌더 없음. 줌 애니메이션 중에는 preview 경로
   (`updateRenderedPageZoomPreview`) 재사용으로 변환 규약 보존.

## 검증 (이슈 명시 기준 전항 충족)

| 기준 | 결과 |
| --- | --- |
| 결정 재현 하니스 red-check | 수정 전 REPRODUCED:true(uniqueLefts 2종) → 수정 후 false(1종) |
| both-stale 변형(전체 15px 어긋남) | 수정 후 전 페이지 현행 좌표 추종(산술 정답 일치) |
| dev·확장 headless 회귀 | 기존 정렬 조건 6종 매트릭스 정렬 유지 |
| studio 테스트 / 빌드 | 678/678 · tsc + vite + 확장 build.mjs 성공 |
| **작업지시자 실기동 시각 판정** | **통과** (2026-07-30, Windows 확장 첫 로딩 1·2쪽 정렬) |

## 남긴 것

- 재현·판독 도구: 첫 로딩 시계열/조건 매트릭스/결정 재현 하니스 3종(세션 scratchpad,
  필요 시 e2e 편입 후보) + 어긋남 상태 판독 콘솔 스니펫(stage1 보고서).
- 구조 교훈: 레이아웃 상태를 렌더 시점에 복사해 두는 요소는 재계산의 단일 관문에서
  동기화를 보장해야 한다 — `recalcLayout` 이 그 관문이 됐다.
