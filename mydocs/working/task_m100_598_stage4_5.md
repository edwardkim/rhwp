# Task #598 Stage 4-5 완료보고서 — CI 저장 테스트 회귀 보정

## 작업 범위

- PR #642 `Build & Test` 실패 원인 분석
- `wasm_api::tests::test_save_text_only` 회귀 보정
- #598 각주 앞 Backspace anchor/Undo 보정 유지 확인

## 원인

Stage 4-4에서 `insert_text_at()` 에 추가한 inline control 앞 삽입 보정이 모든 control position에 적용됐다.

`template/empty.hwp` 의 첫 문단은 텍스트가 비어 있지만 `SectionDef`, `ColumnDef` 컨트롤 2개를 가진다. 기존 동작은 텍스트를 두 컨트롤 뒤 UTF-16 offset `16`부터 삽입해야 한다. 그러나 새 조건이 두 컨트롤의 position `0`을 inline control 앞 삽입으로 오판해 offset `0`부터 텍스트를 삽입했고, 저장 후 caret 위치가 `24`가 아니라 `8`로 기록됐다.

## 구현 내용

- `insert_text_at()` 의 `inserts_before_inline_control` 조건을 실제 본문 흐름 inline control로 제한했다.
- 대상 컨트롤:
  - `Shape`
  - `Table`
  - `Picture`
  - `Equation`
  - `Footnote`
  - `Endnote`
- `SectionDef`, `ColumnDef` 같은 문단 메타 컨트롤은 해당 분기에 들어가지 않도록 했다.

## 검증

실행 명령:

```bash
cargo test wasm_api::tests::test_save_text_only --lib -- --nocapture
cargo test --test issue_598_footnote_marker_nav
cargo test --lib
git diff --check
```

결과:

- `test_save_text_only`: 통과
- `issue_598_footnote_marker_nav`: 4 passed
- `cargo test --lib`: 1135 passed, 0 failed, 2 ignored
- `git diff --check`: 통과

## 판단

CI 실패는 GitHub Actions 환경 문제가 아니라 Stage 4-4 구현 범위가 과하게 넓었던 실제 회귀였다. inline control 판정을 본문 흐름 컨트롤로 제한해 빈 문서 저장 caret 동작과 각주 앞 Backspace Undo 동작을 모두 만족하도록 보정했다.
