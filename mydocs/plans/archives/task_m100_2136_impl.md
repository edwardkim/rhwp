# 구현 계획서 — Task M100 #2136

**수행계획서**: `task_m100_2136.md` / **브랜치**: `local/task2136`

## 1. Stage 1 조사 확정

136건 서브버킷: PARA_FLOW+LS 76(56%) / TABLE_ANCHOR+LS 42 / TITLE_BEFORE_TABLE 12 / NOLS 9.
PARA_FLOW+LS 경계 빈-문단 스캔: **PUSH+TEXT 39(최대 실차이 버킷)** / PULL+TEXT 16 /
PULL+EMPTY 21(캐럿 의미차 오탐성 — #1920 역방향, 오라클 개선 대상으로 분리).

### PUSH+TEXT 대표 분해 (148753276 pi46)

- 직전 pi45(TAC 그림 줄) 저장 하단 = 29080+39171 = 68251HU(>60000).
- pi46 저장 **vpos=2500 = 문단 sb(5000유닛=2500HU)와 정확 일치** — 저장 흐름이 "새 쪽
  상단"을 인코딩. 한글 p5 배치.
- rhwp 는 측정 fit(870+53.3 ≤ 929.6)으로 p4 말미에 과적(used 942 > body 933.6,
  hwp_used 와 +876.7px 괴리).
- 기존 가드 `native_near_top_reset`(typeset.rs:2866)이 정확히 이 형상용인데 상한
  `cv <= 2000` 에서 cv=2500 이 500HU 차로 배제됨.

## 2. 수정

`native_near_top_reset` 의 `cv <= 2000` → `cv <= 2500`. (sb 일치 ±150 조건이 유지되어
오발동 억제; #1750 의 split-precheck 상한 2500 과 정합.)

## 3. 검증

1. 대표 2건(148753276/1490000) 오라클 MATCH 전환.
2. PUSH+TEXT 39건 목록 재오라클 — FIXED 계수.
3. 광역: 359문서 재검 REGRESSED 0, byeolpyo4/1, 표 계열 테스트, cargo test.
4. 잔여(PULL+TEXT/TABLE_ANCHOR/NOLS/캐럿 오탐)는 최종보고서에서 재분류 인계.
