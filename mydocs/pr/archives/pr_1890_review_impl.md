# PR #1890 리뷰 구현 로그

## Stage 1. 메타 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1890
- base: `devel`
- head: `local/task1882`
- 최신 head SHA: `6d1eda74a7713e42612333685063bec73fca9ef1`
- merge commit: `4708f08beecfacfe590af55a53418102a904a757`
- 작성 시점 상태: open, not draft, `MERGEABLE`, `CLEAN`
- 작성 시점 CI: Build & Test, CodeQL, Canvas visual diff 통과
- reviewer assign: `jangster77`

## Stage 2. 로컬 fetch 및 merge 시뮬레이션

완료.

```bash
git fetch upstream pull/1890/head:local/pr1890
git switch -c pr1890-merge-test local/pr1890
git merge upstream/devel --no-commit --no-ff
```

결과:

- `Already up to date`
- 충돌 없음
- 검토 브랜치: `pr1890-merge-test`

## Stage 3. 변경 내용 검토

완료.

확인한 변경 범위:

- `src/ooxml_chart/mod.rs`
- `src/ooxml_chart/parser.rs`
- `src/ooxml_chart/renderer.rs`
- `tests/issue_1882_chart_style_gaps.rs`
- #1882 계획/단계/결과 보고 문서

중점 확인:

- PR 목적은 #1431 Track C 하위 #1882 C1c 스타일 4갭 보정이다.
- 변경은 OOXML chart 렌더러와 관련 테스트에 집중되어 있다.
- 코드상 샘플 파일명/페이지 번호를 직접 맞추는 분기는 발견하지 못했다.
- 다만 원형 차트 제목 정책은 기준 PDF와 불일치했다.

## Stage 4. 로컬 검증

완료.

검증 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다.

```bash
git diff --check
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo test --lib ooxml_chart
env CARGO_INCREMENTAL=0 cargo test --test issue_1882_chart_style_gaps
env CARGO_INCREMENTAL=0 cargo test --test issue_1431_scatter
env CARGO_INCREMENTAL=0 cargo test --test issue_1453_chart_3d_ofpie_routing
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

결과:

- `git diff --check`: 통과
- `cargo fmt --check`: 통과
- `cargo test --lib ooxml_chart`: 54 passed
- `cargo test --test issue_1882_chart_style_gaps`: 4 passed
- `cargo test --test issue_1431_scatter`: 1 passed
- `cargo test --test issue_1453_chart_3d_ofpie_routing`: 2 passed
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --profile release-test --tests`: 통과

## Stage 5. visual sweep

완료.

대표 7개 chart sample 을 기준 PDF와 비교했다.

산출물:

- summary: `output/pr1890-visual/summary.json`
- column: `output/pr1890-visual/chart-column/review/review_001.png` → `mydocs/pr/assets/pr_1890_chart_column_review.png`
- stacked column: `output/pr1890-visual/chart-stacked-column/review/review_001.png` → `mydocs/pr/assets/pr_1890_chart_stacked_column_review.png`
- bar: `output/pr1890-visual/chart-bar/review/review_001.png` → `mydocs/pr/assets/pr_1890_chart_bar_review.png`
- scatter: `output/pr1890-visual/chart-scatter/review/review_001.png` → `mydocs/pr/assets/pr_1890_chart_scatter_review.png`
- pie: `output/pr1890-visual/chart-pie/review/review_002.png` → `mydocs/pr/assets/pr_1890_chart_pie_review.png`
- 3D column: `output/pr1890-visual/chart-3d-column/review/review_003.png` → `mydocs/pr/assets/pr_1890_chart_3d_column_review.png`
- 3D stacked column: `output/pr1890-visual/chart-3d-stacked-column/review/review_003.png` → `mydocs/pr/assets/pr_1890_chart_3d_stacked_column_review.png`

판정:

- 자동 후보는 모든 대표 샘플에서 0건이다.
- PR 내용 기준으로 판단했다. 즉, PDF와 픽셀 단위로 모두 같은지보다 PR 이 약속한 제목/팔레트/범례/축 4갭 보정이 맞는지를 우선했다.
- 팔레트, 축 범위, 우측 범례는 여러 대표 샘플에서 반영됐다.
- 3D 입체감, marker shape, 차트 크기/위치 차이는 PR 이 C2 후보 또는 픽셀 parity 범위 밖으로 분리한 항목이라 merge blocker 로 보지 않는다.
- 원형 `2차원원형` 기준 PDF는 제목이 `판매`인데 rhwp는 `차트 제목`으로 렌더한다.
- 이 차이는 PR 범위 밖 C2 fidelity 가 아니라 #1882 의 ① 제목 gap 정책과 직접 관련된 차이다.
- 다만 이번 PR 이 공통 4갭을 넓게 개선하고 테스트를 추가했으므로, 이 잔여 차이는 merge blocker 가 아니라 후속 작업 요청으로 분리한다.

## Stage 6. 결론

merge 수용 + 후속 작업 요청으로 정리한다.

후속 요청 후보:

- 원형 차트에서 series name `판매`가 제목 fallback 으로 쓰이는 기준 PDF 동작을 반영한다.
- 원형 제목 기대값 테스트를 기준 PDF 기준으로 수정한다.
- 결과보고서의 "시각판정 대기/완료"와 "4갭 전부 정합" 표현을 현재 재검증 결과와 맞춘다.

## Stage 7. 후속

PR #1890 은 merge 완료됐다.

남은 후속 처리:

- review 문서와 visual sweep asset 을 docs-only PR 로 반영
- #1882 에 merge 결과와 남은 원형 제목 정책 후속 요청을 코멘트
- PR #1890 에 감사/검증/후속 요청 코멘트
- docs-only PR merge 후 브랜치와 worktree 정리
