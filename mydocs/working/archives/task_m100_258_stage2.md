# Task M100-258 Stage 2 — 양식 모드 MVP 구현

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `aeec7be1` (`task 258: 누름틀 양식 모드 착수 계획 문서화`)

## 1. 작업 목적

기존 ClickHere 누름틀 문서를 대상으로 rhwp-studio의 양식 모드 MVP를 구현한다.

양식 모드에서는 `양식 모드에서 편집 가능` 속성을 가진 ClickHere 내부 텍스트만 수정할 수
있고, 일반 본문과 문서 구조를 바꾸는 명령은 차단한다.

## 2. 구현 범위

이번 Stage 2에서 구현한다:

- rhwp-studio 편집 모드 상태 추가
- 양식 모드 토글 명령 추가
- 현재 커서의 ClickHere가 양식 모드에서 편집 가능한지 판정
- 직접 키 입력/IME 입력/Backspace/Delete/Enter/Tab/paste 경로 가드
- `field:remove`와 주요 문서 변경 명령 가드
- field info JSON에 editable 여부 노출

이번 Stage 2에서 제외한다:

- 새 누름틀 삽입 대화상자
- 사용자 정보/문서 요약/작성한 날짜/파일 이름·경로 필드 동적 갱신
- 모든 양식 개체의 완전한 입력 정책
- Tab 기반 다음 누름틀 이동 UX 완성

## 3. 예상 수정 파일

- `src/document_core/queries/field_query.rs`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/command/types.ts`
- `rhwp-studio/src/command/dispatcher.ts`
- `rhwp-studio/src/command/commands/edit.ts`
- `rhwp-studio/src/command/commands/view.ts`
- `rhwp-studio/src/main.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/src/engine/input-handler-text.ts`

## 4. 정책

허용:

- 양식 모드 OFF의 기존 편집 흐름
- 양식 모드 ON에서 editable ClickHere 내부 문자 입력
- 양식 모드 ON에서 editable ClickHere 내부 Backspace/Delete
- 양식 모드 ON에서 editable ClickHere 내부 복사/선택

차단:

- 일반 본문 입력
- editable이 아닌 ClickHere 수정
- 필드 밖 선택 삭제
- 누름틀 삭제
- 문단 분할/병합
- Tab 문자 삽입
- 붙여넣기/잘라내기
- 표/그림/도형/수식 삭제 또는 이동
- 일반 삽입/서식 변경 명령

## 5. 검증 계획

- `git diff --check`
- `cargo fmt --check`
- ClickHere 관련 focused test
- `cargo test --test issue_838_field_set_value`
- `npm run build` 또는 최소 `npx tsc --noEmit` 성격의 스튜디오 타입 검증

## 6. 진행 기록

- Stage 2 문서 작성 후 구현 착수.
- `getFieldInfoAt*` 반환 JSON에 `editableInForm`을 추가했다.
- rhwp-studio `EditorContext`에 `editMode`, `isFormMode`, `canEditFormField`를 추가했다.
- `view:form-mode` 명령을 추가해 스튜디오 런타임 양식 모드를 토글할 수 있게 했다.
- `CommandDispatcher`에서 양식 모드 중 문서 구조 변경 명령을 차단했다.
- `InputHandler`에 양식 모드 정책 helper를 추가했다.
  - editable ClickHere 범위 안의 `insertText`/`deleteText`/`deleteSelection`만 허용한다.
  - snapshot 기반 붙여넣기/개체 삭제/표 삭제 등은 차단한다.
  - IME/raw insert/delete 경로도 같은 정책을 적용한다.
- `field:edit`, `field:remove`, cut/paste/delete, format/insert/table/page 계열 명령은 양식 모드에서
  비활성 또는 차단되도록 정리했다.
- 작업지시자가 추가한 누름틀 샘플 파일을 확인했다.
  - `samples/누름틀-2024.hwp`
  - `samples/누름틀-2024.hwpx`
  - `pdf/누름틀-2024.pdf`
- 새 HWPX 샘플의 ClickHere 2개는 모두 `editable="1"`이다.
  - `<hp:fieldBegin id="1115866406" type="CLICK_HERE" name="" editable="1" dirty="1" ...>`
  - `<hp:fieldBegin id="1115866407" type="CLICK_HERE" name="" editable="1" dirty="1" ...>`
- 샘플 기반 회귀 테스트 `tests/issue_258_clickhere_form_mode.rs`를 추가했다.

## 7. 검증 결과

- `cargo fmt --check` — 통과
- `cargo test --test issue_258_clickhere_form_mode` — 통과
- `cargo test --test issue_838_field_set_value` — 통과
- `cargo test --lib test_task231_get_click_here_props -- --nocapture` — 통과
- `npm run build` (`rhwp-studio`) — 통과
- `git diff --check` — 통과

## 8. 남은 범위

- Stage 3에서 양식 모드 중 Tab/F11 기반 다음 편집 가능 누름틀 이동 UX를 정리한다.
- Stage 4에서 `insert:field` 스텁을 실제 누름틀 삽입 대화상자로 바꾼다.
- HWPX 샘플에서 `fieldBegin id`는 서로 다르지만 `fieldid`가 같은 점을 관찰했다. Stage 2의
  editable 판정에는 문제가 없으나, 후속 필드 ID 정규화 작업에서 별도 검토가 필요하다.

## 9. 커밋 대기

Stage 2 구현과 focused 검증은 완료했다. 작업지시자 승인 후 Stage 2 변경분을 커밋한다.
