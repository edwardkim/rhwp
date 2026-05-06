# Task #598 Stage 4-3 완료보고서 — 각주 삭제 확인창/취소/Undo 검증

## 작업 개요

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **브랜치**: `feature/issue-598-footnote-marker-delete`
- **단계 범위**: Delete/Backspace 각주 삭제 확인창, 취소 동작, Undo 검증 보강

이슈 본문 요구사항 재점검 결과, 기존 PR 구현은 각주 삭제 핵심 경로는 동작했지만 동일 확인창과 취소/Undo 명시 검증이 부족했다. 본 단계에서 해당 범위를 보강했다.

## 구현 내용

- rhwp-studio 본문 각주 삭제 분기에서 기존 공통 `showConfirm()` 확인 다이얼로그를 호출하도록 연결했다.
- Delete/Fn+Delete 및 Backspace 양쪽 모두 같은 제목/메시지를 사용한다.
  - 제목: `각주 삭제`
  - 메시지: `각주를 삭제하시겠습니까?`
- 사용자가 취소하면 `deleteFootnote` 를 호출하지 않고 textarea 포커스만 복원한다.
- 사용자가 확인하면 기존 `SnapshotCommand` 기반 `deleteFootnote` 삭제 작업을 실행한다.
- 확인 후 textarea 포커스를 복원해 바로 Ctrl+Z 입력이 Undo 핸들러로 전달되도록 보정했다.

## 검증 결과

실행 명령:

```bash
cd rhwp-studio && npm run build
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/footnote-delete-confirm.test.mjs --mode=headless
git diff --check
```

결과:

- `npm run build`: 통과
- `footnote-delete-confirm.test.mjs`: 통과
  - Delete 경로 확인창 메시지 표시
  - 취소 후 각주 마커/본문/번호 유지
  - Backspace 경로 동일 확인창 메시지 표시
  - 확인 후 각주 마커/본문 삭제 및 후속 각주 재번호화
  - Ctrl+Z 후 각주 마커/본문/번호 복원
- `git diff --check`: 통과

참고:

- `npm run build` 에서 Vite chunk size warning 이 출력됐다. 기존 번들 크기 경고이며 빌드는 성공했다.
- 첫 E2E 시도에서 확인 후 포커스가 textarea로 복원되지 않아 Ctrl+Z가 입력 핸들러에 전달되지 않는 문제가 확인됐고, 포커스 복원으로 수정 후 재실행해 통과했다.

## 산출물

- `rhwp-studio/src/engine/input-handler-text.ts`
- `rhwp-studio/e2e/footnote-delete-confirm.test.mjs`
