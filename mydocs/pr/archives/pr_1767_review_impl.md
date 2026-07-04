# PR #1767 리뷰 실행 기록

## Stage 1. 사전 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1767
- 작성자: @kkyu8925, FIRST_TIME_CONTRIBUTOR
- reviewer assign: `jangster77`
- 관련 이슈: #1663
- 문서 작성 시점 상태: `MERGEABLE`, `BEHIND`

## Stage 2. 변경 내용 검토

완료.

- `typeset.rs`의 후속 co-anchored RowBreak 표 orphan control 로직 확인.
- `typeset.rs`의 표 뒤 trailing empty paragraph 흡수 로직 확인.
- 신규 fixture `samples/issue1663_coanchored_float_orphan.hwpx`와 `tests/issue_1663.rs`가 PR 목적을 직접 검증하는지 확인.
- first-time contributor PR 이므로 후속 코멘트는 환영/감사 표현을 포함한다.

## Stage 3. 로컬 적용 및 검증

완료.

- 최신 `upstream/devel` 기준 `local/pr1767-review` 생성.
- PR 실제 커밋 `3dd698ce73ff2fc12079ddc7b57c41bf9cddf3b0` cherry-pick.
- 충돌 없음.

실행한 검증:

```bash
git diff --check upstream/devel...HEAD
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo test --test issue_1663 --test issue_1686
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
```

결과:

- `issue_1663`: 2 passed
- `issue_1686`: 4 passed
- clippy: pass

## Stage 4. 시각 검증

완료.

기준 PDF:

- `pdf/issue1663_coanchored_float_orphan-2024.pdf`

실행:

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target pr1767-issue1663 \
  samples/issue1663_coanchored_float_orphan.hwpx \
  pdf/issue1663_coanchored_float_orphan-2024.pdf \
  --out output/pr1767-visual-review
```

결과:

- SVG/PDF pages: 2/2
- flagged: 0/2
- page 1 review: `output/pr1767-visual-review/pr1767-issue1663/review/review_001.png`
- page 2 review: `output/pr1767-visual-review/pr1767-issue1663/review/review_002.png`
- page 1 asset: `mydocs/pr/assets/pr_1767_issue1663_visual_review_p1.png`
- page 2 asset: `mydocs/pr/assets/pr_1767_issue1663_visual_review_p2.png`

사람 판정:

- page 1: 선행 표 A만 남고 후속 표 B의 머리 row orphan 이 없다.
- page 2: 후속 표 B가 통째로 배치되고 추가 blank page 가 없다.

## Stage 5. 결론

완료.

- PR #1767은 #1663의 좁은 point-fix 로 merge 후보.
- 최신 PR head 기준 required checks 통과 후 merge 가능.
- GitHub 코멘트는 first-time contributor 톤으로 작성한다.
- 실제 코멘트에는 다음 PR부터 한컴 2020/2024 등 기준 프로그램에서 저장한 PDF를 함께 업로드해 달라는 요청을 포함한다.
- visual asset 은 merge 후 `devel` 기준 `mydocs/pr/assets/pr_1767_issue1663_visual_review_p1.png`, `mydocs/pr/assets/pr_1767_issue1663_visual_review_p2.png` 링크로 안내한다.
