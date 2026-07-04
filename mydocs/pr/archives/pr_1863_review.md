# PR #1863 Review — 빈 앵커 host_line_spacing 소스 무관화

## 메타

| 항목 | 내용 |
|---|---|
| PR | #1863 |
| 제목 | `Task #1836: 빈-앵커 host_line_spacing 억제 소스 무관화 (seoul_0776 라운드트립 회귀 해소)` |
| 작성자 | `planet6897` |
| base | `devel` |
| head | `pr/devel-1836` |
| 관련 이슈 | #1836 |
| 문서 작성 시점 head | `61c967303fac62eec1d5939785fcbaa1a19d95d7` |
| merge 전 확인 | CI 통과 / `MERGEABLE` / `CLEAN` 확인됨 |
| merge commit | `0e7ddc83f0f1ee1034011aeba7f11c4fd8777214` |
| auto-close | PR metadata 기준 closing issue 없음. #1836은 merge 후 수동 close 확인 대상 |

## 변경 범위

- `src/renderer/typeset.rs`
  - `format_table` 의 `is_hwpx_source` 인자를 제거한다.
  - 빈 `TopAndBottom` 비-TAC 표 앵커의 `host_line_spacing` 억제를 HWPX 전용이 아니라 소스 무관 규칙으로 바꾼다.
  - 다음 문단이 빈 TAC 표 앵커인 경우는 표-표 스택으로 보고 `host_line_spacing` 을 보존한다.
- `mydocs/plans/task_m100_1836.md`
- `mydocs/report/task_m100_1836_report.md`

## 로컬 검증

검증은 `/private/tmp/rhwp-pr1863-review` worktree 에서 최신 PR head 기준으로 수행했다.

| 항목 | 결과 |
|---|---|
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test svg_snapshot` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_rowbreak_chart_overlap` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | 통과 |

`cargo test --profile release-test --tests` 에 `tests/svg_snapshot.rs` 가 포함되는 것도 확인했다.
focused `svg_snapshot`은 렌더 영향권 빠른 확인용으로 별도 실행했다.

## 시각 검증

이 PR 은 `src/renderer/typeset.rs` 의 pagination/spacing 규칙을 바꾸므로 visual sweep 대상이다.
기준 PDF가 있는 대표 영향권 샘플 `rowbreak-problem-pages` HWP/HWPX 양쪽을 p11-13 범위로 확인했다.

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target pr1863-rowbreak-hwp samples/rowbreak-problem-pages.hwp pdf/rowbreak-problem-pages-2024.pdf \
  --file-target pr1863-rowbreak-hwpx samples/rowbreak-problem-pages.hwpx pdf/rowbreak-problem-pages-2024.pdf \
  --pages 11-13 \
  --out output/pr1863-visual \
  --rhwp-bin target/debug/rhwp
```

결과:

| target | 페이지 수 | selected pages | 자동 후보 | 사람 판정 |
|---|---:|---|---|---|
| `pr1863-rowbreak-hwp` | SVG 18 / PDF 18 | 11-13 | p11 tail/column 후보 | p12 PartialTable overflow 및 p13 첫 텍스트 상단 침범 없음 |
| `pr1863-rowbreak-hwpx` | SVG 18 / PDF 18 | 11-13 | p11 tail 후보 | p12 PartialTable overflow 및 p13 첫 텍스트 상단 침범 없음 |

대표 산출물:

- HWP p11 review: `/private/tmp/rhwp-pr1863-review/output/pr1863-visual/pr1863-rowbreak-hwp/review/review_011.png`
- HWP p12 review: `/private/tmp/rhwp-pr1863-review/output/pr1863-visual/pr1863-rowbreak-hwp/review/review_012.png`
- HWP p13 review: `/private/tmp/rhwp-pr1863-review/output/pr1863-visual/pr1863-rowbreak-hwp/review/review_013.png`
- HWPX p12 review: `/private/tmp/rhwp-pr1863-review/output/pr1863-visual/pr1863-rowbreak-hwpx/review/review_012.png`

PR 기록용 asset:

- `mydocs/pr/assets/pr_1863_rowbreak_hwp_p011_review.png`
- `mydocs/pr/assets/pr_1863_rowbreak_hwp_p012_review.png`
- `mydocs/pr/assets/pr_1863_rowbreak_hwp_p013_review.png`
- `mydocs/pr/assets/pr_1863_rowbreak_hwpx_p012_review.png`

`visual_accuracy_proxy_percent`:

- HWP p11: 16.47996
- HWP p12: 13.73782
- HWP p13: 53.11426
- HWPX p11: 16.44237
- HWPX p12: 13.91085
- HWPX p13: 53.07545

코멘트: 내용 픽셀 중심 자동 일치율 보조값은 낮은 페이지가 있으나, 이 값은 사람 판정 정확도가 아니라
raster 일치율 보조값이다. 본 PR 의 핵심 확인 대상인 p12/p13의 overflow/상단 침범 회귀는 눈으로 확인한
범위에서 재현되지 않았다.

## PR 내용 검토

기존 코드는 `is_hwpx_source` 조건 때문에 HWPX 입력에서만 빈 앵커 spacing 억제가 적용되고, HWP5 재파스
경로에서는 같은 레이아웃 상황에서도 `host_line_spacing` 이 남을 수 있었다. PR 은 이 조건을 제거해
typeset pagination 과 layout 렌더 경로의 소스 무관 규칙을 맞춘다.

두 번째 커밋은 소스 무관화가 native HWP5 표-표 스택을 과하게 좁히는 회귀를 막기 위해
`para_is_empty_tac_table_anchor` 를 추가했다. 빈 `TopAndBottom` 표 앵커 뒤에 빈 TAC 표 앵커가 오는 경우는
일반 문단으로 넘기지 않고 표 사이 간격으로 보존하는 해석이라, PR 설명과 로컬 회귀 테스트 범위에 맞다.

## workflow 검증 메모

- `mydocs/manual/pr_review_workflow.md` 의 reviewer assign 규칙에 따라 `jangster77` review request 를 확인했다.
- update branch 이후 이전 SHA run 은 모두 completed 상태였고, 최신 head 의 CI 는 취소하지 않았다.
- 4.2 merge simulation 에서 `upstream/devel` 이 이미 포함되어 `Already up to date` 가 나왔다. 이 케이스는
  `git merge --abort` 대상이 아니므로 workflow 문서에 조건부 abort 문구를 보강했다.
- Cargo 검증은 package/artifact lock 경합을 피하기 위해 순차 실행해야 함을 workflow 문서에 보강했다.
- 후속 처리도 원 PR merge, 문서/asset PR, issue close/comment, PR comment, branch/worktree cleanup 순서로
  순차 진행하도록 workflow 문서에 보강했다.
- visual sweep asset 을 실제 review 근거로 사용했으므로 대표 PNG 를 `mydocs/pr/assets/` 에 복사해야 하는
  게이트를 workflow 문서에 보강했다.
- 옵션 2 후속 문서/asset PR 이 필요한 경우 asset raw URL 이 유효해진 뒤 issue close/comment 와 원 PR comment 를
  남기도록 7장 후속 처리 순서를 보정했다.

## 판단

로컬 검증, GitHub Actions, 대표 visual sweep 기준으로 핵심 동작은 PR 목적과 일치했고, PR #1863은
`0e7ddc83f0f1ee1034011aeba7f11c4fd8777214` 로 merge 완료됐다.

현재 review 문서, workflow 보강, visual asset 은 원 PR #1863 diff 에 포함되어 있지 않으므로 별도
문서/asset fast-pass PR 로 반영한다. 그 후 `devel` 기준 asset URL 을 사용해 #1863 comment 와 #1836 수동
close/comment 를 처리한다.
