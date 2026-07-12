# PR #1968 리뷰 구현 기록

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1968
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1921
- 작성자: planet6897
- 문서 작성 시점 head SHA: `d0ddd4faffb500ddbe1e56157557c12e455e9c2c`
- 실제 변경 commit: `750477549840af6d75c6f23d970739fe990594c4`
- 체리픽 commit: `fd176b441`

## Stage 1. 메타 확인 및 체리픽

완료.

- PR base가 `devel`이고 draft가 아님을 확인했다.
- reviewer `jangster77`를 assign했다.
- GitHub Actions CI/CodeQL/Render Diff가 통과한 상태를 확인했다.
- #1968 실제 변경 commit을 먼저 체리픽했고, 이어서 #1969를 체리픽했다.
- 체리픽 충돌은 없었다.

## Stage 2. 변경 내용 검토

완료.

- 표 각주가 있는 문단에서 다음 문단의 저장 vpos가 새 쪽 시작을 가리킬 때만 각주 안전마진을 완화한다.
- 보정 근거는 현재/다음 문단의 non-synthetic `LineSeg.vertical_pos`와 각주 높이 계산이다.
- 샘플명이나 PR 번호로 동작을 바꾸는 분기는 없다.
- `vertical_pos <= 500` 조건은 상단 재시작 판정을 위한 휴리스틱이므로 리뷰 문서에 회귀 관찰 포인트로 남겼다.

## Stage 3. 시각 검증

완료.

- #1968 본문 기준 `75828` 원본/PDF는 현재 체크아웃에 없어 직접 visual sweep하지 못했다.
- 통합 체리픽 상태에서 #1969 회귀 guard인 `36389312` 1쪽 visual sweep을 수행했다.
- 결과는 `flagged=0/1`, 페이지 수 1/1, 대표 asset `mydocs/pr/assets/pr_1968_1969_36389312_review_177.png`다.

## Stage 4. 로컬 검증

완료.

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo build`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1658_page_bottom_fixed_exclusion`: 3 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1611_footer_page_bottom_pagination`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1624_footer_overpush_pagination`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1858`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1858_bottom_anchor_flush`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## Stage 5. 결론

#1969와 함께 통합 체리픽 PR로 처리한다. 원 PR은 통합 PR merge 이후 PR review workflow에 따라 반영 사실을 코멘트하고 close한다.

