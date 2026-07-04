# Task 1293 Stage 50: 미주 separator 실제 거리 sweep 보강

## 목적

`mydocs/plans/task_m100_1293_impl.md`의 4단계는 visual sweep에서 미주 모양 정규화 값뿐 아니라
separator line과 첫 미주 내용 사이의 실제 거리도 측정하도록 요구한다.

Stage49 기준 전체 target의 page count와 renderer `LAYOUT_OVERFLOW`는 0건이 되었지만,
현재 `scripts/task1274_visual_sweep.py`는 `note_shape` 값과 overlap/overflow 후보만 기록한다.
구분선 위/아래/미주 사이 설정이 실제 화면 간격으로 맞는지 확인하려면 separator line과 첫 미주
content 간격을 RHWP/PDF 양쪽에서 직접 측정하는 지표가 필요하다.

## 작업 계획

1. sweep 분석 단계에 horizontal separator line 후보 검출을 추가한다.
2. 각 페이지에서 separator line 아래 첫 content band까지의 gap을 RHWP/PDF 각각 측정한다.
3. target summary에 separator gap drift page와 대표 delta를 기록한다.
4. `note_shape` normalized 값과 같은 manifest에 남겨 수동 시각 검증 시 바로 대조할 수 있게 한다.

## 구현 내용

- `scripts/task1274_visual_sweep.py`의 `note_shape` 요약에 다음 값을 추가했다.
  - `separatorEnabled`
  - `separatorLengthMm`
- 분석 단계에서 RHWP render tree의 `Line` 노드 중 예상 구분선 길이에 맞는 후보를
  separator line으로 사용하도록 했다.
- PDF 쪽은 이미지의 horizontal rule 후보를 사용하되 RHWP separator y 좌표를 anchor로 걸어,
  표 선/수식 선/본문 밑줄을 separator로 오인하는 경우를 줄였다.
- separator 아래 첫 미주 내용의 시작점은 빨간 미주 marker를 우선 사용한다.
  본문 content band가 marker보다 먼저 잡히는 페이지에서 0mm 구분선 아래 간격이 오탐으로
  커지는 것을 막기 위한 보정이다.
- target summary에 다음 지표를 추가했다.
  - `endnote_separator_gap_drift_pages`
  - `endnote_separator_observed_pages`
  - `endnote_separator_gap_pages`

## 대표 검증

- 명령:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-shape987 --target 2024-11-practice-above0-between0-below0 --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage50_separator_sweep --rhwp-bin target/debug/rhwp`
- 결과:
  - `2024-11-practice-shape987`: page count `21/21/21`, renderer overflow `0`, separator drift `[]`
  - `2024-11-practice-above0-between0-below0`: page count `21/21/21`, renderer overflow `0`, separator drift `[]`
  - `2024-11-practice-no-separator-above20-between20-below20`: page count `23/23/23`, renderer overflow `0`, separator observed/gap `[]`

## 전체 sweep 검증

- 명령:
  - `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage50_full_sweep --rhwp-bin target/debug/rhwp`
- 결과:
  - 전체 target 수: 15개
  - SVG/PDF/render tree page count mismatch: 0개
  - renderer `LAYOUT_OVERFLOW`: 0건
  - `frame_overflow_pages`: 0건
  - `question_title_text_overlap_pages`: 0건
  - `line_order_overlap_pages`: 0건
  - `equation_text_overlap_pages`: 0건
  - `endnote_separator_gap_drift_pages`: 0건
- 구분선이 있는 target은 첫 미주 구분선 gap 측정 페이지가 기록되며, 구분선 없음 target
  `2024-11-practice-no-separator-above20-between20-below20`은 `endnote_separator_observed_pages`와
  `endnote_separator_gap_pages`가 모두 비어 있다.

## 추가 검증

- `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile scripts/task1274_visual_sweep.py`
- `git diff --check`

## 판단

Stage50은 구현 계획서 4단계의 남은 요구사항인 “separator line과 첫 미주 내용 사이의 실제 거리
측정”을 sweep에 추가했다. Stage49 기준의 15개 target page count 1:1 및 renderer overflow 0건
상태도 유지된다.
