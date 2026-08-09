---
kind: review_plan
status: remote-validation-in-progress
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4350 메인터너 보정 실행 기록

## commit 경계

| 역할 | SHA | 상태 |
| --- | --- | --- |
| contributor 기능 | `621122fce8e6654b1cd365e24d370c476a73edf8` | 보존 |
| contributor CI 수리 / 원 head | `c0592f5f1bfaa156f87f755723c58816ad776931` | 보존 |
| 메인터너 프로세스 수명 보정 | `0d9aa7f177955e22fa654f409043d4e36f35ce61` | 완료 |
| 1차 review 및 implementation 기록 | `390514add555817919677245bb18dd1aaebfbc0b` | 완료 |
| 메인터너 Windows process-tree 보정 | `d1dad3c07c4bb5569e3e94e37e3e9b3d31edee28` | 완료 |
| 후속 review 및 implementation 기록 | `23a20d32b4576c23962589100c71d8d83c40e500` | 완료 |
| POSIX fixture readiness race 보정 | `8429ce9f49c6bd48d8a357a419c36a3d053dba82` | 완료 |
| 원격 CI 후속 기록 | 본 문서와 `pr_4350_review.md`의 trailing 문서 commit | 완료 후 SHA를 Git 이력에서 확인 |

가시성 branch는 `review/kevin9327-20260810-pr4350` 하나만 사용한다. 원 contributor commits 위에
메인터너 commit과 review 문서 commit을 선형으로 쌓으며 원 contributor·기존 메인터너 commit을 다시
작성하지 않는다.

## 실행 단계

1. **접수·기준 고정 — 완료.** `origin/pr/4350` head와 `origin/devel` 조상 관계를 확인하고 원 head에서
   가시성 branch/worktree를 만들었다. GitHub reviewer assign이나 comment는 수행하지 않았다.
2. **source/test 보정 — 완료.** `rhwp-mcp.cjs`에 POSIX signal forwarding, grace timeout,
   second-signal 강제 종료 및 best-effort exit cleanup을 추가했다. integration test의 정상 종료를
   stdin EOF로 바꾸고 POSIX SIGINT와 SIGTERM 자식 회수 회귀를 추가했다.
3. **로컬 검증 — 완료.** Node 구문·TypeScript·focused integration·diff 검사를 실행했다. Windows에서
   POSIX 전용 한 건이 skip된 사실을 [검토 기록](pr_4350_review.md)에 제한 조건으로 남겼다.
4. **1차 review 기록 — 완료.** source/test commit과 섞지 않고 이 두 문서를 별도 commit으로 만들었다.
5. **Windows process-tree 조사·보정 — 완료.** direct child `TerminateProcess`가 손자를 남기는 경로는
   wrapper가 통제할 때 `taskkill /T /F`로 바꿨다. 실제 parent·grandchild 회귀는 통과했다. 외부가 wrapper
   자체를 abrupt 종료하는 경우는 handler가 실행되지 않으므로 Job Object 없이는 보장하지 않는다.
6. **후속 검증·기록 — 완료.** Node syntax, TypeScript typecheck, focused integration 3 passed/1 POSIX skipped,
   diff check를 통과했다. 이 두 문서는 source/test 뒤의 trailing commit으로 만든다.
7. **원격 반영과 CI 관찰 — 진행.** 원 contributor branch에 correction을 비강제 fast-forward push했다.
   첫 새-head run에서 POSIX signal fixture가 handler 등록 전 `READY`를 출력하는 경쟁을 재현했다.
8. **CI 후속 보정 — 완료.** handler 두 개를 먼저 등록한 뒤 `READY`를 출력하도록 `8429ce9f`에
   분리했다. Windows에서는 해당 POSIX test가 skip되므로 TypeScript와 Node 구문을 통과시킨 뒤 Linux
   원격 재실행을 필수 gate로 유지한다.
9. **최신 CI와 merge — 진행/별도 승인 대기.** `8429ce9f` head의 required checks를 다시 받는다.
   성공 뒤에도 별도의 명시적 merge 승인이 있어야 하며, 그 전에는 merge·close를 수행하지 않는다.

## rollback

- 후속 기록만 철회할 때는 trailing 문서 commit만 revert한다. source/test 보정과 섞지 않는다.
- 전체 메인터너 보정을 철회할 때는 최신 문서, `8429ce9f`, `23a20d32`, `d1dad3c0`, `390514ad`,
  `0d9aa7f1`을 이력 역순으로 revert한다. 그러면 branch는 원 contributor head `c0592f5f...`의
  내용으로 돌아간다.
- contributor의 두 commit은 reset, amend, rebase 또는 squash하지 않는다.
- 원격 반영 뒤이므로 rollback도 force-push 없이 새 revert commit으로 수행하고 최신 CI를 다시 받는다.

## 권한 경계

이 기록은 merge 허가가 아니다. correction push는 완료했지만 GitHub review/comment, merge, close는
최신 상태 재확인과 별도 판단이 필요한 단계다.
