# PR #4938 검토 - DocInfo raw provenance와 편집 저장 계약

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4938](https://github.com/edwardkim/rhwp/pull/4938) |
| 작성자 | `planet6897` |
| 검토자 | `jangster77` |
| 검토 방식 | 외부 기여 PR 누적 체리픽 검토 |
| base / 원본 head | `devel` / `7cef482a0741100ea86b009dd7f81bcaf87cd744` |
| 검토 브랜치 반영 | `84135dd6e` |
| 검토일 | 2026-08-16 |
| 작성 시점 병합 상태 | `CLEAN` |

## 검토 범위와 판단

- DocInfo 전체와 record별 digest를 분리해, 원문 raw 재사용은 모델이 원문과 일치할 때만 허용한다.
- 문서 속성 또는 char shape를 public API로 변경하면 stale raw를 재사용하지 않고 해당 범위를 재직렬화한다.
- raw stream을 바꿔 끼우는 경우도 digest 검증으로 차단한다.
- 추가 결함은 발견하지 못했다.

## 검증

- `cargo fmt --all -- --check` 통과
- `CARGO_TARGET_DIR=target/pr-review-planet6897 cargo test --test issue_4493_docinfo_provenance` 통과, 6건
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

원본 #4938은 독립적으로 병합하지 않으며, 해당 변경을 포함해 전체 local validation을 통과한
#4941의 최신 head CI가 통과하면 #4941만 병합한다.
