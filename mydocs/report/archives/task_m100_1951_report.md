# Task M100 #1951 결과보고서

- 이슈: [#1951](https://github.com/edwardkim/rhwp/issues/1951)
- 브랜치: `codex/task_m100_1951`
- 기준 브랜치: `upstream/devel`
- 재현 원본: `samples/복학원서.hwp`
- 작성일: 2026-07-10

## 1. 문제와 원인

첫 표의 `Name of College` 값 셀에 장문을 빠르게 입력하면, page-local 지연 페이지네이션이
기존 셀 높이의 render tree를 유지한 채 새 줄을 compose했다. 그 결과 raw cursor y가 셀 bbox
아래로 내려갔고, Canvas clip을 공유하지 않는 DOM caret 및 IME 조합창도 셀 밖으로 배치될 수 있었다.

또한 one-depth `cellPath`를 전달하는 IME 삽입은 일반 셀 삽입과 달리 셀 폭 리플로우 및 셀 문단
vpos 재계산을 건너뛰고 있었다.

## 2. 수정

1. 단일 셀 및 cell path cursor query가 일치하는 `TableCell` bbox를 `cellBounds`로 반환한다.
2. raw TextRun 좌표가 셀 bbox를 넘으면 반환 caret 좌표와 높이를 가시 bbox 안으로 제한하고
   `cellOverflowed=true`를 표시한다.
3. Studio는 일반 caret과 IME 조합창의 DOM 위치·폭·높이를 `cellBounds`로 한 번 더 제한한다.
4. 지연 입력 뒤 `cellOverflowed`를 확인하면 해당 입력에만 즉시 페이지네이션을 flush하여 표 행의
   실제 내용 높이를 다시 계산한다.
5. one-depth `cellPath` 삽입은 일반 셀 삽입 경로를 재사용해 셀 reflow와 vpos 재계산을 보장한다.

문서 모델의 text/offset/LINE_SEG와 저장 표 크기는 바꾸지 않았다. 정상적인 짧은 셀 입력은 기존
page-local 갱신 경로를 그대로 사용한다.

## 3. 회귀 테스트

- `tests/issue_1951_table_cell_cursor_clip.rs`
  - 지연 삽입 뒤 direct/path cursor가 모두 셀 bbox 안에 머무르고 overflow 상태를 전달하는지 검증
  - one-depth path 삽입이 즉시 reflow되어 overflow 없이 셀 bbox 안에 머무는지 검증
- `rhwp-studio/tests/caret-cell-bounds.test.ts`
  - IME 조합 DOM의 cell bounds 제한과 overflow 시 즉시 flush 계약을 검증
- `src/document_core/queries/cursor_rect.rs` 단위 테스트
  - 셀 높이보다 큰 caret도 bbox 안으로 축소되는지 검증

## 4. 수행 검증

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1951_table_cell_cursor_clip -- --nocapture`: 2 passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib document_core::queries::cursor_rect::tests::cell_cursor_bounds_clamp_coordinates_and_height -- --nocapture`: 1 passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_717_table_cell_hit_test -- --nocapture`: 4 passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2164_cell_enter_overlap -- --nocapture`: 3 passed
- `rhwp-studio`: `npm test` 185 passed, `npx tsc --noEmit` 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `cargo fmt --check`, `git diff --check`: 통과
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `localhost:7700` 실제 검증: `복학원서.hwp` 값 셀에 `가` 160자를 입력한 뒤 첫 행이 확장되고
  caret이 셀 안에 유지됨. 브라우저 error/warn log 없음.

## 5. PR 전 남은 검증

- 실제 Windows Chrome 한글 IME 조합 상태에서 검은 조합창이 셀 경계 안에 유지되는지 수동 확인

## 6. PR 본문 초안

```markdown
## Summary

- 표 셀 장문 입력에서 cursor query가 셀 bbox를 반환하고, caret 및 IME 조합창을 그 범위 안으로 제한했습니다.
- 지연 입력이 셀 높이를 초과한 경우에만 즉시 페이지네이션을 수행해 표 행을 확장합니다.
- one-depth `cellPath` IME 삽입도 일반 셀 삽입 경로를 사용하도록 통일했습니다.

## Tests

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
- `rhwp-studio`: `npm test`, `npx tsc --noEmit`
- `wasm-pack build --target web --out-dir pkg`
- `samples/복학원서.hwp` 브라우저 장문 입력 검증

Closes #1951
```
