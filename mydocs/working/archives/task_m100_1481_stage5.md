# Task M100 #1481 Stage 5

- 이슈: #1481 표 줄/칸 편집 회귀 및 표 키보드 탈출 보정
- 브랜치: `task_m100_1481`
- 작성일: 2026-06-22
- 상태: 구현/검증 중

## 증상

표 생성 후 행 높이를 크게 조절한 상태에서 커서가 표 내부에 있으면 사용자가 표 밖으로 빠져나가기 어렵다.

한컴처럼 첫 줄에 표가 만들어지면서 첫 줄의 조판부호가 표 앞 위치로 남아 있어야 한다. 이 조판부호가 사라지거나, 별도 빈 줄로 분리되어 표가 아래로 밀리면 첫 셀에서 `왼쪽`/`위쪽`/마우스 클릭으로 표 밖 위치를 잡기 어렵다.

또한 `create_table_native()`는 표 뒤에 빈 문단을 만들고 있지만, 키보드 입력 흐름에서 `Tab`이 표 셀 이동이 아니라 탭 문자 삽입으로 처리되어 표 마지막 셀에서 다음 문단으로 나가는 경로가 노출되지 않는다.

추가 확인 결과, 기본 `자리 차지` 표는 데이터상 같은 문단에 생성되어도 렌더러의 `FullParagraph` 경로가 빈 host 문단 줄을 먼저 그려 표를 다음 y로 밀 수 있었다. 이 경우 화면에는 표 위에 별도 조판부호 줄이 남고, 한컴처럼 첫 조판부호가 표 상단과 겹쳐 보이지 않는다.

또한 상세 `표 만들기...` 대화상자의 `글자처럼 취급` 기본값이 켜져 있어, 한컴 기본 표 속성(해제)과 달리 TAC 표 생성 경로를 탈 수 있었다.

## 원인

`CursorState`에는 이미 셀 읽기 순서에 따라 다음/이전 셀로 이동하고, 마지막/첫 셀 경계에서는 `exitTable()`로 표 밖 문단으로 이동하는 `moveToCellNext()`/`moveToCellPrev()`가 있다.

하지만 `input-handler-keyboard.ts`의 `Tab` 처리에서는 표 셀 여부를 확인하지 않고 본문·표 셀·글상자 공통으로 `InsertTabCommand`를 실행한다. 따라서 표 셀 내부에서 표 이동/탈출용 `Tab` 경로가 동작하지 않는다.

첫 셀에서 뒤로 나가는 경계도 이전 문단 끝 또는 구역 시작 실패로 처리되어, 새 문서 첫 줄 표에서는 같은 표 문단의 시작 조판부호 위치로 나가지 못한다.

## 구현 방향

- `create_table_native()`는 새 문서 첫 빈 문단을 별도 줄로 남기지 않고 표 문단으로 사용한다.
- 표 문단의 시작 위치(`paragraphIndex = table para`, `charOffset = 0`)를 첫 줄 조판부호/표 밖 커서 위치로 유지한다.
- 첫 셀에서 `왼쪽`/`Shift+Tab`/`위쪽`으로 나갈 때 같은 표 문단의 시작 위치로 이동한다.
- 폼 필드 이동은 기존처럼 최우선 유지한다.
- 커서가 표 셀 내부이고 글상자 내부가 아니면 `Tab`은 `moveToCellNext()`, `Shift+Tab`은 `moveToCellPrev()`로 연결한다.
- 본문과 글상자에서는 기존 `Shift+Tab` 내어쓰기, `Tab` 문자 삽입 동작을 유지한다.
- 회귀 테스트로 `Tab` 처리 순서가 표 셀 이동을 탭 문자 삽입보다 먼저 수행하는지 고정한다.
- 빈 TopAndBottom/문단 기준 표 host 문단은 `FullParagraph`에서 별도 빈 줄로 진행하지 않는다.
- 표 렌더 시 host 문단부호를 표 시작 y에 직접 렌더링해 첫 조판부호가 표와 겹쳐 보이게 한다.
- 상세 표 만들기 대화상자의 기본 `글자처럼 취급`은 한컴 기본값처럼 해제한다.
- `resize_table_cells_native()`의 표시 height 보존은 여러 행 표에만 적용해, 1행 표의 `table.common.height == cell.height` 회귀 규칙을 유지한다.

## 검증 계획

```bash
cargo test --release issue_1481 --lib
cargo test --release document_core::commands::object_ops::issue_1151_v2_tac_toggle_tests::v6_resize_cell_then_tac_toggle_picture_below_table --lib
cargo test --release --lib
cd rhwp-studio && node --test tests/table-keyboard-navigation.test.ts
cd rhwp-studio && node --test tests/table-create-dialog.test.ts
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
git diff --check
```

## 현재 검증 결과

- `cargo fmt`: 통과
- `cargo test --release issue_1481 --lib`: 통과 (6 passed)
- `cargo test --release document_core::commands::object_ops::issue_1151_v2_tac_toggle_tests::v6_resize_cell_then_tac_toggle_picture_below_table --lib`: 통과
- `cargo test --release --lib`: 통과 (1920 passed, 6 ignored)
- `cd rhwp-studio && node --test tests/table-keyboard-navigation.test.ts tests/table-create-dialog.test.ts`: 통과 (3 passed)
- `wasm-pack build --target web --out-dir pkg`: 통과
