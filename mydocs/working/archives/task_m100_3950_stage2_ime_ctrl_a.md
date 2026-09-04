# Task M100 #3950 Stage 2 — 물리 KeyA와 IME dispatcher 보정

## 변경

1. `defaultShortcuts`의 `edit:select-all` 정의에 `code: 'KeyA'`를 추가했다. 이로써 한글 IME가
   `e.key`를 `ㅁ` 또는 `Process`로 제공해도 물리 키 코드로 `Ctrl+A`를 찾는다.
2. `onKeyDown()`의 IME 조합 분기에서 Ctrl+M chord를 먼저 유지한 뒤, `matchShortcut()`으로 찾아진
   Ctrl/Meta 기본 명령을 기존 dispatcher에 전달하도록 했다. 매칭되지 않은 키와 탐색키 보류는 종전
   경로를 유지한다.
3. 한글 IME 일반·조합 상태 매핑 테스트와 IME 분기 dispatcher 계약 테스트를 추가했다.

## 집중 검증

```text
node --test tests/shortcut-map.test.ts tests/ime-shortcut-routing.test.ts
결과: 8 passed, 0 failed

git diff --check
결과: 통과
```
