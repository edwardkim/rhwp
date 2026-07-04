# Task M100 #1633 Stage 9 - 각 셀마다 대각선 UI 적용 렌더링 보정

## 배경

- Stage 8 커밋 `afecce80f` 이후 사용자가 7700 화면에서 다시 확인했다.
- 대화상자 미리보기에서는 대각선 X가 선택되어 보이지만, 확인 후 본문 렌더링에는 선택 셀 대각선이 적용되지 않았다.
- 이후 사용자가 중심선 CROSS도 미리보기에는 표시되지만 본문 렌더링에는 적용되지 않는다고 추가 확인했다.
- 이후 렌더링은 되지만 HWP 파일로 저장 후 한컴에서 열면 중심선이 보이지 않는다고 추가 확인했다.
- 사용자는 Stage 8을 커밋한 뒤 다음 스테이지에서 이 문제를 수정하라고 지시했다.

## 관측

- 현재 회귀 테스트는 Rust API에서 `set_cell_properties(..., cell_idx=1, diagonal_props)` 를 직접 호출하는 경로만 검증한다.
- 사용자 재현은 rhwp-studio UI 경로다.
  - 컨텍스트 메뉴 `셀 테두리/배경 - 각 셀마다 적용`
  - 대각선 탭에서 X 선택
- `선택된 셀` 범위로 확인
- 따라서 남은 결함은 렌더러 자체보다 UI 선택 범위/대화상자 상태/wasm 호출 payload 중 하나일 가능성이 높다.
- `cell_idx=0` 은 전체 cellzone 대각선의 시작 셀이라 Stage 8의 중복 억제 플래그에 의해 셀 고유 대각선 렌더링이 숨겨졌다.
- 중심선도 같은 억제 플래그에 걸려 개별 적용 결과가 본문에 렌더되지 않았다.
- `/Users/tsjang/Downloads/test.hwp` 와 `samples/대각선샘플3.hwp` 비교 결과:
  - 대각선샘플3은 전체 cellzone + 셀[0] BF 중심선으로 저장되고, 1x1 중심선 cellzone은 없다.
  - 한컴 저장본 셀 LIST_HEADER는 47바이트이며, 새 rhwp 저장본은 34바이트였다.
  - 한컴 저장본의 추가 13바이트는 `셀 폭(u32) + 9바이트 0` 형태다.

## 목표

- UI 경로에서 `각 셀마다 적용`으로 선택한 개별 셀 대각선이 본문 렌더링에 반영되게 한다.
- 같은 경로로 선택한 개별 셀 중심선도 본문 렌더링에 반영되게 한다.
- cellzone 전체 X는 유지한다.
- 한컴 저장본에서 들어온 원본 중복 대각선 억제 정책은 유지하되, 사용자가 명시 적용한 셀 표시는 보존한다.
- HWP 저장 구조도 cellzone BF와 개별 셀 BF가 분리되도록 유지한다.
- HWP 저장본은 대각선샘플3처럼 중심선을 1x1 cellzone이 아니라 셀 BF로 보존한다.

## 처리

- `set_cell_properties_native` 에서 전체 대각선 cellzone 시작 셀에 사용자가 개별 대각선을 적용하면 1x1 cellzone override를 추가하도록 했다.
- 중심선은 한컴 저장 호환성을 위해 1x1 cellzone을 만들지 않고 셀 BF 자체로 유지한다.
- 렌더러는 cellzone origin 억제 중에도 셀 BF가 중심선-only이면 렌더하도록 조정했다.
- HWP serializer는 raw 확장 정보가 없는 새 표 셀도 한컴식 47바이트 LIST_HEADER로 저장하도록 `width_ref=0x0400`, `cell.width + 9바이트 0` 확장 영역을 보강했다.
- 대각선 해제 입력은 1x1 대각선 override를 제거해 기존 억제 정책을 유지한다.
- `issue_1633_cellzone_origin_cell_diagonal_renders_after_each_cell_apply` 테스트를 추가했다.
- `issue_1633_cellzone_origin_centerline_renders_after_each_cell_apply` 테스트를 추가했다.

## 검증 계획

- UI payload/선택 범위 경로 점검.
- 필요한 경우 Rust/WASM API 회귀 테스트 추가.
- `cargo test --test issue_1623_cellzone_diagonal`
- `cargo fmt --check`
- `git diff --check`
- `wasm-pack build --target web --out-dir pkg`
- `npm --prefix rhwp-studio run build`

## 검증 결과

- `cargo test --test issue_1623_cellzone_diagonal issue_1633_cellzone_origin_cell_diagonal_renders_after_each_cell_apply -- --nocapture`: 통과
- `cargo test --test issue_1623_cellzone_diagonal issue_1633_cellzone_origin_centerline_renders_after_each_cell_apply -- --nocapture`: 통과
- `cargo test --test issue_1623_cellzone_diagonal`: 통과, 19 passed
- `cargo test --lib serializer::control`: 통과, 10 passed
- `cargo test --test hwpx_to_hwp_adapter hwp5_cell_header -- --nocapture`: 0 tests matched
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
  - 현재 Codex 실행 환경에서는 `wasm-bindgen` prebuilt 미제공 경고 후 `cargo install` fallback이 발생했다.
- `npm --prefix rhwp-studio run build`: 통과
  - 기존 CanvasKit browser compatibility 경고와 chunk size 경고만 확인.
- 7700 브라우저 재로드: `rhwp-studio` 로드, warn/error 로그 없음
