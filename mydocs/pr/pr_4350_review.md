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
| 메인터너 code head | `0d9aa7f177955e22fa654f409043d4e36f35ce61` |
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
| `bindings/node/bin/rhwp-mcp.cjs` | 첫 SIGINT/SIGTERM을 자식에 전달하고 5초 뒤 강제 종료, 두 번째 신호는 즉시 강제 종료한다. 부모의 명시적 exit에도 자식 종료를 시도하고 자식 exit까지 기다려 회수한다. |
| `bindings/node/test/mcp-bin.integration.test.ts` | initialize 뒤 stdin EOF로 정상 종료한다. POSIX에서는 가짜 server로 SIGINT와 SIGTERM을 각각 전달하고 자식 PID가 사라졌는지 확인한다. |

contributor commits는 수정·squash·rebase하지 않았으며 위 메인터너 commit 한 개만 추가했다.

## 완료한 검증

| 게이트 | 결과 |
| --- | --- |
| `node --check bindings/node/bin/rhwp-mcp.cjs` | 통과 |
| `npm.cmd run typecheck` | 통과 |
| `npm.cmd exec -- vitest run test/mcp-bin.integration.test.ts --project integration` | 최종 tree에서 2 passed, 1 skipped |
| 실제 `rhwp.exe` initialize 및 stdin EOF 종료 | Windows 통합 테스트에서 통과 |
| POSIX SIGINT/SIGTERM 전달·자식 PID 회수 계약 | 테스트를 추가했으나 현재 Windows host에서는 platform skip |
| `git diff --check origin/pr/4350..0d9aa7f1` | 통과 |

`npm ci`는 89 packages를 설치했고 tracked dependency 파일은 바꾸지 않았다. 설치 과정의 audit 결과
1 low·1 high는 기존 lock dependency 상태이며 이 보정에서 자동 수정하지 않았다.

## 잔여 위험

- Windows의 강제 `TerminateProcess`/`taskkill`은 JavaScript signal·exit handler를 실행하지 않을 수 있다.
  정상 stdin EOF와 console signal 수명은 이번 범위지만, OS 강제 종료 전체를 Job Object 없이 보증하지 않는다.
- 현재 host가 Windows라 POSIX 신호 회귀는 실행되지 않았다. 원격 후보를 만들면 Linux GitHub Actions에서
  해당 테스트의 실제 통과가 필수다.
- source/test를 추가한 local head이므로 원 contributor head의 기존 녹색 CI를 메인터너 보정의 근거로
  재사용하지 않는다. push 승인이 나면 새 head 전체 required checks가 필요하다.

## 조건부 권고

**로컬 blocker는 메인터너 보정으로 해소되어 조건부 통합 권고다.** 다만 현재 branch는 로컬 전용이며
push 또는 merge 승인이 없다. 작업지시자가 반영 경로를 승인한 뒤 새 원격 head에서 package integration과
Linux 신호 회귀를 포함한 required checks가 모두 통과하고, 별도의 명시적 merge 승인이 있을 때만
통합할 수 있다.
