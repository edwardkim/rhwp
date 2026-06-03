# task 1274 stage24: 2022-10 11쪽 문20 line order overlap 검출 정밀화

## 배경

- 작업지시자가 `3-10월_교육_통합_2022.hwp` 11쪽 오른쪽 단 하단의 `문20)` 풀이에서 본문과 수식이 겹친다고 지적했다.
- 기존 `task1274_visual_sweep.py`는 해당 페이지를 `equation_text_overlap` 후보로 잡았지만, 실제 문제의 형태인 “다음 TextLine이 이전 TextLine 위로 올라와 줄 순서가 겹치는 현상”을 별도 유형으로 설명하지 못했다.
- 따라서 render tree의 `Equation`/`TextRun` bbox 교차뿐 아니라, 인접 `TextLine` 간 세로 overlap과 순서 역전을 직접 검출해야 한다.

## 현재 관찰

- 대상: `samples/3-10월_교육_통합_2022.hwp`
- 페이지: 11쪽 (`0-based page=10`)
- 위치: 오른쪽 단 하단 `문20） 226`
- render tree 기준:
  - `pi=582`: `문20）   226`, y=`953.0`
  - `pi=586`: `이차식 [EQ]에 대하여 [EQ]라 하자.`, y=`1035.1`, h=`12.0`
  - `pi=587`: 다음 수식 line `[EQ]`, y=`1032.7`, h=`28.8`
- `pi=587`의 y가 `pi=586`보다 위쪽이라 두 line bbox가 세로로 완전히 겹친다.

## 수정 방향

1. `scripts/task1274_visual_sweep.py`에 render tree 인접 line overlap 검출을 추가한다.
   - `TextLine` 단위로 텍스트, bbox, pi, path를 수집한다.
   - 같은 단/같은 x 영역으로 보이는 인접 line의 세로 bbox가 큰 비율로 겹치면 `line_order_overlap` 후보로 기록한다.
   - 후보에는 현재 문항 번호, 이전 line/다음 line pi, text, bbox, overlap ratio를 포함한다.
2. annotation PNG가 실제 후보 위치를 보여주도록 rhwp 쪽에 bbox overlay를 그린다.
   - line order 후보는 이전 line과 다음 line을 색으로 구분해 표시한다.
   - 기존 frame annotation만으로는 문20 overlap 지점을 찾기 어려웠던 문제를 보완한다.
3. `summary.json`에 `line_order_overlap_pages`를 추가한다.
   - `2022-10` page 11이 명확히 이 지표로 잡혀야 한다.

## 검증 계획

- `python3 -m py_compile scripts/task1274_visual_sweep.py`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
  - `line_order_overlap_pages`에 11쪽이 포함되는지 확인한다.
  - `metrics.json` page 11의 후보가 `question=문20`, `prev_pi=586`, `next_pi=587`을 가리키는지 확인한다.
  - `analysis/annotated_011.png`에서 겹친 line bbox가 표시되는지 확인한다.
- 필요 시 `python3 scripts/task1274_visual_sweep.py --target all`로 6종 전체 지표 변화를 확인한다.

## 상태

- 작업지시자 요청에 따라 stage24 착수.
- `scripts/task1274_visual_sweep.py` 수정과 검증을 완료했다.

## 구현 결과

- `line_order_overlap` 지표를 추가했다.
  - render tree의 인접 `TextLine`을 순회하면서 같은 단/x 영역에서 세로 bbox가 `0.65` 이상 겹치는 후보를 찾는다.
  - 같은 `pi` 내부의 큰 수식 bbox는 제외한다. 이 후보는 문단 내부 수식 줄 높이 문제일 수 있으므로, stage24의 목적과 같은 “서로 다른 pi의 줄 순서 역전”과 분리했다.
  - 후보에는 `question`, `question_text`, `prev_pi`, `next_pi`, `prev_text`, `next_text`, `prev_bbox`, `next_bbox`, `overlap_ratio`, `overlap_px`, `y_delta`를 기록한다.
- `analysis/annotated_*.png`에 render tree bbox overlay를 추가했다.
  - `line_order_overlap`은 이전 line과 다음 line을 서로 다른 색으로 표시한다.
  - 기존 `equation_text_overlap`, `question_title_text_overlap` 후보도 annotation PNG에 bbox로 표시한다.
- `summary.json`에 `line_order_overlap_pages`를 추가하고, 콘솔 summary line에도 `order=[...]`를 출력한다.

## 검증 결과

- `python3 -m py_compile scripts/task1274_visual_sweep.py` 통과.
- 기존 `output/task1274/2022-10/render_tree/render_tree_011.json` 직접 검사에서 문20 후보를 정확히 잡았다.
  - `question=문20`
  - `question_text=문20）   226`
  - `prev_pi=586`, text=`이차식 [EQ]에 대하여 [EQ]라 하자.`
  - `next_pi=587`, text=`[EQ]`
  - `overlap_ratio=1.0`, `overlap_px=12.0`, `y_delta=-2.4`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10` 통과.
  - SVG/PDF/render tree 모두 18쪽.
  - `line_order_overlap_pages=[9, 11]`
  - 11쪽 문20 후보가 `metrics.json`에 위 값으로 기록됐다.
  - `output/task1274/2022-10/analysis/annotated_011.png`에 문20 하단 line bbox overlay가 표시됐다.
- `python3 scripts/task1274_visual_sweep.py --target all` 통과.
  - `2022-09`: SVG/PDF/render tree 23/23/23쪽, `order=[]`.
  - `2023-09`: SVG/PDF/render tree 20/20/20쪽, `order=[]`.
  - `2024-09-below20`: SVG/PDF/render tree 23/23/23쪽, `order=[]`.
  - `2024-09-between20`: SVG/PDF/render tree 24/24/24쪽, `order=[]`.
  - `2022-10`: SVG/PDF/render tree 18/18/18쪽, `order=[9, 11]`.
  - `2022-11-practice`: SVG/PDF/render tree 21/21/21쪽, `order=[18]`.

## 참고

- `2022-10` 9쪽과 `2022-11-practice` 18쪽도 같은 “서로 다른 pi의 인접 line overlap” 후보로 새로 잡힌다. 문20처럼 실제 시각 검증이 필요한 후보로 분류한다.
- stage24는 검출 정밀화 단계이며, 문20 배치 자체의 수정은 별도 stage에서 진행할 수 있다.
