# Task M100-3414 Stage 2 — 포커스 공백 단축키와 개체 명령 단일화

- 이슈: https://github.com/edwardkim/rhwp/issues/3414
- 작성일: 2026-08-13
- 브랜치: `local/task_m100_3414`

## 목표

Stage 1의 마지막 모달 포커스 복원 뒤에도 남을 수 있는 body 포커스 구간에서는 Ctrl+Z/Y를
dispatcher로 전달한다. 또한 그림·표 개체 선택 상태의 Ctrl+C, Ctrl+X, Delete/Backspace가
`InputHandler`의 이미 존재하는 `edit:copy`, `edit:cut`, `edit:delete` 명령을 사용하게 해
키보드 분기의 중복된 실행·undo 경로를 없앤다.

## 경계

- Ctrl+V는 브라우저가 사용자 제스처에서 제공하는 `paste` 이벤트와 `ClipboardEvent.clipboardData`를
  통해 실제 내용을 전달받는다. 따라서 선택 해제 뒤 native paste 이벤트를 유지하며,
  `document.execCommand('paste')`로 대체하지 않는다.
- Escape, Enter, 방향키, 개체 이동·크기 조절, 셀 선택·중첩 표의 동작은 변경하지 않는다.
- 한컴 2022 오라클이 필요한 #3438·#3416·#3351은 이 stage 범위가 아니다.

## 구현·검증

1. global shortcut fallback에 활성 handler의 undo/redo 명령만 추가한다. textarea가 이벤트 대상이면
   기존 handler가 먼저 소유하므로 이 fallback은 작동하지 않는다.
2. 그림·표 선택 keydown의 Ctrl+C/X와 Delete/Backspace를 dispatcher로 라우팅한다. canonical
   `performCut`에 중첩 표 안전 가드를 맞춘다.
3. 소스 가드 테스트를 추가하고 focused test, TypeScript, 전체 Studio test, 로컬 브라우저 Ctrl+Z
   스모크를 순차 검증한다.
