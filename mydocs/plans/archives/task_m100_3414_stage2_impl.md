# 구현 계획서 — Task M100-3414 Stage 2

- 수행 계획서: `mydocs/plans/task_m100_3414_stage2.md`
- 작성일: 2026-08-13

## 수정 예상 지점

- `rhwp-studio/src/main.ts`: non-textarea 활성 편집 상태에서 undo/redo만 dispatcher fallback으로 전달.
- `rhwp-studio/src/engine/input-handler-keyboard.ts`: 그림·표 선택 Ctrl+C/X와 Delete/Backspace의
  직접 구현을 `edit:*` dispatcher 호출로 치환. Ctrl+V의 native event 전달은 유지.
- `rhwp-studio/src/engine/input-handler.ts`: `performCut()`의 표 분기가 keyboard 경로와 같은
  중첩 표 안전 계약을 지키게 한다.
- `rhwp-studio/tests/issue-3414-shortcut-routing.test.ts`: source-level ownership·paste 보존·global
  undo/redo fallback 회귀 가드.

## 수용 기준

- body 포커스 상태의 활성 문서에서 Ctrl+Z/Y가 dispatcher에 도달한다.
- 그림·표 개체 선택 Ctrl+C/X와 Delete/Backspace가 하나의 command execution 경로를 사용한다.
- Ctrl+V는 native paste 이벤트를 차단하지 않는다.
