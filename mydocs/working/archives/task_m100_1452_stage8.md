# Task M100 #1452 Stage 8 기록

- 이슈: #1452 `rhwp-studio: 그림 삽입/배치 속성 및 Shift+Tab 내어쓰기 개선`
- 브랜치: `local/task_m100_1452`
- 작성일: 2026-06-21
- 선행 커밋:
  - `c84deb3a task 1452: TAC 그림 Enter 줄 유지 보정`

## 1. 배경

파일 메뉴에서 `열기`를 누른 뒤 파일 선택 다이얼로그를 `Esc`로 닫으면, 닫히자마자 다시 `열기`
파일 선택 다이얼로그가 열리는 버그가 보고됐다.

## 2. 초기 확인

- 파일 메뉴 항목은 `rhwp-studio/src/ui/menu-bar.ts`에서 클릭 이벤트로 `file:open` 명령을 dispatch한다.
- `file:open`은 `rhwp-studio/src/command/commands/file.ts`에서 File System Access API를 먼저 시도하고,
  지원하지 않으면 숨김 `<input type="file">`의 `click()`으로 fallback한다.
- `pickOpenFileHandle()`은 File System Access API 미지원과 사용자의 native picker 취소를 모두 `null`로 반환한다.
- 따라서 Chrome 계열에서 `showOpenFilePicker()`가 `Esc` 취소로 `AbortError`를 냈을 때도 `file:open`이 숨김
  `<input type="file">` fallback을 실행해 파일 선택창이 곧바로 다시 열린다.

## 3. 개선 목표

- File System Access API 지원 여부와 native picker 취소 결과를 구분한다.
- native picker가 지원되는 환경에서 사용자가 취소한 경우 숨김 file input fallback을 열지 않는다.
- 사용자가 취소한 뒤에는 다음 명시적 `열기` 클릭 또는 `Ctrl+O`는 정상 동작해야 한다.
- File System Access API를 지원하지 않는 브라우저에서는 기존 숨김 file input fallback을 유지한다.

## 4. 변경 내용

- `rhwp-studio/src/command/file-system-access.ts`
  - `canUseOpenFilePicker()` helper를 추가해 native open picker 지원 여부를 명시적으로 판별한다.
  - `pickOpenFileHandle()`은 helper를 사용하되, 취소 시 기존처럼 `null`을 반환한다.
- `rhwp-studio/src/command/commands/file.ts`
  - `file:open`에서 native picker가 사용 가능한 상태였으면 `null` 반환을 사용자 취소로 보고 즉시 종료한다.
  - native picker가 없는 환경에서만 숨김 file input fallback을 실행한다.
- `rhwp-studio/tests/file-system-access.test.ts`
  - native picker 지원 여부 판별 테스트와 `AbortError` 취소 반환 테스트를 추가했다.

## 5. 검증 결과

- `cd rhwp-studio && npx tsc --noEmit` 통과.
- `cd rhwp-studio && node --test tests/file-system-access.test.ts` 통과.
- `cd rhwp-studio && npm test` 통과. 89개 테스트 통과.
- 작업지시자 수동 시각 검증 완료.
