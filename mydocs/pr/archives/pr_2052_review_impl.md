# PR #2052 리뷰 구현 메모

## Stage 1. 메타 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/2052
- 작성자: @planet6897
- base/head: `devel` / `fix/2007-nested-cell-pagination`
- 검토 기준 head: `4af4e7694757df5f8259800c70bae24e3bde0944`
- reviewer assign: @jangster77 지정 완료.
- `maintainerCanModify=true`.
- merge state: `CLEAN`.
- 원격 CI: WASM Build skipped 외 전부 통과.

## Stage 2. 기준 PDF 확보

완료.

PR/이슈에 기준 PDF가 첨부되어 있지 않아 HWP 2020 MCP로 생성했다.

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 출력: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- HWP SHA-256: `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626`
- PDF SHA-256: `1f9d2f5705a64899c2b081832d2e6548dfe7bc3b9d1fb1b92f41221d39c8b3e7`
- MCP job id: `f69ed018-28b7-4306-86af-190afe8e7fba`
- run_status/validation: `0` / `ok`
- `pdfinfo`: 17 pages, A4.

## Stage 3. 변경 내용 검토

완료.

- PR은 1x1 RowBreak 표 안의 중첩 1x1 표를 기존 atomic unit 하나로 두지 않고,
  다중 페이지급 콘텐츠일 때 fragment 단위로 분할한다.
- 코드상 실제 게이트는 `page_avail * 2.0`이며, PR 본문과 테스트 주석 일부에 남은 `>1000px`
  설명은 최신 구현과 다르다.
- #1891 회귀 대응 주석은 코드에 반영되어 있다.

## Stage 4. 로컬 검증

완료.

- `git diff --check` 통과.
- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 항목 삭제.
- `cargo fmt --check` 통과.
- targeted tests 통과:

```bash
CARGO_INCREMENTAL=0 cargo test --profile release-test \
  --test issue_2007_nested_cell_pagination \
  --test issue_1891 \
  --test issue_1488_rowbreak_empty_overlay_pages \
  --test issue_1749_saved_bounds_page_break \
  --test issue_2015_saved_bounds_rowbreak
```

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib` 통과: 2150 passed, 7 ignored.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과.
- `cargo build --bin rhwp` 통과.
- `target/debug/rhwp info samples/basic/issue2007_nested_cell_pagination_42065.hwp`: 15쪽.

## Stage 5. 시각 검증

완료.

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr2052-issue2007-42065 \
  --hwp samples/basic/issue2007_nested_cell_pagination_42065.hwp \
  --pdf pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf \
  --out output/pr2052_visual
```

결과:

- SVG pages: 15
- PDF pages: 17
- flagged: 10/15
- visible frame overflow: 없음
- line/column/tail drift: 6쪽 이후
- render tree tail overflow 후보: 6-15쪽
- 평균 pixel match: 88.7247%
- 평균 ink match: 12.12421%

대표 파일:

- p6 compare: `output/pr2052_visual/pr2052-issue2007-42065/compare/compare_006.png`
- p6 overlay: `output/pr2052_visual/pr2052-issue2007-42065/overlay/overlay_006.png`
- p6 review: `output/pr2052_visual/pr2052-issue2007-42065/review/review_006.png`
- p15 compare: `output/pr2052_visual/pr2052-issue2007-42065/compare/compare_015.png`
- p15 overlay: `output/pr2052_visual/pr2052-issue2007-42065/overlay/overlay_015.png`
- p15 review: `output/pr2052_visual/pr2052-issue2007-42065/review/review_015.png`

asset 보존:

- `mydocs/pr/assets/pr_2052_issue2007_review_contact_sheet.png`
- `mydocs/pr/assets/pr_2052_issue2007_review_p006.png`
- `mydocs/pr/assets/pr_2052_issue2007_review_p015.png`

옵션 1 보존 대상:

- `mydocs/pr/archives/pr_2052_review.md`
- `mydocs/pr/archives/pr_2052_review_impl.md`
- `mydocs/pr/assets/pr_2052_issue2007_review_contact_sheet.png`
- `mydocs/pr/assets/pr_2052_issue2007_review_p006.png`
- `mydocs/pr/assets/pr_2052_issue2007_review_p015.png`
- `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- `mydocs/orders/20260708.md`

판정:

- 수정 전 6쪽 붕괴는 재현되지 않고 15쪽으로 분할된다.
- MCP 기준 HWP 2020 PDF는 17쪽이므로 완전 정합은 아니다.
- visible frame overflow는 없지만, 6쪽 이후 drift와 render tree tail overflow 후보가 남는다.

## Stage 6. 권장 리뷰 처리

대기.

권장 코멘트 방향:

- 핵심 개선은 확인했다.
- MCP 기준 PDF 생성 결과 17쪽, rhwp 15쪽으로 PR의 "6p -> 15p" 주장은 맞다.
- PR 본문/테스트 주석의 stale `>1000px` 설명은 `> 2 x page height` 기준으로 정정하면 좋다는
  일반 코멘트로 남긴다.
- `pages >= 12`만 보는 테스트 assertion은 상한 또는 현 구현 기대값을 추가하면 더 좋다는
  일반 코멘트로 남긴다.
- visual sweep상 render tree tail overflow 후보가 남는 점은 잔여로 기록한다.
- 위 항목들은 non-blocking 이며, 이 사유로 merge 를 보류하지 않는다.
- #2007은 이 PR로 크램 붕괴 개선을 인정하고, 남은 pagination fidelity 축은 후속 잔여로 유지한다.

GitHub 코멘트/리뷰 제출 및 remote push는 사용자 승인 후 진행한다.
