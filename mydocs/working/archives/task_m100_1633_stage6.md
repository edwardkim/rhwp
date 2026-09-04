# Task M100 #1633 Stage 6 작업 문서

## 목표

cellzone 대각선이 이미 적용된 표에서 `셀 테두리/배경 - 각 셀마다 적용`을 열 때 한컴처럼 개별 셀 대각선은 비어 보이도록 하고, 확인 시 화면이 찌그러지는 부작용을 막는다.

## 관찰

- `하나의 셀처럼 적용` 대각선은 TABLE cellzone overlay이다.
- `각 셀마다 적용` 대화상자는 개별 셀 속성을 편집해야 하므로 cellzone overlay의 대각선을 초기값으로 보여주면 안 된다.
- 현재 `CellBorderBgDialog.show()`가 항상 `getCellProperties`를 호출하고, 이 API는 Stage 1 이후 cellzone overlay를 effective 값으로 반환한다.
- 그 결과 `각 셀마다 적용` 대화상자에도 X가 보이고, 확인 시 기존 cellzone 위에 개별 셀 대각선이 덧입혀져 화면이 깨질 수 있다.

## 작업 범위

1. 개별 셀 고유 `border_fill_id` 기준 속성을 조회하는 별도 API를 추가한다.
2. `각 셀마다 적용` 대화상자는 개별 셀 속성 조회 API를 사용한다.
3. `각 셀마다 적용` 확인 시 셀 선택 범위가 있으면 선택 범위 각 셀에 개별 적용한다.
4. `하나의 셀처럼 적용` 및 effective 조회 동작은 유지한다.

## 검증 계획

- cellzone 적용 후 개별 셀 속성 조회는 대각선 없음, effective 조회는 대각선 있음을 확인하는 Rust 테스트 추가.
- `cargo test --test issue_1623_cellzone_diagonal`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `wasm-pack build --target web --out-dir pkg`

## 구현 결과

- `getCellProperties`는 기존처럼 cellzone overlay를 합성한 표시용 값을 유지했다.
- `getCellOwnProperties` WASM API를 추가해 셀 자체 `border_fill_id` 기준 속성을 조회할 수 있게 했다.
- `셀 테두리/배경 - 각 셀마다 적용` 대화상자는 고유 셀 속성을 초기값으로 사용하게 했다.
  - cellzone X 대각선이 있는 선택 영역에서도 개별 셀 대각선 방향 비트가 없으면 대화상자 미리보기에는 대각선이 보이지 않는다.
- `각 셀마다 적용` 확인 시 셀 선택 범위가 있으면 현재 커서 셀 하나가 아니라 선택 범위와 겹치는 각 셀에 개별 속성을 적용하게 했다.
- `하나의 셀처럼 적용`은 기존대로 cellzone 적용 경로를 사용한다.

## 검증 결과

- PASS: `cargo test --test issue_1623_cellzone_diagonal`
  - cellzone overlay 조회는 대각선을 반환하고, 고유 셀 조회는 개별 셀 대각선 방향 비트가 0임을 확인했다.
- PASS: `cd rhwp-studio && npx tsc --noEmit`
- PASS: `cd rhwp-studio && npm test`
- PASS: `git diff --check`
- PASS: `wasm-pack build --target web --out-dir pkg`
  - 사용자가 실행했고 `wasm-bindgen` fallback 경고 없이 완료됐다.
