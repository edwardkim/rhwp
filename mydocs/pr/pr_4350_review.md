---
kind: pr_review
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4350 검토 — Node MCP wrapper의 자식 프로세스 수명

## 라우팅과 접수

기본 경로는 `maintainer_general.md`, 보조 경로는 `intake_and_review.md`와
`local_validation.md`다. contributor code 위에 메인터너 source/test 보정을 추가했으므로
[implementation 기록](pr_4350_review_impl.md)을 함께 유지한다.

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4350](https://github.com/edwardkim/rhwp/pull/4350) / `kevin9327` |
| 관련 이슈 | [#4349](https://github.com/edwardkim/rhwp/issues/4349), #4327, #4337 |
| 원 base / head | `devel` / `c0592f5f1bfaa156f87f755723c58816ad776931` |
| 원 변경 규모 | 3 files, +117/-0, contributor commits 2개 |
| 검토 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` — 원 head의 조상임을 확인 |
| 작성 시점 원격 참고 상태 | `MERGEABLE` / `CLEAN`, 원 head GitHub checks 성공. merge 전 재확인 필요 |
| 가시성 branch | `review/kevin9327-20260810-pr4350` |
| 메인터너 code head | `d1dad3c07c4bb5569e3e94e37e3e9b3d31edee28` |
| GitHub 상태 변경 | reviewer assign, comment, push, review, merge 모두 미수행 |

이번 작업지시는 로컬 메인터너 보정과 기록까지이며 GitHub mutation 승인을 포함하지 않는다. 따라서
접수 가이드의 reviewer assign도 실행하지 않았고, 이 문서의 원격 상태는 2026-08-10 조회 시점 참고값이다.

## contributor 변경 범위

원 PR은 `@rhwp/node`에 `rhwp-mcp` bin을 추가한다. wrapper는 패키지의 기존 `findBinary()` 탐색 순서를
재사용해 `rhwp mcp-serve`를 stdio 상속으로 실행하고, 패키지 manifest와 실제 바이너리 통합 테스트를
함께 추가한다. 후속 contributor commit은 바이너리가 필요한 테스트를 integration project로 옮기고,
클론·CI에서 `dist/index.cjs`가 없으면 테스트 준비 단계에서 package build를 수행하게 했다.

renderer, layout, WASM, sample, golden 및 fixture 변경은 없다. 따라서 시각 검증 대상이 아니며 별도
PDF/SVG/브라우저 증적을 만들지 않았다.

## 확인한 blocker

wrapper는 자식의 `exit`만 부모 종료 코드로 반영하고 `SIGINT`·`SIGTERM`을 전달하지 않았다. MCP host가
wrapper PID만 종료하면 실제 `rhwp mcp-serve`가 계속 실행될 수 있고, 기존 initialize 테스트도 응답을 받은
뒤 wrapper만 `kill()`해 같은 누수 가능성을 만들었다. stdio MCP server의 프로세스 수명 계약을 깨므로
원 head 그대로는 merge blocker로 판정했다.

## 메인터너 보정

commit `0d9aa7f177955e22fa654f409043d4e36f35ce61`
(`fix(node): reap MCP child on wrapper shutdown`)을 원 contributor head의 직계 자식으로 추가했다.

| 파일 | 보정 |
| --- | --- |
| `bindings/node/bin/rhwp-mcp.cjs` | POSIX 첫 SIGINT/SIGTERM을 자식에 전달하고 5초 뒤 강제 종료, 두 번째 신호는 즉시 강제 종료한다. 정상 경로에서는 자식 exit를 관찰해 부모 종료 코드를 정한다. `exit` hook은 동기 종료 요청만 시도하며 await/reap을 보장하지 않는다. |
| `bindings/node/test/mcp-bin.integration.test.ts` | initialize 뒤 stdin EOF로 정상 종료한다. POSIX에서는 가짜 server로 SIGINT와 SIGTERM을 각각 전달하고 자식 PID가 사라졌는지 확인한다. |

contributor commits는 수정·squash·rebase하지 않았으며 위 1차 메인터너 commit을 별도로 추가했다.

후속 조사에서는 Windows의 `child.kill()`이 direct process만 `TerminateProcess`해 손자를 남길 수 있고,
외부에서 wrapper 자체를 `TerminateProcess`하면 JavaScript signal/exit hook이 아예 실행되지 않는 경계를
확인했다. 작은 안전 보정이 가능한 wrapper 통제 경로만 후속 commit
`d1dad3c07c4bb5569e3e94e37e3e9b3d31edee28`
(`fix(node): terminate Windows MCP child trees`)로 강화했다.

| 파일 | 후속 보정 |
| --- | --- |
| `bindings/node/bin/rhwp-mcp.cjs` | Windows에서 JS handler·grace timeout·second signal·explicit exit hook이 실행되는 강제 종료 경로는 동기 `taskkill.exe /PID <pid> /T /F`를 사용한다. POSIX 첫 신호의 graceful forwarding은 유지한다. |
| `bindings/node/test/mcp-bin.integration.test.ts` | bin을 server 기동 없이 require할 수 있게 내부 helper를 노출하고, Windows에서 parent가 만든 grandchild까지 두 PID가 사라지는지 실제 process tree로 검증한다. |

이 보정은 Job Object가 아니며 외부 abrupt wrapper 종료를 가로채지 않는다는 한계를 코드 주석과 아래
잔여 위험에 함께 고정했다. contributor와 기존 메인터너 commit은 재작성하지 않았다.

## 완료한 검증

| 게이트 | 결과 |
| --- | --- |
| `node --check bindings/node/bin/rhwp-mcp.cjs` | 통과 |
| `npm.cmd run typecheck` | 통과 |
| `.\node_modules\.bin\vitest.cmd run test\mcp-bin.integration.test.ts --project integration` | filesystem sandbox의 config resolution 거부 뒤 승인된 local rerun에서 3 passed, 1 skipped |
| 실제 `rhwp.exe` initialize 및 stdin EOF 종료 | Windows 통합 테스트에서 통과 |
| POSIX SIGINT/SIGTERM 전달·자식 PID 회수 계약 | 테스트를 추가했으나 현재 Windows host에서는 platform skip |
| Windows wrapper 통제 강제 종료 | 실제 parent·grandchild tree를 만들고 `taskkill /T /F` 뒤 두 PID 종료 확인 |
| `git diff --check origin/pr/4350..d1dad3c0` | 통과 |

`npm ci`는 89 packages를 설치했고 tracked dependency 파일은 바꾸지 않았다. 설치 과정의 audit 결과
1 low·1 high는 기존 lock dependency 상태이며 이 보정에서 자동 수정하지 않았다.

## 잔여 위험

- 외부가 wrapper PID 자체를 `TerminateProcess`하거나 `/T` 없는 `taskkill /F`로 끊으면 JavaScript
  signal·exit handler가 실행되지 않아 이번 tree helper도 호출되지 않는다. wrapper가 죽는 순간 kernel이
  자식 tree를 닫는 보장은 Windows Job Object/native 연동 없이는 제공하지 않는다.
- `process.once('exit', ...)` 경로는 동기 tree 종료를 요청하지만 event loop를 재개하거나 자식 exit를
  await/reap할 수 없다. 따라서 명시적 `process.exit()`·native crash의 완전한 회수 계약으로 주장하지 않는다.
- Windows 회귀는 wrapper가 직접 호출하는 `forceKillChildTree()`의 parent/grandchild 종료를 입증한다.
  외부 abrupt wrapper 종료를 재현하거나 보증하는 테스트는 아니다.
- 현재 host가 Windows라 POSIX 신호 회귀는 실행되지 않았다. 원격 후보를 만들면 Linux GitHub Actions에서
  해당 테스트의 실제 통과가 필수다.
- source/test를 추가한 local head이므로 원 contributor head의 기존 녹색 CI를 메인터너 보정의 근거로
  재사용하지 않는다. push 승인이 나면 새 head 전체 required checks가 필요하다.

## 조건부 권고

**POSIX signal·정상 EOF와 wrapper 통제 Windows 강제 종료에서 확인한 blocker는 해소되어 조건부 통합
권고다.** 다만 현재 branch는 로컬 전용이며
push 또는 merge 승인이 없다. 작업지시자가 반영 경로를 승인한 뒤 새 원격 head에서 package integration과
Linux 신호 회귀를 포함한 required checks가 모두 통과하고, 별도의 명시적 merge 승인이 있을 때만
통합할 수 있다.
