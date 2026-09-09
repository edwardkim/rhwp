# Task M100-258 Stage 15 — 누름틀 삭제 후 커서 위치 복귀

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `2cbd1d1e` (`task 258: 누름틀 삭제 시 내용 제거`)

## 1. 문제

Stage14에서 `Delete` 확인 시 누름틀 내용과 field/control은 함께 삭제되지만,
커서가 삭제 전 field 끝 위치에 남아 있다. field text가 사라진 뒤에도 커서가 삭제된
range의 끝 offset을 유지해 한컴처럼 삭제된 누름틀 이전 위치로 돌아가지 않는다.

## 2. 수정 방향

- 삭제 전 `getFieldInfoAt()`으로 ClickHere field의 `startCharIdx`를 저장한다.
- `removeFieldAt()` 성공 후 커서를 같은 문단/셀 컨텍스트의 `startCharIdx`로 이동한다.
- 커서 이동을 `afterEdit()` 전에 수행해 재렌더와 caret 갱신이 삭제 후 위치를 기준으로
  동작하게 한다.

## 3. 검증 계획

- `cd rhwp-studio && npm run build`
- `git diff --check`
- `http://localhost:7700/`에서 `누름틀-2024.hwp` 첫 누름틀 삭제 후 커서가 `charOffset=0`으로
  돌아가는지 Playwright로 확인

## 4. 수정 결과

- `removeCurrentField()`가 삭제 전 ClickHere `FieldInfoResult.startCharIdx`를 저장한다.
- `removeFieldAt()` 성공 후 커서를 같은 문단/셀 컨텍스트의 field 시작 offset으로 이동하고
  selection/preferredX를 초기화한다.
- 커서 이동 후 `afterEdit()`를 호출해 문서 재렌더와 caret 갱신이 삭제 후 위치를 기준으로
  동작하게 했다.
- `WasmBridge.removeFieldAt()` 주석도 Stage14 이후 실제 동작인 `필드와 내용 제거`로 정리했다.

## 5. 검증 결과

- `cd rhwp-studio && npm run build`: 통과
- `git diff --check`: 통과
- `http://localhost:7700/` Playwright 검증 통과
  - 삭제 전 커서: `sectionIndex=0`, `paragraphIndex=0`, `charOffset=8`
  - 삭제 후 첫 문단 text: 빈 문자열
  - 삭제 후 커서: `sectionIndex=0`, `paragraphIndex=0`, `charOffset=0`
  - 삭제 후 `getFieldInfoAt(0,0,0)={"inField":false}`
  - 스크린샷: `/tmp/task258-stage15-delete-cursor.png`
