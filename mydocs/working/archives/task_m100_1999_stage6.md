# #1999 Stage 6 - issue1686 회귀 보정

## 목적

Stage 5 이후 `cargo test --profile release-test --tests`를 실행하던 중 `tests/issue_1686.rs`의
HWPX 페이지 수 회귀를 발견했다.

- 기대: `samples/hwpx/pr-1674.hwpx` 35쪽
- 회귀: 36쪽
- `upstream/devel` 기준 같은 테스트는 통과하므로 현재 브랜치에서 도입된 회귀다.

## 원인

Stage 1에서 RowBreak hard break 완화 조건을 다음처럼 과도하게 제한했다.

- HWPX 소스이면 완화하지 않음
- TopAndBottom flow가 있는 RowBreak 셀이면 완화하지 않음

issue1686의 HWPX 샘플은 HWPX라는 이유만으로 저장 `LineSeg` hard break가 너무 보수적으로 보존되어
1쪽이 늘었다. 반면 issue1999 샘플은 TopAndBottom flow가 있는 RowBreak 거대 셀이므로 완화를 그대로
허용하면 114쪽으로 줄어든다.

따라서 분기 근거는 파일 포맷(HWPX)이 아니라 문서 구조인 TopAndBottom flow 존재 여부여야 한다.

## 구현

`advance_row_cut` / `advance_row_block_cut`의 `relaxed_hard_break` 조건에서 `!self.is_hwpx_source` 제한을 제거하고,
`!row_has_top_and_bottom_flow` / `!block_has_top_and_bottom_flow` 제한만 유지했다.

## 검증

- `env CARGO_INCREMENTAL=0 cargo test --test issue_1686 -- --nocapture`: 4 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1949_giant_cell_render_perf`: 1 passed

## 결론

issue1686 HWPX 35쪽 회귀가 복구되었고, issue1999 PDF/HWPX/HWP 115쪽 정합도 유지된다.
