# PR #4928 검토 - HWPX 고아 fieldEnd 순서 보존

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4928](https://github.com/edwardkim/rhwp/pull/4928) |
| 작성자 | `planet6897` |
| 검토자 | `jangster77` |
| 검토 방식 | 외부 기여 PR 누적 체리픽 검토 |
| base / 원본 head | `devel` / `0558d8dfb36292e1f6649eefe56bbc0f8455f53b` |
| 검토 브랜치 반영 | `review/planet6897-20260816`의 `18256a12c` |
| 검토일 | 2026-08-16 |
| 작성 시점 병합 상태 | `CLEAN` |

## 검토 범위와 판단

- 동일 control slot에서 fieldBegin과 고아 fieldEnd가 함께 나타날 때, fieldBegin 및 짝 fieldEnd를
  먼저 방출하고 그 뒤 고아 fieldEnd를 방출하도록 HWPX 직렬화 순서를 제한한다.
- field가 아닌 slot은 기존 고아 fieldEnd 선행 순서를 유지한다. 따라서 순서 보정 범위가 교차 문단
  field에만 한정된다.
- 추가 결함은 발견하지 못했다.

## 검증

- `cargo fmt --all -- --check` 통과
- `CARGO_TARGET_DIR=target/pr-review-planet6897 cargo test --lib issue4902` 통과
- 누적 검토 브랜치에서 `git diff --check` 통과

## 최종 권고

원본 PR head의 CI와 병합 가능 상태를 merge 직전에 다시 확인한 뒤 병합을 권고한다.


## 통합 병합 판단 및 local validation

- 통합 PR: [#4941](https://github.com/edwardkim/rhwp/pull/4941)
- 통합 코드 후보: `1afe41afb`를 포함한 `integrate/planet6897-open-prs-20260816`
- `cargo fmt --all -- --check`: 성공
- `cargo test --profile release-test --tests`: 성공 (exit 0)
- `cargo clippy --all-targets --all-features -- -D warnings`: 성공 (exit 0)
- `cargo build --release`: 성공 (exit 0)

위 전체 local validation은 통합 코드 후보에서 전용 target 디렉터리
`/Users/tsjang/rhwp/target/pr-review-planet6897`로 수행했다. 원본 #4928은 독립적으로
병합하지 않으며, 해당 변경을 포함한 #4941의 최신 head CI가 통과하면 #4941만 병합한다.
