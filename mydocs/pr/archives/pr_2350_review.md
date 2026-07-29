# PR #2350 검토 — 책갈피 추가/삭제/이름변경 히스토리 기록 (#2349, 연작 6)

- PR: https://github.com/edwardkim/rhwp/pull/2350 (lpaiu-cs) — Fixes #2349
- 충돌 0 (연작 1~5 merge 후 클린)

## 변경 본질

책갈피 다이얼로그 3동작의 직접 호출 → #2077 수식 속성 다이얼로그와 동형
snapshot 라우팅. result 캡처는 operation 밖 지역 변수로 상태 라벨 흐름
유지, 수동 refresh 제거. 범위 외(각주 삽입 서브모드·form 게이팅)는 얽힘
사유와 함께 명시 — 후속 예고.

## 로컬 재실증 (merged tree)

가드 1/1 · studio **346/346** · tsc 0 · e2e 24/0 · 충돌 0.

## 판단

**merge 권고.** 연작 6건 완결 — 원장 게이지 기준 이관 진행이 한 사이클
마무리됨. #2327 원장 baseline 잔여는 컨트리뷰터 후속 예고 참조.
