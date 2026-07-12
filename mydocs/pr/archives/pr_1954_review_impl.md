# PR #1954 리뷰 구현 로그

## Stage 1. 메타 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1954
- 제목: `C1c 후속: 단일 시리즈 자동 제목 fallback (#1882 잔여)`
- 작성자: `johndoekim`
- base: `devel`
- head: `local/task1882_v2`
- 문서 작성 시점 head SHA: `297141762fbd0e342b616f31a94278dd605e60a2`
- merge commit: `79b4ce216039a08dd0d1957facf027e714f425e0`
- mergedAt: `2026-07-05T14:43:19Z`
- 상태: merged, draft 아님, merge 전 `MERGEABLE`
- CI: 최신 head 기준 CI/CodeQL/Render Diff checks 통과
- reviewer assign: `jangster77`
- 참고: `closingIssuesReferences`는 비어 있음

## Stage 2. 로컬 fetch 및 merge 확인

완료.

```bash
git fetch upstream +pull/1954/head:local/pr1954
git switch -C review/pr1954 local/pr1954
git merge upstream/devel --no-commit --no-ff
```

결과:

- 최신 head `297141762fbd0e342b616f31a94278dd605e60a2`로 갱신했다.
- `upstream/devel` 기준 `Already up to date`로 충돌 없음.
- 이전 head `cae36e5896078fe9bf70429771a125f9e8eeb696`의 Actions run은 모두 완료 상태라 force-cancel 대상 없음.

## Stage 3. 변경 내용 검토

완료.

확인한 변경 범위:

- `src/ooxml_chart/renderer.rs`
- `tests/issue_1882_chart_style_gaps.rs`
- `samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwp`
- `samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwpx`
- `pdf/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목-2022.pdf`
- `mydocs/plans/task_m100_1882_v2.md`
- `mydocs/report/task_m100_1882_v2_report.md`

중점 확인:

- `effective_title` 우선순위가 `명시 제목 -> 단일 시리즈 이름 -> "차트 제목"`으로 바뀐다.
- 기존 `has_title_elem && !auto_title_deleted` 조건은 유지된다.
- model/parser 변경 없이 기존 `OoxmlSeries.name`을 사용한다.
- 다계열 chart는 기존 placeholder 동작을 유지한다.
- 명시 제목, 빈 이름, `autoTitleDeleted` 경계가 단위 테스트에 포함됐다.

## Stage 4. 로컬 검증

완료.

검토 시작 시 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다. PR head 갱신 후 최신 head 기준으로 아래 명령을 순차 실행했다.

```bash
git diff --check upstream/devel...HEAD
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo build
env CARGO_INCREMENTAL=0 cargo test --lib ooxml_chart
env CARGO_INCREMENTAL=0 cargo test --test issue_1882_chart_style_gaps
env CARGO_INCREMENTAL=0 cargo test --test issue_1431_scatter
env CARGO_INCREMENTAL=0 cargo test --test issue_1453_chart_3d_ofpie_routing
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

결과:

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `cargo build`: 통과
- `cargo test --lib ooxml_chart`: 56 passed
- `cargo test --test issue_1882_chart_style_gaps`: 4 passed
- `cargo test --test issue_1431_scatter`: 1 passed
- `cargo test --test issue_1453_chart_3d_ofpie_routing`: 2 passed
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --profile release-test --tests`: 통과, `svg_snapshot.rs` 8 tests 포함 통과

## Stage 5. visual sweep

완료.

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1954-hwp \
  --hwp "samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwp" \
  --pdf "pdf/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목-2022.pdf" \
  --page 1 \
  --out output/pr1954-visual-hwp

python3 scripts/task1274_visual_sweep.py \
  --key pr1954-hwpx \
  --hwp "samples/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목.hwpx" \
  --pdf "pdf/chart/특이케이스/가로막대형_하나만있을떄_단일시리즈제목-2022.pdf" \
  --page 1 \
  --out output/pr1954-visual-hwpx
```

산출물:

- HWP review: `output/pr1954-visual-hwp/pr1954-hwp/review/review_001.png`
- HWPX review: `output/pr1954-visual-hwpx/pr1954-hwpx/review/review_001.png`
- HWP asset: `mydocs/pr/assets/pr_1954_hwp_single_series_title_review_001.png`
- HWPX asset: `mydocs/pr/assets/pr_1954_hwpx_single_series_title_review_001.png`

결과:

- HWP/HWPX 모두 SVG/PDF 페이지 수 1/1
- HWP/HWPX 모두 자동 후보 `flagged=0/1`
- HWP/HWPX 모두 pixel match `97.37344%`
- HWP/HWPX 모두 내용 픽셀 중심 자동 일치율 보조값 `42.02253%`
- 사람 판정: PR 핵심인 chart 제목이 HWP/HWPX 출력과 기준 PDF 모두 `계열 1`로 일치

## Stage 6. 결론

merge 완료로 정리한다.

근거:

- 최신 head 기준 충돌 없음.
- GitHub Actions 최신 head checks 통과.
- 로컬 focused test, 전체 integration test, Clippy 통과.
- visual sweep에서 PR이 약속한 단일 시리즈 제목 fallback이 확인됨.
- admin merge 완료: `79b4ce216039a08dd0d1957facf027e714f425e0`

후속:

- merge 후 #1882 상태와 코멘트 필요 여부 확인.
- review 문서와 visual asset은 절차에 따라 archive/docs-only 경로로 정리.
