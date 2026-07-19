# PR #1894 리뷰 구현 로그

## Stage 1. 메타 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1894
- base: `devel`
- head: `planet6897:fix/1858-bottom-anchor-offset`
- 최종 head SHA: `d4790d381cf9edceba715e4dcc0374ea199335d5`
- merge commit: `f0dfb58b95f7f8446e99499767925196b15832a3`
- 최종 상태: merged, `MERGEABLE`, `CLEAN`
- 최종 CI: CI/CodeQL/Render Diff preflight, Build & Test, Native Skia tests, Canvas visual diff, CodeQL 통과
- reviewer assign: `jangster77`

## Stage 2. 로컬 fetch 및 merge 시뮬레이션

완료.

```bash
git fetch upstream pull/1894/head:local/pr1894
git switch -c pr1894-merge-test local/pr1894
git merge upstream/devel --no-commit --no-ff
```

결과:

- 충돌 없음
- PR 이 `BEHIND` 라 upstream/devel 변경이 staged 되는 merge simulation 상태가 됨
- 검증 완료 후 `git merge --abort`, `git switch devel`, `git merge --ff-only upstream/devel` 로 정리

## Stage 3. 변경 내용 검토

완료.

확인한 변경 범위:

- `src/renderer/layout/table_layout.rs`
- `tests/issue_1858_bottom_anchor_flush.rs`
- `mydocs/troubleshootings/bottom_anchor_declared_vs_rendered_height.md`

중점 확인:

- PR 목적은 #1858 발현 2인 `vert=쪽/용지` + `valign=Bottom` 하단앵커 표의 상향 부유를 해소하는 것이다.
- 변경은 depth 0 Page/Paper anchor pre-pass 의 Bottom/Outside y 계산에 국한되어 있다.
- 보정은 `MeasuredTable` 의 실측 행 높이 합과 `VertAlign` 속성을 사용하며, 특정 샘플명·페이지 번호·이슈 번호 기반 분기는 없다.
- `Outside` 는 기존 `compute_table_y_position` 에서 Bottom 과 같은 분기로 처리되던 계열이라 같은 effective height 경로에 포함된다.

## Stage 4. 로컬 검증

완료.

검증 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다.

```bash
git diff --check HEAD
git diff --cached --check
env CARGO_INCREMENTAL=0 cargo test --test issue_1858_bottom_anchor_flush
env CARGO_INCREMENTAL=0 cargo test --test issue_1858
env CARGO_INCREMENTAL=0 cargo test --test issue_1611_footer_page_bottom_pagination
env CARGO_INCREMENTAL=0 cargo test --test issue_1624_footer_overpush_pagination
env CARGO_INCREMENTAL=0 cargo test --test issue_1658_page_bottom_fixed_exclusion
env CARGO_INCREMENTAL=0 cargo test --test issue_1459_topbottom_picture_reflow
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

결과:

- `git diff --check HEAD`: 통과
- `git diff --cached --check`: 통과
- `cargo test --test issue_1858_bottom_anchor_flush`: 1 passed
- `cargo test --test issue_1858`: 1 passed
- `cargo test --test issue_1611_footer_page_bottom_pagination`: 1 passed
- `cargo test --test issue_1624_footer_overpush_pagination`: 1 passed
- `cargo test --test issue_1658_page_bottom_fixed_exclusion`: 3 passed
- `cargo test --test issue_1459_topbottom_picture_reflow`: 3 passed
- `cargo fmt --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --profile release-test --tests`: 통과

## Stage 5. visual sweep

완료.

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --out output/pr1894-visual \
  --rhwp-bin target/debug/rhwp \
  --key pr1894-36389312 \
  --hwp "samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx" \
  --pdf "pdf/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177)-2024.pdf"
```

산출물:

- summary: `output/pr1894-visual/summary.json`
- compare: `output/pr1894-visual/pr1894-36389312/compare/compare_177.png`
- overlay: `output/pr1894-visual/pr1894-36389312/overlay/overlay_177.png`
- review: `output/pr1894-visual/pr1894-36389312/review/review_177.png`
- 증적 asset: `mydocs/pr/assets/pr_1894_36389312_p177_review.png`

결과:

- SVG/PDF 페이지 수: 1 / 1
- 자동 후보: `flagged=0/1`
- frame overflow: 0
- content bottom delta: `-3.0px`
- pixel match: `92.64419%`
- 내용 픽셀 중심 자동 일치율 보조값: `15.94925%`

render tree 확인:

```text
Body  y=18.9 h=1084.7 bottom=1103.6
Table pi=6 ci=0 y=783.2 h=316.7 bottom=1099.9
Table pi=5 ci=0 y=856.3 h=247.3 bottom=1103.6
```

판정:

- PR 핵심인 하단앵커 표의 하단 밀착은 신규 테스트와 visual sweep 모두에서 확인된다.
- 폰트/글리프 차이 때문에 잉크 기반 자동 일치율 보조값은 낮지만, 이번 PR 의 merge 판단 핵심인 y 위치와 frame overflow 에서는 blocker 를 발견하지 못했다.

## Stage 6. 결론

merge 완료로 정리한다.

결과:

- 초기 검토 시점 `BEHIND` 상태를 `Update branch` 로 해소했다.
- update branch 후 로컬 `local/pr1894` 도 `upstream/devel` 기준으로 rebase 하고 `issue_1858_bottom_anchor_flush` 를 재확인했다.
- 최종 head `d4790d381cf9edceba715e4dcc0374ea199335d5` 기준 GitHub Actions 가 모두 통과했다.
- admin merge 완료: `f0dfb58b95f7f8446e99499767925196b15832a3`

후속:

- #1858 close/후속 코멘트 확인
- PR #1894 감사/검증 코멘트
- review 문서와 visual asset 을 docs-only PR 로 archive 반영
- 로컬 PR review 브랜치 정리
