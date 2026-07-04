# Task M100-258 Stage 25 — 인접 누름틀 연속 선택

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `af42b1d0` (`task 258: 누름틀 빈 입력칸 경계 이동 보정`)

## 1. 문제

한컴은 값이 들어간 누름틀 두 개가 인접해 있을 때 두 누름틀 값을 하나의 연속 텍스트처럼 선택할 수 있다.
예를 들어 `abc[123][123]`에서 두 누름틀을 모두 선택하면 `123123` 전체가 선택된다.

현재 rhwp-studio는 누름틀 boundary 상태에서 바깥으로 이동하거나 선택 확장하는 흐름이 끊겨,
인접한 누름틀 두 개를 동시에 선택하기 어렵다.

## 2. 원인 조사 방향

- `Shift+Arrow` 선택 확장 시 start-exit/end-exit 경계 상태가 일반 커서 이동과 다르게 동작하는지 확인한다.
- selection renderer가 field boundary와 겹친 range를 잘라서 그리는지 확인한다.
- 인접 field의 끝/시작이 같은 문단 안에서 연속 range로 표현될 때 selection range가 `3..9`처럼 유지되는지 확인한다.

조사 결과, field range가 정상적으로 `3..6`, `6..9`로 분리돼 있으면 `Shift+ArrowRight` 선택은
`3..9` range를 만들고 selection rect도 `123123` 전체를 덮는다. 문제는 두 번째 빈 누름틀을
첫 번째 누름틀 끝과 같은 charOffset에 삽입한 직후, shared boundary 조회가 앞 누름틀을 먼저
반환하는 데 있었다. 이 상태에서 첫 입력이 앞 누름틀에 붙어 첫 field range가 `3..9`로 늘고,
새 field range가 `6..9`로 겹쳐 인접 누름틀 선택/진입이 깨졌다.

## 3. 수정 방향

- selection 확장 중에는 누름틀 내부/외부 boundary 상태를 선택 범위 확장에 방해되지 않게 정리한다.
- 값이 있는 인접 누름틀은 일반 본문 텍스트처럼 연속 선택 range를 만들 수 있게 한다.
- 기존 Stage24의 단일 누름틀 boundary 이동, start-exit Backspace, 빈 누름틀 `←/→` 동작은 유지한다.

수정은 `field_query`의 shared boundary 해석으로 좁혔다.

- `getFieldInfoAt`과 `setActiveField` 내부 필드 탐색에서 같은 charOffset에
  `앞 누름틀 끝`과 `다음 빈 누름틀 시작/끝`이 겹치면 빈 누름틀을 우선 반환한다.
- 이 우선순위는 비어 있지 않은 단일 누름틀 끝 경계에는 영향을 주지 않는다.
- 인접 누름틀 첫 입력 회귀 테스트를 추가해 `abc[123][123]`가
  text `abc123123`, ranges `(3,6)`, `(6,9)`로 유지되는지 고정했다.

## 4. 검증 계획

- `cd rhwp-studio && npm run build`
- `git diff --check`
- `http://localhost:7700/` Playwright 검증
  - `abc[123][123]` 구성 후 첫 누름틀 시작에서 두 누름틀 전체 선택
  - selection range가 두 field 값을 모두 포함하고, 선택 렌더가 `123123` 전체를 덮는지 확인
  - Stage24 회귀: start-exit `←`, start-exit Backspace, 빈 누름틀 `←/→`

## 5. 검증 결과

- `cargo test --test issue_258_clickhere_form_mode adjacent_clickhere_input_prefers_new_empty_field_at_shared_boundary -- --nocapture`: 통과
- `cargo test --test issue_258_clickhere_form_mode`: 통과 (`11 passed`)
- `wasm-pack build --target web --out-dir pkg`: 통과
- `cd rhwp-studio && npm run build`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- Browser plugin: `http://localhost:7700/` 로드, title `rhwp-studio`, console error/warn 없음 확인
- Playwright 검증: `abc[123][123]` 구성 후 두 field가 `3..6`, `6..9`로 분리되고
  `Shift+ArrowRight` 6회 선택 range가 `3..9`, selection rect 폭 `44px`로 `123123` 전체를 덮음.
  스크린샷: `/tmp/rhwp-task258-stage25-adjacent-selection.png`
