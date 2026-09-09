# Task M100-258 Stage 9 — 빈 누름틀 안내문 클릭 후 첫 입력 복구

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `384a0f83` (`task 258: 누름틀 경계 입력과 삭제 확인 보정`)

## 1. 문제

새 누름틀 삽입 후 안내문은 표시되지만, 안내문을 클릭해 빨간 낫표만 보이는 상태에서
`123`을 입력해도 실제 값이 들어가지 않는다.

## 2. 원인 조사 방향

- 빈 ClickHere는 `startCharIdx == endCharIdx`라 클릭 후 커서가 필드 시작/끝 경계에 놓인다.
- Stage8의 “필드 끝 밖 이탈” 판정이 빈 ClickHere 클릭 또는 첫 입력 경로와 충돌하는지 확인한다.
- 마우스 hit-test와 `updateFieldMarkers()`가 active field를 올바르게 설정하는지 확인한다.
- 첫 입력은 빈 ClickHere 내부 값으로 들어가고, 입력 후 오른쪽 이동 입력은 field 밖 본문으로 들어가야 한다.

## 3. 검증 계획

- `cargo test --test issue_258_clickhere_form_mode`
- `npm run build`
- `git diff --check`

## 4. 원인

빈 ClickHere 안내문은 실제 문서 텍스트가 아니라 렌더러가 만든 guide run이다. 안내문 클릭 후
커서는 필드 경계에 놓이지만, 입력 경로는 클릭 직후의 선택/커서 상태를 그대로 사용해 첫
입력 전에 빈 field start를 다시 보장하지 않았다.

## 5. 수정

- `InputHandler.prepareClickHereInputPosition()`을 추가해 빈 ClickHere guide 상태의 입력 위치를
  `startCharIdx`로 정규화하고 active field를 보장했다.
- 일반 입력, IME 조합 시작, iOS 입력 폴백에서 같은 정규화 함수를 사용하도록 했다.
- 회귀 테스트는 빈 ClickHere를 active로 잡은 뒤 첫 입력이 field 값으로 들어가는지 확인하도록
  보강했다.

## 6. 검증 결과

- `cargo fmt` 통과
- `cargo test --test issue_258_clickhere_form_mode` 통과
- `npm run build` 통과
