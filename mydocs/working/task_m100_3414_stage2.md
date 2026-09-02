# Task M100-3414 Stage 2 — 포커스 공백 단축키·개체 명령 단일화

- Issue: #3414
- 브랜치: `local/task_m100_3414`
- 기준: `upstream/devel` (`8e81cbd996b66f873d21b74085a9dcee78ae3901`)
- 완료일: 2026-08-13

## 변경

- `main.ts`의 global shortcut fallback은 textarea가 아닌 요소에 포커스가 남은 활성 편집기에서
  `edit:undo`·`edit:redo`만 기존 dispatcher로 보낸다. textarea 이벤트는 InputHandler가 계속
  단독 소유하므로 이중 실행되지 않는다.
- 그림·표 개체 선택 keydown의 Ctrl+C, Ctrl+X, Delete/Backspace는 `edit:copy`, `edit:cut`,
  `edit:delete` command로 모았다. clipboard `copy`·`cut` 이벤트와 canonical `perform*` 구현은
  유지해 메뉴·컨텍스트 메뉴·키보드가 같은 실행 경로를 쓴다.
- Ctrl+V는 selection을 해제한 뒤 native `paste` 이벤트를 그대로 통과시킨다. 이 이벤트의
  `clipboardData`는 사용자 제스처에서만 신뢰할 수 있으므로 `execCommand('paste')`로 바꾸지 않는다.
- canonical 표 cut은 중첩 표에서 복사·삭제를 하지 않고, 중첩 표 delete 뒤 caret을 갱신해 종전
  keydown 안전 계약과 맞췄다.

## 검증

| 명령 또는 절차 | 결과 |
| --- | --- |
| `node --test tests/issue-3414-modal-focus.test.ts tests/issue-3414-shortcut-routing.test.ts` | 5 passed |
| `npx.cmd tsc --noEmit` | 통과 |
| `npm.cmd test` | 874 passed, 1 skipped, 0 failed |
| `npm.cmd run build` | 통과 |
| 로컬 Studio 브라우저 UI | 새 문서에 `fallback-undo-check` 입력 → `폭 맞춤` 버튼으로 textarea 밖 포커스 이동 → Ctrl+Z 후 빈 페이지 확인. 브라우저 오류 로그 없음. |

## 후속

이번 stage는 #3414가 지적한 Ctrl+Z 포커스 공백과 개체 키보드 명령의 중복 경로를 해소한다.
원격 PR·이슈 close는 아직 수행하지 않았다.
