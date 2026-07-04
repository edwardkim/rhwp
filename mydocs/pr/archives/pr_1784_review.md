# PR #1784 리뷰 — Task #1772 HWPX 표 outMargin 동기화

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1784 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1772` |
| 관련 이슈 | #1772 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | planet6897 open PR 묶음 누적 cherry-pick |

## 변경 범위

- `src/parser/hwpx/section.rs`
- `tests/issue_1772_table_outer_margin_sync.rs`
- `samples/task1772/table_outer_margin_common_sync.hwpx`
- 관련 계획/보고/원인 문서

HWPX 표 `outMargin`을 파싱할 때 내부 `common.margin`에도 동기화해 후속 layout 단계가 같은 값을 보도록 한다.

## 검토 결과

파서 계층에서 표 외부 여백의 단일 모델 값을 맞춰 renderer 쪽 보정 없이 margin 손실을 막는다. 회귀 테스트가
HWPX 파싱 후 `common.margin` 동기화를 직접 확인한다. 기준 PDF visual sweep은 1쪽 모두 자동 후보 없이 통과했다.

## 시각 검증

| target | SVG/PDF pages | flagged | 대표 asset |
|---|---:|---:|---|
| `pr1784-1788-table-margin` | 1/1 | 0 | `mydocs/pr/assets/pr1784_pr1788_table_margin_review_p001.png` |

원본 산출물: `output/pr1818-planet6897-visual/pr1784-1788-table-margin/review/review_001.png`

## 검증

- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1772_table_outer_margin_sync`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

PR 내용 기준으로 parser margin 동기화가 테스트와 visual sweep으로 확인됐다. merge 후보로 판단한다.
