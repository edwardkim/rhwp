# PR #3463 검토 기록 — Studio test TypeScript compiler의 Windows 이식성

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3463](https://github.com/edwardkim/rhwp/pull/3463) — `test(studio): tsc 컴파일 스텝 Windows 중단 해소` |
| 작성자·검토자 | `@planet6897` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `74448394c3f49130402204e950133fabe593d06c` |
| 통합 검토 | `review/planet6897-20260727`; 적용 `74448394…` → `08fdad18a` |
| 작성 시점 source 상태 | `MERGEABLE` / `BEHIND`, source CI 전체 성공 |
| 라우팅 | `collaborator_external_pr` + `intake_and_review`, `local_validation`, `multi_pr_update_branch` |

## 판정 및 검증

`.bin/tsc` shell script와 `.cmd`를 shell 없이 spawn하면 Windows에서 중단하는 문제를, 현재 Node로
`node_modules/typescript/bin/tsc` JS 진입점을 실행하도록 바꾼다. `RHWP_STUDIO_TSC` override는 유지한다.
macOS focused Studio test 32/0에 더해 Windows `win10-ted` PowerShell의 source SHA 전용 임시
worktree에서 `node --test tests/cell-flow-boundary.test.ts tests/deferred-pagination-runner.test.ts`가
17/0으로 통과했다. 원격 기본 worktree는 변경하지 않았고 검증 worktree·junction은 제거했다.

## 최종 권고

**기술적으로 수용 가능**. 최신 통합 PR CI·mergeable과 작업지시자 승인을 최종 조건으로 둔다.
