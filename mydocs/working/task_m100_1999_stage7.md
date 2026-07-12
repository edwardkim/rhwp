# #1999 Stage 7 - RowBreak 순서 역전 회귀 보정

## 목적

Stage 6 이후 전체 회귀 테스트에서 `tests/issue_rowbreak_chart_overlap.rs`의
`rowbreak_page17_keeps_database_separation_line_before_example_box`가 실패했다.

- `issue_1949_giant_cell_render_perf`: 통과
- `issue_1686`: 통과
- 회귀: `samples/rowbreak-problem-pages.hwpx` 17쪽에서 "예시" 캡션이 "별도" 문장보다 위로 렌더링됨

## 원인

Stage 6은 HWPX 소스 여부가 아니라 문서 구조를 기준으로 RowBreak hard break 완화 여부를 결정하도록 고쳤다.
다만 일부 RowBreak 표에서는 저장 `LineSeg` hard break를 완화할 때 후속 caption/box가 선행 문장보다 먼저
나오는 순서 역전이 발생할 수 있다.

실제 실패 샘플은 `RowBreak` 1×1 표 안에 다음 구조를 가진다.

- 앞쪽에는 `Square` 계열 텍스트 박스 flow가 있다.
- 뒤쪽에는 빈 anchor 문단에 매달린 `TopAndBottom` TAC 사각형 2개가 있다.
- Stage 4의 non-inline flow atomic 처리 때문에 `Square` 텍스트 박스 flow가 이전 쪽에서 전부 소비되어,
  다음 쪽에서 `TopAndBottom` 예시 박스가 먼저 렌더링됐다.

보정 근거는 특정 샘플명이 아니라 다음 문서 속성/레이아웃 유닛에 둔다.

- 표 `page_break = RowBreak`
- 표 `common.treat_as_char = false`
- `CellUnit`의 non-inline flow fragment
- 후속 빈 anchor의 `TextWrap::TopAndBottom` control
- `CellUnit`의 `empty_spacer` run

## 구현

- `TopAndBottom` flow 유닛과 `Square/Tight/Through` flow fragment를 구분하도록 `CellUnit`에
  `top_and_bottom_flow` 플래그를 추가했다.
- `TopAndBottom` flow는 기존 Stage 4 의도대로 atomic 유지한다.
- `Square/Tight/Through` flow는 기존처럼 16px fragment로 유지해 텍스트 박스 꼬리가 continuation에서
  보존될 수 있게 했다.
- RowBreak cut이 뒤쪽 `TopAndBottom` flow 직전까지 앞쪽 non-inline fragment run을 전부 소비하려 할 때,
  마지막 fragment 하나를 다음 조각에 남겨 선행 설명 박스가 사라지지 않도록 했다.

## 검증

- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_rowbreak_chart_overlap`: 20 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_rowbreak_chart_overlap rowbreak_page17_keeps_database_separation_line_before_example_box`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1686 -- --nocapture`: 4 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1949_giant_cell_render_perf`: 1 passed

visual sweep:

- 명령:
  `python3 scripts/task1274_visual_sweep.py --key issue1999-p14-16-stage7 --file-target issue1999-hwp-stage7 samples/issue1949_giant_cell_nested_tables_perf.hwp pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf --file-target issue1999-hwpx-stage7 samples/issue1949_giant_cell_nested_tables_perf.hwpx pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf --pages 14-16 --out output/task1999_issue1999_p14_16_stage7 --rhwp-bin target/debug/rhwp`
- summary: `output/task1999_issue1999_p14_16_stage7/summary.json`
- HWP: SVG/PDF/render-tree 115/115/115쪽, 14~16쪽 `flagged=0/3`
- HWPX: SVG/PDF/render-tree 115/115/115쪽, 14~16쪽 `flagged=0/3`
- 평균 pixel match: `92.3305%`
- 평균 내용 픽셀 중심 자동 일치율 보조값: `13.64366%`

## 결론

Stage 7 보정으로 `rowbreak-problem-pages` p17 순서 역전 회귀를 복구했고, #1999의 PDF/HWPX/HWP 115쪽
정합 및 14~16쪽 visual sweep 결과도 유지된다.
