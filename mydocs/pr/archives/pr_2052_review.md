# PR #2052 리뷰 — 1x1 중첩셀 콘텐츠 페이지 분할

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/2052 |
| 작성자 | @planet6897 |
| 제목 | `Issue #2007: 1×1 중첩셀 콘텐츠 페이지 분할 (6p→15p, intra-cell pagination)` |
| base / head | `devel` / `fix/2007-nested-cell-pagination` |
| 검토 기준 head | `4af4e7694757df5f8259800c70bae24e3bde0944` |
| reviewer assign | @jangster77 지정 완료 |
| maintainerCanModify | true |
| GitHub merge state | `CLEAN` |

## 변경 범위

- `samples/basic/issue2007_nested_cell_pagination_42065.hwp` 추가.
- `src/renderer/layout/table_layout.rs`
  - 빈 텍스트 문단 안의 1x1 중첩 표가 명백한 다중 페이지 콘텐츠일 때
    `nested_table_mixed_fragment_heights` 기반 fragment 분해를 적용한다.
  - 현재 구현 게이트는 `frags.len() > 1 && total_frag_h > page_avail * 2.0`
    (`page_avail <= 0` fallback: `1800.0`)이다.
- `tests/issue_2007_nested_cell_pagination.rs`
  - 42065 샘플이 수정 전 6쪽 수준으로 붕괴하지 않는지 회귀 테스트한다.
- `mydocs/working/task_m100_pagination_redesign_stageA3.md` 추가.

## MCP 기준 PDF

PR에 기준 PDF가 첨부되어 있지 않아 HWP 2020 MCP로 기준 PDF를 생성했다.

| 항목 | 내용 |
|------|------|
| 입력 HWP | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| 입력 SHA-256 | `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626` |
| 출력 PDF | `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf` |
| 출력 SHA-256 | `1f9d2f5705a64899c2b081832d2e6548dfe7bc3b9d1fb1b92f41221d39c8b3e7` |
| MCP job id | `f69ed018-28b7-4306-86af-190afe8e7fba` |
| run_status / validation | `0` / `ok` |
| pdfinfo | 17 pages, A4, Producer `cairo 1.18.0` |

## 로컬 검증

검토 worktree: `/Users/tsjang/rhwp`

- `git diff --check` 통과.
- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 항목 삭제 완료.
- `cargo fmt --check` 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2007_nested_cell_pagination --test issue_1891 --test issue_1488_rowbreak_empty_overlay_pages --test issue_1749_saved_bounds_page_break --test issue_2015_saved_bounds_rowbreak` 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib` 통과: 2150 passed, 7 ignored.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과: `svg_snapshot` 포함 전체 integration tests 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과.
- `cargo build --bin rhwp` 통과.
- `target/debug/rhwp info samples/basic/issue2007_nested_cell_pagination_42065.hwp`
  - rhwp 페이지 수: 15쪽.

## GitHub CI

검토 기준 head `4af4e7694757df5f8259800c70bae24e3bde0944` 기준:

- CodeQL / Analyze python / Analyze javascript-typescript / Analyze rust: 통과.
- Build & Test / Build default-feature tests / Native Skia tests: 통과.
- Canvas visual diff: 통과.
- preflight 계열: 통과.
- WASM Build: skipped.

## 시각 검증

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr2052-issue2007-42065 \
  --hwp samples/basic/issue2007_nested_cell_pagination_42065.hwp \
  --pdf pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf \
  --out output/pr2052_visual
```

요약:

- SVG pages: 15
- render tree pages: 15
- MCP PDF pages: 17
- compared pages: 15
- `flagged=10/15`
- visible frame overflow pages: 없음
- red marker drift pages: 없음
- line/column/tail/large drift: 주로 6쪽 이후
- render tree tail overflow 후보: 6-15쪽
- overlay 평균 pixel match: 88.7247%
- overlay 평균 ink match: 12.12421%

대표 산출물:

| 페이지 | compare | overlay | review |
|--------|---------|---------|--------|
| p6 | `/Users/tsjang/rhwp/output/pr2052_visual/pr2052-issue2007-42065/compare/compare_006.png` | `/Users/tsjang/rhwp/output/pr2052_visual/pr2052-issue2007-42065/overlay/overlay_006.png` | `/Users/tsjang/rhwp/output/pr2052_visual/pr2052-issue2007-42065/review/review_006.png` |
| p15 | `/Users/tsjang/rhwp/output/pr2052_visual/pr2052-issue2007-42065/compare/compare_015.png` | `/Users/tsjang/rhwp/output/pr2052_visual/pr2052-issue2007-42065/overlay/overlay_015.png` | `/Users/tsjang/rhwp/output/pr2052_visual/pr2052-issue2007-42065/review/review_015.png` |

보존 asset:

- `mydocs/pr/assets/pr_2052_issue2007_review_contact_sheet.png`
- `mydocs/pr/assets/pr_2052_issue2007_review_p006.png`
- `mydocs/pr/assets/pr_2052_issue2007_review_p015.png`

옵션 1 처리:

- 리뷰 문서는 `mydocs/pr/archives/pr_2052_review.md`, `mydocs/pr/archives/pr_2052_review_impl.md` 로 archive 경로에 보존한다.
- MCP 기준 PDF `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf` 를 PR head 에 포함한다.
- 대표 visual asset 3건을 `mydocs/pr/assets/` 아래 PR 번호 포함 파일명으로 보존한다.
- 오늘할일 `mydocs/orders/20260708.md` 에 #2052 옵션 1 처리 항목을 추가한다.

사람 판정:

- PR의 핵심 주장인 "42065가 수정 전 6쪽 수준으로 붕괴하지 않고 15쪽으로 분할된다"는 사실과 일치한다.
- MCP 기준 HWP 2020 PDF는 17쪽이므로 한컴과 페이지네이션이 완전히 일치하지는 않는다.
- visible frame 밖으로 실제 픽셀이 새는 심각 후보는 없지만, 6쪽 이후 line/column/tail drift와 render tree tail overflow 후보가 남아 있다.
- 따라서 이 PR은 #2007의 완전 해결이 아니라 under-pagination 크램 붕괴를 줄이는 부분 개선으로 보는 것이 맞다.

## 리뷰 포인트

1. `tests/issue_2007_nested_cell_pagination.rs:12` 설명이 현재 구현과 다르다.
   - 테스트 주석은 `>1000px` 게이트를 말하지만 실제 구현은 `src/renderer/layout/table_layout.rs:5003`-`5010`의 `page_avail * 2.0` 게이트다.
   - PR 본문도 아직 `total_frag_h > 1000px`라고 적고, 이후 코멘트에서만 `2 x page height`로 정정한다.
   - 이는 구현 결함이 아니라 설명 stale 이므로 일반 코멘트로 남기고 merge blocker 로 보지 않는다.

2. `tests/issue_2007_nested_cell_pagination.rs:34`-`36`의 assertion이 하한(`pages >= 12`)만 검증한다.
   - 현재 PR의 개선값은 15쪽이고 MCP 기준 PDF는 17쪽이다.
   - 하한만 있으면 향후 과분할로 30쪽, 100쪽이 되어도 테스트가 통과한다.
   - 최소한 `12..=17` 범위 또는 현 구현 기준 `15`쪽 고정값을 함께 검증하면 false confidence를 줄일 수 있다.
   - 다만 이번 PR은 under-pagination 붕괴를 막는 목적이므로 이 보강도 non-blocking comment 로 처리한다.

3. visual sweep에서 `render_tree_frame_tail_overflow_pages=6-15`가 남는다.
   - 실제 visible frame overflow는 없으므로 즉시 merge blocker로 보지는 않는다.
   - 다만 render tree를 소비하는 기능이 있으면 후속 리스크가 될 수 있어, PR 코멘트나 후속 이슈에 잔여로 남기는 것이 좋다.

## 결론

PR #2052는 #2007 계열의 "거대 1x1 중첩셀 콘텐츠가 한 페이지에 크램되는 under-pagination"을 실제로 줄인다.
MCP 기준 PDF 없이도 검증 PDF를 생성해 확인한 결과, 기준은 17쪽이고 rhwp는 15쪽이다. 즉 개선 주장은 맞지만,
한컴 2020 기준과 완전 정합하지는 않는다.

메인테이너 판단으로는 부분 개선 PR로 merge 가능한 상태다. 테스트/본문의 stale gate 설명과 페이지 수 assertion
상한 보강은 일반 코멘트로 남기되, 이 사유로 merge 를 보류하지 않는다. #2007은 이 PR로 크램 붕괴 개선이
확인되었고, 남은 한컴 17쪽 대비 rhwp 15쪽 pagination fidelity 는 별도 잔여 축으로 유지한다.
