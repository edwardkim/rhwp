# Task M100 #1486 Stage 3: 13-14쪽 RowBreak 병합행 분할 정합 개선

- 이슈: #1486
- 브랜치: `local/task_m100_1486`
- 작성일: 2026-06-24
- 방법론: Hyper-Waterfall
- 선행 커밋:
  - `60eeccf1 task 1486: HWPX 분할 표 TAC 배치 1차 보정`
  - `99812698 task 1486: RowBreak 잔여 조각 페이지 보정`

## 배경

Stage 2에서 전체 페이지 수는 한컴 PDF와 동일한 29쪽으로 맞아졌고, 22쪽 `lisfranc` 밀림은 해소되었다. 이후 작업지시자가 13쪽과 14쪽을 다시 확인하도록 지시했고, 해당 구간에서 아직 한컴 PDF와 다른 표 내부 분할이 남아 있음을 확인했다.

## 관찰된 문제

- 문제 페이지: 13쪽, 14쪽
- 문제 표: `pi=127`, 13행x5열 배점기준표
- rhwp dump:
  - 13쪽: `PartialTable pi=127 rows=0..12 start_cut=[] end_cut=[5, 9, 2, 1]`
  - 14쪽: `PartialTable pi=127 rows=9..13 start_cut=[5, 9, 2, 1] end_cut=[]`
- rhwp 텍스트 흐름:
  - 13쪽 끝에 `국민기초생활...`, `영구임대주택...` 항목이 먼저 배치된다.
  - 14쪽 첫머리에 `(자)`, `(카)`가 배치된다.
- 한컴 PDF 텍스트 흐름:
  - 13쪽은 `(아)` 항목에서 끝난다.
  - 14쪽에서 `(자)`, `(카)` 다음에 `국민기초생활...`, `영구임대주택...` 항목이 이어진다.
- 14쪽 PDF에는 continuation 표의 왼쪽 셀 경계와 행 경계가 유지되지만, rhwp는 상단 continuation에서 같은 구조가 빠져 보인다.

## Stage 3 목표

- `pi=127` RowBreak 병합행 분할에서 셀별 컷 순서가 한컴 PDF와 다르게 잡히는 원인을 확인한다.
- 큰 rowspan/rowspan-like block continuation에서 모든 셀을 독립적으로 끝까지 밀어 넣는 대신, 한컴 PDF처럼 같은 행의 셀 컷을 함께 묶는 조건을 찾는다.
- 13/14쪽 텍스트 흐름과 continuation border가 한컴 PDF에 더 가깝게 나오도록 보정한다.

## 구현

- `src/renderer/layout/table_layout.rs`
  - `advance_row_block_cut_with_row_offsets`를 추가했다.
  - RowBreak rowspan 블록에서 아래쪽 행 셀은 블록 top 기준 예산에서 해당 행의 top offset을 뺀 만큼만 소비하도록 했다.
  - `row_block_cut_row_content_height`를 추가해 블록 컷 벡터를 특정 행의 per-row 컷으로 변환하고, 실제 cut이 없는 행은 전체 행 높이 기준을 유지하도록 했다.
  - `CellUnit`과 `cell_units`는 sibling partial table renderer가 같은 컷 기준을 쓰도록 layout 내부 가시성으로 열었다.
- `src/renderer/typeset.rs`
  - RowBreak rowspan block split에서 offset-aware 컷을 사용하도록 분기했다.
  - 첫 조각의 `end_row`는 실제 cut line이 지나간 행까지만 렌더하도록 조정했다. 이로써 row10/row11의 `국민기초생활`, `영구임대주택` 행이 13쪽에서 먼저 소비되지 않는다.
  - continuation 조각의 높이는 cut이 없는 row10/row11을 전체 행 높이로 계산한다.
- `src/renderer/layout/table_partial.rs`
  - block split row height override에서도 cut이 없는 행은 전체 행 높이를 유지하도록 맞췄다.
- `tests/issue_1486_hwpx_partial_tac_table.rs`
  - 13쪽에는 `제2조제10호` 행이 없어야 하고, 14쪽에는 `(자)`와 `제2조제10호`가 남아야 하는 회귀 테스트를 추가했다.

## 검증 결과

- `cargo fmt --check` 통과.
- `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture` 통과.
  - 3개 테스트 통과.
  - 9쪽 TAC overflow 회귀 없음.
  - 22쪽 `lisfranc` 위치 회귀 없음.
  - 13/14쪽 RowBreak 병합행 텍스트 흐름 보정 확인.
- `cargo build --release` 통과.
- `./target/release/rhwp info samples/hwpx_sample2.hwpx` 기준 페이지 수는 29쪽으로 한컴 PDF와 동일하다.
- 텍스트 검산:
  - `제2조제10호`, `국민기초생활`, `영구임대주택` 항목은 rhwp 14쪽에 위치한다.
  - `lisfranc` 항목은 rhwp 22쪽에 유지된다.
- 13/14쪽 PDF/SVG PNG side-by-side v2 생성:
  - p13: `changed_pct_gt24=15.358`, `rmse=54.685`
  - p14: `changed_pct_gt24=11.588`, `rmse=54.198`
  - 텍스트 흐름과 continuation 표 구조는 이전보다 한컴 PDF에 가까워졌다.
  - 세부 세로 간격은 아직 한컴 PDF보다 일부 조밀한 차이가 남아 있어 작업지시자 시각 판정이 필요하다.

## 분석 산출물

- 13-14쪽 연속 비교: `output/poc/task1486/stage2_compare_99812698/report/pages13_14/pages13_14_side_by_side_scaled.png`
- 13쪽 원본 비교: `output/poc/task1486/stage2_compare_99812698/report/page13/page13_side_by_side_full.png`
- 14쪽 원본 비교: `output/poc/task1486/stage2_compare_99812698/report/page14/page14_side_by_side_full.png`
- `pi=127` drift 로그: `output/poc/task1486/stage2_compare_99812698/report/pages13_14_pi127_drift.log`
- Stage 3 v2 13-14쪽 연속 비교: `output/poc/task1486/stage3_compare_v2/report/pages13_14_side_by_side_scaled.png`
- Stage 3 v2 13쪽 원본 비교: `output/poc/task1486/stage3_compare_v2/report/page13_side_by_side_full.png`
- Stage 3 v2 14쪽 원본 비교: `output/poc/task1486/stage3_compare_v2/report/page14_side_by_side_full.png`
- Stage 3 v2 텍스트: `output/poc/task1486/stage3_text_v2/`

## 검증 계획

- focused test: `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture`
- release build 후 13/14쪽 SVG/PDF PNG side-by-side 재생성
- 전체 페이지 수가 29쪽으로 유지되는지 확인
- 22쪽 `lisfranc` 회귀가 없는지 확인
