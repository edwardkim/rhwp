---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1658/README.md
last_verified: 2026-07-16
---

# 조사 — #1658 구조적 reset 판별자 (음성 결과 + 진짜 원인 재규명)

- 일자: 2026-06-29 / 브랜치: local/task1658 / 목적: 별표4 잔여 Δ+3 의 3~4유닛 낭비를
  #1488 회귀 없이 흡수할 판별자(구조적 reset vs 진짜 문단 reset) 탐색.

## 1. 가설과 검증 (음성)
- 가설: #1488 reset 은 문단 경계(첫 줄, vis_start==0), 법령 거대 셀 낭비 reset 은 문단 내부(vis_start>0).
- 검증(`RHWP_UNIT_DEBUG`, CellUnit.vis_start 출력):
  - 별표1 row14 reset: u1 vis=(0,1), u36 vis=(3,4), u71 vis=(1,2) — **혼재**.
  - 별표4 거대 셀 reset: vis_start ∈ {0,1,2,4,5} — **혼재**(0=문단시작도 다수).
- ∴ **vis_start(문단경계 여부)로 #1488 과 구별 불가.** nested_row 도 전부 None(텍스트 줄). 판별자 없음.

## 2. 진짜 원인 재규명 — per-page 용량 결손
- 별표4 row31 한글 reset(=한글 페이지 경계): u=6, 41, 76, 111 … (~35유닛 ≈ 1쪽 간격).
- rhwp 는 페이지당 ~34유닛만 담고 **fit-break** → 한글 reset(35유닛째) 직전에서 끊겨 잔여 유닛이
  다음 fragment(2~4유닛)로 orphan. (페이지8=3줄, 페이지21=4줄.)
- 산식: 34유닛×28.8 = 979px, avail body 1009px. unit35(+28.8=1008<1009) 들어가야 하나 못 들어감.
  - `table_available = available − pagination_tolerance_px`(table_layout.rs:10646)가
    텍스트(+tolerance 허용)보다 표 row cut 의 용량을 `pagination_tolerance_px`(문서설정,
    `page_def.pagination_bottom_tolerance`, ~1유닛) 만큼 깎음 → 한글보다 ~1유닛 일찍 break.
  - 단, orphan 결손은 2~4유닛으로 변동 → tolerance(~1유닛) 외 추가 px 누적(행높이 미세 과대 등) 동반.

## 3. 결론
- 잔여 Δ+3 은 **reset 분류 문제 아님**(판별자 없음) → **per-page 용량 결손**(rhwp 가 한글보다
  ~2~4유닛 적게 적재) 의 close-call 계열. 1·2유닛 낭비는 `tiny_fragment_waste` 가드로 이미 흡수.
- 더 줄이려면: (a) 표 row cut 에 한글처럼 tolerance 용량 허용(클리핑 trade-off), 또는
  (b) fit-break 를 다음 reset(=한글 경계)로 스냅(소량 overflow 흡수). 둘 다 px 튜닝·회귀위험 동반 →
  양 게이트 + 시각(클리핑) 검증 동반한 별도 정밀 작업 필요.
- 현 landed 상태(별표1 일치, 별표4 Δ+8→Δ+3, 무회귀)가 안전 ceiling. fit-aware 전면화(Δ+1)는
  #1488(rowbreak-problem-pages) 회귀 위험으로 보류.

## 4. (b) reset-snap 시도 — 실패(되돌림)
- 구현: fit-break 시 다음 reset 이 `avail + HARD_BREAK_REMAINING_TOLERANCE_PX`(32px, body 내) 안에
  있으면 거기까지 채워 한글 경계와 정렬(advance_row_cut + advance_row_block_cut).
- 결과: 별표4 28 **불변**(orphan 결손 2~4유닛 ≈ 60~115px > 32px → 도달 불가) +
  **대형 게이트 442→440 회귀**(32px overflow 가 일부 문서 mis-align). → **되돌림.**
- 교훈: 안전 tolerance(32px≈1유닛)로는 2~4유닛 orphan 에 못 미치고, 더 키우면 클리핑·회귀.
  reset-snap 은 깨끗한 해법 아님.

## 5. 상태
- 소스 = Phase 3 커밋(table_layout.rs +11, tiny_fragment_waste ≤2). 무회귀.
- **안전 ceiling 확정**: 별표1 일치 / 별표4 Δ+8→Δ+3 / 대형 442(97.8%) / 소형 75 / lib·roundtrip 통과.
  추가 개선은 per-page 용량 정합(px 누적·클리핑 trade-off)이라 시각 검증 동반 별도 과제.
