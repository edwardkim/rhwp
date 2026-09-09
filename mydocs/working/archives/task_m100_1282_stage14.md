# Task 1282 Stage 14 - PR 검증 중 RowBreak clip 회귀 보정

## 배경

PR 준비 검증 중 `cargo test --profile release-test --tests`가
`issue_713_rowbreak_table_no_intra_row_split`에서 실패했다.

`upstream/devel`에서는 같은 테스트가 통과하므로 task 1282 브랜치의 회귀로 판단했다.

## 원인 후보

Stage 8 이후 partial table 렌더 경로에서 `TableCellNode.clip`을 `is_in_split_row`가
아닌 항상 `true`로 설정했다.

이 값은 렌더러의 셀 클리핑뿐 아니라 기존 RowBreak 테스트에서 "행 내부 분할 여부"를
판정하는 신호로도 쓰이므로, 모든 셀이 분할된 것처럼 보이는 부작용이 생겼다.

## 수정 방향

- `TableCellNode.clip`의 기존 의미를 복원한다.
- task 1282의 그림/셀 경계 시각 동작은 별도 검증으로 유지 여부를 확인한다.

## 검증 계획

- `cargo test --profile release-test --test issue_713`
- `cargo test --test issue_1282_rotated_cell_picture_resize`
- task 1282 E2E 시각 검증 스크립트
- PR 준비 전체 검증 재개

## 수정

- `src/renderer/layout/table_partial.rs`의 partial table cell 생성에서 `clip` 값을
  `true` 고정이 아니라 기존 의미인 `is_in_split_row`로 복원했다.

## 검증 결과

통과:

```text
cargo test --profile release-test --test issue_713
cargo test --test issue_1282_rotated_cell_picture_resize
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && node e2e/table-picture-resize-1282.test.mjs --mode=headless
cargo fmt --check
cd rhwp-studio && npm run build
cargo build --release
```

판정:

- `issue_713_rowbreak_table_no_intra_row_split` 회귀가 해소됐다.
- task 1282 전용 Rust/E2E 검증은 유지된다.
- PR 준비 전체 검증은 Stage14 커밋 이후 이어서 수행한다.
