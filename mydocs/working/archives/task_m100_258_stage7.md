# Task M100-258 Stage 7 — 누름틀 삽입 직후 안내문 표시

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `2b767790` (`task 258: 누름틀 대화상자 바깥 클릭 닫힘 방지`)

## 1. 문제

`필드 입력`에서 누름틀을 넣은 직후 화면에 안내문 텍스트가 표시되지 않고 빨간 낫표만
보인다. 한컴 동작은 삽입 직후 안내문이 빨간 글씨로 표시되고, 사용자가 해당 누름틀을
클릭/진입하면 입력 상태가 된다.

## 2. 원인

`insert:field` 명령이 삽입 후 `InputHandler.triggerAfterEdit()`를 호출하면서 현재 커서
위치의 빈 ClickHere를 바로 활성 필드로 설정한다. 렌더러는 활성 빈 누름틀의 안내문을
숨기므로, 새로 만든 안내문이 보이지 않는다.

## 3. 수정 범위

- `insert:field` 삽입 성공 후에는 active field를 해제한다.
- 공통 편집 후 caret/field marker 갱신 대신 문서 변경 이벤트만 발행해 새 렌더가 안내문
  표시 상태를 사용하게 한다.
- 저장/직렬화 로직은 변경하지 않는다.

## 4. 검증 계획

- `npm run build`
- `git diff --check`

## 5. 진행 기록

- `insert:field` 삽입 성공 후 `triggerAfterEdit()` 대신 `clearActiveField()`와 문서 변경 이벤트를
  직접 호출하도록 바꿨다.
- 삽입 직후 렌더가 활성 필드 상태를 사용하지 않으므로 빈 ClickHere 안내문이 표시된다.
- 최종 보고서에 Stage 6/7 UX 보정 내용을 반영했다.

## 6. 검증 결과

- `npm run build` 통과
- `git diff --check` 통과
