# PR #2297 최종 보고 — cross-backend manifest + profile 전달 (seo-rii, P36)

- 결정: **merge** (2026-07-16, CLEAN) — CHANGES_REQUESTED 1건(export-png
  placeholder 계약 회귀, 실증 잉크 5,776) → 2시간 내 근인 반영(기본
  HighQuality + PNG 축 가드 + CI 배선) → 재실증 후 merge.
- 게이트: default 3,230/0 · studio 295/0 · CI 전 항목 green.
- 교훈: #2225 계약의 백엔드 고정 억제가 profile 정책化될 때 진입점
  기본값이 계약 경계 — 회귀 가드가 SVG 축만 있던 공백을 이번에 봉합.
- 상세: `pr_2297_review.md` (v2 포함).
