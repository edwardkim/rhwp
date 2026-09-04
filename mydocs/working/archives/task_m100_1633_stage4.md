# Task M100 #1633 Stage 4 작업 문서

## 목표

여러 셀 선택 후 `셀 테두리/배경 - 하나의 셀처럼 적용`에서 대각선을 적용했을 때, 한컴처럼 선택 영역 전체를 하나의 셀처럼 가로지르는 대각선으로 표시되게 한다.

## 관찰

- 한컴은 여러 셀 선택 상태에서 `하나의 셀처럼 적용`을 선택한 뒤 대각선 X를 적용하면 선택 영역 전체의 외곽 기준으로 대각선을 표시한다.
- 현재 rhwp는 동일 동작 후 선택 영역의 첫 셀 쪽에만 대각선이 표시된다.
- 작업지시자가 한컴/rhwp 결과의 시각 차이를 확인했다.

## 작업 범위

1. rhwp-studio `CellBorderBgDialog`의 `asOne` 적용 payload를 확인한다.
2. Rust table command에서 `asOne`/cellzone 적용 시 대각선 속성이 선택 영역 전체 cellzone에 보존되는지 확인한다.
3. 필요한 경우 `asOne` 적용은 개별 셀 대각선이 아니라 cellzone overlay로 처리해 렌더러가 영역 bbox 기준으로 그리도록 보정한다.
4. 기존 `각 셀마다 적용` 동작은 유지한다.

## 검증 계획

- 관련 Rust 단위/통합 테스트 추가 또는 갱신.
- `cargo test --test issue_1623_cellzone_diagonal`
- `cd rhwp-studio && npx tsc --noEmit`
- 필요 시 `cd rhwp-studio && npm test`
- WASM 갱신이 필요한 경우 `wasm-pack build --target web --out-dir pkg`

## 구현 결과

- Rust core에 `set_cell_zone_properties_native`를 추가했다.
  - 선택 영역 좌표를 표 범위로 clamp한다.
  - 동일 범위 cellzone이 있으면 `border_fill_id`를 갱신하고, 없으면 새 `TableZone`을 추가한다.
  - `create_border_fill_from_json`을 재사용해 테두리/배경/대각선 속성을 cellzone overlay의 BorderFill로 저장한다.
- WASM API `setCellZoneProperties`와 rhwp-studio `WasmBridge.setCellZoneProperties`를 추가했다.
- `CellBorderBgDialog`는 `applyMode === 'asOne'`일 때 선택 범위 전체에 cellzone을 적용한다.
  - `scope=selected`: F5 셀 선택 범위.
  - `scope=all`: 표 전체 범위.
  - 기존 `각 셀마다 적용` 경로는 유지한다.
- `table:border-one` 실행 시 현재 선택 범위를 dialog에 전달한다.
- `issue_1633_as_one_cell_diagonal_uses_cellzone_range` 회귀 테스트를 추가했다.

## 검증 결과

- `cargo fmt --check` 통과.
- `cargo test --test issue_1623_cellzone_diagonal` 통과: 5개.
- `cd rhwp-studio && npx tsc --noEmit` 통과.
- `cd rhwp-studio && npm test` 통과: 147개.
- `wasm-pack build --target web --out-dir pkg` 통과.
  - 이 환경용 prebuilt `wasm-bindgen`이 없어 cargo install fallback 경고가 있었으나 빌드는 완료됐다.
- `cd rhwp-studio && npm run build` 통과.
- Playwright runtime smoke (`http://127.0.0.1:7700/`) 통과.
  - `setCellZoneProperties` 호출 결과: `startRow=0,startCol=0,endRow=1,endCol=1`.
  - zone 내부 셀 `getCellProperties`가 `diagonalLine=1`, `diagonalSlash=2`, `diagonalBackSlash=2`를 반환했다.
  - 렌더 SVG에서 2×2 cellzone bbox를 가로지르는 긴 대각선을 확인했다.
