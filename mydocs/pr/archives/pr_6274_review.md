---
kind: pr-review
status: active
pr: 6274
issue: 6271
author: davindev
base: devel
reviewed_at: 2026-08-28 16:30 KST
---

# PR #6274 검토 기록

## 라우팅

- base route: maintainer 일반
- modifiers: 접수·리뷰 기록, 로컬 검증, 시각·fixture 증적
- loaded documents: `mydocs/manual/pr_review_workflow.md`,
  `mydocs/manual/pr_review/README.md`, `mydocs/manual/pr_review/maintainer_general.md`,
  `mydocs/manual/pr_review/intake_and_review.md`,
  `mydocs/manual/pr_review/local_validation.md`,
  `mydocs/manual/pr_review/visual_fixture_evidence.md`,
  `mydocs/manual/verification/visual_sweep_guide.md`
- current PR head: `92f1ace655c419e02ba052815bcb70157a2647d0`
- current upstream/devel: `11f5a80fd17d9895e4a2c9cab921a0d6ace07836`
- local review branch: `review/davindev-6274-trailing-20260828`
- pre-trailing current-base merge commit: `1d011557a94293452a369c94bab9c70a4b7f9fe5`
- rebased PR source commit: `c6acdd3497b01062d9d804b00569a89087fb2617`

## Metadata

| 항목 | 값 |
| --- | --- |
| PR | #6274 |
| 제목 | `fix(renderer): RowBreak 자리차지 표의 꼬리 줄 vpos snap 이 표 배치를 깨뜨리지 않는다 (#6271)` |
| 작성자 | `davindev` |
| base | `devel` |
| head repo | `kidsnote/rhwp` |
| head branch | `fix/6271-rowbreak-float-whole-fit` |
| head SHA | `92f1ace655c419e02ba052815bcb70157a2647d0` |
| draft | false |
| 규모 | +75 / -2, 4 files |
| GitHub mergeability | `MERGEABLE` / `CLEAN` (2026-08-28 16:30 KST 기준, merge 전 재확인 필요) |
| reviewer | REST API로 `jangster77` review request 등록 완료 |
| 작성자 맥락 | 저장소 PR 검색 기준 첫 PR로 보임 |

## 변경 범위

- `samples/issue-6271-rowbreak-float-tail-line.hwp`
  - #6271 합성 재현 fixture 추가. 원 실문서의 텍스트·이미지를 더미로 치환하고 배치 기하만 유지한 샘플.
- `src/renderer/typeset.rs`
  - RowBreak + TopAndBottom + Para 기준 table에서 line segment stored vpos로 snap할 때,
    그 snap이 선언 table 전체 높이의 현재 페이지 배치를 깨뜨리면 snap하지 않도록 제한.
- `src/renderer/layout/paragraph_layout.rs`
  - TAC picture가 이미 sibling TopAndBottom 예약 아래의 최종 line y에 놓인 경우 예약 높이를 다시 더하지 않도록 제한.
- `tests/cases/issue_6271_rowbreak_float_tail_snap.rs`
  - 1쪽 유지와 tail TAC picture의 본문 하단 내 위치를 고정하는 회귀 테스트 2건 추가.

## GitHub CI

PR head `92f1ace655c419e02ba052815bcb70157a2647d0` 기준 모든 required check가 완료됐다.

- CI: `Build & Test`, `Lint (fmt, clippy, WASM check)`, `Native Skia tests`,
  test archive A/B/C/D 및 각 shard 통과
- CodeQL: javascript-typescript, python, rust 통과
- Adapter inter-diff: 통과
- Proptest roundtrip: 통과
- Render Diff: `Render Diff preflight`, `Canvas visual diff` 통과

단, PR head는 현재 `upstream/devel`을 조상으로 포함하지 않는다. current-base 검증은 아래 로컬 merge
branch에서 별도로 수행했다.

## 로컬 검증

### Pre-trailing current-base merge

```text
git fetch upstream devel
git fetch upstream pull/6274/head:refs/remotes/upstream/pr6274-head
git merge-base --is-ancestor upstream/devel upstream/pr6274-head
=> pr-head-does-not-contain-upstream-devel

git merge-tree --write-tree upstream/devel upstream/pr6274-head
=> merge-tree-clean

git switch -c review/davindev-6274-20260828 upstream/devel
git merge --no-ff --no-edit upstream/pr6274-head
=> conflict 없이 merge
```

### Latest devel rebase

```text
git fetch upstream devel
git rebase upstream/devel
=> PR source commit은 제품 코드 충돌 없이 재적용
=> review 증적 commit 적용 중 mydocs/orders/20260828.md만 충돌
=> 최신 devel의 오늘할일 기록을 보존하고 #6274 항목을 다시 추가
```

### Format / manifest

```text
cargo fmt --all -- --check
=> PASS

node scripts/rust-unit-test-tiers.mjs --check
=> PASS: 4221 tests / 299 modules / ready 0 / support 87 / white-box 4130 / cfg support items 28

node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check
=> PASS: 1007 sources / 4483 static test attrs / 32 suites + 16 exceptions = 48/48 integration targets
```

### Focused regression

```text
node scripts/run-rust-test.mjs issue_6271_rowbreak_float_tail_snap -- \
  --cargo-profile release-test --target-dir target/pr-review
=> PASS: 2 tests run, 2 passed, 138 skipped
```

추가 주변 회귀 확인:

```text
issue_5700_tac_reset_tail_above_flow => PASS
issue_5734_cell_float_stack_stored_vpos => PASS
issue_5871_ws_host_float_double_charge => PASS
issue_6167_leading_space_tac_table_own_line => PASS
issue_5788_tac_table_anchor_line_spacing => PASS
issue_5807_coanchored_float_tac_order => PASS
```

전체 nextest는 GitHub Full CI가 같은 source head에서 이미 통과했고, 로컬에서는 current-base merge
충돌 없음과 핵심 renderer 주변 focused test로 중복 범위를 줄였다. merge 전에는 최신 PR head와 required
check를 다시 확인해야 한다.

## 시각·fixture 증적

### 원본 fixture

```text
samples/issue-6271-rowbreak-float-tail-line.hwp
sha256: 9d533fde6caa7ce388fd0a06893933116dea06e4ef755bb3906c0417b8902dfa
file: Hancom HWP file, version 5.0
```

`rhwp info --json` 결과:

```text
format: hwp5
version: 5.1.1.0
lastSavedWith.product: hancom-office-2022
lastSavedWith.version: 12.0.0.535
pageCount: 1
printMethod: 0
asset: mydocs/pr/assets/pr_6274_issue6271_info.json
```

`hancom-office-2022` 저장본이므로 PR review 기준 PDF는 engine `2020`, suffix `-2020.pdf`로 생성했다.

### MCP 기준 PDF

```text
engine: 2020
job_id: 7473b2ce-81ec-4476-b030-22958e18081e
status: succeeded -> download success
hancom_version: 12.0.0.4605
output: pdf/pr_6274/by_saved_version/pr6274_issue6271_rowbreak_float_tail_line-2020.pdf
sha256: 96a1686140fb6a19362687f19204f74f72aca229923c1e65fac99e69623c2448
pdfinfo: Pages 1, A4, PDF 1.6
assets:
- mydocs/pr/assets/pr_6274_issue6271_mcp2020_start.json
- mydocs/pr/assets/pr_6274_issue6271_mcp2020_status.json
- mydocs/pr/assets/pr_6274_issue6271_mcp2020_download.json
```

### Visual sweep

```text
venv/bin/python scripts/visual_sweep.py \
  --key pr6274_issue6271_rowbreak_float_tail_line \
  --hwp samples/issue-6271-rowbreak-float-tail-line.hwp \
  --pdf pdf/pr_6274/by_saved_version/pr6274_issue6271_rowbreak_float_tail_line-2020.pdf \
  --page 1 \
  --rhwp-bin target/pr-review/release-test/rhwp \
  --out output/visual_sweep_pr6274
```

단일 페이지 fallback으로 선택 page 1이 내부 page 번호 `6271` 산출물에 1:1 대응됐다.

```text
exported_svg_pages: 1
exported_pdf_pages: 1
completed_pages: [6271]
pixel_match_percent: 78.88796
visual_accuracy_proxy_percent: 23.29933
flags: frame_overflow_pixels, render_tree_frame_tail_overflow
```

보존 asset:

- `mydocs/pr/assets/pr_6274_issue6271_visual_sweep_summary.json`
- `mydocs/pr/assets/pr_6274_issue6271_visual_sweep_metrics.json`
- `mydocs/pr/assets/pr_6274_issue6271_overlay_metrics.json`
- `mydocs/pr/assets/pr_6274_issue6271_compare_p6271.png`
- `mydocs/pr/assets/pr_6274_issue6271_overlay_p6271.png`
- `mydocs/pr/assets/pr_6274_issue6271_review_p6271.png`

사람 확인 결과:

- review PNG에서 rhwp와 Hancom PDF 모두 1쪽이다.
- 표 본체가 2쪽으로 밀리지 않고 첫 페이지 본문에 유지된다.
- 표 아래 꼬리 텍스트와 그림이 하단에 표시되어 원 결함의 "그림이 쪽 밖으로 사라짐" 축은 해소됐다.
- 자동 flag의 `frame_overflow_pixels`와 `render_tree_frame_tail_overflow`는 양쪽 모두 하단 bleed가 있는
  잔여 후보다. #6271의 blocker였던 2쪽 이월·그림 소실과 동일 결함으로 보이지 않으므로 merge 차단 사유로
  보지 않는다. 다만 하단 frame boundary strictness를 별도 개선축으로 삼으려면 follow-up issue로 분리할 수 있다.

## Review 판단

### Findings

차단 결함은 발견하지 못했다.

### Risk

- `paragraph_layout.rs`의 sibling 예약 높이 suppression은 heuristic이다. 기존 double-charge 계열 주변
  fixture 6건을 추가 실행해 통과했지만, future fixture에서 다른 line y 해석이 발견되면 조건을 더 좁혀야 한다.
- visual sweep 자동 지표는 23.29933%로 낮고 하단 overflow 후보를 보고한다. 이는 전체 fidelity 통과가 아니라
  #6271의 핵심 현상(1쪽 유지, tail 그림 표시) 확인 증적으로만 사용한다.

### 권고

수용 권고. merge 전 최신 head `92f1ace655c419e02ba052815bcb70157a2647d0` 유지 여부, GitHub required check,
mergeability를 다시 확인한 뒤 admin merge 가능하다.

첫 PR로 보이므로 merge comment에는 다음을 포함한다.

- 첫 기여 감사
- #6271 재현 fixture를 더미화해 배치 기하를 보존한 점
- 회귀 테스트가 핵심 불변식(1쪽 유지, tail TAC picture 본문 내)을 잘 고정한 점
- maintainer visual sweep에서 자동 하단 overflow 후보는 남았지만 원 blocker는 해소된 것으로 본다는 점
