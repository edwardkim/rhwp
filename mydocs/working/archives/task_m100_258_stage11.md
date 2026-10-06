# Task M100-258 Stage 11 — 빈 누름틀 첫 입력 커서 좌표 보정

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `bcd34028` (`task 258: 누름틀 첫 입력 렌더 갱신`)

## 1. 문제

Stage 10 보정 뒤 빈 ClickHere 안내문을 클릭하고 `123`을 입력하면 값 자체는 들어가지만,
커서를 움직여 보면 누름틀 값 안에 들어간 것처럼 보이지 않았다. 자동 검증에서도
`getCursorRect(0, 0, 0)`과 `getCursorRect(0, 0, 3)`의 x 좌표가 모두 `113.4`로 같아,
field end 마커가 실제 입력값 끝을 따라가지 못했다.

## 2. 원인

입력 직후 render tree에는 다음 두 종류의 TextRun이 함께 생성된다.

- source offset `0..3`을 가진 0폭 `U+FFFC` placeholder TextRun
- 같은 source offset `0..3`을 가진 실제 표시 문자열 `123` TextRun

`get_cursor_rect_native()`는 먼저 만나는 0폭 placeholder TextRun을 cursor hit로 채택해
모든 field 내부 offset을 같은 x 좌표로 반환했다.

## 3. 수정 내용

- ClickHere field range가 있는 문단은 텍스트 삽입 뒤 `char_offsets`와 `char_count`를
  즉시 재계산하도록 했다.
- 빈 field가 텍스트를 갖게 된 뒤에도 field 끝 마커와 문단 끝 마커가 `char_count`에
  반영되도록 `rebuild_char_offsets()`를 보강했다.
- `get_cursor_rect_native()`에서 0폭 `U+FFFC` placeholder TextRun이 실제 문단 텍스트와
  같은 source range를 가리키면, 문단 텍스트 기준으로 문자별 caret x를 재계산한다.

## 4. 검증 결과

- `cargo test --test issue_258_clickhere_form_mode first_input_into_empty_clickhere_is_rendered -- --nocapture` 통과
- `cargo test --test issue_258_clickhere_form_mode` 통과
- `cargo test --lib rebuild_` 통과
- `cargo fmt --check` 통과
- `git diff --check` 통과
- `wasm-pack build --target web --out-dir pkg` 통과

## 5. 프론트 검증

`browser:control-in-app-browser` 절차를 확인했지만 현재 세션에서는 in-app Browser가
`Browser is not available: iab`로 연결되지 않았다. 대신 `rhwp-studio`의 Playwright로
`http://localhost:7700/`에 접속해 같은 시나리오를 검증했다.

- 새 문서 생성 후 빈 editable ClickHere 삽입
- 안내문 위치 클릭과 동일한 위치로 커서 이동
- `123` 입력
- `getFieldList()` 결과: `value="123"`, `startCharIdx=0`, `endCharIdx=3`
- `renderPageSvg(0)` 결과: `>1<`, `>2<`, `>3<` 포함, `입력하세요` 미포함
- `getCursorRect(0..3)` x 좌표: `113.4`, `120.7`, `128.1`, `135.4`
- field marker 위치: 시작 `left=358.7`, 끝 `left=388.7`
- 스크린샷: `/tmp/task258-stage11-ui-after-input-fixed.png`
