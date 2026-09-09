# Task M100-258 Stage 24 — 삽입 직후 빈 누름틀 키보드 진입

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `e3bfc806` (`task 258: 누름틀 시작 낫표 위치 보정`)

## 1. 문제

한컴은 누름틀 삽입 직후 커서를 입력 안내문 오른쪽 바깥에 둔다. 이 상태에서
`←`를 누르면 안내문이 사라지고 빈 입력칸만 보이는 누름틀 내부 상태로 들어가며,
아무것도 입력하지 않고 `→`를 누르면 다시 안내문이 보이는 오른쪽 바깥 상태로 나온다.

현재 rhwp-studio는 빈 누름틀의 내부/외부 경계를 충분히 구분하지 못해 삽입 직후
키보드만으로 이 한컴 동작을 재현하기 어렵다.

추가 확인 중 값이 들어간 누름틀의 시작 바깥 상태(`abc|123`)에서도 같은 경계 문제가 있었다.
`←`를 누르면 왼쪽 본문으로 이동해야 하지만 같은 위치에 머물렀고, Backspace는 앞 글자 `c`를
지우지 않고 `[누름틀]을 지울까요?` 확인창을 표시했다.

## 2. 원인 판단

- 새로 삽입된 빈 누름틀은 `startCharIdx == endCharIdx`라 내부/외부가 같은 `charOffset`이다.
- 삽입 대화상자의 적용 경로는 field를 추가하고 문서를 다시 그리지만, 커서 상태를
  “누름틀 끝 바깥”으로 명시하지 않는다.
- `tryExitCurrentFieldEnd()`와 `markCurrentFieldEndOutside()`는 빈 누름틀을 거부하므로,
  빈 입력칸에서 `→`로 다시 오른쪽 바깥 상태로 나오는 동작도 성립하지 않는다.

## 3. 수정 방향

- 누름틀 삽입 직후 커서를 삽입 위치에 두되, 빈 누름틀 끝 바깥 상태로 표시한다.
- 빈 누름틀에서도 `markCurrentFieldEndOutside()`와 `tryExitCurrentFieldEnd()`가 end-exit 상태를 만들 수 있게 한다.
- 사용자가 `←`를 누르면 기존 `tryEnterExitedFieldEnd()` 경로로 field 내부 빈 입력칸에 진입한다.
- 사용자가 빈 입력칸에서 `→`를 누르면 다시 end-exit 상태가 되어 안내문을 복원한다.
- 이미 start-exit/end-exit 상태인 경우에는 같은 boundary exit 분기를 반복하지 않고 일반 방향키 이동으로 넘긴다.
- start-exit 상태의 Backspace는 누름틀 삭제 확인이 아니라 앞쪽 본문 글자 삭제로 처리하고,
  삭제 후 이동한 field 시작 경계를 다시 start-exit 상태로 유지한다.

## 4. 검증 계획

- `cd rhwp-studio && npm run build`
- `git diff --check`
- `http://localhost:7700/`에서 새 누름틀 삽입 직후 `←`로 빈 field 내부 진입, `→`로 안내문 복원 확인

## 5. 구현 결과

- `insert:field` 적용 직후 삽입된 누름틀 위치로 커서를 이동하고, 해당 위치를 누름틀 끝 바깥 상태로 표시했다.
- 빈 누름틀(`startCharIdx == endCharIdx`)에서도 끝 바깥 상태를 만들 수 있도록 `tryExitCurrentFieldEnd()`와
  `markCurrentFieldEndOutside()`의 빈 field 거부 조건을 제거했다.
- 끝 바깥 상태의 guide 누름틀은 caret을 guide 텍스트 오른쪽에 표시하도록 layer tree의 `textRun` bbox를 참조했다.
- 끝 바깥/시작 바깥 상태 전환 직후 `document-changed`와 caret 갱신을 발생시켜 marker 표시가 한 박자 늦게 남지 않도록 했다.
- 값이 들어간 누름틀의 시작 바깥 상태에서 `←`는 일반 본문 왼쪽 이동으로 통과시키고,
  Backspace는 누름틀 삭제 확인 대신 앞 글자 삭제 후 start-exit 상태를 유지하도록 했다.

## 6. 검증 결과

- `cd rhwp-studio && npm run build`: 통과
- `git diff --check`: 통과
- `http://localhost:7700/` Playwright 검증 통과
  - 삽입 직후: guide `사용자 이름`이 보이고 caret이 guide 오른쪽 바깥에 표시됨
  - `←`: guide가 사라지고 빈 누름틀 입력칸만 표시됨
  - `→`: 입력 없이 빠져나오면 guide `사용자 이름`이 다시 표시되고 caret이 오른쪽 바깥에 표시됨
  - `abc|123` start-exit 상태에서 `←`: 커서가 `ab|c123` 위치로 이동하고 field 상태가 해제됨
  - `abc|123` start-exit 상태에서 Backspace: 확인창 없이 `ab|123`이 되고 field range가 `2..5`로 보정됨
  - 관련 warning/error 로그 없음
