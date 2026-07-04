# PR #1867 리뷰 기록

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1867 |
| 작성자 | `jangster77` |
| base | `devel` |
| head | `task/m100-1733-residual-overpagination-v2` |
| 관련 이슈 | https://github.com/edwardkim/rhwp/issues/1733 |
| 처리 방식 | 옵션 1. 현재 PR 에 review 문서 포함 |
| 작성 시점 head | `79b9fecda24fd83f3cd56cec8499ab7de2b0bba1` |
| 작성 시점 상태 | Open, non-draft, update branch 완료, CI 대기 |

## PR 요지

#1733 의 국제고속선기준 tail/vpos-reset 잔여 over-pagination 을 다시 추적해,
`samples/task1725/text_footnote_tail_overpagination.{hwpx,hwp}` 렌더 페이지 수를 기준 PDF
`pdf/text_footnote_tail_overpagination-2024.pdf` 의 242쪽과 맞춘다.

초기 `upstream/devel` 기준 page count 는 다음과 같았다.

| 파일 | 수정 전 rhwp | 기준 PDF |
|---|---:|---:|
| `samples/task1725/text_footnote_tail_overpagination.hwpx` | 250쪽 | 242쪽 |
| `samples/task1725/text_footnote_tail_overpagination.hwp` | 249쪽 | 242쪽 |

이 PR 은 저장 `LINE_SEG`/vpos 증거가 있는 partial paragraph tail 과 vpos-reset 직전 하단 빈 문단 bridge 에
한정해 현재 쪽 배치를 허용한다. 공통 tolerance 확산과 RowBreak 인접 흐름 회귀는 별도 가드로 막았다.

## 변경 범위

- `src/renderer/typeset.rs`
  - partial paragraph tail split 완화 조건을 저장 vpos 증거가 있는 tail 흐름으로 제한한다.
  - #1733 전용 경로만 128px tolerance 를 사용하고, 공통 `saved_bounds_fit_at_flow_tail` tolerance 는 유지한다.
  - 하단 빈 문단 bridge 흡수는 `PartialTable` 이 이미 있는 페이지와 RowBreak 인접 문단에서 제외한다.
- `tests/issue_1733.rs`
  - HWPX/HWP 두 샘플 모두 242쪽이어야 한다는 회귀 게이트를 추가한다.
- `mydocs/plans/task_m100_1733_v2.md`
- `mydocs/working/task_m100_1733_stage1.md`
- `mydocs/working/task_m100_1733_stage2.md`
- `mydocs/manual/pr_review_workflow.md`
  - 옵션 1 후속 코멘트와 issue URL 참조 규칙을 보강한다.

## update branch 확인

PR 생성 후 GitHub `Update branch` 가 수행되어 다음 merge commit 이 PR head 에 추가됐다.

- `79b9fecda24fd83f3cd56cec8499ab7de2b0bba1`
- message: `Merge branch 'devel' into task/m100-1733-residual-overpagination-v2`

해당 update 로 들어온 최신 `devel` 변경은 #1667 Render Diff cache 관련 workflow/문서 변경이다. #1733 코드 변경은
그대로 유지되며, 로컬 브랜치는 위 head 로 fast-forward 했다.

## 로컬 검증

update branch 직전 PR 코드 기준 full 검증을 수행했다. update branch 로 추가된 `devel` 변경은 workflow/문서
범위라 #1733 렌더 코드 자체는 변하지 않았다.

```bash
cargo build --release
```

결과: 통과.

```bash
env CARGO_INCREMENTAL=0 cargo test --release --lib
```

결과: 통과. `2075 passed; 0 failed; 6 ignored`.

```bash
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

결과: 통과.

```bash
cargo fmt --check
git diff --check
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
cargo test --doc
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
cargo test --test svg_snapshot
```

결과: 모두 통과. `npm test` 는 153개 테스트 통과, `svg_snapshot` 은 8개 테스트 통과.

집중 회귀 검증도 별도 수행했다.

```bash
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1733
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1073_nested_table_split
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_rowbreak_chart_overlap
```

결과: 각각 2개, 3개, 20개 테스트 통과.

## 시각/페이지 검증

이번 PR 의 핵심 판정 기준은 전체 페이지 수 정합이다. 옵션 1 기준에 따라 대표 visual sweep PNG 도 PR asset 으로
함께 남긴다.

- 기준 PDF: `pdf/text_footnote_tail_overpagination-2024.pdf`, 242쪽
- HWPX: `samples/task1725/text_footnote_tail_overpagination.hwpx`, 242쪽
- HWP: `samples/task1725/text_footnote_tail_overpagination.hwp`, 242쪽
- 회귀 게이트: `tests/issue_1733.rs`

대표 페이지는 마지막 페이지 p242 로 잡았다. 이 페이지가 생성된다는 것은 추가 빈 페이지 없이 기준 PDF와 같은
242쪽에서 문서가 끝난다는 점을 보여준다.

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1867-issue1733-hwpx-p242 \
  --hwp samples/task1725/text_footnote_tail_overpagination.hwpx \
  --pdf pdf/text_footnote_tail_overpagination-2024.pdf \
  --page 242 \
  --out output/pr1867_visual \
  --rhwp-bin target/release/rhwp
```

결과:

- SVG pages: 242
- PDF pages: 242
- selected pages: `[242]`
- `flagged=0/1`
- `visual_accuracy_proxy_percent`: 약 `21.54%`
- `content_bottom_delta_px`: `1.0`
- line band drift: max `2.5px`, mean `0.7~0.8px`

산출물:

- compare: `output/pr1867_visual/pr1867-issue1733-hwpx-p242/compare/compare_242.png`
- overlay: `output/pr1867_visual/pr1867-issue1733-hwpx-p242/overlay/overlay_242.png`
- review: `output/pr1867_visual/pr1867-issue1733-hwpx-p242/review/review_242.png`
- 보존 asset: `mydocs/pr/assets/pr_1867_issue1733_hwpx_review_p242.png`

사람 판정:

- p242 기준 rhwp/PDF 모두 같은 마지막 페이지를 렌더링한다.
- 자동 후보는 없고, 본문 line band drift 는 1~2.5px 수준이다.
- `visual_accuracy_proxy_percent` 는 낮지만 이는 글꼴/잉크 픽셀 중심 보조값이며, 사람 판정 정확도가 아니다.
- 이번 PR 의 merge blocker 는 페이지 수 over-pagination 이므로, 242쪽 정합과 마지막 페이지 존재 확인을 핵심 근거로 본다.

## GitHub CI

작성 시점 상태:

- PR head: `79b9fecda24fd83f3cd56cec8499ab7de2b0bba1`
- mergeable: `MERGEABLE`
- mergeStateStatus: `BLOCKED`
- status check: update branch 직후 대기 상태

CI 완료 후 merge 가능 여부를 최종 확인한다.

## 결론

로컬 full 검증과 page-count 회귀 게이트 기준으로 merge 후보로 판단한다. 최종 merge 는 #1867 최신 head 기준
GitHub Actions 통과 후 진행한다.
