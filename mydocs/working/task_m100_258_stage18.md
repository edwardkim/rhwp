# Task M100-258 Stage 18 — 누름틀 값 복사와 경계 커서 이동

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `e9660132` (`task 258: 누름틀 복사 필드 보존`)

## 1. 문제

Stage17 보정 뒤 누름틀 컨트롤 자체는 복사되지만, 붙여넣은 누름틀의 값이 비어 보인다.
또한 한컴처럼 누름틀 이전 위치, 누름틀 내부, 누름틀 이후 위치로 커서가 이동되어야 한다.

## 2. 기준

- 누름틀 전체 선택 후 복사/붙여넣기하면 `ClickHere` 필드와 내부 텍스트 값이 함께 복제된다.
- 커서는 필드 시작 전, 필드 내부, 필드 끝 뒤 위치를 구분할 수 있어야 한다.
- Shift+방향키 또는 마우스 드래그 선택 시 한컴처럼 누름틀 텍스트 전체가 선택되어야 한다.
- Home/End는 한컴처럼 현재 줄의 첫 위치/마지막 위치로 이동하되, 누름틀이 줄 시작/끝에
  있으면 각각 누름틀 이전/이후 경계 위치로 이동할 수 있어야 한다.

## 3. 조사 계획

- `Paragraph::split_at()`이 선택 범위 클립보드의 `field_ranges`를 어떻게 자르는지 확인한다.
- `paste_internal_native()`가 붙여넣은 field range의 시작/끝 offset을 값 길이에 맞게 재배치하는지 확인한다.
- 프런트엔드 커서 이동에서 field end 위치가 항상 필드 내부로만 처리되는지 확인한다.
- `moveToLineStart()`/`moveToLineEnd()`와 Shift+방향키 선택이 field start/end 경계를 보존하는지 확인한다.

## 4. 원인

- `copy_selection_native()`와 `copy_selection_in_cell_native()`가 선택 앞부분을 자를 때
  `Paragraph::split_at(start)`를 사용했다. 이 메서드는 오른쪽 조각으로 텍스트만 넘기고
  컨트롤/field range는 왼쪽 문단에 유지하므로, 누름틀이 문단 중간에 있으면 선택 텍스트와
  `Control::Field`가 분리될 수 있다.
- 프런트엔드는 누름틀 끝에서 오른쪽 이동한 상태만 별도로 기억했다. 누름틀 시작 이전 상태와
  Home/End로 만든 줄 시작/끝 경계 상태는 없어서 같은 charOffset이 항상 필드 내부처럼 처리됐다.

## 5. 수정

- 텍스트 선택 클립보드 전용 범위 추출 헬퍼를 추가했다.
  - 선택 범위 안에 완전히 포함된 `FieldRange`만 유지한다.
  - 유지한 `Control::Field`와 `ctrl_data_records`를 새 문단 기준으로 재매핑한다.
  - 문단 중간 누름틀도 `start/end`를 0 기반 선택 범위로 보정해 값 텍스트와 함께 붙여넣는다.
- 누름틀 시작/끝 경계의 바깥 상태를 각각 `fieldStartExitKey`, `fieldEndExitKey`로 구분했다.
- 방향키는 누름틀 시작 밖/끝 밖에서 같은 charOffset의 내부 경계로 다시 들어갈 수 있게 했다.
- Home/End는 줄 시작/끝이 누름틀 경계이면 한컴처럼 누름틀 이전/이후 위치로 표시되게 했다.

## 6. 검증

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test issue_258_clickhere_form_mode copying_clickhere_after_prefix_preserves_field_value -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `cd rhwp-studio && npm run build`
- `wasm-pack build --target web --out-dir pkg`
