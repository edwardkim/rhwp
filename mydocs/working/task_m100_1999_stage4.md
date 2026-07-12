# #1999 Stage 4 - TopAndBottom flow 유닛 고립 방지

## 목적

Stage 3에서 p14 하단 불일치의 직접 원인을 확인했다. RowBreak 컷이 pi=310의
`TopAndBottom` non-inline 그림 flow 유닛은 p14에 넣고, 바로 뒤 빈 spacer는 p15로
넘기면서 그림만 페이지 하단에 고립된다.

이 스테이지에서는 특정 샘플명/페이지 번호/이슈 번호가 아니라 CellUnit 구조에 기반해 다음 조건을 보정한다.

- 현재 유닛이 non-inline control flow 전용 유닛이다.
- 다음 유닛이 hard break가 아닌 빈 spacer다.
- 현재 유닛만 넣으면 페이지 예산 안에 들어간다.
- 현재 유닛과 다음 spacer를 함께 넣으면 페이지 예산을 넘는다.
- 현재 유닛이 조각의 시작 유닛은 아니다. 시작 유닛은 진행 보장을 위해 그대로 소비한다.

위 조건이면 그림 flow 유닛을 현재 쪽에 고립시키지 않고 다음 쪽에서 spacer와 함께 배치되도록 컷을 멈춘다.

## 검증 계획

- `cargo fmt --check`
- `env CARGO_INCREMENTAL=0 cargo test row_cut_tests --lib`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1949_giant_cell_render_perf`
- PDF/HWPX/HWP 쪽수 115 / 115 / 115 확인
- HWP/HWPX 14~16쪽 visual sweep 재수행

## 진행 기록

### Stage 4 시작 상태

- 선행 커밋:
  - `a592b0677 task 1999: 14~16쪽 그림 흐름 원인 분석`
- 현재 미커밋 코드:
  - `TopAndBottom` flow height에 top/bottom margin을 포함하는 실험 변경
- 이 변경은 Stage 4의 구조적 orphan 보정과 함께 최종 검증한다.

### 구현

- 표 셀 TopAndBottom flow height에 `margin.top` / `margin.bottom`을 포함했다.
- `advance_row_cut` / `advance_row_block_cut`에 `non-inline flow unit + following empty spacer` orphan 방지 조건을 추가했다.
- partial table continuation 렌더에서 실제 non-inline flow 유닛이 컷에 포함된 경우, `LineSeg.vertical_pos` 전체 셀 좌표 대신 현재 fragment의 `para_y_before_compose`를 anchor로 사용하도록 보정했다.

근거 문서 속성:

- `CommonObjAttr.text_wrap = TopAndBottom`
- `CommonObjAttr.vert_rel_to = Para`
- `CommonObjAttr.flow_with_text = true`
- `CommonObjAttr.margin.top/bottom`
- RowBreak 셀의 `CellUnit` 구조: non-inline control flow 유닛과 뒤따르는 빈 spacer

### 검증

- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test row_cut_tests --lib`: 19 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1949_giant_cell_render_perf`: 1 passed
- 쪽수:
  - 기준 PDF: 115쪽
  - `samples/issue1949_giant_cell_nested_tables_perf.hwpx`: 115쪽
  - `samples/issue1949_giant_cell_nested_tables_perf.hwp`: 115쪽

visual sweep:

- HWP:
  - `output/task1999_issue1999_p14_16_hwp_stage4b/summary.json`
  - SVG/PDF/render-tree: 115 / 115 / 115쪽
  - 14~16쪽 `flagged=0/3`
  - 평균 pixel match: `91.9511%`
  - 평균 내용 픽셀 중심 자동 일치율 보조값: `12.63558%`
- HWPX:
  - `output/task1999_issue1999_p14_16_hwpx_stage4b/summary.json`
  - SVG/PDF/render-tree: 115 / 115 / 115쪽
  - 14~16쪽 `flagged=0/3`
  - 평균 pixel match: `91.9511%`
  - 평균 내용 픽셀 중심 자동 일치율 보조값: `12.63558%`

시각 판정:

- 개선됨: p14 하단에 그림 2개가 고립되던 현상은 사라졌다.
- 개선됨: p15에서 그림 2개가 기준 PDF처럼 페이지 상단으로 이동했다.
- 남은 차이: 기준 PDF는 p14에 3.4.6.3/3.4.6.4 텍스트를 더 채운 뒤 p15 상단에 그림을 배치한다.
  현재 rhwp는 그림 뒤의 3.4.6.3/3.4.6.4도 p15로 같이 넘어가므로 p15~p16 텍스트 흐름이 약간 뒤로 밀린다.

### 결론

Stage 4는 반복 그림/하단 고립 문제를 줄이는 구조 보정으로 유효하지만, 기준 PDF와 같은 최종 흐름에는 아직 부족하다.
다음 스테이지에서는 한컴이 `TopAndBottom` 그림을 다음 쪽 상단으로 지연하면서도 후속 텍스트 일부는 이전 쪽에
계속 채우는 동작을 문서 속성 기반으로 모델링할 수 있는지 확인한다.
