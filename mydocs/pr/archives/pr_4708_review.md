---
kind: review
status: self-review-ci-pending
pr: 4708
issue: 3414
author: jangster77
base: devel
---

# PR #4708 검토 기록

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4708](https://github.com/edwardkim/rhwp/pull/4708) |
| 작성자 | `jangster77` |
| 관련 이슈 | [#3414](https://github.com/edwardkim/rhwp/issues/3414) |
| head / base | `task_m100_3414` / `devel` |
| code candidate | `f90f1a30a4247438c3408ec0c85d2b62ab39c6c8` |
| 변경 규모 | 14 files, +366 / -146 |
| 문서 작성 시점 상태 | `MERGEABLE`, `BLOCKED` 참고값 — CI preflight·CodeQL preflight·Render Diff preflight 실행 중 |
| 검토 | `jangster77` collaborator 셀프 검토 코멘트 제출 완료 (`5717d14b7`) |

### 적용 절차

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, local_validation.md,
docs_and_git_workflow.md
current code candidate: f90f1a30a4247438c3408ec0c85d2b62ab39c6c8
```

최신 `upstream/devel` `8e81cbd996b66f873d21b74085a9dcee78ae3901` 위에서
`task_m100_3414`를 만들고 `upstream`에 게시했다. 구현이 완료된 상태이므로 draft가 아닌
PR로 열었다. 작업지시자 지시에 따라 외부 reviewer 요청은 제거하고 collaborator 셀프 검토
코멘트를 제출했다. merge와 이슈 종료는 최신 head의 필수 CI와 작업지시자 승인 조건을 다시
확인한 뒤에만 진행한다. 별도의 `review_impl`은 보정·복수 PR 누적·merge 작업이 없는 단일
자체 PR이므로 만들지 않았다.

## 변경 검토

마지막 `ModalDialog` overlay가 닫힌 뒤에만 공통 종료 이벤트를 내고, 앱은 활성
`InputHandler`의 textarea 포커스를 복원한다. 중첩 모달에서 자식만 닫혔을 때는 부모 overlay가
남아 있어 이벤트를 내지 않으므로 모달 안의 포커스를 빼앗지 않는다.

textarea 밖으로 포커스가 이동한 활성 편집기의 전역 keydown 경로는 기존 shortcut map으로
`edit:undo`·`edit:redo`만 dispatcher에 보낸다. 다른 전역 단축키 소유 범위를 넓히지 않고,
textarea 내부의 키 입력은 기존 `InputHandler`가 계속 처리한다. 그림·표 개체 선택의
copy/cut/delete는 canonical `edit:*` 명령으로 통일했으며, 사용자 제스처가 필요한 Ctrl+V의
native paste 이벤트는 막지 않는다. 표의 중첩 cellPath guard와 delete 뒤 caret 갱신도
공통 구현에서 유지한다.

renderer/layout, 문서 fixture, baseline 또는 시각 자산은 변경하지 않았다. 따라서 formal
renderer visual sweep은 선택하지 않았고, 실제 Studio 브라우저에서 모달과 버튼 포커스의
입력 동작을 별도로 확인했다.

## 완료한 검증

다음 검증은 code candidate에서 Windows PowerShell로 완료했다.

| 검증 | 결과 |
| --- | --- |
| `node --test tests/issue-3414-modal-focus.test.ts tests/issue-3414-shortcut-routing.test.ts` | 5 passed |
| `npx.cmd tsc --noEmit` | 통과 |
| `npm.cmd test` | 874 passed, 1 skipped, 0 failed |
| `npm.cmd run build` | 통과 (기존 Vite 경고만 발생) |
| 로컬 Studio 브라우저 스모크 | 일반·중첩 F6 스타일 모달의 마지막 종료 후 textarea 포커스 복원, 부모 모달이 남은 자식 종료에서는 비복원, 버튼 포커스의 Ctrl+Z undo 확인 |
| `git diff --check upstream/devel...f90f1a30a4247438c3408ec0c85d2b62ab39c6c8` | 통과 |

이 검토 기록과 오늘할일은 code candidate 뒤에 추가하는 docs-only 후속 commit이다. 따라서
실제 merge 전에는 이 문서를 포함한 최신 PR head의 GitHub Actions와 mergeability를 다시
확인한다.

PR #4708의 `5717d14b7` head에 collaborator `jangster77`가 `COMMENTED` 셀프 검토를
제출했다. 이 문서의 후속 정정은 docs-only 변경이므로, 셀프 검토 범위인 code candidate와
로컬 검증 결과는 바꾸지 않는다.

## 최종 조건과 권고

코드 검토와 로컬 검증에서 blocker는 발견하지 못했다. 다만 문서 작성 시점에는 필수 CI가
실행 중이므로 **현재는 merge 보류**다. 최신 PR head의 CI 성공, 셀프 검토의 유효성, 최신
mergeability와 작업지시자 승인이 모두 확인되면 수용·merge를 진행하고, merge 뒤 `Closes #3414`에
따른 이슈 종료 상태와 후속 정리를 post-merge 절차로 확인한다.
