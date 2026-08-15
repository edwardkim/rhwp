---
kind: pr-review
status: pending-ci
pr: 4880
author: jangster77
base: devel
head: codex/local-worktree-cleanup-policy
last_verified: 2026-08-16
---

# PR #4880 자체검토 - PR 후 local worktree 정리 명시

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#4880](https://github.com/edwardkim/rhwp/pull/4880) |
| 작성자 | `jangster77` (collaborator self-review) |
| base / head | `devel` / `codex/local-worktree-cleanup-policy` |
| code candidate | `7b427b40b` |
| 변경 범위 | `mydocs/manual/pr_review/` 아래 절차 문서 2개 |

collaborator 자체 PR이므로 reviewer를 지정하지 않았다. 현재 문서는 review·오늘할일 trailing commit을
추가하는 중의 기록이며, merge 직전에 최신 head, GitHub Actions, mergeability를 다시 확인한다.

## 변경 검토

- self-merge의 merge·후속 처리 승인은 해당 PR 전용 local branch와 local worktree의 cleanup까지 포함한다고
  명시했다.
- cleanup은 선택적 상태 보고가 아니라 merge 후 종료 게이트이며, 대상 worktree 자신이 아닌 보존 worktree에서
  실행해야 한다.
- remote branch, 기본 작업공간, 공유 `target/pr-review`, 사용자 또는 다른 도구 소유 대상은 이 승인 범위에서
  제외해 기존 소유권 보호를 유지한다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| `git diff --cached --check` | 통과 |
| 변경 경로 확인 | `collaborator_self_merge.md`, `post_merge.md`만 변경 |
| 링크 경로 확인 | `post_merge.md` 상대 링크가 존재함을 확인 |
| Cargo/npm/WASM | `mydocs` 전용 변경이므로 `local_validation.md` 4.3에 따라 미적용 |

## 위험과 병합 조건

- local cleanup은 이번 PR 전용이고 clean하며 다른 작업의 소유가 아님을 확인한 경우에만 자동으로 수행된다.
- cleanup 승인 범위가 remote branch나 기본 작업공간까지 확장되지 않도록 절차 문구를 제한했다.
- review·오늘할일 trailing head의 GitHub Actions 통과, `MERGEABLE`/`CLEAN` 확인, 작업지시자 승인이 필요하다.

## 최종 권고

**보류.** 문서 전용 local 검증은 통과했다. trailing 문서 commit의 CI와 최신 mergeability를 확인한 뒤
작업지시자 승인에 따라 병합한다.
