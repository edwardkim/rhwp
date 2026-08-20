---
kind: pr-review
status: approved
pr: 5691
issue: 5690
---

# PR #5691 검토 기록 - F3 블록 선택 undo 확장 단계 복원

- PR: [#5691](https://github.com/edwardkim/rhwp/pull/5691) `fix(#5690): F3 블록 선택 삭제 undo 가 확장 단계까지 되살리게 한다`
- 관련 이슈: [#5690](https://github.com/edwardkim/rhwp/issues/5690)
- 작성자: `@lpaiu-cs`, `maintainer_can_modify=true`
- source code candidate: `a4c90a830208ccd4cf6a3699ba1d569389d041dd`
- 검토 기준: `upstream/devel@1139f28d1` 위 `review/open-prs-20260820`
- 체리픽: `0fbc09b2e` (`-x`, 원 작성자·원 SHA 보존)
- 라우팅: `collaborator_external_pr` + `intake_and_review` + `local_validation` + `multi_pr_update_branch`

## 검토 범위

- F3 블록 선택 삭제 command가 삭제 전 확장 단계를 보관하고, undo가 범위와 단계를 한 번에 복원한다.
- `0`(블록 선택이나 아직 확장 전)과 `null`(블록 선택 아님)을 구분해 일반 드래그 선택을 블록 모드로 잘못 복원하지 않는다.
- redo의 기존 해제 규칙과 #2339의 유령 선택 차단은 유지한다.

## 검증 근거

- 최신 `devel` 위 체리픽은 충돌 없이 적용됐고 manifest prepare/check, source unit-tier 정책, format·diff 검사를 통과했다.
- `node --test rhwp-studio/tests/issue-3416-selection-restore.test.ts rhwp-studio/tests/issue-5690-block-selection-phase.test.ts`는 `10/10` 통과했다. 범위 유효성 거절, 구역 경계, undo 복원, redo 해제, block phase 보존을 포함한다.
- source head의 Frontend package gate, Canvas visual diff, CodeQL, Proptest, adapter inter-diff가 성공했고 해당 Rust worker들은 영향 없음으로 skip됐다.

## 결론

**승인.** 한컴 실측이 요구한 “undo 직후 다음 F3가 문단 전체까지 이어 확장”하는 계약을 `CursorState` 소유 경계에서 복원하며, 일반 선택과 redo의 기존 동작도 보존한다. #5690은 통합 후보 PR의 CI 성공 뒤 수용 결과와 함께 닫는다.
