# Phase 3 보고 — #1658 법령 표 행분할 1유닛 낭비 페이지 수정 (LANDED, 무회귀)

- 브랜치: local/task1658 / 일자: 2026-06-29 / 결과: **소스 수정 landing 성공(첫 실질 개선).**

## 1. 수정 내용
`src/renderer/layout/table_layout.rs` `advance_row_cut` (+11줄):
- 거대 셀이 페이지를 가로질러 분할될 때, 직전 fragment 가 vpos reset(hard_break_before)으로 끝나고
  바로 다음 유닛(j==start+1)도 reset 이며 잔여공간이 충분하면(>32px), 그 reset 을 페이지 break 로
  honor 하면 **단 1유닛만 담은 낭비 페이지**가 생긴다(연속 reset). 이 경우 break 를 건너뛰고 흡수.
- `single_unit_waste` 가드 추가. #1488(가시 문단 사이 reset 을 충분 유닛 소비 후 보존)은
  j>start+1 이라 영향 없음.

## 2. 근인 (Phase B RCA 연계)
- 법령 별표 거대 셀(행14=2557.9px 등)이 페이지 분할 시, cell content vpos reset 이 연속(예: unit 35,36)
  되면 `advance_row_cut` 가 1유닛 fragment 를 만들어 낭비 페이지 발생 → over-pagination.
- avail_for_rows 는 full(1009px)인데 1유닛만 cut → 측정·산술 정상, cut 경계 honor 버그(국소).

## 3. 검증 (양 오라클 게이트 + 단위/통합 테스트)

| 항목 | 베이스 | 수정 후 |
|------|--------|---------|
| 별표1(국토부) | 5쪽(Δ+1) | **4쪽 일치** ✅ |
| 별표4(산업통상부) | 33쪽(Δ+8) | **28쪽(Δ+3)** 개선 |
| 소형 controlset | 75 | **75 무회귀** |
| 대형 오라클(452) | 441 | **442 (+1)** |
| 신규 −1(under) | — | **0** |
| lib 전체 | 1984 | **1984 passed** (실패했던 #1488 단위 테스트 보존) |
| hwpx_roundtrip | 4 | **4 passed** |

- 변경 범위: table_layout.rs +11줄(국소). typeset.rs/engine.rs 무변경(=upstream).

## 4. 잔여 (후속)
- 가드 범위: 미세 fragment ≤2유닛 흡수(`tiny_fragment_waste`). #1488(3유닛 후 reset 보존)은
  j>start+2 로 무영향.
- 산업통상부 별표4 Δ+3 잔존(3~4유닛 fragment 낭비 — 페이지8=3줄/페이지21=4줄). #1488 의 3유닛
  break 보존과 충돌해 유닛수 가드로는 추가 흡수 불가. 구조적 reset(거대 셀 내부) vs 진짜 문단
  reset 을 구별하는 판별자(향후) 필요. fit-aware 전면화는 Δ+1 까지 가나 #1488(rowbreak-problem-pages)
  회귀 위험으로 보류.
- 법무부 별표2 Δ+1, 일부 −1(결재 .hwpx 하단표/별표9) 잔존 — 다른 메커니즘.

## 5. 의의
- 8종 실패(하단표 fixpoint) 후, **국소 cut 버그를 양 게이트 무회귀로 실제 수정한 첫 개선**.
- 대형 오라클 over 지배항(법령 표 행분할)의 1유닛 낭비를 제거 → 스케일 페이지수 정합 97.6→97.8%.
