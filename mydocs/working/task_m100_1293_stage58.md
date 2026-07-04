# Task 1293 Stage 58: no-separator 잔여 qflow 후보 분류

## 목적

Stage57에서 no-separator large block의 stale vpos tail advance를 보정해 page 15~16 흐름을
한컴 PDF에 가깝게 당겼다. targeted sweep 결과 core overlap/overflow는 모두 0이지만
`question_marker_flow_drift_pages`가 `[18, 21, 22]`로 남았다.

이번 단계에서는 이 세 후보가 실제 미주 흐름 실패인지, red marker count/짝짓기 기반 sweep 오탐인지
직접 compare PNG와 render tree로 분류한다. 실제 구조 차이면 추가 보정하고, 오탐이면 sweep의 qflow
판단 기준을 더 정교화한다.

## 대상

- latest sweep: `output/task1293_stage57_no_separator_tail_guard`
- target: `2024-11-practice-no-separator-above20-between20-below20`
- 후보 페이지:
  - page 18
  - page 21
  - page 22

## 분석 계획

1. `compare_018.png`, `compare_021.png`, `compare_022.png`를 직접 비교한다.
2. 각 페이지의 red marker 개수, y drift, 문항 번호 흐름을 PDF와 대조한다.
3. qflow 후보가 실제 page/column 분배 차이인지 확인한다.
4. 오탐이면 qflow 조건을 red marker count만이 아니라 실제 문항 번호 시퀀스/큰 line drift와 함께 판단하도록 보강한다.

## 분류 결과

### page 18

- compare PNG상 Stage57에서 문27/문28/문29 흐름은 Stage55보다 PDF에 가까워졌다.
- 그러나 여전히 `line_band_drift`가 구조 기준을 넘고, 큰 도형/수식 영역의 배치 차이가 남아 있다.
- qflow 후보로 유지한다.

### page 21

- Stage55에서는 큰 도형이 page 21 맨 위에 남아 PDF와 명백히 달랐다.
- Stage57에서는 문29와 도형 흐름이 PDF 위치에 가까워졌다.
- metrics:
  - red marker count: rhwp 6, PDF 4
  - red max drift: `34px`
  - line mean/p90: `17.7px` / `26.5px`
- red marker y 차이가 작고 line drift도 구조 기준 미만인데, 큰 ink region 조각 수 차이만으로
  qflow에 포함됐다. 직접 비교 기준으로는 실제 문항 흐름 실패가 아니라 sweep 오탐이다.

### page 22

- 문30 도형/풀이 tail의 page/column 분배 차이가 아직 남아 있다.
- `line_band_drift`도 구조 기준을 넘으므로 qflow 후보로 유지한다.

## 수정

`scripts/task1274_visual_sweep.py`의 qflow 판정을 보강했다.

- red marker 개수 차이가 있더라도 red y drift가 `80px` 미만이고 line drift가 구조 기준 미만이면
  qflow로 승격하지 않는다.
- 이는 page21처럼 marker count/large ink segmentation만 다른 경우를 제외하기 위한 조건이다.
- red y drift가 크거나 line drift가 구조 기준을 넘는 page18/page22는 계속 qflow 후보로 남는다.

## 검증 결과

문법 확인:

```bash
python3 -m py_compile scripts/task1274_visual_sweep.py
```

targeted sweep:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage58_no_separator_qflow_refined \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF/render tree page count: `23/23/23`
- renderer `LAYOUT_OVERFLOW`: 0
- frame/title/order/equation overlap: 0
- qflow: `[18, 22]`
- page21 qflow 오탐 제거 확인

## 검증 계획

- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20`
- 필요 시 `python3 scripts/task1274_visual_sweep.py --target all`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
