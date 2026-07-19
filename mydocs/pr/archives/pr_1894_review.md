# PR #1894 리뷰 — #1858 하단앵커 표 실측 높이 배치

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1894 |
| 제목 | Issue #1858: vert=쪽/용지 valign=Bottom 표를 실측 높이로 anchor 하단에 밀착 |
| 작성자 | planet6897 |
| base | `devel` |
| head | `planet6897:fix/1858-bottom-anchor-offset` |
| 최종 head SHA | `d4790d381cf9edceba715e4dcc0374ea199335d5` |
| merge commit | `f0dfb58b95f7f8446e99499767925196b15832a3` |
| 규모 | 3 files, +156 / -1 |
| mergeable | merge 전 최종 확인: `MERGEABLE`, `CLEAN` |
| CI | 최종 head 기준 CI/CodeQL/Render Diff preflight, Build & Test, Native Skia tests, Canvas visual diff, CodeQL 통과 |

## 변경 범위

- `src/renderer/layout/table_layout.rs`
  - depth 0 Page/Paper anchor pre-pass 에서 `valign=Bottom|Outside` 표의 y 배치 높이를 선언 높이(`common.height`)가 아니라 `MeasuredTable.total_height - caption_height` 기준으로 계산한다.
  - 실측 높이가 없거나 0 이하이면 기존 선언 높이를 유지한다.
- `tests/issue_1858_bottom_anchor_flush.rs`
  - `samples/hwpx/opengov/36389312_...hwpx` 의 하단앵커 표 pi=5/pi=6 하단이 body 하단에 밀착하는지 회귀 테스트를 추가했다.
- `mydocs/troubleshootings/bottom_anchor_declared_vs_rendered_height.md`
  - #1858 발현 2 조사 기록과 PR #1894 보정 결과를 문서화했다.

## 렌더 영향 및 visual sweep 판정

`table_layout.rs` 의 하단앵커 표 y 배치가 바뀌므로 visual sweep 대상이다. PR 이 직접 언급한 `36389312` 샘플과 한컴 2024 기준 PDF가 저장소에 있으므로 해당 1페이지를 비교했다.

실행 명령:

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

visual sweep 결과:

- SVG/PDF 페이지 수: 1 / 1
- 자동 후보: `flagged=0/1`
- frame overflow: 0
- content bottom delta: `-3.0px`
- pixel match: `92.64419%`
- 내용 픽셀 중심 자동 일치율 보조값: `15.94925%`

사람 판정 메모:

- `review_177.png` 기준으로 하단 발신/결재 블록 위치가 기준 PDF와 같은 하단 영역에 배치된다.
- render tree 기준 `Body` 하단은 `1103.6px`, 하단앵커 표 pi=6 하단은 `1099.9px`, pi=5 하단은 `1103.6px` 이다. 신규 테스트의 허용오차 6px 안에 들어 수정 전 40.5px 부유 회귀와 구분된다.
- `visual_accuracy_proxy_percent` 가 낮은 것은 한컴 PDF와 rhwp SVG raster 의 폰트/글리프 잉크 차이가 크게 반영된 값이다. 이번 PR 의 핵심 판단은 하단앵커 블록 y 위치와 frame overflow 여부로 본다.

## 로컬 검증

새 PR review 시작 전 cargo cache 비대화 영향을 줄이기 위해 `/Users/tsjang/rhwp/target` 하위 항목을 삭제한 뒤 순차 실행했다.

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
- `cargo test --profile release-test --tests`: 통과. `svg_snapshot` integration test 포함 실행 확인

## 검토 결과

### 1. 보정 근거는 문서 속성과 측정 레이아웃에 기반한다

보정은 특정 파일명이나 페이지 번호를 직접 맞추는 분기가 아니라, Page/Paper anchor 와 `VertAlign::Bottom|Outside` 조건에서 이미 계산된 `MeasuredTable` 의 행 높이 합을 사용한다. 입력 문서에서 읽은 표 속성(`vert_align`)과 렌더러의 실측 테이블 높이를 근거로 top 을 다시 계산하므로 하드코딩성 보정으로 보지 않는다.

### 2. 기존 하단 고정·footer 계열 회귀는 focused test 와 전체 테스트에서 발견되지 않았다

변경 지점은 모든 하단앵커 표에 영향을 줄 수 있으므로 #1611, #1624, #1658, #1459 계열 focused test 를 함께 확인했다. 전체 integration test 도 release-test profile 로 통과했다.

### 3. update branch 후 최종 CI 통과

초기 검토 시점에는 `MERGEABLE / BEHIND` 였다. `Update branch` 이후 최종 head `d4790d381cf9edceba715e4dcc0374ea199335d5` 기준으로 GitHub Actions 가 모두 통과했고, 최종 merge 상태는 `MERGEABLE / CLEAN` 으로 확인했다.

### 4. #1858 auto-close 는 기대하지 않는다

PR 본문은 `closes #1858` 를 포함하지만 GitHub metadata 의 `closingIssuesReferences` 는 빈 배열이었다. `devel` 대상 PR 의 auto-close 실패 가능성이 있으므로 merge 후 #1858 상태를 확인하고, 열려 있으면 수동 close 또는 후속 코멘트를 남긴다. 단 #1858 코멘트에는 아웃라이어와 별도 기전이 이미 분리되어 있으므로 close 여부는 후속 이슈 분리 상태까지 확인한 뒤 결정한다.

## 최종 권고

merge 완료됐다.

- merge commit: `f0dfb58b95f7f8446e99499767925196b15832a3`
- merge 방식: admin merge
- merge 시각: 2026-07-04T11:04:04Z

merge 후 후속:

- #1858 상태 확인 및 후속 코멘트
- PR #1894 감사/검증 코멘트
- review 문서와 `mydocs/pr/assets/pr_1894_36389312_p177_review.png` 는 docs-only PR 로 archive 반영
- 로컬 `local/pr1894`, `pr1894-merge-test` 등 검토 브랜치 정리
