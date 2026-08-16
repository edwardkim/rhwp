# PR #4933 검토 - HWPX 그림 shape component offset 보존

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4933](https://github.com/edwardkim/rhwp/pull/4933) |
| 작성자 | `planet6897` |
| 검토자 | `jangster77` |
| 검토 방식 | 외부 기여 PR 누적 체리픽 검토 |
| base / 원본 head | `devel` / `004799dc8d32fb9e464c9257a854507f2566dac4` |
| 검토 브랜치 반영 | `6d3b70af2`, `f430db8e6` |
| 검토일 | 2026-08-16 |
| 작성 시점 병합 상태 | `CLEAN` |

## 검토 범위와 판단

- HWPX picture 직렬화가 common position 대신 shape component의 저장 offset을 사용하게 해,
  음수 좌표를 포함한 원문 offset이 위치 보정으로 덮이지 않도록 한다.
- 실물 HWPX 샘플을 이용한 통합 테스트가 저장 전후 offset 값을 확인한다.
- 추가 결함은 발견하지 못했다.

## 검증

- `cargo fmt --all -- --check` 통과
- `CARGO_TARGET_DIR=target/pr-review-planet6897 cargo test --test issue_4668_pic_offset_preserved` 통과
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

원본 #4933은 독립적으로 병합하지 않으며, 해당 변경을 포함해 전체 local validation을 통과한
#4941의 최신 head CI가 통과하면 #4941만 병합한다.
