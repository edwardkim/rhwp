# PR #1764 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1764
- 작성자: @planet6897
- 관련 이슈: #1763
- #1756 위 스택 PR이었으나, #1756 merge 후 clean head로 메인터너 보정 완료
- 문서 작성 시점 참고값: `mergeable=MERGEABLE`, `mergeStateStatus=BEHIND`
- reviewer assign: `@jangster77`

## Stage 2. 체리픽 누적 검토

완료.

```bash
git cherry-pick 4e47d0fb1283cefcadd87bba08da0348c7b04230
```

충돌 없음.

## Stage 3. 검증

완료.

- `cargo fmt --check`
- `git diff --check upstream/devel..HEAD`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1763_cell_trailing_ls_expand -- --nocapture`
- GitHub Actions 최신 head success 확인

## Stage 4. 시각 검증

완료.

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target pr1764-cell-trailing-hwp samples/task1763/cell_trailing_ls_expand.hwp samples/task1763/cell_trailing_ls_expand-2024.pdf \
  --file-target pr1764-cell-trailing-hwpx samples/task1763/cell_trailing_ls_expand.hwpx samples/task1763/cell_trailing_ls_expand-2024.pdf \
  --page 1 \
  --out output/pr1764-visual-review
```

- HWP/HWPX 모두 페이지 수 1 / 1
- HWP/HWPX 모두 자동 후보 `0/1`
- review PNG:
  - `output/pr1764-visual-review/pr1764-cell-trailing-hwp/review/review_001.png`
  - `output/pr1764-visual-review/pr1764-cell-trailing-hwpx/review/review_001.png`
- visual_accuracy_proxy_percent: 3.75233

## Stage 5. 샘플 처리

- 기존 PR diff의 HWP 재현 샘플에 더해, 메인터너/콜라보레이터가 추가한 기준 PDF/HWPX 샘플을 검증 근거로 포함
- `samples/task1763/cell_trailing_ls_expand-2024.pdf`
- `samples/task1763/cell_trailing_ls_expand.hwpx`
- 옵션 1 처리에 따라 review 문서/archive, visual asset, 기준 샘플을 같은 PR 에 포함

## Stage 6. 다음 작업

- #1761 → #1762 → #1764 순서로 merge 판단
- merge 후 #1763 자동 close 여부 확인, 실패 시 수동 close
