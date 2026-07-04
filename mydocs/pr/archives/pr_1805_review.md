# PR #1805 리뷰 — Task #1794 exclusion probe is_hwpx_source 게이트 제거

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1805 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1794` |
| 관련 이슈 | #1794 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | #1791 위에 누적 cherry-pick |
| conflict 해소 | contributor branch를 최신 `devel` 위에 #1791/#1794 순서로 재작성해 head `0bd167f44`로 갱신, GitHub `MERGEABLE` 확인 |

## 변경 범위

- `src/renderer/layout.rs`
- 관련 계획/보고/오늘할일 문서

자리차지 표 잉크-겹침 probe의 `is_hwpx_source` 게이트를 제거해 HWP/HWPX 입력 모두 같은 exclusion probe를
사용하도록 한다.

## 검토 결과

#1791의 line spacing 제외 계약 위에서 입력 형식 게이트만 제거한다. 따라서 PR 내용 기준 검증은 #1791 샘플과
같은 visual sweep 및 전체 회귀 테스트로 확인했다. 기준 PDF 2쪽 모두 자동 후보가 없었다.

GitHub conflict는 스택 PR의 오늘할일 문서 add/add 충돌 때문이었다. 기존 문서를 유지하고 #1794 행을 M100 표에
추가하는 방식으로 해소한 뒤 contributor branch를 force-with-lease 갱신했다.

## 시각 검증

| target | SVG/PDF pages | flagged | 대표 asset |
|---|---:|---:|---|
| `pr1791-1805-exclusion` | 2/2 | 0 | `mydocs/pr/assets/pr1791_pr1805_exclusion_review_p001.png` |

원본 산출물: `output/pr1818-planet6897-visual/pr1791-1805-exclusion/review/review_001.png`

## 검증

- 최신 PR head의 실제 non-merge 변경 커밋 확인
- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1789_exclusion_probe_line_spacing`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

PR 내용 기준으로 source gate 제거가 #1791 계약과 함께 검증됐다. merge 후보로 판단한다.
