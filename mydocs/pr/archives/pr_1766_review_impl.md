# PR #1766 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1766
- 작성자: @planet6897
- 관련 이슈: #1765, refs #1759
- reviewer assign: `@jangster77`
- 문서 작성 시점 참고값: `mergeable=MERGEABLE`, `mergeStateStatus=BLOCKED`
- PR head: `b4218b6a5ff6a1d8088a55b655a58c659cd042ba`

## Stage 2. 로컬 적용

완료.

PR head에는 `devel` merge commit `b4218b6a5ff6a1d8088a55b655a58c659cd042ba`가 포함되어 있어,
검토 브랜치에서는 실제 조사 커밋만 적용했다.

```bash
git checkout -B local/pr1766-review upstream/devel
git cherry-pick a200cb0b833a82d32e9a921c6566a24b80178f2b
```

충돌 없음.

## Stage 3. 기본 검증

완료.

- `git diff --check upstream/devel..HEAD`
- `cargo fmt --check`

소스 변경 없음. 전체 cargo test/clippy는 아직 수행하지 않음.

## Stage 4. 리뷰 판단

초기 검토에서 문서 상충을 확인했고, 메인터너 보정 후 merge 후보로 변경.

- 최종 보고서/Stage 2는 가설 기각으로 정리되어 있음.
- PR merge 목적은 병합 셀 경로 가설 기각이 맞음을 샘플/기준 PDF/visual sweep 기록으로 보존하는 것.
- 수행계획서/구현계획서/샘플 README가 병합 셀 trailing ls 가설을 사실처럼 적던 문제를 보정.
- 기준 PDF/HWPX 샘플은 추가되었고, visual sweep으로 HWP/HWPX p2 자동 후보 없음 확인.

## Stage 5. 다음 작업

- review 문서 archive 이동
- visual asset 추가
- 추가된 기준 PDF/HWPX 샘플 포함
- PR head 에 remote push
- 최신 CI 통과 후 merge 판단

## Stage 6. 시각 검증

완료.

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target pr1766-merged-cell-hwp samples/task1765/merged_cell_trailing_ls.hwp samples/task1765/merged_cell_trailing_ls-2024.pdf \
  --file-target pr1766-merged-cell-hwpx samples/task1765/merged_cell_trailing_ls.hwpx samples/task1765/merged_cell_trailing_ls-2024.pdf \
  --page 2 \
  --out output/pr1766-visual-review
```

- HWP/HWPX 모두 SVG/PDF 페이지 수 4 / 4
- 선택 페이지: 2
- HWP/HWPX 모두 자동 후보 `0/1`
- review PNG:
  - `output/pr1766-visual-review/pr1766-merged-cell-hwp/review/review_002.png`
  - `output/pr1766-visual-review/pr1766-merged-cell-hwpx/review/review_002.png`
- asset:
  - `mydocs/pr/assets/pr_1766_merged_cell_hwp_visual_review_p2.png`
  - `mydocs/pr/assets/pr_1766_merged_cell_hwpx_visual_review_p2.png`
- visual_accuracy_proxy_percent: 15.17532
