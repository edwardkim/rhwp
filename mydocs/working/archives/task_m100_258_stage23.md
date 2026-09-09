# Task M100-258 Stage 23 — 누름틀 시작 낫표 위치 보정

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `9faa1efd` (`task 258: 누름틀 시작 바깥 입력 active 상태 방어`)

## 1. 문제

새 누름틀에 `123`을 입력한 뒤 왼쪽 방향키 4회로 누름틀 시작 바깥으로 나가고
`abc`를 입력한 다음 오른쪽 방향키로 다시 누름틀 안에 들어가면, 실제 field value는
`123`으로 유지되지만 화면에서는 시작 낫표가 `c` 앞에 걸려 `c`가 누름틀 안에 들어간 것처럼 보인다.

## 2. 원인

`FieldMarkerRenderer.show()`가 시작 낫표를 `startRect.x - markerWidth`에 배치한다.
`startRect.x`는 field 시작 경계이지만, 낫표 전체 폭만큼 왼쪽으로 이동하면서 field 앞 글자 영역에
겹쳐 보인다.

## 3. 수정 방향

- 시작 낫표의 left 좌표를 field 시작 경계인 `startRect.x` 기준으로 배치한다.
- end 낫표 배치는 기존처럼 field 끝 경계 기준을 유지한다.

## 4. 검증 계획

- `cd rhwp-studio && npm run build`
- `git diff --check`
- `http://localhost:7700/`에서 `123 ← ← ← ← abc →` 흐름 후 시작 낫표가 `1` 앞에 표시되는지 확인

## 5. 수정 결과

- 시작 낫표 left 좌표에서 `markerWidth`를 빼던 보정을 제거했다.
- 이제 시작 낫표는 field start cursor rect의 x 좌표에 맞춰 표시된다.
- field range와 value 계산은 기존 Stage22와 동일하게 유지된다.

## 6. 검증 결과

- `cd rhwp-studio && npm run build`: 통과
- `git diff --check`: 통과
- `http://localhost:7700/` Playwright 검증 통과
  - 새 누름틀 삽입 후 `123` 입력
  - 왼쪽 방향키 4회, `abc`, 오른쪽 방향키 1회
  - 본문 text: `abc123`
  - field value: `123`
  - field range: `3..6`
  - field 내부 진입 후 시작 낫표가 field 시작 경계 기준에 표시
  - 스크린샷: `/tmp/task258-stage23-marker-start.png`
