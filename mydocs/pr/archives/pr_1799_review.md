# PR #1799 리뷰 — Task #1658 라운드 5 페이지 하단 고정 표 하단 배타 예약

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1799 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1658-r5` |
| 관련 이슈 | #1658 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | planet6897 open PR 묶음 누적 cherry-pick |

## 변경 범위

- `src/renderer/float_placement.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/typeset.rs`
- `tests/issue_1658_page_bottom_fixed_exclusion.rs`
- `samples/hwpx/opengov/*.hwpx`
- 관련 계획/작업/보고 문서

페이지 하단 fixed table의 하단 배타 예약을 조정해 패턴 B over-pagination을 해소한다.

## 검토 결과

PR 목적은 renderer pagination/exclusion 보정이므로 기준 PDF visual sweep을 실제 판단 근거로 사용했다.
두 opengov 샘플 모두 페이지 수가 기준 PDF와 맞고 자동 후보가 없었다. 테스트는 page-bottom fixed exclusion
경로를 직접 확인한다.

## 시각 검증

| target | SVG/PDF pages | flagged | 대표 asset |
|---|---:|---:|---|
| `pr1799-36389312` | 1/1 | 0 | `mydocs/pr/assets/pr1799_36389312_review_p177.png` |
| `pr1799-36398366` | 1/1 | 0 | `mydocs/pr/assets/pr1799_36398366_review_p001.png` |

## 검증

- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1658_page_bottom_fixed_exclusion`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

PR 내용 기준으로 renderer pagination 보정이 테스트와 기준 PDF visual sweep으로 확인됐다. merge 후보로 판단한다.
