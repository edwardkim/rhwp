# Task M100-3414 Stage 1 — 마지막 모달 종료 뒤 편집기 포커스 복원

- Issue: #3414
- 브랜치: `local/task_m100_3414`
- 기준: `upstream/devel` (`8e81cbd996b66f873d21b74085a9dcee78ae3901`)
- 완료일: 2026-08-13

## 범위와 변경

이 단계는 #3414의 “활성이지만 미포커스” 구간 중 모달 종료 직후를 공통 경로에서 해소한다.

- `ModalDialog.hide()`는 overlay 제거와 기존 `afterClose` 후처리 뒤, 다른 `.modal-overlay`가
  남아 있지 않을 때만 `rhwp-modal-dialog-closed` 이벤트를 발행한다.
- `main.ts`는 이 이벤트에서 활성 `InputHandler`가 있을 때만 공개 `focus()`를 호출한다.
  따라서 hidden textarea가 포커스를 되찾고 기존 키보드 Ctrl+Z 경로가 다시 입력 handler로
  들어간다.
- 중첩 스타일 편집처럼 부모 모달이 남아 있는 자식 종료에서는 이벤트를 발행하지 않아 포커스가
  모달 밖으로 빠지지 않는다.
- 회귀 가드는 마지막 모달 조건, `afterClose` 순서, 활성 handler 조건, 초기화 순서를 고정한다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `node --test tests/issue-3414-modal-focus.test.ts` | 2 passed |
| `npx.cmd tsc --noEmit` | 통과 |
| `npm.cmd test` | 871 passed, 1 skipped, 0 failed |
| `npm.cmd run build` | 통과 |

로컬 Studio 개발 서버(`http://127.0.0.1:5173`)에서 새 문서를 만들고 브라우저 UI 스모크를
수행했다. 편집 용지 모달을 취소한 뒤 편집기 textarea가 활성화됐고, 입력한 문자열은 Ctrl+Z로
지워졌다. F6 스타일 모달에서 스타일 편집 자식 창을 열어 취소했을 때는 부모 overlay 1개가
남고 textarea가 활성화되지 않았다. 이어 부모 모달을 취소하자 overlay가 0개가 되고 textarea가
활성화됐으며, 새 문자열에 Ctrl+Z가 정상 동작했다. 해당 흐름 중 브라우저 오류 로그는 없었다.

## 범위 외와 후속

개체 선택 모드 Ctrl+C/X/V/Delete의 dispatcher 이관과 전역 fallback 자체의 리팩터링은 이 단계에
포함하지 않았다. 이번 변경은 모달 종료가 만드는 포커스 공백을 직접 제거하지만, #3414 전체 이슈를
close할 근거는 아니다.
