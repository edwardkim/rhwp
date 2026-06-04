# task 1284 stage1: PDF bbox 기반 문항 흐름 drift 검출 고도화

## 배경

- GitHub Issue: [#1284](https://github.com/edwardkim/rhwp/issues/1284)
- #1274 및 PR #1277의 후속 작업이다.
- 작업지시자가 `3-09월_교육_통합_2024-미주사이20.hwp` 13쪽 화면에서 rhwp와 한컴 배치가 다르다고 지적했다.
- 기존 sweep은 해당 페이지를 `red_marker_drift`, `line_band_drift` 후보로 잡을 수 있지만, 어느 문항이 PDF/한컴 기준 위치와 다른지 설명하지 못한다.
- 따라서 sweep이 문항 번호별 위치 차이를 직접 보고하도록 고도화한다.

## 수정 방향

1. 한컴 PDF에서 `pdftotext -bbox-layout`로 `문17）`, `문18）`, `문19）` 같은 문항 marker bbox를 추출한다.
2. rhwp render tree의 `TextLine` 중 `문\d+` marker를 같은 문항 번호로 매칭한다.
3. 같은 문항 번호의 PDF 기준 page/column/y 위치와 rhwp 위치를 비교한다.
4. page 차이, column 차이, y drift가 기준 이상이면 `question_marker_drift`로 flag한다.
5. annotation PNG에 rhwp marker bbox와 PDF marker bbox를 같이 표시해 사용자가 바로 확인할 수 있게 한다.

## 검증 계획

- `python3 -m py_compile scripts/task1274_visual_sweep.py`
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - 13쪽/14쪽에서 `question_marker_drift`가 문항 번호와 함께 표시되는지 확인한다.
  - `flagged_pages.json`, `metrics.json`, `annotated_013.png`, `annotated_014.png`를 확인한다.
- 필요 시 `python3 scripts/task1274_visual_sweep.py --target all`

## 구현 내용

- `scripts/task1274_visual_sweep.py`
  - 필수 도구에 `pdftotext`를 추가했다.
  - 각 PDF를 `pdftotext -bbox-layout`로 변환해 `pdf_bbox.html`을 산출한다.
  - PDF의 `문\d+` word bbox를 PNG 좌표계로 변환해 `pdf_question_markers`로 저장한다.
  - rhwp render tree의 `TextLine` 중 `문\d+` marker를 `rhwp_question_markers`로 수집한다.
  - PDF 안에 같은 문항 번호가 여러 번 나오는 경우가 있어, 같은 번호 중 page/column/y가 가장 가까운 PDF marker와 rhwp marker를 매칭한다.
  - page/column/y 차이가 기준을 넘으면 `question_marker_drift` 후보로 기록한다.
  - annotation PNG에는 rhwp marker bbox를 빨간색, PDF 기준 marker bbox를 초록색으로 표시한다.
  - `analysis/question_flow.json`에 PDF/rhwp marker와 drift 후보를 저장한다.

## 검증 결과

- `python3 -m py_compile scripts/task1274_visual_sweep.py`: 통과
- `cargo build --bin rhwp`: 통과
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`: 통과
  - `question_marker_drift_pages=[13, 14, 18, 21, 23]`
  - 13쪽:
    - `문17`: 같은 13쪽/오른쪽 단, `y_delta_px=-88.2`
    - `문18`: 같은 13쪽/오른쪽 단, `y_delta_px=-88.2`
    - 작업지시자가 지적한 “한컴보다 위쪽에 배치된 문항 흐름”을 문항 번호와 좌표 차이로 검출한다.
  - 14쪽:
    - `문19`: 같은 14쪽/왼쪽 단, `y_delta_px=-77.2`
    - `문20`: 같은 14쪽/왼쪽 단, `y_delta_px=-89.5`
- `python3 scripts/task1274_visual_sweep.py --target all`: 통과
  - `2022-09`: `question=[]`
  - `2023-09`: `question=[15, 20]`
  - `2024-09-below20`: `question=[]`
  - `2024-09-between20`: `question=[13, 14, 18, 21, 23]`
  - `2022-10`: `question=[]`
  - `2022-11-practice`: `question=[]`

## 산출물

- `output/task1274/2024-09-between20/analysis/question_flow.json`
- `output/task1274/2024-09-between20/analysis/annotated_013.png`
- `output/task1274/2024-09-between20/analysis/annotated_014.png`

## 상태

- stage26 구현 및 자동 검증 완료.
