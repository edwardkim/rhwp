# Task 1293 Stage 54: 문항 흐름 drift 자동 게이트 보강

## 목적

Stage53에서 `2024-11-practice-no-separator-above20-between20-below20`의 21쪽을 직접 비교한 결과,
page count와 기존 core gate가 모두 맞아도 문항 흐름이 PDF와 구조적으로 다른 상태를 확인했다.

이번 단계에서는 sweep이 이 상태를 자동으로 core 후보로 표시하도록 보강한다. 렌더링 원인 수정은
다음 단계에서 이어가되, 먼저 잘못된 완료 판단을 막는다.

## 대상 증상

- `compare_020.png`: PDF는 문23/문24를 왼쪽 단 하단에 남기지만 rhwp는 문23부터 오른쪽 단으로
  보낸다.
- `compare_021.png`: rhwp는 문27/문28 흐름이고 PDF는 문28 continuation/문29 흐름이다.
- 기존 `red_marker_drift`, `line_band_drift`, `large_ink_region_drift` 후보가 잡혔지만, Stage52 요약에서
  core gate로 취급하지 않아 완료 판단을 흐렸다.

## 구현 계획

- `scripts/task1274_visual_sweep.py`에 `question_marker_flow_drift` flag를 추가한다.
- 단순 빨간 marker y 차이 전체가 아니라 다음처럼 구조 차이가 큰 페이지만 flag한다.
  - 빨간 marker 개수 차이가 2개 이상이고 line/large drift가 동반된다.
  - 또는 marker 최대 y drift가 매우 크고 line drift가 동반된다.
- target summary에 `question_marker_flow_drift_pages`를 추가하고 콘솔 출력에도 포함한다.

## 검증 계획

- no-separator target sweep에서 21/22쪽이 `question_marker_flow_drift`로 잡히는지 확인한다.
- 기존 core overlap/page count 지표는 그대로 유지되는지 확인한다.

## 실행 결과

문법 확인:

```bash
python3 -m py_compile scripts/task1274_visual_sweep.py
```

native CLI 갱신:

```bash
cargo build --bin rhwp
```

targeted sweep:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage54_qflow_no_separator \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF page count: 23/23
- 기존 core 후보: `frame=[]`, `sep=[]`, `title=[]`, `order=[]`
- 새 문항 흐름 drift 후보: `qflow=[18, 20, 21, 22, 23]`

Stage53에서 직접 확인한 `compare_021.png`가 자동으로 `question_marker_flow_drift`에 포함된다.
이제 이 target은 page count만 맞아도 완료로 오판되지 않는다.

## 다음 단계

렌더링 원인 수정은 다음 stage에서 계속한다. 우선 문22 tail → 문23 title 경계에서 왜 PDF는 왼쪽
단 하단에 문23/문24를 남기고 rhwp는 오른쪽 단으로 넘기는지 추적한다.
