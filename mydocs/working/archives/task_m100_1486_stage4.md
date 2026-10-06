# Task M100 #1486 Stage 4: 13쪽 footer 겹침과 19쪽 마지막 그림 잘림 추가 개선

- 이슈: #1486
- 브랜치: `local/task_m100_1486`
- 작성일: 2026-06-24
- 방법론: Hyper-Waterfall
- 선행 커밋:
  - `60eeccf1 task 1486: HWPX 분할 표 TAC 배치 1차 보정`
  - `99812698 task 1486: RowBreak 잔여 조각 페이지 보정`
  - `41155a5c task 1486: RowBreak 병합행 분할 순서 보정`

## 배경

Stage 3에서 13-14쪽 배점기준표의 텍스트 흐름은 한컴 PDF에 더 가깝게 보정했다. 작업지시자 시각 확인 결과 다음 잔여 증상을 Stage 4에서 추가로 다룬다.

## 추가 증상

1. 19쪽 마지막 그림 잘림
   - 19쪽 하단의 마지막 그림 또는 그림 포함 표 조각이 한컴 PDF 기준보다 잘려 보인다.
   - 우선 PDF/SVG PNG side-by-side와 render tree bbox로 어떤 이미지 노드가 페이지 하단에서 잘리는지 확인한다.

2. 13쪽 페이지 번호와 문단 겹침
   - 13쪽 하단에서 페이지 번호와 본문/표 문단이 겹치는 증상이 있다.
   - Stage 3 보정 후 `pi=127` 분할 표가 13쪽 하단까지 내려오므로, footer/page number 안전 여백과 PartialTable 실제 하단을 함께 확인한다.

## Stage 4 목표

- 19쪽 마지막 그림 clipping 원인을 확인하고, 한컴 PDF 기준으로 잘림을 줄인다.
- 13쪽 footer/page number와 본문/표 조각이 겹치지 않도록 페이지 하단 안전 여백 또는 분할 높이 산정을 보정한다.
- Stage 2/3 회귀 항목을 유지한다.
  - 전체 페이지 수 29쪽 유지
  - 22쪽 `lisfranc` 위치 유지
  - 13/14쪽 `제2조제10호` 흐름 유지

## 분석 계획

- 13쪽, 19쪽을 한컴 PDF와 원본 크기 PNG로 다시 비교한다.
- `export-render-tree`에서 13쪽 페이지 번호와 하단 표 bbox, 19쪽 마지막 image/table bbox를 확인한다.
- `dump-pages`로 해당 페이지의 문단/표/그림 항목과 사용 높이를 확인한다.
- 필요 시 디버그 오버레이 SVG를 생성해 문제 노드의 `pi`/`ci`를 특정한다.

## 검증 계획

- focused test: `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture`
- release build 후 13쪽, 19쪽 PDF/SVG PNG side-by-side 생성
- 전체 페이지 수와 22쪽/14쪽 텍스트 위치 검산

## 구현 내용

1. 19쪽 마지막 그림 잘림 보정
   - 셀 내부 `Square`/`Tight`/`Through` 비-TAC 그림·도형의 시각 하단을 셀 콘텐츠 높이 후보에 반영했다.
   - `table_layout`, `table_partial`, `height_measurer`가 같은 기준을 사용하도록 맞췄다.
   - 19쪽 하단 내부 표의 왼쪽 그림은 기존 `y=1023.7, h=102.5`로 페이지 하단을 넘어갔으나, 보정 후 `y=978.4, h=102.5`, 하단 `1080.9`로 페이지 안에 들어온다.

2. 13쪽 쪽번호와 하단 표 간격 보정
   - `footer_area.height == 0`인 문서에서 쪽번호가 본문 하단 바로 아래에 붙는 문제를 줄이기 위해, 실제 꼬리말 여백 중앙을 기준으로 쪽번호 y를 계산하도록 보정했다.
   - 13쪽 하단 표 bbox 하단은 `1081.4`, 쪽번호 TextRun y는 `1106.9`로 약 `25.5px` 간격을 확보했다.

3. 회귀 테스트 추가
   - 19쪽 하단 그림 bbox가 페이지 아래로 넘어가지 않는지 검사한다.
   - 13쪽 하단 표와 `-13-` 쪽번호 사이 간격이 최소 12px 이상인지 검사한다.

## 검증 결과

- `cargo fmt` 통과.
- `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture` 통과.
  - 5개 테스트 통과.
  - 기존 9쪽 TAC 중첩 표, 22쪽 `lisfranc`, 13/14쪽 `제2조제10호` 회귀 검증 유지.
- `cargo build --release` 통과.
- `./target/release/rhwp info samples/hwpx_sample2.hwpx` 기준 페이지 수 29쪽 유지.
- clippy는 작업지시자의 이전 중지 지시에 따라 실행하지 않았다.

## Stage 4 산출물

- 13쪽/19쪽 SVG: `output/poc/task1486/stage4_after/svg/`
- 13쪽/19쪽 render tree: `output/poc/task1486/stage4_after/render_tree/`
- 13쪽/19쪽 rhwp PNG: `output/poc/task1486/stage4_after/svg_png/`
- 13쪽/19쪽 PDF PNG: `output/poc/task1486/stage4_after/pdf_png/`
- 13쪽/19쪽 side-by-side:
  - `output/poc/task1486/stage4_after/report/page13_bottom_side_by_side.png`
  - `output/poc/task1486/stage4_after/report/page19_bottom_side_by_side.png`
  - `output/poc/task1486/stage4_after/report/page13_side_by_side_full.png`
  - `output/poc/task1486/stage4_after/report/page19_side_by_side_full.png`
