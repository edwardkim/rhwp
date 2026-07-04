# PR #1752 처리 계획

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1752
- 작성자: `planet6897`
- 원 head: `pr/devel-1749`
- 적용 커밋: `699208e240d3943ca8f891294b3f69e3921e474c`
- 누적 통합 브랜치 로컬 커밋: `5a349f5b0`

## Stage 1. 체리픽 통합

- 순서: #1746 -> #1751 -> #1752 -> #1754
- #1746, #1751 적용 후 `upstream/devel` 기준 충돌 없이 적용됐다.

## Stage 2. 검증

- 로컬 검증: `cargo fmt --check`, `git diff --check`, targeted tests, `cargo test --profile release-test --tests`, `cargo clippy --all-targets -- -D warnings` 통과.
- PR 핵심 조건: `saved_bounds_cumulative_vpos`는 pi18이 2쪽 시작으로 이동하고, `saved_bounds_cumulative_page_break`는 pi26이 2쪽 마지막 문단으로 유지된다.
- 시각 검증: HWP 기준은 두 샘플 모두 자동 후보 0건이다.
- 보완 후보: HWPX `saved_bounds_cumulative_page_break` p5에서 tail/line drift 후보가 남았다.
- PR 코멘트 대표 PNG: `mydocs/pr/assets/pr_1752_visual_review_p5_followup.png`.

## Stage 3. 후속 처리

- 통합 PR CI 통과 후 #1746, #1751 다음 순서로 반영된 것으로 처리한다.
- 원 PR #1752에는 보완 후보 PNG 1장과 함께 PR 핵심 조건은 통과했으나 p5 drift 후보는 후속 이슈로 분리해 추적한다고 코멘트한다.
- 통합 PR merge 후 원 PR #1752는 superseded 처리로 close한다.

완료:

- #1810 merge commit `716fbca92ef4d5c67194ec6575bcc06413beacf6`
- 원 PR #1752 코멘트/close 완료
- 이슈 #1749 close 완료
- p5 tail/line drift 후속 이슈 #1811 생성
