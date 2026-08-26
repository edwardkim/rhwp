---
kind: review
status: self-review-ci-pending
pr: 4710
issue: 3438
author: jangster77
base: devel
---

# PR #4710 검토 기록

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4710](https://github.com/edwardkim/rhwp/pull/4710) |
| 작성자 | `jangster77` |
| 관련 이슈 | [#3438](https://github.com/edwardkim/rhwp/issues/3438) |
| head / base | `task_m100_3438` / `devel` |
| code candidate | `b3ceeb3303ad033218237a61e4d24dea8a8d7227` |
| 변경 규모 | 10 files, +320 / -56 |
| 문서 작성 시점 상태 | `MERGEABLE`, `BLOCKED` 참고값 — CI·CodeQL·Render Diff preflight 실행 중 |
| 검토 | `jangster77` collaborator 셀프 검토 코멘트 제출 완료 (`PRR_kwDORyECHM8AAAABJYyrLA`, `1d95af512`) |

### 적용 절차

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, local_validation.md,
visual_fixture_evidence.md, docs_and_git_workflow.md
current code candidate: b3ceeb3303ad033218237a61e4d24dea8a8d7227
```

최신 `upstream/devel` `d15b63eb46552ac61da24ad4a1b0c56208f71544` 위에서
`task_m100_3438`을 만들고 `upstream`에 게시했다. 구현이 완료된 상태이므로 draft가 아닌 PR로
열었다. 작업지시자 지시에 따라 external reviewer request는 만들지 않고 collaborator 셀프
검토를 제출했다. merge와 이슈 종료는 최신 head의 필수 CI, mergeability와 작업지시자 승인 조건을
다시 확인한 뒤에만 진행한다.

## 변경 검토

한컴 Office 2022의 새 빈 문서에서 F6 스타일 대화상자 중 Ctrl+Z가 문서 undo를 실행하지 않는 것을
확인했다. Studio `ModalDialog`도 capture 단계에서 같은 입력 계약을 적용하므로, `StyleDialog`에서
도달할 수 없는 `history-jumped` 구독·동기화·해제 핸들을 제거한 것은 적절하다. 일반적인
스타일 추가·편집 뒤 `refresh()` 경로와 삭제 snapshot 경로는 유지한다.

`StyleEditDialog`는 새 스타일 생성 API가 실패용 음수 ID를 반환하지 않는 실제 계약을 따르며,
기존 ID를 갱신하는 `updateStyle`·`updateStyleShapes`의 `false`는 각각 no-op 또는 rollback 경로로
처리한다. `onConfirm()`은 validation, `false`, 예외에서 `false`를 반환해 `ModalDialog`가 닫히지
않도록 하고, 저장 성공 때만 `onSave`와 document-changed 후속 동작을 실행한다.

renderer/layout, HWP/HWPX fixture, golden, 기준 PDF, Canvas 출력은 변경하지 않았다. formal visual
fixture evidence는 merge 판단 보조 경로로 선택하지 않았고, 실제 Studio 브라우저와 한컴 UI의 F6
입력 동작을 기능 검증으로 실행했다.

## 완료한 검증

다음 검증은 code candidate에서 Windows PowerShell로 완료했다.

| 검증 | 결과 |
| --- | --- |
| 한컴 Office 2022 새 문서 F6 → Ctrl+Z | 스타일 모달 유지, Escape 뒤 `10글자` 유지 |
| `node --test tests/style-undo-routing.test.ts tests/undo-noop-skip.test.ts` | 12 passed |
| `npx.cmd tsc --noEmit` | 통과 |
| `npm.cmd test` | 875 passed, 1 skipped, 0 failed |
| `npm.cmd run build` | 통과 (기존 Vite 경고만 발생) |
| 로컬 Studio fixture F6 → Ctrl+Z | 스타일 모달 유지 |
| 로컬 Studio 모달 닫기 → Ctrl+Z | textarea 포커스 유지, console error/warning 없음 |
| `git diff --check upstream/devel...b3ceeb3303ad033218237a61e4d24dea8a8d7227` | 통과 |

이 검토 기록과 오늘할일은 code candidate 뒤에 추가하는 docs-only 후속 commit이다. 따라서 실제
merge 전에는 이 문서를 포함한 최신 PR head의 GitHub Actions와 mergeability를 다시 확인한다.

## 최종 조건과 권고

코드 검토와 로컬 검증에서 blocker는 발견하지 못했다. 문서 작성 시점에는 필수 CI가 실행 중이므로
**현재는 merge 보류**다. 최신 PR head의 CI 성공, 셀프 검토의 유효성, 최신 mergeability와
작업지시자 승인이 모두 확인되면 수용·merge를 진행하고, merge 뒤 `Closes #3438`에 따른 이슈 종료
상태와 후속 정리를 post-merge 절차로 확인한다.
