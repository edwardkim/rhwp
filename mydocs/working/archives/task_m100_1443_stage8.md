# Task M100 #1443 Stage 8 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `aa03cf43 task 1443: 표 선택 후 셀 편집 진입 복구`

## 1. Stage 8 목표

셀 블록 선택 상태에서 단축키로 셀 높이를 조정할 때 한컴과 다르게 보이는 문제를 수정한다.

사용자 제보 화면 기준으로 rhwp-studio에서는 전체 셀 블록 선택 후 셀 높이를 키보드로 조정할 때 선택된 모든 행이 같이 커지지 않는다.
한컴은 전체 셀 블록이 선택된 상태에서 높이를 조정하면 선택된 셀들이 모두 같은 방향으로 조정되고, 선택 블록 표시도 유지된다.

## 2. 확인할 내용

- 현재 `Alt+ArrowDown`/`Ctrl+ArrowDown` 셀 높이 조정이 선택 범위 전체에 적용되는지 확인한다.
- `table:cell-height-equal`이 선택 범위의 전체 높이를 보존하는지 함께 확인한다.
- 실행 후 셀 선택 상태가 유지되는지 확인한다.
- `셀 너비를 같게`와 구현이 비슷하므로 함께 회귀 여부를 확인한다.

## 3. 회귀 방지 기준

- 단축키 셀 높이 조정은 선택된 모든 셀 높이를 같은 delta로 조정해야 한다.
- 선택 범위 내부 이웃 행/열이 반대 delta로 상쇄되면 안 된다.
- 실행 후 셀 선택 블록이 유지되어야 한다.
- 병합/제외 셀이 있는 비직사각형 선택은 기존처럼 실행하지 않는다.

## 4. 원인

`resizeCellByKeyboard`가 선택된 각 셀에 delta를 적용한 뒤, 같은 선택 범위 안에 있는 오른쪽/아래쪽 이웃 셀에도 반대 delta를 넣고 있었다.
전체 표를 셀 블록으로 선택한 뒤 `Alt+ArrowDown`을 누르면 첫 행은 `+300`이 적용되지만, 나머지 행은 위쪽 행의 이웃 보정 `-300`과 자기 자신의 `+300`이 상쇄되어 높이가 그대로 남았다.

## 5. 구현 결과

- 셀 선택 범위 내부 이웃에 반대 delta를 넣는 보정을 제거했다.
- 선택된 셀은 중복 없이 한 번만 업데이트하도록 `updatedCells` 집합을 추가했다.
- `Alt+ArrowDown` 기준 전체 셀 블록의 모든 셀이 같은 delta로 증가하도록 했다.

## 6. 검증 결과

- 수정 전 `node /private/tmp/rhwp_1443_cell_height_keyboard_check.mjs` 실패 확인.
  - 전체 5x5 선택 후 `Alt+ArrowDown` 실행 시 첫 행만 `1200 -> 1500`, 나머지 행은 `1200` 유지.
- 수정 후 `node /private/tmp/rhwp_1443_cell_height_keyboard_check.mjs` 통과.
  - 전체 5x5 선택 후 25개 셀이 모두 `1200 -> 1500`.
  - 셀 선택 모드와 25개 하이라이트 유지.
- `node /private/tmp/rhwp_1443_cell_height_equal_check.mjs` 통과.
  - `셀 높이를 같게`는 선택 범위 전체 높이 합을 유지하며 5개 행을 `1340`으로 균등화.
  - 셀 선택 모드와 25개 하이라이트 유지.
- `node /private/tmp/rhwp_1443_cell_drag_check.mjs` 통과.
  - F5 없이 셀 드래그 선택, 선택 후 키보드 폭 조정, 같은 셀 텍스트 드래그 선택 유지 확인.
- `node /private/tmp/rhwp_1443_protected_cell_drag_check.mjs` 통과.
  - 보호 셀 클릭 후 드래그 선택 유지 확인.
- `cd rhwp-studio && npx tsc --noEmit` 통과.
- `cd rhwp-studio && npm test` 통과.
- `cd rhwp-studio && npm run build` 통과.
- `git diff --check` 통과.

최종 PR 준비 단계가 아니므로 `cargo clippy --all-targets -- -D warnings`는 수행하지 않았다.
