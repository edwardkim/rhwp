# PR #1844 리뷰 구현 메모

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1844
- 작성자: @planet6897
- 제목: `Task #1831: 같은 문단 float 스택 이월 규칙 — 다쪽 표 continuation 상수 오프셋 해소`
- base / head: `devel` / `pr/devel-1831`
- 검토 head: `ff0c7ac2c3629ec780179f5e682f685bc0c085d2`
- 최종 merge head: `93312688500e6a026758947e57e203fbe100e808`
- merge commit: `a987ae10e85128f75997d7b826b61a668b0eef7f`

## Stage 1. 메타 확인

완료.

- reviewer assign 완료.
- PR은 Draft가 아니다.
- `maintainerCanModify=true`.
- mergeable 상태는 `MERGEABLE`.
- 작성 시점 댓글은 없음.
- #1831은 본 PR로 해결 대상이고, #1842는 분리된 잔여 이슈로 확인했다.

## Stage 2. 변경 내용 검토

완료.

- `typeset_block_table` 분할 진입부에서 같은 문단 선행 Table/PartialTable 존재 여부를 `para_index`로 판정한다.
- row-break 표 전체 행 높이와 caption base overhead가 현재 단 잔여 공간에 들어가지 않으면 `prefill_before_deferred_table`
  후 다음 단/쪽으로 넘긴다.
- 2px 허용은 첫 fragment, start_cut 없음, 전체 행 높이가 단 상단 기준을 2px 이하로만 넘는 경우로 제한되어 있다.
- `dump`의 `hdr=` 진단 추가는 기능 동작에는 영향이 없고 리뷰 재현성을 높이는 변경이다.
- `tools/patch_cell_flags.py`는 인과 실험용 도구이며 `py_compile` 기준 문법 검증을 통과했다.

## Stage 3. 시각 검증

완료.

- 기준 PDF: `pdf/float-stack-defer-2022.pdf`
  - Creator: Hwp 2022 12.0.0.4547
  - Producer: Hancom PDF 1.3.0.550
  - Pages: 2
- 샘플: `samples/float-stack-defer.hwp`
  - `rhwp info`: 2 pages
  - `dump-pages`: p1은 제목+표1, p2는 표2 전체
- visual sweep:
  - command: `python3 scripts/task1274_visual_sweep.py --key pr1844-float-stack-defer --hwp samples/float-stack-defer.hwp --pdf pdf/float-stack-defer-2022.pdf --pages 1-2 --out output/pr1844_visual --rhwp-bin target/debug/rhwp`
  - flagged: 0/2
  - review p1: `output/pr1844_visual/pr1844-float-stack-defer/review/review_001.png`
  - review p2: `output/pr1844_visual/pr1844-float-stack-defer/review/review_002.png`
  - asset p1: `mydocs/pr/assets/pr_1844_float_stack_defer_review_p1.png`
  - asset p2: `mydocs/pr/assets/pr_1844_float_stack_defer_review_p2.png`
- baseline compare:
  - p1 median: -1.00pt
  - p2 median: -2.27pt

## Stage 4. 로컬 검증

완료.

- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 항목 삭제.
- `env CARGO_INCREMENTAL=0 cargo build` 통과.
- `git diff --check upstream/devel...HEAD` 통과.
- `cargo fmt --check` 통과.
- `python3 -m py_compile tools/patch_cell_flags.py tools/compare_line_baselines.py` 통과.
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과.
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과.

## Stage 5. merge 및 후속 처리

완료.

- 최초 merge 시도는 PR head 가 최신 `devel`보다 뒤처져 branch protection 에 막혔다.
- `gh pr update-branch 1844 --repo edwardkim/rhwp` 로 최신 `devel`을 반영했다.
- 최신 head `93312688500e6a026758947e57e203fbe100e808` 기준 GitHub Actions 재실행 결과:
  - Analyze rust/python/javascript-typescript 통과.
  - Build & Test 통과.
  - Canvas visual diff 통과.
  - CodeQL 통과.
- PR #1844 merge 완료.
- 옵션 2 방식으로 review 문서와 visual asset 은 별도 docs-only 후속 PR 로 반영한다.
- #1831 close 여부와 #1842 open 유지 여부를 후속 처리에서 확인한다.
