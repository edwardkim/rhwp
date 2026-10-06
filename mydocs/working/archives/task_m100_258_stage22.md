# Task M100-258 Stage 22 — 누름틀 시작 바깥 입력 전 active field 방어

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `ffaf0549` (`task 258: 누름틀 시작 바깥 입력 유지`)

## 1. 문제

사용자 확인에서 누름틀 값 `123` 입력 후 왼쪽 방향키로 누름틀 시작 바깥에 머문 상태에서
문자를 계속 입력하면, 마지막 입력 문자가 누름틀 안으로 들어간 것처럼 보이는 사례가 남았다.

## 2. 조사

fresh 로드 Playwright 검증에서는 `ArrowLeft` 후 `abcd` 입력 결과가 다음처럼 정상이다.

- 본문 text: `abcd123`
- field value: `123`
- field range: `4..7`
- `fieldStartExitKey` 유지

다만 경계 위치는 `charOffset == startCharIdx`를 공유하므로, 입력 직전 WASM `active_field`가
늦게 살아 있으면 Rust 삽입 로직이 해당 입력을 field 내부 입력으로 해석할 수 있다.

## 3. 수정 방향

- `insertText` 실행 전 커서가 `fieldStartExitKey`로 표시된 누름틀 시작 바깥 위치라면,
  command 실행 전에 `wasm.clearActiveField()`를 한 번 더 호출한다.
- Stage21의 삽입 후 shifted start 위치에 exit key를 다시 거는 동작은 유지한다.

## 4. 검증 계획

- `cd rhwp-studio && npm run build`
- `cargo test --test issue_258_clickhere_form_mode`
- `git diff --check`
- `http://localhost:7700/`에서 새 누름틀 `123` 입력 후 왼쪽 바깥에서 `abcd` 연속 입력 검증

## 5. 수정 결과

- `executeOperation()`에서 `insertText` 실행 전 현재 위치가 `fieldStartExitKey`로 표시된
  누름틀 시작 바깥 위치이면 `wasm.clearActiveField()`를 먼저 호출한다.
- 삽입 후 shifted field start에 `fieldStartExitKey`를 다시 거는 Stage21 동작은 유지했다.
- 이 보강으로 UI 상태와 WASM active field 상태가 어긋난 경우에도 시작 바깥 입력이
  field value로 흡수되지 않는다.

## 6. 검증 결과

- `cd rhwp-studio && npm run build`: 통과
- `cargo test --test issue_258_clickhere_form_mode`: 통과
- `git diff --check`: 통과
- `http://localhost:7700/` Playwright 검증 통과
  - 새 누름틀 삽입 후 `123` 입력
  - 왼쪽 방향키 4회로 누름틀 시작 바깥 이동
  - `abcd` 연속 입력
  - 최종 본문 text: `abcd123`
  - field value: `123`
  - field range: `4..7`
  - 스크린샷: `/tmp/task258-stage22-active-guard-abcd.png`
- `http://localhost:7700/` Playwright 추가 검증 통과
  - 새 누름틀 삽입 후 `123` 입력
  - 왼쪽 방향키 4회, `abc` 입력, 오른쪽 방향키 1회
  - `abc` 입력 직후 본문 text: `abc123`, field value: `123`, field range: `3..6`
  - `abc` 입력 직후 `fieldStartExitKey` 유지
  - 오른쪽 방향키 후 `fieldStartExitKey` 해제, 같은 `charOffset=3`에서 field 내부 시작으로 진입
  - 스크린샷: `/tmp/task258-stage22-abc-right.png`
