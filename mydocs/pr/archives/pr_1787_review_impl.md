# PR #1787 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1787
- reviewer assign: `@jangster77`
- 실제 커밋: `9483cf286f747c57aebbb5a2b85f4d94fef07fe0`

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick 9483cf286f747c57aebbb5a2b85f4d94fef07fe0
```

`src/renderer/layout/table_partial.rs` 자동 병합. 충돌 없음.

## Stage 3. 코드 검토

완료.

- `cell_units_fitting_height` 추가.
- RowBreak straddling rowspan 셀만 별도 height-based unit cut 적용.
- clip/top-align 조건을 straddling 셀에도 확장.
- 신규 회귀 테스트 3개가 컷 페이지와 연속 페이지의 overflow/중복 렌더를 guard.

## Stage 4. 검증

완료.

- `git diff --check upstream/devel..HEAD`
- `cargo fmt --check`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
- `cargo run --bin rhwp -- export-pdf samples/table_scattered_header_rowbreak.hwp -o target/tmp-pr1787/table_scattered_header_rowbreak-rhwp.pdf`
- p6 bbox: `dBot=-4`, `|dBot| <= 5`

## Stage 5. 판단

merge 후보.
