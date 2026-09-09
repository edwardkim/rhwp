# Task M100 #1486 Stage 2: 한컴 PDF 정합 잔여 차이 분석

- 이슈: #1486
- 브랜치: `local/task_m100_1486`
- 작성일: 2026-06-24
- 방법론: Hyper-Waterfall
- 선행 커밋: `60eeccf1 task 1486: HWPX 분할 표 TAC 배치 1차 보정`

## 배경

Stage 1에서 9쪽 상단 TAC 중첩 표의 오른쪽 overflow는 1차로 보정했다. 그러나 작업지시자 시각 확인 결과 한컴 PDF 기준 출력과 아직 맞지 않는 페이지가 확인되었다.

## 관찰된 잔여 문제

- 한컴 PDF 기준과 rhwp 렌더 결과의 페이지 분할/위치가 다르다.
- 작업지시자 캡처 기준으로 앞 페이지 하단 문단과 다음 페이지 첫 줄 위치가 한컴 PDF 기준과 맞지 않는다.
- 따라서 Stage 2에서는 단순 x overflow가 아니라 페이지 분할 또는 분할 표/문단 높이 산정 차이를 추적한다.

## Stage 2 목표

- 한컴 PDF 기준과 rhwp SVG/Canvas 산출물의 불일치 페이지를 특정한다.
- 해당 페이지의 `dump-pages`, render tree, SVG/PDF PNG 비교로 어떤 문단/표 조각이 밀리거나 당겨졌는지 기록한다.
- 원인이 Stage 1 TAC x 보정의 부작용인지, 별도의 분할 표 높이/라인 세그먼트 문제인지 분리한다.

## 검증 계획

- 우선 시각 비교와 render tree/dump 분석만 수행한다.
- `cargo test`/`cargo clippy`는 작업지시자 지시에 따라 현재 중지 상태를 유지한다.
- 수정이 필요하면 코드 변경 전 분석 결과를 이 문서에 갱신한 뒤 진행한다.
- 작업지시자 추가 지시에 따라 Stage 2 변경을 먼저 커밋한 뒤 전체 페이지를 한컴 PDF와 대조한다.

## 현재 분석

- `pdftotext` 페이지별 추출 결과, 한컴 PDF 기준 `발목발허리관절(lisfranc joint)` 줄은 PDF 22쪽 하단에 있다.
- rhwp HWPX `export-text` 기준 같은 줄은 rhwp 23쪽 하단에 있다.
- rhwp HWP 입력도 같은 위치로 렌더되어 HWPX 파서 차이가 아니라 렌더/페이지네이션 공통 문제로 판단한다.
- `dump-pages` 기준 원인은 `pi=185 ci=0` RowBreak 표의 terminal fragment다.
  - rhwp 20쪽: `PartialTable pi=185 rows=0..4 start_cut=[] end_cut=[5, 4]`
  - rhwp 21쪽: `PartialTable pi=185 rows=3..5 start_cut=[5, 4] end_cut=[2, 19]`, render tree bbox 높이 `1045.5px`
  - rhwp 22쪽: `PartialTable pi=185 rows=4..5 start_cut=[2, 19] end_cut=[]`, render tree bbox 높이 `17.1px`
- PDF 기준은 `pi=185`에 해당하는 유의사항 표가 PDF 20~21쪽에서 끝나고, PDF 22쪽은 바로 `장애인 편의증진시설 설치` 표와 후속 문단으로 시작한다.
- 따라서 현재 잔여 문제는 9쪽 TAC 표 x 좌표가 아니라, RowBreak 분할 표의 마지막 17px 잔여 조각이 별도 빈 페이지로 emit되는 문제다.

## 원인 후보

- `src/renderer/typeset.rs`의 컷 기반 분할 루프가 마지막 fragment 조건(`end_row >= row_count && split_end_limit == 0.0`)에서 `partial_height=17.1px`짜리 terminal fragment를 그대로 `PageItem::PartialTable`로 추가한다.
- `RHWP_TABLE_DRIFT=1 dump-pages` 로그:
  - `TABLE_SPLIT_AVAIL: pi=185 ... cursor_row=4 cont=true ... start_cut=[2, 19]`
  - `TABLE_SPLIT_RESULT: pi=185 ... end_row=5 consumed=17.1 partial_h=17.1 split_end_limit=0.0`
- 한컴 PDF 기준으로는 이 terminal sliver를 별도 페이지로 만들지 않는 것으로 보인다. 다음 단계에서는 해당 조각이 trailing empty content/border-only fragment인지 확인하고, 안전하게 흡수하거나 생략할 수 있는 조건을 좁힌다.

## 구현

- `src/renderer/layout/table_layout.rs`
  - RowBreak 컷 범위에 실제 보이는 내용이 남아 있는지 판정하는 `row_cut_range_has_visible_content`를 추가했다.
  - 빈 문단/패딩만 남은 terminal fragment를 식별할 때 nested row, 문단 텍스트, control 존재 여부를 확인한다.
  - 일반 `advance_row_cut`에서는 hard break 직전 orphan rewind를 적용하지 않도록 조정했다. 기존 rowspan block cut 경로의 rewind는 유지한다.
- `src/renderer/typeset.rs`
  - continuation RowBreak의 마지막 조각이 작고, caption overhead가 없으며, 컷 범위에 보이는 내용이 없을 때 `PageItem::PartialTable` emit을 생략한다.
- `tests/issue_1486_hwpx_partial_tac_table.rs`
  - 한컴 PDF 기준 22쪽 하단의 `lisfranc` 줄이 rhwp 22쪽에 남고 23쪽으로 밀리지 않는 회귀 테스트를 추가했다.

## 검증 상태

- 1차 terminal sliver 생략 변경 후 `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture` 통과를 확인했다.
- 이후 hard break orphan rewind 범위를 좁히는 추가 변경 중 작업지시자가 "커밋 후 전체 페이지 pdf 와 대조"를 지시했다.
- 따라서 clippy/test 추가 수행 없이 Stage 2를 먼저 커밋하고, 커밋 이후 `cargo build --release`와 전체 페이지 PDF/SVG PNG 대조를 수행한다.

## 산출물

- PDF 페이지별 텍스트: `output/poc/task1486/pdf_text_pages/page_022.txt`
- rhwp 텍스트: `output/poc/task1486/text_hwpx_after/hwpx_sample2_022.txt`, `output/poc/task1486/text_hwpx_after/hwpx_sample2_023.txt`
- dump: `output/poc/task1486/dump_hwpx_page_020_after.txt`
- dump: `output/poc/task1486/dump_hwpx_page_021_after.txt`
- dump: `output/poc/task1486/dump_hwpx_page_022_after.txt`
- drift log: `output/poc/task1486/table_drift_page_022.log`
