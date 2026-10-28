# Task M100-258 Stage 20 — 방향키 누름틀 바깥 이동 보정

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `ab32d07a` (`task 258: 누름틀 시작 경계 입력 보정`)

## 1. 문제

누름틀 시작/끝 경계에서 키보드 방향키로 누름틀 밖 위치로 이동할 수 있어야 하지만,
현재 화면에서는 방향키 이동 후에도 누름틀 내부 상태로 남는다.

## 2. 기준

- 누름틀 시작 내부에서 왼쪽 방향키를 누르면 같은 charOffset의 누름틀 이전 위치가 된다.
- 누름틀 끝 내부에서 오른쪽 방향키를 누르면 같은 charOffset의 누름틀 이후 위치가 된다.
- 누름틀 이전 위치에서 오른쪽 방향키를 누르면 내부 시작으로 들어간다.
- 누름틀 이후 위치에서 왼쪽 방향키를 누르면 내부 끝으로 들어간다.

## 3. 조사 계획

- `tryExitCurrentFieldStart/End()`가 만든 exit key가 `updateCaret()`/`updateFieldMarkers()` 중
  지워지는지 확인한다.
- exit 상태가 단순 표시 상태인지, 실제 입력/삭제 허용 판정에도 반영되는지 확인한다.
- 방향키 이동 시 불필요한 `document-changed` 이벤트가 즉시 재렌더/상태 재계산을 유발하는지 확인한다.

## 4. 수정

- `tryExitCurrentFieldStart/End()`는 문서 내용을 바꾸지 않는 커서 경계 상태 전환이다.
- 따라서 누름틀 시작/끝 밖 상태로 전환할 때 `document-changed`를 발생시키지 않도록 제거했다.
- 기존처럼 field marker 숨김, active field 해제, 상태 표시줄 갱신은 유지한다.

## 5. 검증

- `cd rhwp-studio && npm run build`: 통과
- Chrome 자동 검증은 시도했으나 현재 Codex 세션에서 Chrome 확장 백엔드가 노출되지 않아 보류했다.
  - Chrome 실행: 정상
  - Codex Chrome Extension 설치/활성화: 정상
  - Native host manifest: 정상
  - `agent.browsers.get("extension")`: `Browser is not available: extension`

## 6. 판단

- 방향키 밖 전환은 동일한 `charOffset`에서 exit key로 내부/외부를 구분하는 UI 상태다.
- 이 상태 전환에서 문서 재렌더 이벤트를 발생시키면 active field/marker 재계산이 즉시 개입할 수 있으므로
  한컴식 경계 이동에는 부적합하다.
