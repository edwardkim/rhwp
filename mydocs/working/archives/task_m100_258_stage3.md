# Task M100-258 Stage 3 — 양식 모드 누름틀 이동 UX

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `93e6172a` (`task 258: 양식 모드 MVP 구현`)

## 1. 작업 목적

Stage 2에서 양식 모드 입력 제한을 구현했다. Stage 3에서는 양식 모드에서 사용자가
키보드만으로 편집 가능한 누름틀 사이를 이동할 수 있게 한다.

## 2. 구현 범위

- 양식 모드에서 `Tab`을 누르면 다음 editable ClickHere로 이동한다.
- 양식 모드에서 `Shift+Tab`을 누르면 이전 editable ClickHere로 이동한다.
- 이동 대상은 `양식 모드에서 편집 가능` 속성이 켜진 ClickHere로 제한한다.
- 이동 후 active field와 상태 표시줄이 기존 누름틀 진입 흐름과 같은 방식으로 갱신되게 한다.

## 3. 비범위

- 새 누름틀 삽입 대화상자는 Stage 4로 유지한다.
- 한컴 `EditMode=2` HwpCtrl 호환 property는 별도 후속으로 둔다.
- 본 Stage에서는 ClickHere 외 FormObject와 editable cell 이동까지 일반화하지 않는다.

## 4. 예상 수정 파일

- `src/document_core/queries/field_query.rs`
- `src/wasm_api.rs`
- `rhwp-studio/src/core/types.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/engine/input-handler-keyboard.ts`

## 5. 검증 계획

- `cargo fmt --check`
- `cargo test --test issue_258_clickhere_form_mode`
- `npm run build`
- `git diff --check`

## 6. 진행 기록

- Stage 3 문서 작성 후 구현 착수.
- `getFieldList()` 반환 JSON에 `startCharIdx`, `endCharIdx`, `editableInForm`을 추가했다.
- field list range 계산을 위해 `FieldLocation`의 읽기 전용 문단 탐색 helper를 추가했다.
- rhwp-studio `InputHandler.moveToAdjacentFormField(delta)`를 추가했다.
  - 양식 모드에서 editable ClickHere만 후보로 사용한다.
  - 현재 필드는 후보에서 제외하고, 문서 순서상 다음/이전 필드로 이동한다.
  - 끝에 도달하면 처음/마지막 editable ClickHere로 순환한다.
- 양식 모드에서 `Tab`은 다음 editable ClickHere로, `Shift+Tab`은 이전 editable ClickHere로 이동한다.
- `tests/issue_258_clickhere_form_mode.rs`에 field list가 navigation에 필요한 range/editable 정보를 노출하는지 확인하는 검증을 추가했다.

## 7. 검증 결과

- `cargo fmt --check` — 통과
- `cargo test --test issue_258_clickhere_form_mode` — 통과
- `npm run build` (`rhwp-studio`) — 통과
- `git diff --check` — 통과

## 8. 남은 범위

- Stage 4에서 `insert:field` 스텁을 누름틀 삽입 대화상자로 교체한다.
- HwpCtrl 호환 `EditMode` property는 별도 후속으로 분리할지 Stage 4 이후 판단한다.
- FormObject/editable cell까지 양식 모드 이동 대상을 넓히는 작업은 ClickHere MVP 이후로 둔다.
