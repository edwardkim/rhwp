# Task M100-3438 Stage 2 — 한컴 모달 undo 판정과 저장 실패 모달 유지

- Issue: #3438
- 브랜치: `local/task_m100_3438`
- 기준: `upstream/devel`의 #3414 병합 뒤 `d15b63eb46552ac61da24ad4a1b0c56208f71544`
- 완료일: 2026-08-13

## 한컴 Office 2022 오라클

새 빈 문서에 `oracle3438`을 입력해 상태 표시줄 `10글자`를 확인한 뒤 F6 스타일 대화상자를 열었다.
대화상자가 열린 상태에서 Ctrl+Z를 누르면 창은 열린 채로 남았으며, Escape로 닫은 뒤에도 상태
표시줄은 `10글자`였다. 이 테스트 문서는 저장하지 않고 닫았다.

따라서 한컴 Office 2022는 스타일 모달 중 undo를 실행하지 않는다. Studio의 모달 capture 정책도
같은 계약이므로, 스타일 모달 중 발생할 수 없다고 가정되는 `history-jumped` 구독과 목록 동기화,
해제 핸들을 제거했다.

## 구현

- `StyleEditDialog.onConfirm()`은 유효성 검사 실패, WASM `false`, 예외 때 `false`를 반환한다.
  `ModalDialog`는 이 결과에서 모달을 닫지 않는다.
- 입력 핸들러 snapshot은 저장 성공일 때만 위치를 반환한다. 실패는 no-op으로 남고, 모양 갱신의
  예외는 기존 snapshot rollback 경로로 전달된다.
- services 미주입 fallback도 동일한 `saved` 결과를 사용해 실패를 document-changed 성공으로
  알리지 않는다.

## 검증

| 항목 | 결과 |
| --- | --- |
| 한컴 Office 2022 F6 → Ctrl+Z | 모달 유지, Escape 뒤 `10글자` 유지 |
| `node --test tests/style-undo-routing.test.ts tests/undo-noop-skip.test.ts` | 12 passed |
| `npx.cmd tsc --noEmit` | 통과 |
| `npm.cmd test` | 875 passed, 1 skipped, 0 failed |
| `npm.cmd run build` | 통과 (기존 Vite 경고만 출력) |
| 로컬 Studio fixture F6 → Ctrl+Z | 스타일 모달 유지 |
| 로컬 Studio 스타일 모달 닫기 → Ctrl+Z | 입력 textarea 포커스 유지, console error/warning 없음 |
| `git diff --check` | 통과 |

로컬 Studio 검증은 `rhwp-studio/public/samples/shift-return.hwp`를 URL 입력 경로로 열어 수행했다.
원본 fixture는 저장하거나 변경하지 않았다.

## 범위 외

- `ModalDialog`의 Ctrl+Z capture 정책을 전역적으로 바꾸는 일
- 한컴 Office 2022 결과를 다른 한컴 버전으로 일반화하는 일
- remote push, PR 생성, issue close
