# PR #1969 리뷰 구현 기록

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1969
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1920
- 작성자: planet6897
- 문서 작성 시점 head SHA: `b0a0477a58e8bf14a4d2cb42bff25be0daec7cc7`
- 실제 변경 commit: `b0a0477a58e8bf14a4d2cb42bff25be0daec7cc7`
- 체리픽 commit: `2cfac2f4e`

## Stage 1. 메타 확인 및 체리픽

완료.

- PR base가 `devel`이고 draft가 아님을 확인했다.
- reviewer `jangster77`를 assign했다.
- GitHub Actions CI/CodeQL/Render Diff가 통과한 상태를 확인했다.
- #1968을 먼저 체리픽한 뒤 #1969 실제 변경 commit을 체리픽했다.
- 체리픽 충돌은 없었다.

## Stage 2. 변경 내용 검토

완료.

- `available`이 배타 영역을 이미 차감했다는 가정을 제거하고 `available - prospective_excl`로 같은 쪽 편입 여부를 판정한다.
- 하단 고정 틀이 본문 flow에서 롤백한 소비 높이를 활성 vpos base에 반영해 후속 문단 스냅의 이중 계산을 줄인다.
- 보정 근거는 bottom-fixed control의 배치 속성, block 높이, 저장 vpos base, flow 좌표다.
- 샘플명/페이지 번호/issue 번호로 결과를 맞추는 분기는 없다.

## Stage 3. 시각 검증

완료.

- PR 본문 기준 `36373162` 원본/PDF는 현재 체크아웃에 없어 직접 visual sweep하지 못했다.
- PR 본문에서 회귀 guard로 언급한 `36389312` 샘플과 Hancom 2024 기준 PDF로 1쪽 visual sweep을 수행했다.
- 결과는 `flagged=0/1`, SVG/PDF 1/1쪽이다.
- 대표 asset은 `mydocs/pr/assets/pr_1968_1969_36389312_review_177.png`에 저장했다.

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

#1968 뒤에 적용하는 통합 체리픽 PR로 처리한다. 원 PR은 통합 PR merge 이후 PR review workflow에 따라 반영 사실을 코멘트하고 close한다.

