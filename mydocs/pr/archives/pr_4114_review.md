---
kind: pr_review
status: accepted-pending-remote-checks
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4114 검토 - capabilities 하위 명령 자기서술

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4114](https://github.com/edwardkim/rhwp/pull/4114) / @kevin9327 |
| 원 head | `594398d0fcc121e06ba1b9781e1b4fbaefe274bb` |
| 규모 | 6개 파일, +306/-3, 2 commits |
| 원격 참고 상태 | `MERGEABLE` / `BLOCKED`; 현재 source branch에 GitHub check가 보고되지 않음 |
| 시각 검증 | 비대상. CLI capability 봉투와 계약 테스트만 바꾼다. |

`edit` 6종과 `inspect` 3종 하위 명령을 capabilities 봉투의 `subcommands` 객체 배열로 노출하고,
`--search`가 하위 이름·요약을 찾게 한다. 두 번째 commit은 실제 봉투와 JSON Schema의 드리프트를
수정한다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --test capabilities_subcommands_contract -- --nocapture` | 4 passed |
| `cargo test --profile release-test --test capabilities_schema_contract -- --nocapture` | 17 passed |
| `cargo fmt --check` | 성공 |
| `cargo clippy --all-targets -- -D warnings` | 성공 |

Cargo 전체 회귀는 수행하지 않았다. 변경 계약에 직접 대응하는 테스트와 정적 Rust 검증만 실행했다.

통합 보정은 R7을 canonical 로드맵에서 완료로 정렬한다. **통합 수용 보류 조건은 현재 원 head의 GitHub
CI·CodeQL 생성 및 성공 확인**이다. 2026-08-07 확인 시 `gh pr checks 4114`는 check가 없다고 반환했다.
