# #1999 Stage 5 - TopAndBottom 지연 개체와 후속 텍스트 흐름 분석

## 목적

Stage 4에서 p15 상단 그림 배치는 개선됐지만, 기준 PDF와 달리 3.4.6.3/3.4.6.4 텍스트까지 p15로 같이 밀렸다.
기준 PDF는 그림을 p15 상단으로 지연하면서도, 그림 뒤의 일부 텍스트는 p14에 계속 채운다.

이 스테이지에서는 RowBreak 셀의 순차 CellUnit 모델로 이 동작을 표현할 수 있는지 확인한다.

## 검증 기준

- 특정 파일명, 페이지 번호, 이슈 번호, 임의 계수로 분기하지 않는다.
- 보정 근거는 `TopAndBottom`, `flow_with_text`, `VertRelTo::Para`, `LineSeg`, `CellUnit` 구조에 둔다.
- 모델 변경이 필요하다면, 먼저 현재 `PageItem::PartialTable`의 `start_cut/end_cut`만으로 지연 개체를 표현할 수 있는지 기록한다.

## 진행 기록

### Stage 5 시작 상태

- 선행 커밋:
  - `a88dd0285 task 1999: RowBreak 그림 flow 고립 방지`
- 현재 상태:
  - PDF/HWPX/HWP 쪽수는 모두 115쪽
  - HWP/HWPX 14~16쪽 visual sweep은 `flagged=0/3`
  - p15 상단 그림 배치는 개선됨
  - p14/p15 텍스트 흐름은 기준 PDF보다 약간 뒤로 밀림

### 구현

RowBreak 셀의 `CellUnit` 생성 후처리에서 다음 조건을 만족하는 flow 유닛을 다음 가시 hard break 직전으로 지연했다.

- unit이 non-inline control flow 전용 유닛이다.
- 해당 문단의 텍스트가 비어 있다.
- control 속성이 `TopAndBottom + VertRelTo::Para + flow_with_text`이다.
- 뒤쪽에 가시 텍스트 hard break가 있다.

이렇게 하면 기준 PDF처럼 그림은 다음 쪽 상단으로 넘어가면서도, anchor 뒤 후속 텍스트는 이전 쪽에 계속 채워질 수 있다.

근거 문서 속성:

- `CommonObjAttr.text_wrap = TopAndBottom`
- `CommonObjAttr.vert_rel_to = Para`
- `CommonObjAttr.flow_with_text = true`
- anchor 문단의 빈 텍스트
- 저장 `LineSeg` 기반 `hard_break_before`

### 검증

- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test row_cut_tests --lib`: 20 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1949_giant_cell_render_perf`: 1 passed
- 쪽수:
  - 기준 PDF: 115쪽
  - `samples/issue1949_giant_cell_nested_tables_perf.hwpx`: 115쪽
  - `samples/issue1949_giant_cell_nested_tables_perf.hwp`: 115쪽

visual sweep:

- HWP:
  - `output/task1999_issue1999_p14_16_hwp_stage5/summary.json`
  - SVG/PDF/render-tree: 115 / 115 / 115쪽
  - 14~16쪽 `flagged=0/3`
  - 평균 pixel match: `92.3305%`
  - 평균 내용 픽셀 중심 자동 일치율 보조값: `13.64366%`
- HWPX:
  - `output/task1999_issue1999_p14_16_hwpx_stage5/summary.json`
  - SVG/PDF/render-tree: 115 / 115 / 115쪽
  - 14~16쪽 `flagged=0/3`
  - 평균 pixel match: `92.3305%`
  - 평균 내용 픽셀 중심 자동 일치율 보조값: `13.64366%`

시각 판정:

- p14: 기준 PDF처럼 3.4.6.3/3.4.6.4 텍스트까지 이전 쪽에 남는다.
- p15: 그림 2개가 기준 PDF처럼 상단에 배치되고 3.4.7부터 이어진다.
- p16: 3.5.3 이후 흐름이 기준 PDF와 같은 순서로 이어진다.

### 결론

Stage 5 보정으로 p14~p16의 주요 시각 흐름이 기준 PDF와 맞아졌다. 남은 차이는 글꼴/렌더링 세부 위치 차이이며,
자동 sweep도 drift 후보를 내지 않았다.
