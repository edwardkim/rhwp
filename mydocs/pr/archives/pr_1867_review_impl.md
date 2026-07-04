# PR #1867 리뷰 구현 기록

## Stage 1. 메타 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1867
- base: `devel`
- head: `task/m100-1733-residual-overpagination-v2`
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1733
- 기존 PR #1856 은 closed 상태이며, #1867 은 최신 `devel` 기준 정리본이다.

## Stage 2. 변경 내용 검토

완료.

- `typeset.rs` 변경은 저장 `LINE_SEG`/vpos 증거가 있는 tail split 과 vpos-reset 직전 하단 빈 문단 bridge 에
  한정된다.
- 공통 tolerance 는 넓히지 않고, #1733 전용 helper 경로에서만 넓은 tolerance 를 사용한다.
- `PartialTable` 이 이미 있는 페이지와 RowBreak 인접 문단은 완화 대상에서 제외한다.
- HWPX/HWP 샘플 모두 기준 PDF 242쪽과 맞는지 `tests/issue_1733.rs` 로 고정했다.
- `pr_review_workflow.md` 는 옵션 1 후속 코멘트 및 issue URL 참조 규칙을 보강한다.

## Stage 3. 로컬 검증

완료.

- `cargo build --release`
  - 통과
- `env CARGO_INCREMENTAL=0 cargo test --release --lib`
  - 통과: `2075 passed; 0 failed; 6 ignored`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
  - 통과
- `cargo fmt --check`
  - 통과
- `git diff --check`
  - 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
  - 통과
- `cargo test --doc`
  - 통과: `0 passed; 0 failed; 1 ignored`
- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `cd rhwp-studio && npm test`
  - 통과: `153 pass`
- `wasm-pack build --target web --out-dir pkg`
  - 통과
- `cargo test --test svg_snapshot`
  - 통과: `8 passed`

집중 회귀:

- `issue_1733`: 2개 통과
- `issue_1073_nested_table_split`: 3개 통과
- `issue_rowbreak_chart_overlap`: 20개 통과

## Stage 4. update branch 확인

완료.

- GitHub `Update branch` 로 `79b9fecda24fd83f3cd56cec8499ab7de2b0bba1` merge commit 이 추가됐다.
- update branch 로 들어온 최신 `devel` 변경은 #1667 Render Diff cache workflow/문서 변경이다.
- 로컬 브랜치는 `upstream/task/m100-1733-residual-overpagination-v2` 로 fast-forward 했다.

## Stage 5. 옵션 1 문서 반영

진행 완료.

- `mydocs/pr/archives/pr_1867_review.md`
- `mydocs/pr/archives/pr_1867_review_impl.md`
- `mydocs/pr/assets/pr_1867_issue1733_hwpx_review_p242.png`
- `mydocs/orders/20260703.md`

대표 visual sweep:

- p242, `flagged=0/1`
- `visual_accuracy_proxy_percent`: 약 `21.54%`
- 임시 경로: `output/pr1867_visual/pr1867-issue1733-hwpx-p242/review/review_242.png`
- 보존 asset: `mydocs/pr/assets/pr_1867_issue1733_hwpx_review_p242.png`

위 문서를 PR head 에 포함해 remote push 한다. CI 완료 후 merge 및 후속 처리는
`mydocs/manual/pr_review_workflow.md` 기준으로 진행한다.
