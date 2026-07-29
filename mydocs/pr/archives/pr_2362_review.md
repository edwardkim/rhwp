# PR #2362 검토 — 레이아웃 다이얼로그 히스토리 기록 (#2361, Track 1)

- PR: https://github.com/edwardkim/rhwp/pull/2362 (lpaiu-cs) — Fixes #2361
- 도전 과제 #2359(다이얼로그 services 주입)의 레이아웃 클러스터 실현 —
  컨트리뷰터 자체 로드맵 #2369 Track 1

## 변경 본질

편집 용지·구역·다단·쪽 테두리 4개 다이얼로그에 `services?` 주입 →
#2077/책갈피와 동형 snapshot 라우팅. **미주입 구 호출부는 직접 적용+emit
fallback 유지** — 옵셔널 주입의 비파괴 설계. 호출부 5곳 전달, 소스 가드 4.

## 로컬 재실증 (merged tree)

가드 5/5 · studio **351/351** · tsc 0 · e2e undo-contracts 24/0 · 충돌 0.
브라우저 왕복(다이얼로그 실구동 → 라우터 도달 end-to-end)은 컨트리뷰터 실측.

## 판단

**merge 권고.** merge 후 #2359 는 레이아웃 클러스터 해소 — close 여부는
#2369 로드맵과의 관계 정리 후 작업지시자 판단.
