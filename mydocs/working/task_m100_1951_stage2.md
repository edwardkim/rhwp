# Task M100 #1951 Stage 2 완료보고서

- 이슈: #1951
- 브랜치: `codex/task_m100_1951`
- 재현 원본: `samples/복학원서.hwp`
- 작성일: 2026-07-10

## 1. 수정

- `cursor_rect.rs`
  - 단일 셀과 cell path cursor query가 일치하는 `TableCell` bbox를 `cellBounds`로 반환한다.
  - raw TextRun 좌표가 bbox 밖이면 caret 좌표를 가시 범위 안으로 제한하고
    `cellOverflowed=true`를 함께 반환한다.
- `text_editing.rs`
  - one-depth `cellPath` 삽입은 일반 셀 삽입 경로를 재사용한다. 따라서 IME가 path를 전달해도
    셀 폭 리플로우, vpos 재계산, 페이지네이션을 빠뜨리지 않는다.
- Studio
  - DOM caret과 IME 조합창의 위치/폭/높이를 `cellBounds` 안으로 제한한다.
  - 지연 입력 뒤 `cellOverflowed`가 감지되면 그 순간에만 전체 페이지네이션을 flush해 표 행을
    최신 내용 높이로 다시 계산한다.

## 2. 검증

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1951_table_cell_cursor_clip -- --nocapture`: 2 passed
- `node --test tests/caret-cell-bounds.test.ts`: 2 passed
- `npx tsc --noEmit`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `localhost:7700` 실제 브라우저에서 장문 160자 입력: 첫 행이 확장되고 caret이 행 내부에 유지됨
- 브라우저 error/warn log: 없음

## 3. 남은 수동 검증

자동 입력은 OS 한글 IME 조합 이벤트를 만들지 않는다. Windows Chrome에서 실제 한글 조합 중 검은
조합창의 위치와 폭이 셀 안에 유지되는지 확인해야 한다.
