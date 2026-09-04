# 구현 계획서 — Task M100-3438 Stage 2

- 이슈: [#3438](https://github.com/edwardkim/rhwp/issues/3438)
- 수행 계획서: `mydocs/plans/task_m100_3438_stage2.md`
- 기준 branch: `local/task_m100_3438`
- 기준 devel: `d15b63eb46552ac61da24ad4a1b0c56208f71544`

## 구현 순서

1. `style-dialog.ts`에서 `historyJumpOff`, 생성자 구독, `syncAfterHistoryJump()`, `hide()` 해제를
   제거한다. `services`와 `eventBus`는 삭제·적용 경로에서 계속 쓰므로 유지한다.
2. `style-edit-dialog.ts`의 `onConfirm()`을 boolean으로 만들어 입력 검증, `updateStyle=false`,
   `updateStyleShapes=false`, 예외에서 `false`를 반환한다. 성공한 저장 때만 `onSave`를 호출한다.
3. `style-undo-routing.test.ts`에서 모달 중 `history-jumped` 소비가 없고 저장 실패가 modal을 유지하는
   구조를 검사한다.

## 안전성

`SnapshotCommand`는 operation 예외 시 before snapshot으로 rollback한다. 따라서 기존 스타일 메타 수정
뒤 모양 수정이 실패해도 InputHandler 경로에서는 부분 변경을 남기지 않는다. services 미주입 경로는
새 mutation routing을 만들지 않고, bool/예외 실패를 성공 후처리로 취급하지 않는다.
