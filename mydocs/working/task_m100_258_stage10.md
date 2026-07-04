# Task M100-258 Stage 10 — 누름틀 첫 입력 즉시 표시와 마커 갱신

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `c2dd4251` (`task 258: 빈 누름틀 첫 입력 복구`)

## 1. 문제

빈 누름틀 안내문을 클릭한 뒤 `123`을 입력하면 값은 들어가지만 바로 화면에 보이지 않고,
Enter 등 다른 편집으로 재렌더가 발생한 뒤에야 보인다. 커서를 움직여도 누름틀 값 안에
들어간 것처럼 마커가 안정적으로 보이지 않는다.

## 2. 원인 가설

- 빈 ClickHere 첫 입력은 guide run을 실제 field value로 바꾸는 구조 변화다.
- 기존 입력 후 갱신 경로가 일반 텍스트 편집과 같은 방식으로 처리되어, active field와
  field marker 좌표가 새 field range 기준으로 즉시 재계산되지 않는다.

## 3. 수정 방향

- 빈 ClickHere 첫 입력은 일반 텍스트 입력보다 강한 refresh를 수행한다.
- 입력 직후 커서 위치에서 field marker와 active field를 다시 계산한다.
- Enter/다른 편집 없이도 입력한 값과 누름틀 마커가 바로 표시되는지 확인한다.

## 4. 수정 내용

- `onInput`에서 입력 위치가 빈 ClickHere guide인지 먼저 판정한다.
- guide 첫 입력 후 active field와 field marker를 비우고 `document-changed`를 즉시 발생시킨다.
- 다음 animation frame에서 `updateCaret()`로 커서/마커를 새 field value 기준으로 다시 계산한다.

## 5. 검증 계획

- `cargo test --test issue_258_clickhere_form_mode`
- `npm run build` (`rhwp-studio/`)
- `git diff --check`

## 6. 검증 결과

- `cargo test --test issue_258_clickhere_form_mode` 통과
- 루트 `npm run build`는 스크립트가 없어 실행 불가 확인
- `cd rhwp-studio && npm run build` 통과
- `git diff --check` 통과
