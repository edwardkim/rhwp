# Task M100 #1633 Stage 8 - 셀존 위 개별 셀 대각선 렌더링 보정

## 배경

- Stage 7 에서 전체 cellzone 대각선과 개별 셀 대각선/중심선의 중복 렌더링을 억제했다.
- 사용자가 전체 cellzone 대각선이 있는 표의 1행 2열에 개별 대각선을 적용했을 때, 대화상자 미리보기에는 표시되지만 문서 화면에는 렌더링되지 않는다고 보고했다.

## 관측

- 현 렌더러는 cellzone 대각선을 렌더한 뒤 해당 영역 전체를 `cellzone_diagonal_covered` 로 표시한다.
- 이후 셀 레이아웃 단계에서 coverage 안의 셀 고유 대각선을 기본적으로 숨긴다.
- 이 정책은 기존 HWP 샘플의 중복 대각선 억제에는 맞지만, 사용자가 나중에 개별 셀에 적용한 대각선까지 숨기는 부작용이 있다.
- `samples/대각선샘플4.hwpx` 기준으로 전체 cellzone 은 BF#4, 시작 셀(0,0)은 BF#5 로 같은 X 대각선을 가진다. 이 시작 셀 BF 는 한컴 저장본의 중복 표현이라 숨겨야 한다.
- 반면 사용자가 나중에 1행 2열(cell index 1)에 대각선을 적용하면 별도 셀 BF 가 실제 편집 결과이므로 cellzone 위에 렌더링되어야 한다.

## 목표

- 전체 cellzone 대각선은 유지한다.
- cellzone 위에 나중에 적용한 개별 셀 대각선은 렌더링한다.
- 기존 비교군에서 불필요한 첫 셀 중복 대각선은 다시 나타나지 않게 한다.

## 구현

- `cellzone_diagonal_covered` 의 의미를 전체 cellzone 영역에서 cellzone 시작 셀 suppress 표식으로 좁혔다.
- cellzone 대각선 자체는 계속 전체 영역에 렌더링한다.
- 시작 셀의 중복 BF 만 숨기고, 1행 2열처럼 이후 편집된 개별 셀 대각선은 셀 레이아웃 단계에서 렌더링한다.
- `issue_1633_cell_diagonal_renders_over_existing_cellzone_diagonal` 회귀 테스트를 추가했다.

## 검증 계획

- `cargo test --test issue_1623_cellzone_diagonal`
- `cargo fmt --check`
- `git diff --check`
- `wasm-pack build --target web --out-dir pkg`

## 검증 결과

- `cargo test --test issue_1623_cellzone_diagonal issue_1633_cell_diagonal_renders_over_existing_cellzone_diagonal -- --nocapture` 통과.
- `cargo test --test issue_1623_cellzone_diagonal issue_1633_sample4_renders_cellzone_diagonal_without_cell_overlay -- --nocapture` 통과.
- `cargo test --test issue_1623_cellzone_diagonal` 통과: 16 passed.
- `cargo fmt --check` 통과.
- `git diff --check` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
  - 현재 Codex 실행 환경에서는 `wasm-bindgen` prebuilt 미지원 경고 후 `cargo install` fallback 으로 완료됐다.

## 추가 관측 - 대각선샘플5 vs Downloads/test.hwp

- `samples/대각선샘플5.hwp`:
  - 10행 x 10열, cellzone `0..9 x 0..9`, zone BF#4.
  - 셀[0] BF#6, 셀[1] BF#5, 나머지 기본 BF#3.
  - BF#4 는 zone용 대각선 BF, BF#5 는 1행 2열 셀용 대각선 BF, BF#6 은 1행 1열 중심선 BF.
- `/Users/tsjang/Downloads/test.hwp`:
  - 8행 x 10열, cellzone `0..7 x 0..9`, zone BF#4.
  - 셀[0] BF#5, 셀[1] BF#4.
  - generated BF#4 payload가 샘플5의 셀용 BF#5와 같아서, zone BF가 셀 테두리 포함 대각선 BF로 저장되어 있다.
- `rhwp convert samples/대각선샘플5.hwp output/task1633_stage8_convert/sample5_resave.hwp` 무편집 재저장은 행 수, cellzone, BF#4/#5/#6 payload를 보존했다.
- 따라서 차이는 일반 저장기 전체 문제가 아니라, UI 편집 후 선택 셀 속성 적용 경로에서 cellzone BF와 셀 BF를 분리하지 못한 결과로 추정된다.

## 추가 구현 - cellzone BF 오염 방지

- `setCellProperties` 에 방어 로직을 추가했다.
  - incoming `borderFillId` 가 대상 셀을 덮는 cellzone BF이고, 대상 셀의 own BF와 다르면 own BF를 base로 사용한다.
  - incoming border JSON 이 zone BF와 같으면 own BF의 셀 테두리 값을 복구한 뒤 새 BF를 만든다.
  - `borderFillId` 단독 변경도 대상 셀을 덮는 cellzone BF를 직접 셀 BF로 저장하지 않게 했다.
- `format copy` 의 셀 속성 복사 경로도 `getCellProperties` 에서 `getCellOwnProperties` 로 변경했다.
  - cellzone overlay 표시값을 셀 고유 서식으로 복사해 저장하는 것을 막는다.
- `samples/대각선샘플5.hwp/.hwpx` 를 비교군으로 추가하고, zone BF와 1행 2열 셀 BF가 분리되어야 한다는 회귀 테스트를 추가했다.

## 추가 검증 결과

- `cargo test --test issue_1623_cellzone_diagonal issue_1633_cell_edit_does_not_store_cellzone_borderfill_as_cell_borderfill -- --nocapture` 통과.
- `cargo test --test issue_1623_cellzone_diagonal` 통과: 18 passed.
- `npm --prefix rhwp-studio run build` 통과.
  - 기존 CanvasKit `fs/path` externalized 경고 및 chunk size 경고만 발생.
- `cargo fmt --check` 통과.
- `git diff --check` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
  - 현재 Codex 실행 환경에서는 기존과 동일하게 `wasm-bindgen` prebuilt 미지원 경고 후 `cargo install` fallback 으로 완료됐다.

## 추가 관측 - 2026-06-29 저장본

- 사용자가 `/Users/tsjang/Downloads/test.hwp` 를 한컴 Viewer에서 확인한 결과, rhwp 화면과 저장 후 한컴 표시가 여전히 다르다고 보고했다.
- 최신 저장본 구조:
  - 8행 x 9열, cellzone `0..7 x 0..8`, zone BF#4.
  - 셀[0] BF#5, 셀[1] BF#3.
  - 즉 샘플5처럼 1행 2열 개별 대각선 BF가 있는 상태가 아니라, 전체 cellzone X 위에 1행 1열 중심선 BF만 있는 상태였다.
- 한컴 샘플3/5의 중심선 BF는 HWP5 attr `0x2300` 이고 HWPX 에서는 `centerLine="VERTICAL"` + `<hh:slash Crooked="3">` 로 저장되어 있다.
- rhwp 가 새로 만든 중심선 BF는 `CROSS` 입력 시 HWP5 attr `0x2000` 으로 저장되어 한컴 샘플의 중심선 보조 비트와 달랐다.
- 렌더러도 cellzone 시작 셀 suppress 표식에서 중심선만 예외로 풀고 있어, 한컴 Viewer에서 보이지 않는 시작 셀 중심선이 rhwp 화면에는 보일 수 있었다.

## 추가 구현 - 중심선 HWP5/HWPX 호환 보정

- `CenterLine::hwp_attr_bits()` 와 `hwp_binary_attr_bits()` 를 한컴 샘플 기준으로 조정했다.
  - `VERTICAL`: `0x2300`
  - `CROSS`: `0x2700`
- HWP/HWPX 직렬화와 HTML table import 의 중심선 정규화 마스크가 slash Crooked bit9까지 지우도록 보정했다.
  - 기존 마스크는 bit8만 지워 stale bit9가 남을 수 있었다.
- HWPX parser/serializer 테스트를 한컴 샘플의 `Crooked="3"` 기준으로 갱신했다.
- 렌더러에서 cellzone 시작 셀은 중심선도 한컴처럼 억제한다.
  - cellzone 전체 X는 유지한다.
  - cellzone 시작 셀의 중복/중심선은 숨긴다.
  - 1행 2열처럼 별도 셀에 나중에 적용한 개별 대각선은 계속 렌더링한다.

## 추가 검증 결과 - 2026-06-29

- `cargo test --test issue_1623_cellzone_diagonal issue_1633_cellzone_origin_centerline_is_hidden_like_hancom -- --nocapture` 통과.
- `cargo test --lib test_serialize_border_fill_cross_centerline_uses_hwp5_center_bits -- --nocapture` 통과.
- `cargo test --lib test_center_line_vertical_sets_attr_and_direction -- --nocapture` 통과.
- `cargo test --lib write_border_fill_preserves_center_line_type -- --nocapture` 통과.
- `cargo test --test issue_1623_cellzone_diagonal` 통과: 18 passed.
- `cargo fmt --check` 통과.
- `git diff --check` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
  - Codex 실행 환경에서는 기존과 동일하게 `wasm-bindgen` prebuilt 미지원 경고 후 `cargo install` fallback 으로 완료됐다.
- `npm --prefix rhwp-studio run build` 통과.
  - 기존 CanvasKit `fs/path` externalized 경고 및 chunk size 경고만 발생.
