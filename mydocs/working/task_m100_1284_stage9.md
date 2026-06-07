# task 1284 stage9: 전체 sweep 잔여 frame/question 후보 분석

## 배경

- stage8 커밋: `1b849b08 task 1284: 미주사이20 late 문항 gap 보정`
- stage8 이후 전체 sweep 결과에서 `2024-09-between20`의 `question=[]`, `frame=[]`는 유지됐다.
- 전체 sweep 잔여 후보는 `2023-09`와 `2022-11-practice`에 남았다.

## 잔여 후보

### 2023-09

- page15: `question_marker_drift`
  - 문23 `pi=759`
  - rhwp page15 column0 y=`90.7`
  - PDF page16 column0 y=`147.2`
  - page drift `-1`, y drift `-56.5`
- page20: `question_marker_drift`
  - 문30 `pi=972`
  - rhwp page20 column1 y=`90.7`
  - PDF page20 column0 y=`1021.8`
  - column drift, y drift `-931.1`
- page19: `frame_overflow_pixels`
  - `rhwp_outside_frame_pixels=276`
  - `rhwp_outside_frame_max_y=1102`

### 2022-11-practice

- page12: `frame_overflow_pixels`
  - `rhwp_outside_frame_pixels=9269`
  - `rhwp_outside_frame_max_y=1096`
  - `content_bottom_delta_px=-185.0`
- page19: `frame_overflow_pixels`
  - `rhwp_outside_frame_pixels=60`
  - `rhwp_outside_frame_max_y=1102`

## 진행 순서

1. `2023-09` page15/page20 문항 drift의 dump-pages/render-tree/PDF marker를 비교한다.
2. question drift가 page/column 분기 문제인지, sweep의 marker matching 오탐인지 구분한다.
3. 실제 렌더 차이면 pagination/render 공통 조건으로 보정한다.
4. 보정 후 `2023-09` focused sweep과 `issue_1139_inline_picture_duplicate`를 확인한다.
5. 이후 `2022-11-practice` frame overflow 후보를 별도 stage 또는 같은 stage 후반에서 이어간다.

## 분석 결과

- `2023-09` page15 drift는 sweep 오탐이 아니었다.
  - 한컴/PDF는 page14 오른쪽 단 하단에 `문23(pi=759)` 제목 한 줄을 남기고, page15 왼쪽 단 상단에서 `문23` 본문과 `문24`가 이어진다.
  - 기존 rhwp는 새 미주 그룹 시작 전 advance가 먼저 걸려 `문23` 제목까지 page15 상단으로 넘어갔고, 그 결과 page15 왼쪽 단 문항들이 한 줄씩 아래로 밀렸다.
- `2023-09` page20 drift도 실제 pagination 차이였다.
  - 한컴/PDF는 page20 왼쪽 단 하단에 `문30(pi=972)` 제목과 `pi=973..975` 첫 풀이 3줄을 남기고, `pi=976`부터 오른쪽 단 상단에서 이어진다.
  - 기존 rhwp는 직전 미주 rewind 이후 새 미주 그룹 전체 높이를 기준으로 advance하여 `문30`을 오른쪽 단 상단으로 넘겼다.
- 단순히 모든 기본 7mm 미주 제목 tail을 허용하면 2022/2024 9월 공통 문서의 page16 `문27(pi=875)`처럼 빈/TAC 식이 뒤따르는 제목이 하단에 orphan으로 남아 frame overflow가 발생했다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 기본 7mm 미주에서도 `새 미주 제목 한 줄`이 column 하단에 들어갈 수 있는지 판단하는 공통 조건을 추가했다.
  - 새 미주 그룹 advance 판단에도 같은 title-tail 조건을 반영했다.
  - 단, 비마지막 단에서는 `문29/문30`처럼 이어지는 앞부분이 실제 frame 안에 들어가는 경우만 허용한다.
  - 마지막 단에서도 다음 문단이 visible text로 이어지지 않는 빈/TAC 식 시작 케이스는 title-tail을 허용하지 않아 `문27` orphan overflow를 막았다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - `2023-09` page14 `문23(pi=759)` 제목 tail / page15 본문 이어짐 회귀 테스트 추가.
  - `2023-09` page20 `문30(pi=972..976)` 왼쪽 단 하단 tail + 오른쪽 단 이어짐 회귀 테스트 추가.
  - `2022-09` page17 `문27(pi=875)`이 page16 하단 orphan으로 남지 않고 PDF처럼 page17 상단에서 시작하는 회귀 테스트 추가.

## 검증 결과

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 59개 통과
- `python3 scripts/task1274_visual_sweep.py --target 2023-09`
  - `SVG pages=20`, `PDF pages=20`
  - `question=[]`
  - 잔여: 기존 `frame=[19]`
- `python3 scripts/task1274_visual_sweep.py --target 2022-09`
  - `SVG pages=23`, `PDF pages=23`
  - `frame=[]`, `question=[]`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20`
  - `SVG pages=23`, `PDF pages=23`
  - `frame=[]`, `question=[]`
- `python3 scripts/task1274_visual_sweep.py --target all`
  - `2022-09`: `frame=[]`, `question=[]`
  - `2023-09`: `frame=[19]`, `question=[]`
  - `2024-09-below20`: `frame=[]`, `question=[]`
  - `2024-09-between20`: `frame=[]`, `question=[]`
  - `2022-10`: `frame=[]`, `question=[]`
  - `2022-11-practice`: `frame=[12, 19]`, `question=[]`

## 남은 항목

- stage9에서 question drift 계열은 정리됐다.
- 남은 frame 후보는 기존 잔여인 `2023-09 page19`, `2022-11-practice page12/page19`이다.
- 다음 단계에서는 frame overflow 후보가 실제 overflow인지, frame 검출 오탐인지, partial tail/수식 흐름 문제인지 분리해서 진행한다.
