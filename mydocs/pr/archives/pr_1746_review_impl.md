# PR #1746 처리 계획

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1746
- 작성자: `planet6897`
- 원 head: `pr/devel-1745`
- 적용 커밋: `4cbf94df7958556b9163303643d2cb7a569157e5`
- 누적 통합 브랜치 로컬 커밋: `018bbed6f`

## Stage 1. 체리픽 통합

- 순서: #1746 -> #1751 -> #1752 -> #1754
- PR 내부 merge commit `881c5ad48b1c0c59cee81e1ce111b64c4e11d24f`는 제외했다.
- `upstream/devel` 기준 충돌 없이 적용됐다.

## Stage 2. 검증

- 로컬 검증: `cargo fmt --check`, `git diff --check`, targeted test, `cargo test --profile release-test --tests`, `cargo clippy --all-targets -- -D warnings` 통과.
- 시각 검증: `samples/task1745/table_text_anchor_wrap.hwp` 기준 PDF 대비 visual sweep `flagged=0/3`.
- PR 코멘트 대표 PNG: `mydocs/pr/assets/pr_1746_visual_review_p2.png`.

## Stage 3. 후속 처리

- 통합 PR CI 통과 후 merge한다.
- 원 PR #1746에는 대표 PNG 1장과 함께 검증 결과 및 기준 PDF 동반 요청을 코멘트한다.
- 통합 PR merge 후 원 PR #1746은 superseded 처리로 close한다.

완료:

- #1810 merge commit `716fbca92ef4d5c67194ec6575bcc06413beacf6`
- 원 PR #1746 코멘트/close 완료
- 이슈 #1745 close 완료
