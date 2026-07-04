# PR #1791 리뷰 — Task #1789 exclusion probe line_spacing 제외

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1791 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1789` |
| 관련 이슈 | #1789 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | planet6897 open PR 묶음 누적 cherry-pick |

## 변경 범위

- `src/renderer/layout.rs`
- `tests/issue_1789_exclusion_probe_line_spacing.rs`
- `samples/task1789/exclusion_probe_line_spacing.hwpx`
- 관련 계획/보고/샘플 문서

자리차지 exclusion 겹침 probe에서 paragraph line spacing을 중복 반영하지 않도록 제외한다.

## 검토 결과

PR 목적은 line spacing이 probe의 ink 영역을 불필요하게 키워 본문 줄을 밀어내는 문제를 막는 것이다. 테스트가
line spacing이 제외된 probe 높이를 확인하고, 기준 PDF visual sweep 2쪽이 자동 후보 없이 통과했다.

## 시각 검증

| target | SVG/PDF pages | flagged | 대표 asset |
|---|---:|---:|---|
| `pr1791-1805-exclusion` | 2/2 | 0 | `mydocs/pr/assets/pr1791_pr1805_exclusion_review_p001.png` |

원본 산출물: `output/pr1818-planet6897-visual/pr1791-1805-exclusion/review/review_001.png`

## 검증

- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1789_exclusion_probe_line_spacing`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

PR 내용 기준으로 exclusion probe line spacing 제외가 테스트와 visual sweep으로 확인됐다. merge 후보로 판단한다.
