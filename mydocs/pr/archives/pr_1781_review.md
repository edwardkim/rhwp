# PR #1781 리뷰 — Task #1771 중첩 그룹 벡터 roundtrip 보존

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1781 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1771` |
| 관련 이슈 | #1771 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | planet6897 open PR 묶음 누적 cherry-pick |

## 변경 범위

- `src/serializer/control.rs`
- `tests/issue_1771_nested_group_roundtrip.rs`
- `samples/task1771/nested_group_vectors.hwpx`
- 관련 계획/샘플 문서

중첩 그룹 직렬화 경계를 보정해 복합 벡터가 roundtrip 과정에서 대량 소실되지 않도록 한다.

## 검토 결과

중첩 그룹 내부 컨트롤을 직렬화할 때 경계가 끊기지 않도록 serializer 계약을 보강했다. 테스트는 샘플의
중첩 그룹 roundtrip 후 벡터 수가 보존되는지 확인한다. visual sweep은 PR 내용과 맞는 보조 검증으로 수행했고
자동 후보가 없었다.

## 시각 검증

| target | SVG/PDF pages | flagged | 대표 asset |
|---|---:|---:|---|
| `pr1781-nested-vectors` | 15/15 | 0 | `mydocs/pr/assets/pr1781_nested_group_vectors_review_p001.png` |

원본 산출물: `output/pr1818-planet6897-visual/pr1781-nested-vectors/review/review_001.png`

## 검증

- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1771_nested_group_roundtrip`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

PR 목적에 맞게 nested group vector roundtrip 보존이 테스트와 대표 visual sweep으로 확인됐다. merge 후보로 판단한다.
