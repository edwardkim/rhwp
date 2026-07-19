# task_m100_2291 stage5 — landscape RowBreak bleed 의 rowspan 걸침 행 게이트

## 배경

#2287 는 범교과 연결맵(`samples/task2287/1342000_edu_curriculum_map.hwp`, 244×10 RowBreak, rs=176 세로 라벨)의
과소분할을 375→380 쪽으로 부분 완화했으나 한글 2022 정답지 415 쪽 대비 **−35** 잔여가 남았다
(`task_m100_2287_report.md`). #2287 는 "RowBreak rowspan 블록 연속 조각의 잔여 증발"(블록 컷 회계)을
공략했지만, 지배 성분은 그 경로가 아니었다.

## 진단 — 근본 원인 규명

행 앵커맵(`tools/task2287/row_anchor_map.py 5:8`, 한글 2022 PDF 고유 앵커 매치)으로 표 5:8 의
쪽-행 배분을 대조한 결과:

| 지표 | 수정 전 | 한글 2022 |
|---|---|---|
| 표 5:8 쪽 범위 | p121~p139 (19쪽) | p120~p144 (24쪽) |
| 행/쪽 (평균) | ~12.8 | ~10.2 |
| 행별 \|rhwp쪽−한글쪽\| 평균 | 시작 +1 → 끝 −5 (단조 증가) | — |

드리프트가 **전 구간 점진적**(단일 걸침 행 점프 아님)이므로 걸침 행 cut 문제가 아니다.

- **행 높이는 정확했다**: `RHWP_TABLE_DRIFT` — 표 총높이 cut_sum=mt_sum=**14220.8px**, 예산 ~582.8px →
  14220.8/582.8 ≈ **24.4쪽**(한글 24쪽과 일치). rs=176 라벨 `known_sum=9965.9 ≈ decl 10028`(≈57px/행).
- **페이지네이션이 예산을 초과 적재**했다: p122 는 rows 5..17(815.9px)을 예산 582.8px 쪽에 적재.

원인은 `scan_block_table_split_rows` 의 **landscape RowBreak 연속 페이지 bleed**
(`typeset.rs` `landscape_short_row_tolerance=260px`, task #1672 편람 과다분할 완화용). 이 관용이
rowspan 걸침 행에서도 발동해 쪽당 ~260px(≈5행) 과다 적재를 누적 → 24→19쪽 과소분할, 문서 −30쪽.

## 판별 신호 — rowspan_touched

`RHWP_DIAG_BLEED` 계측으로 bleed 발동 행을 분류(수정 전 exe):

| 문서 | SHORT bleed | WHOLE bleed | rs_touched |
|---|---|---|---|
| 연결맵 (과소분할 −30) | 316 | 71 | **전부 true** |
| 편람 (task #1672 대상) | 23 | 5 | **전부 false** |

세로 병합 라벨이 확정한 행 그리드(걸침 행)에서는 한글이 병합 셀 높이 배분으로 정한 행 경계를 지켜
예산을 넘겨 적재하지 않는다. 편람의 과다분할 완화 대상은 순수 rs=1 행이라 완전 분리된다.

## 수정

`src/renderer/typeset.rs` — landscape short-row bleed(다행 260px)에 `&& !rowspan_touched[r]` 게이트 추가.
whole-row bleed(1행 ~36px, 한글도 하는 소량 bleed)는 유지 — 이것까지 막으면 +6 과보정(421쪽).

```rust
// [#2291] landscape RowBreak 연속 페이지 bleed 는 순수 rs=1 행에만 적용한다.
// rowspan 걸침 행(세로 병합 라벨이 확정한 행 그리드)은 예산을 넘겨 적재하지 않는다.
if landscape_rowbreak_bleed && ... && r > cursor_row
    && !rowspan_touched[r]                       // ← 추가
    && row_total <= landscape_short_row_max_height
    && consumed + cs_before + row_total <= avail_for_rows + landscape_short_row_tolerance
```

## 검증

| 게이트 | 수정 전 | 수정 후 | 한글 정답 |
|---|---|---|---|
| 연결맵 쪽수 | 385 | **414** | 415 |
| 표 5:8 행 앵커 평균 \|Δ쪽\| | 시작+1→끝−5 | **0.84** | — |
| 편람 hwp / hwpx | 394 / 390 | **394 / 390** (무회귀) | — |
| 92 컨트롤셋 | — | **90/92 정합, −1×2, +1×0** | — |
| issue_2097_band_fill / _squeeze | pass | **pass** | — |
| issue_2070_rowbreak_density | pass | **pass** | — |
| issue_1035_alignment (sample16=64) | pass | **pass** | — |
| issue_2146 / issue_1658 / issue_2063 | pass | **pass** | — |
| byeolpyo1=4 / byeolpyo4=26 | pass | **pass** | — |
| cargo nextest run | — | (진행 중) | — |

- 시각: 표 5:8 p130 SVG 렌더 — 1122.5×793.7px(landscape A4), body-clip 642.56px, 817 text 요소 정상,
  콘텐츠 페이지 경계 내(과다 적재 해소로 오버플로 없음).
- 방향성: 이 게이트는 rowspan 걸침 landscape 행에서 bleed **제거**(쪽 추가 방향)만 하므로 92셋 −1축에
  새 +1 회귀를 만들 수 없다(게이트 결과 +1×0 확인).

## 결론

#2291 잔여(−35, s5류 걸침 행 축)의 근본 원인은 걸침 행 cut 부재가 아니라 landscape short-row bleed 의
rowspan 페이지 과다 적재였다. `rowspan_touched` 판별로 편람(task #1672) 무회귀 + 연결맵 −30→−1 정합.
