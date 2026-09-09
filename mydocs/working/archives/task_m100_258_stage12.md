# Task M100-258 Stage 12 — 기존 HWP 누름틀 삭제와 샘플 렌더 정합

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `2ddc2d39` (`task 258: 빈 누름틀 첫 입력 커서 좌표 보정`)

## 1. 문제

- 누름틀 경계에서 삭제 확인 대화상자의 `확인`을 눌러도 실제 누름틀이 지워지지 않는다.
- `samples/누름틀-2024.hwp`를 열면 한컴오피스처럼 `11223344` 전체가 누름틀 값으로
  잡히지 않고, 렌더/마커/커서 진입 위치가 어긋난다.
- 커서 위치에 따라 언제 누름틀 내부로 들어가는지 판정이 한컴오피스 동작과 다르다.

## 2. 조사 방향

- `confirmRemoveCurrentField()`가 확인 후 실제 `removeFieldAt()` 성공 결과를 문서 갱신에
  반영하는지 확인한다.
- HWP5 파서가 기존 ClickHere의 field begin/end 범위를 정확히 복원하는지 확인한다.
- `누름틀-2024.hwp`의 `getFieldList()`, paragraph text, field range, cursor rect,
  render SVG를 한컴 기준 화면과 비교한다.

## 3. 원인

- 삭제 확인 대화상자는 비동기로 닫힌다. 기존 구현은 확인 버튼을 누른 뒤
  `removeCurrentField()`에서 현재 커서 위치를 다시 읽었기 때문에, 모달 클릭/포커스 변화 후
  누름틀 위치가 아닌 좌표를 제거하려 할 수 있었다.
- `samples/누름틀-2024.hwp` 첫 문단은 본문 텍스트 `11223344` 앞에
  `SectionDef`, `ColumnDef`, `Field` 컨트롤 gap이 있어 `char_offsets[0]=24`이다.
  렌더 전용 placeholder 합성 로직이 이 비가시 컨트롤까지 인라인 개체로 세어
  `U+FFFC` 3개를 앞에 삽입했고, 실제 텍스트 run의 source char가 `0`이 아니라 `3`으로
  밀렸다. 그 결과 `getCursorRect(0..8)`이 중간에서 뒤로 되돌아갔다.

## 4. 수정 내용

- `confirmRemoveCurrentField()`가 대화상자를 띄운 순간의 `DocumentPosition`을 캡처하고,
  `확인` 후 그 위치를 `removeCurrentField(pos)`에 넘기도록 했다.
- `synthesize_marker_paragraph()`의 placeholder 합성 대상을 실제 렌더 자리 차지가 필요한
  그림/도형/표/양식 개체/글자처럼 취급하는 수식으로 좁혔다.
- `누름틀-2024.hwp` 첫 ClickHere 값 `11223344`의 cursor rect가 start부터 end까지
  단조 증가하는 회귀 테스트를 추가했다.

## 5. 검증 결과

- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test --test issue_258_clickhere_form_mode clickhere_hwp_sample_cursor_rects_follow_visible_value -- --nocapture`
- `cargo fmt --check`
- `git diff --check`
- `cd rhwp-studio && npm run build`
- `wasm-pack build --target web --out-dir pkg`

WASM/Node 덤프:

- `getFieldList()` 첫 ClickHere: `value="11223344"`, range `0..8`
- `getCursorRect(0..8)` x: `113.4, 120.7, 128.1, 135.4, 142.7, 150.1, 157.4, 164.7, 172.1`
- 첫 TextRun source key: `section:0/para:0/char:0`

Playwright `http://localhost:7700/` 검증:

- `누름틀-2024.hwp` 로드 성공
- `charOffset=3`과 `charOffset=8` 모두 첫 ClickHere 내부로 판정
- 삭제 전 마커가 `「11223344」` 전체를 감싼다.
  스크린샷: `/tmp/task258-stage12-sample-active.png`
- 끝 경계에서 `[누름틀]을 지울까요?` 확인 대화상자 표시
- `확인` 클릭 후 첫 ClickHere가 `getFieldList()`에서 제거되고 두 번째 ClickHere만 남는다.
  스크린샷: `/tmp/task258-stage12-after-delete.png`
