# Task #1880 v2 Stage 2 — 실측 검증

## 2959953 개별 검증 (측정 기반: devel + PR #1924 + PR #1927 + 본 수정)

- body_area: HWPX h=895.7px / conv h=917.1px(+1600HU) → **양쪽 895.7px 일치**.
- (section,pi)→page 맵: **1,888 entries 완전 일치** (종전 PI_MOVED 5개 pi).
- dump-pages 전체 diff: 페이지 배치 차이 0 (잔여는 `used=` px 수치 표기 차이
  32줄 — 항목 수·배치 동일).

## A/B 하니스 (2,005건, tools/roundtrip_fidelity_harness.py)

| 빌드 | SAME | PI_MOVED | PAGE_DELTA | ERR |
|------|------|----------|------------|-----|
| 수정 전 (devel+#1924+#1927) | 2002 | 2 | 1 | 0 |
| **수정 후 (+v2 게이트)** | **2003** | **1** | 1 | 0 |

- 개선 +1: 2959953 (5개 pi 전부 해소). **신규 divergence 0**.
- 잔존 (본 수정과 무관, 별개 원인 — 상세 수정 전후 동일):
  - 3171755 PI_MOVED 1개 pi (s0:pi213 20→21)
  - 3235145 PAGE_DELTA (3→2) — body_area 가설 불성립 확인, 다른 클래스

산출물: 스크래치 `task1880v2/{integration,v2}.tsv`, pages/pimap 파일.
