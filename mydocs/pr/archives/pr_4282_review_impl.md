---
kind: pr-review-implementation
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# kevin9327 PR #4282-#4308 누적 검토·메인터너 보정 계획

## 검토 경로

```text
base route: maintainer_general.md
modifiers: collaborator_external_pr.md, intake_and_review.md,
           local_validation.md, multi_pr_update_branch.md
review branch: review/kevin9327-20260809
base: upstream/devel f62f7503f
candidate: 5f3e9eaf0
```

## 범위와 적용 순서

`kevin9327`의 열린 `devel` 대상 PR 20건을 오래된 번호 순으로 최신 `upstream/devel` 위에
cherry-pick했다. 문서만 변경하는 #4293-#4296, #4298-#4301, #4306도 동일 후보에서 source
경로·로드맵 집계를 대조했다. 체리픽 충돌은 #4307의 Python test 파일과 roadmap README에서만
발생했으며, 선행·후행 PR의 테스트와 생성기 결과를 모두 보존했다.

## 메인터너 보정

| 대상 | commit | 보정 이유 |
| --- | --- | --- |
| #4282 | `83b5dd93e` | saturating 계산만으로는 행/열 개수 및 span 변경의 overflow·부분 변경을 막지 못해, mutation 전 `u16` 사전 검사를 추가했다. |
| #4302 | `6048d3d7a` | `--stats`의 사용자 입력 도구명을 HashMap key로 쓰면 opt-in 모드에서도 무한 key 축적이 가능해, 고정된 선언 도구명 또는 하나의 unknown bucket만 쓰게 했다. |
| #4293 | `4c0076141` | OWPML 관찰 문서의 `body_text.rs` 경로가 실제 source 경로와 달라 근거를 정정했다. |
| #4304 | `d6206e43a` | 공개 `TimeoutError` import/catch 경로를 깨지 않도록 별칭을 유지하고, 새 public API의 문서를 실제 동작과 맞췄다. |
| #4308 | `5f3e9eaf0` | Session timeout 뒤에도 자식 프로세스가 edit를 계속할 수 있어, timeout이 실행 경계를 종료하고 세션을 재사용 불가로 만들었다. |

## 실행 결과

| 검증 | 결과 |
| --- | --- |
| focused table / HWP3 / HWPX / caption / MCP contract tests | 통과 |
| `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 5,499 passed, 35 skipped, 450.799s |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| Python `pytest`, mypy, ruff | 251 passed, 43 skipped; 정적 검사 통과 |
| Node test, typecheck, build | 427 passed; typecheck/build 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |

## 원격 단계

각 원 PR에 reviewer `jangster77`을 지정했다. 원격 PR head·required checks·mergeability는
계속 변하는 참고값이므로, 실제 source branch 보정 push 또는 merge 전에 해당 PR별 최신 head를
다시 조회해야 한다. 이 로컬 누적 branch는 검토와 충돌 해소용이며 contributor 원 commit을
rewrite하지 않는다.
