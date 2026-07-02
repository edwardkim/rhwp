# PR #1751 처리 계획

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1751
- 작성자: `planet6897`
- 원 head: `pr/devel-1750`
- 적용 커밋: `a7721d7a00bc77ae12cbc2c7a0f9b6b910de1c7a`
- 누적 통합 브랜치 로컬 커밋: `bdc07c181`

## Stage 1. 체리픽 통합

- 순서: #1746 -> #1751 -> #1752 -> #1754
- PR 내부 merge commit `b1a3d7dfea2a6a42723e944317edf3b205b32a06`는 제외했다.
- #1746 적용 후 `upstream/devel` 기준 충돌 없이 적용됐다.

## Stage 2. 검증

- 로컬 검증: `cargo fmt --check`, `git diff --check`, targeted test, `cargo test --profile release-test --tests`, `cargo clippy --all-targets -- -D warnings` 통과.
- PR 핵심 조건: pi=22가 1쪽에 분할 잔류하지 않고 2쪽 시작으로 배치된다.
- 시각 검증: HWP/HWPX 기준 PDF 대비 visual sweep 모두 `flagged=0/5`.
- PR 코멘트 대표 PNG: `mydocs/pr/assets/pr_1751_visual_review_p2.png`.

## Stage 3. 후속 처리

- 통합 PR CI 통과 후 #1746 다음 순서로 반영된 것으로 처리한다.
- 원 PR #1751에는 대표 PNG 1장과 함께 검증 결과 및 기준 PDF 동반 요청을 코멘트한다.
- 통합 PR merge 후 원 PR #1751은 superseded 처리로 close한다.

완료:

- #1810 merge commit `716fbca92ef4d5c67194ec6575bcc06413beacf6`
- 원 PR #1751 코멘트/close 완료
- 이슈 #1750 close 완료
