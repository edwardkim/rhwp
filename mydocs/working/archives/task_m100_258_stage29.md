# Task M100-258 Stage 29 — 누름틀 경계 Home/End 이동 보정

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `13f508c9` (`task 258: 인접 누름틀 붙여넣기 표시 보정`)

## 1. 문제

누름틀 안 또는 인접 누름틀 경계에서 `Home`/`End` 키를 누르면 한컴처럼 누름틀 바깥의 줄 시작/끝
위치로 이동하지 못한다. 작업지시자 재현 기준:

- `abc[123][123]` 또는 붙여넣은 `[123][123]`에서 `Home`을 누르면 첫 컬럼/줄 시작의 누름틀 바깥으로 이동해야 한다.
- 같은 상태에서 `End`를 누르면 줄 끝의 누름틀 바깥으로 이동해야 한다.
- 현재는 경계 밖 상태가 제대로 잡히지 않아 누름틀 내부 또는 잘못된 경계에 머문다.

## 2. 수정 방향

- `Home`/`End` 키가 타는 내비게이션 경로와 `lineStart`/`lineEnd` 처리 위치를 확인한다.
- `moveToLineStart`/`moveToLineEnd` 이후 커서가 ClickHere 경계에 놓이면 `fieldStartExitKey`/`fieldEndExitKey`를
  설정해 누름틀 바깥 상태로 만든다.
- 인접 누름틀에서는 같은 `charOffset`에 앞 필드 끝과 다음 필드 시작이 공존하므로, Home은 시작 경계,
  End는 끝 경계를 우선하도록 고정한다.

## 3. 원인

`Home`/`End`는 본문 키 처리 switch까지 내려오기 전에 `handleNavigationShortcut`의 `lineStart`/`lineEnd`
경로에서 먼저 처리된다. 일반 switch 경로에는 `markCurrentFieldStartOutside`/`markCurrentFieldEndOutside`
호출이 있었지만, 공통 내비게이션 경로에는 없어서 누름틀 경계에 도달해도 바깥 상태 키가 설정되지 않았다.

## 4. 수정

- `executeNavigationAction`의 `lineStart` 처리 직후 `markCurrentFieldStartOutside`를 호출한다.
- `executeNavigationAction`의 `lineEnd` 처리 직후 `markCurrentFieldEndOutside`를 호출한다.
- 이로써 플랫폼별 `Home`/`End`, 매핑된 줄 시작/끝 이동이 모두 같은 누름틀 경계 처리로 수렴한다.

## 5. 검증 결과

- `abc[123][123]` 구성 후 누름틀 내부/경계에서 `Home`, `End` 재현
- Home 후 본문 입력이 첫 누름틀 밖 앞쪽으로 들어가는지 확인
- End 후 본문 입력이 마지막 누름틀 밖 뒤쪽으로 들어가는지 확인
- 작업지시자 시각 검증 완료: Home/End 경계 이동 정상
- `cd rhwp-studio && npm run build`: 통과
- `git diff --check`: 통과
