# PR #1788 리뷰 — Task #1785 셀 안 여백 선택 규칙 단일 출처 통일

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1788 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1785` |
| 관련 이슈 | #1785 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | #1784 위에 누적 cherry-pick |

## 변경 범위

- `src/model/table.rs`
- `src/renderer/height_measurer.rs`
- `src/renderer/layout/table_layout.rs`
- `tests/issue_1785_cell_padding_rule_consistency.rs`
- 관련 계획/보고/원인 문서

셀 안 여백 선택 규칙을 모델의 단일 출처로 모으고, `aim=false` 레거시 보존값을 유지한다.

## 검토 결과

#1784의 margin 동기화 위에서 renderer height/layout이 같은 padding 선택 규칙을 쓰도록 정리한다. 테스트는
height measurer와 layout 경로가 동일한 padding 값을 사용하는지 확인한다. 기준 샘플 visual sweep도 1쪽 자동
후보 없이 통과했다.

## 시각 검증

| target | SVG/PDF pages | flagged | 대표 asset |
|---|---:|---:|---|
| `pr1784-1788-table-margin` | 1/1 | 0 | `mydocs/pr/assets/pr1784_pr1788_table_margin_review_p001.png` |

원본 산출물: `output/pr1818-planet6897-visual/pr1784-1788-table-margin/review/review_001.png`

## 검증

- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1785_cell_padding_rule_consistency`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

PR 내용 기준으로 padding rule 단일 출처화와 #1784 스택 검증이 완료됐다. merge 후보로 판단한다.
