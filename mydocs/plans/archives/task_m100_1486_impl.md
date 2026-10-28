# Task M100 #1486 구현 계획서

## 목표

HWPX 9쪽 분할 표 안의 TAC 중첩 표가 앞 텍스트 뒤 남은 줄에 들어가지 못할 때, 본문 오른쪽으로 밀려나지 않고 다음 줄의 셀 좌측 기준으로 배치되도록 한다.

## 대상 코드

- `src/renderer/layout/table_partial.rs`
  - 분할 표 셀 내부 `Control::Table(nested_table)` 처리
- 테스트 추가 후보:
  - `tests/issue_1486_hwpx_partial_tac_table.rs`

## 구현 방향

### 1. 분할 표 경로의 TAC 중첩 표 배치 보정

현재 partial 경로는 TAC 중첩 표일 때 앞 텍스트 전체 폭을 `tac_text_offset`으로 계산하고, `ctrl_area.x = inner_area.x + tac_text_offset`으로 표를 배치한다.

보정 방향:

- TAC 표 폭 `table_w`를 계산한다.
- 앞 텍스트 폭 `text_w`를 계산한다.
- `text_w + table_w <= inner_area.width + epsilon`이면 기존처럼 같은 줄 배치를 유지한다.
- 초과하면 표를 다음 줄로 본다.
  - `ctrl_area.x = inner_area.x`
  - `ctrl_area.width = inner_area.width`
  - `ctrl_area.y`는 해당 문단의 다음 line segment 또는 표가 속한 line segment 기준으로 내린다.

이 샘플에서는 `text_w + table_w`가 셀 내부 폭을 초과하므로 표가 좌측 본문 안으로 돌아와야 한다.

### 2. 일반 표 경로와의 차이 축소

일반 표 경로(`table_layout.rs`)는 `Control::Table`의 TAC 여부를 먼저 나누고, TAC 표를 `inline_x`/`inline_x_override` 기반으로 배치한다. partial 경로도 최소한 다음 규칙은 맞춘다.

- TAC 표는 앞 텍스트 뒤에 무조건 붙이지 않는다.
- 줄 폭 초과 시 줄바꿈된 TAC처럼 좌측 기준을 사용한다.
- 비-TAC 중첩 표 경로는 기존 동작을 유지한다.

### 3. 회귀 테스트

테스트는 `samples/hwpx_sample2.hwpx` 9쪽 render tree를 사용한다.

검증 조건:

- 페이지 9 body bbox 오른쪽 경계보다 넓게 나가는 `Table` 노드가 없어야 한다.
- 특히 `pi=74` 외곽 표 내부의 3×2 또는 2×? 조회방법 표가 `x < 120px` 범위에서 시작하고, `x + w <= body_right + tolerance`를 만족해야 한다.

테스트는 PDF 픽셀 비교가 아니라 render tree bbox 계약으로 고정한다.

### 4. 시각 검증

수정 후 다음 산출물을 만든다.

```text
output/poc/task1486/after_rt/render_tree_009.json
output/poc/task1486/after_svg/hwpx_sample2_009.svg
```

PDF 정답지 `pdf/hwpx_sample2-2024.pdf` 9쪽 PNG와 나란히 확인한다.

## 위험 및 회귀 관찰점

- 셀 내부 TAC 표가 실제로 같은 줄에 들어가는 문서에서는 기존 inline 배치가 유지되어야 한다.
- `tests/issue_1195_cell_table_empty_line.rs`, `tests/issue_1285_tac_sequence_right_align.rs` 등 TAC 표/인라인 표 관련 테스트가 깨지지 않아야 한다.
- 분할 표 row cut 관련 테스트(`issue_1073`, `issue_1156_rowbreak_fragment_fit`)도 영향을 받을 수 있으므로 focused 후 관련 테스트를 함께 확인한다.

## 1차 검증 명령

```text
cargo test --release --test issue_1486_hwpx_partial_tac_table
cargo test --release --test issue_1195_cell_table_empty_line
cargo test --release --test issue_1285_tac_sequence_right_align
cargo test --release --test issue_1156_rowbreak_fragment_fit
```

이후 stage 검증에서 `cargo test --release --lib`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`를 수행한다.
