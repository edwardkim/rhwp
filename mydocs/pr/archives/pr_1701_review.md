# PR #1701 리뷰 문서

## 개요

| 항목 | 내용 |
|------|------|
| PR | #1701 `Task #1664: CI cache 정책 및 측정 기록 문서화` |
| 작성자 | @postmelee |
| 경로 | collaborator self-merge 후보 |
| base | `devel` |
| head | `postmelee:task1664-docs-policy-measurement` |
| 문서 작성 시점 참고 head | `ed074885ed3abc7f39fac6554882df78f6f5dcbb` |
| 관련 이슈 | #1664, #1668 |
| labels | `ci`, `documentation` |
| milestone | `v1.0.0` |

이 PR은 외부 contributor PR이 아니라 collaborator 본인 PR이다. 따라서 `mydocs/manual/pr_review_workflow.md`
8장 collaborator self-merge 후보 예외 경로를 적용한다.

## 변경 범위

PR #1701은 `mydocs/**` 문서 전용 PR이다.

- `mydocs/orders/20260630.md`
- `mydocs/plans/task_m100_1664.md`
- `mydocs/plans/task_m100_1664_impl.md`
- `mydocs/working/task_m100_1664_stage1.md`
- `mydocs/working/task_m100_1664_stage2.md`
- `mydocs/working/task_m100_1664_stage3.md`
- `mydocs/tech/ci_cache_policy_1664.md`
- `mydocs/report/task_m100_1664_measurement.md`
- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`
- `mydocs/report/task_m100_1664_report.md`

실제 CI workflow 변경인 `.github/workflows/ci.yml`은 PR #1701에 포함하지 않는다. 해당 변경은 후속 코드 PR
#1702에서 다룬다.

## 재검토 중 수정한 사항

1차 검토에서 `task_m100_1664_report.md`와 Stage 보고서 일부가 `.github/workflows/ci.yml` 변경이 이미
`devel`에 반영된 것처럼 읽힐 수 있음을 확인했다.

수정 커밋 `ed074885`에서 다음을 명확히 했다.

- PR #1701은 정책/측정 기록 전용 문서 PR이다.
- PR #1702는 후속/draft 코드 PR이며, workflow 변경과 CI 관측은 #1702 기준이다.
- #1702가 merge되기 전에는 workflow 변경이 `devel`에 반영된 것으로 단정하지 않는다.
- #1702 PR run 측정값은 #1666/#1667 비교 기준으로만 사용한다.

수정 후 동일한 blocking finding은 남지 않았다.

## 검증

| 항목 | 결과 | 비고 |
|------|------|------|
| 별도 worktree 리뷰 | 통과 | `/private/tmp/rhwp-pr1701-review` |
| 변경 범위 | 통과 | `mydocs/**` 10개 파일 |
| `git diff --check upstream/devel...HEAD` | 통과 | whitespace 문제 없음 |
| PR metadata | 통과 | `ci`, `documentation`, `v1.0.0`, assignee @postmelee |
| PR #1702 관측값 교차확인 | 통과 | run `28430353568`, Build & Test 19m08s |
| 테스트 파일 수 확인 | 통과 | `tests/*.rs` 162개, `tests/issue_*.rs` 131개 |

PR #1702 run `28430353568`에서 확인한 핵심 값:

- `Build & Test`: success, 19m08s
- `Restore cargo registry & build cache`: exact hit
- cache size: 약 1476 MB
- `Save cargo registry & build cache`: skipped
- cache reservation/read-only/save 실패 경고: 관측되지 않음

## 최신 PR 상태 참고값

문서 작성 시점 기준:

- `mergeable`: `MERGEABLE`
- `mergeStateStatus`: `CLEAN`
- 최신 PR #1701 checks:
  - `CI preflight`: success
  - `CodeQL preflight`: success
  - `Render Diff preflight`: success
  - `Build & Test`: skipped
  - `Analyze`: skipped
  - `Canvas visual diff`: skipped
  - `WASM Build`: skipped

후속 문서 커밋만 포함된 `mydocs/**` PR이므로 heavy job skip은 fast-pass 조건에 부합한다. merge 전에는 최신
head 기준 상태를 다시 확인해야 한다.

## 판단

현재 PR #1701은 문서 PR과 후속 코드 PR #1702의 경계를 명확히 기록하고 있으며, `mydocs/**` 범위를 벗어나지
않는다.

권고:

- 작업지시자 승인 후 merge 가능
- merge 직전 최신 head SHA, mergeability, checks 상태 재확인 필요
- #1664 이슈 close는 PR #1701 merge만으로 수행하지 않는다. #1702 및 trusted branch push 측정 확인 후 별도
  판단한다.
