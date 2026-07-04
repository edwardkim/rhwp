# Task M100-258 Stage 21 — 누름틀 시작 바깥 입력 상태 유지

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `2f9fd48a` (`task 258: 누름틀 방향키 경계 이동 보정`)

## 1. 문제

누름틀 시작에서 왼쪽 방향키로 같은 `charOffset`의 필드 바깥 위치에 머문 뒤
문자를 입력하면, 첫 글자는 모델상 필드 밖에 들어가지만 커서가 새 필드 시작 offset과
같아지면서 즉시 필드 내부 상태로 되돌아간다.

그 결과 화면에서는 입력 직후 누름틀 안에 들어간 것처럼 보이고, 이어 입력하는 문자는
field value로 들어갈 수 있다.

## 2. 원인 판단

- `charOffset == startCharIdx`는 “필드 시작 내부”와 “필드 이전 바깥”을 동시에 표현한다.
- Stage20은 방향키 이동 시 `fieldStartExitKey`로 이 둘을 구분했다.
- 그러나 `insertText` 실행 후 새 커서 위치가 shifted field start와 같아지고,
  refresh 과정에서 exit key가 유지되지 않아 active field가 다시 설정된다.

## 3. 수정 방향

- `insertText`가 `fieldStartExitKey`가 걸린 위치에서 시작된 경우, 삽입 후 새 커서 위치도
  누름틀 시작 바깥 상태로 다시 표시한다.
- 일반 필드 내부 입력, 빈 누름틀 첫 입력, 필드 끝 바깥 입력은 기존 동작을 유지한다.

## 4. 검증 계획

- `cargo test --test issue_258_clickhere_form_mode`
- `cd rhwp-studio && npm run build`
- `wasm-pack build --target web --out-dir pkg`
- `http://localhost:7700/`에서 `누름틀-2024.hwp` 첫 누름틀 시작에서
  `ArrowLeft` 후 `a`, `b`를 입력해 `ab`가 field 밖 prefix로 유지되는지 확인

## 5. 수정 결과

- `executeOperation()`의 `insertText` 실행 전 커서가 `fieldStartExitKey`로 표시된
  누름틀 시작 바깥 위치인지 기록한다.
- 삽입 후 커서를 새 위치로 이동한 다음 `markCurrentFieldStartOutside()`를 다시 호출해
  shifted field start에서도 바깥 상태를 유지한다.
- 이에 따라 첫 글자 입력 후에도 active field가 재설정되지 않고, 이어지는 입력도
  field value가 아니라 필드 앞 본문 prefix로 들어간다.

## 6. 검증 결과

- `cargo test --test issue_258_clickhere_form_mode`: 통과
- `cd rhwp-studio && npm run build`: 통과
- `git diff --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `http://localhost:7700/` Playwright 검증 통과
  - 첫 누름틀 시작에서 `ArrowLeft` 후 `a`, `b` 입력
  - 첫 문단 text: `ab11223344`
  - 첫 field value: `11223344`
  - 첫 field range: `2..10`
  - 커서 `charOffset=2`, `fieldStartExitKey` 유지
  - 스크린샷: `/tmp/task258-stage21-arrow-left-ab.png`
