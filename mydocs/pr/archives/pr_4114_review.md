---
kind: pr_review
status: accepted-with-maintainer-correction-pending-remote-checks
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

## 발견 사항과 메인터너 보정

최초 통합 PR head `d9c1bc0ae`의 node-binding CI는 `통합 (rhwp 빌드)`와 `생성 타입 최신 검사`에서
실패했다. capabilities가 JSON 명령 34개를 선언하는데 Node binding은 31개 기준 생성물을 유지했고,
`export-plan-schema`, `export-agent-manifest`, `explain`의 1층 wrapper가 없었다.

메인터너 보정 `4ab2df72f`는 세 무상태 wrapper와 argv 회귀를 추가하고, capabilities에서 생성한
`src/envelopes.ts`를 34개 봉투로 재생성했다. CI 워크플로는 수정하지 않았다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --test capabilities_subcommands_contract -- --nocapture` | 4 passed |
| `cargo test --profile release-test --test capabilities_schema_contract -- --nocapture` | 17 passed |
| `cargo fmt --check` | 성공 |
| `cargo clippy --all-targets -- -D warnings` | 성공 |
| Node binding `RHWP_BIN=target/release-test/rhwp npm test` | 17 files, 445 tests passed |
| Node binding `npm run typecheck` · `npm run build` | 성공 |
| Node binding `RHWP_BIN=target/release-test/rhwp npm run gen:check` | IR 41개 정의·capabilities 34개 봉투 최신 확인 |

Cargo 전체 회귀는 수행하지 않았다. 변경 계약에 직접 대응하는 테스트와 정적 Rust 검증만 실행했다.

통합 보정은 R7을 canonical 로드맵에서 완료로 정렬한다. **통합 수용 보류 조건은 보정 head의 최신
GitHub CI·CodeQL 성공 확인**이다. 원 source #4114는 2026-08-07 확인에서 check가 없었고, 최초 통합
CI는 위 Node drift를 발견했다.
