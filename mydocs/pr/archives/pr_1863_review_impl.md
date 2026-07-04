# PR #1863 Review Impl — 빈 앵커 host_line_spacing 소스 무관화

## Stage 1. PR 메타 확인

완료.

- PR #1863 은 open 상태이며 draft 가 아니다.
- base 는 `devel`, head 는 `pr/devel-1836` 이다.
- 작성자는 `planet6897` 이고 maintainer 수정 가능한 PR 이다.
- 최신 head 는 `61c967303fac62eec1d5939785fcbaa1a19d95d7` 이다.
- reviewer 로 `jangster77` 가 assign 되어 있다.

## Stage 2. 이전 SHA CI 정리 확인

완료.

- `910dc729c8c42dac66f046f307336f097f6c98de` 의 run 은 모두 completed 상태였다.
- `d7fe8a5ed34e1e19eaed8593a7abbc2155c59751` 의 run 은 모두 completed 상태였다.
- 최신 head `61c967303fac62eec1d5939785fcbaa1a19d95d7` 의 CI 만 monitoring 대상이다.
- 이전 SHA 에 대해 force-cancel 할 in-progress run 은 없었다.

## Stage 3. 로컬 merge simulation

완료.

```bash
git fetch upstream pull/1863/head:local/pr1863 --force
git worktree add --detach /private/tmp/rhwp-pr1863-review local/pr1863
git merge upstream/devel --no-commit --no-ff
```

결과:

- `local/pr1863` 은 `upstream/devel` 을 이미 포함한다.
- merge simulation 은 `Already up to date.` 로 끝났다.
- 이 경우 active merge state 가 없으므로 `git merge --abort` 는 수행하지 않는다.

## Stage 4. 변경 내용 검토

완료.

- `format_table` 에서 `is_hwpx_source` 인자가 제거되어 빈 앵커 spacing 억제가 소스 무관 규칙이 됐다.
- HWPX 원 케이스는 억제 유지, HWP5 재파스 케이스는 phantom `host_line_spacing` 누적을 피하는 방향이다.
- `para_is_empty_tac_table_anchor` 가 추가되어 빈 TAC 표 앵커 후속 문단은 표-표 스택으로 보존한다.
- 이 보정은 `issue_rowbreak_chart_overlap` 회귀 범위와 PR 설명에 부합한다.

## Stage 5. 로컬 검증

완료.

순차 실행으로 확인했다.

```bash
git diff --check upstream/devel...HEAD
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test svg_snapshot
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_rowbreak_chart_overlap
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
```

결과:

- 전체 통과
- full integration test 에 `tests/svg_snapshot.rs` 가 포함됨을 확인
- clippy 는 warning 없이 완료

## Stage 6. Visual sweep

완료.

대상:

- `samples/rowbreak-problem-pages.hwp` vs `pdf/rowbreak-problem-pages-2024.pdf`
- `samples/rowbreak-problem-pages.hwpx` vs `pdf/rowbreak-problem-pages-2024.pdf`
- pages 11-13

결과:

- HWP/HWPX 모두 SVG 18 pages / 기준 PDF 18 pages 로 페이지 수 일치
- 자동 후보는 p11 tail/column 계열만 감지
- PR 보고서가 지목한 p12 PartialTable overflow, p13 첫 텍스트 상단 침범은 review PNG 확인 범위에서 재현되지 않음
- 대표 review PNG 는 `mydocs/pr/assets/pr_1863_rowbreak_*_review.png` 로 복사했고, 임시 산출물과 최종 asset 경로를 `mydocs/pr/archives/pr_1863_review.md` 에 기록

## Stage 7. GitHub Actions 상태

완료.

- CodeQL 계열: 통과
- Render Diff 계열: 통과
- CI preflight: 통과
- `Build & Test`: 통과
- mergeable: `MERGEABLE`
- merge state: `CLEAN`
- PR metadata 기준 closing issue 없음. #1836은 merge 후 수동 close 확인 대상
- PR #1863 merge 완료: `0e7ddc83f0f1ee1034011aeba7f11c4fd8777214`

## Stage 8. workflow 문서 검증

진행 중 PR 리뷰를 통해 다음 문구 보강이 필요함을 확인하고 `mydocs/manual/pr_review_workflow.md` 에 반영했다.

- merge simulation 이 `Already up to date` 인 경우 `git merge --abort` 를 생략한다.
- Cargo 검증은 병렬 실행하지 않고 순차 실행한다.
- 렌더 영향 PR 은 cargo/svg_snapshot 통과만으로 끝내지 않고 visual sweep 필요 여부를 사전 판정한다.
- 기준 PDF 가 없으면 보류 사유만 적지 않고 PR 작성자 또는 reviewer 에게 기준 PDF 업로드를 요청한다.
- visual sweep 근거 PNG 는 merge 가능/승인 요청 전 `mydocs/pr/assets/` 에 복사하고 review 문서에 최종 asset
  경로를 기록한다.
- 후속 문서/asset PR 이 필요한 경우 raw asset URL 이 유효해진 뒤 issue close/comment 와 원 PR comment 를
  남긴다.
- 후속 처리도 원 PR merge, 문서/asset PR, issue close/comment, PR comment, branch/worktree cleanup 순서로
  순차 실행한다.
- PR head branch 와 로컬 review branch 이름이 다를 수 있으므로 cleanup 전에 실제 `headRefName` 을 확인한다.

## 남은 작업

- 별도 문서/asset fast-pass PR 생성 및 merge
- 문서/asset PR 에 포함할 항목:
  - `mydocs/manual/pr_review_workflow.md`
  - `mydocs/pr/archives/pr_1863_review.md`, `mydocs/pr/archives/pr_1863_review_impl.md`
  - `mydocs/pr/assets/pr_1863_rowbreak_*_review.png`
  - `mydocs/orders/20260703.md` PR 처리 기록
- 문서/asset PR merge 후 #1836 수동 close 확인 및 #1863 감사/comment 처리
- 처리 완료 후 `/private/tmp/rhwp-pr1863-review`, `local/pr1863`, `pr/devel-1836` 정리 여부 확인
