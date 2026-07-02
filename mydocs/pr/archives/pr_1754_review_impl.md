# PR #1754 처리 계획

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1754
- 작성자: `planet6897`
- 원 head: `pr/devel-1753`
- 적용 커밋: `6e07bb98134a5d88959de22102bdbda143ad5c6e`
- 누적 통합 브랜치 로컬 커밋: `b76668bbf`

## Stage 1. 체리픽 통합

- 순서: #1746 -> #1751 -> #1752 -> #1754
- 선행 세 PR 적용 후 `upstream/devel` 기준 충돌 없이 적용됐다.

## Stage 2. 검증

- 로컬 검증: `cargo fmt --check`, `git diff --check`, targeted test, `cargo test --profile release-test --tests`, `cargo clippy --all-targets -- -D warnings` 통과.
- PR 핵심 조건: 지연 자리차지 RowBreak 표 앞의 후속 control-free 문단 pi=52/53이 현재 쪽 잔여 공간에 선행 배치된다.
- 시각 검증: HWP/HWPX 기준 PDF 대비 visual sweep 모두 `flagged=0/21`.
- PR 코멘트 대표 PNG: `mydocs/pr/assets/pr_1754_visual_review_p9.png`.

## Stage 3. 후속 처리

- 통합 PR CI 통과 후 #1746 -> #1751 -> #1752 이후 순서로 반영된 것으로 처리한다.
- 원 PR #1754에는 대표 PNG 1장과 함께 검증 결과 및 기준 PDF 동반 요청을 코멘트한다.
- 통합 PR merge 후 원 PR #1754는 superseded 처리로 close한다.

완료:

- #1810 merge commit `716fbca92ef4d5c67194ec6575bcc06413beacf6`
- 원 PR #1754 코멘트/close 완료
- 이슈 #1753 close 완료
