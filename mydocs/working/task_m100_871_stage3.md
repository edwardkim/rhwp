# Task M100 #871 Stage 3 완료 보고서

## 개요

macOS web 환경의 외부 클립보드 우선순위 회귀를 자동 검증할 수 있도록 전용 e2e 시나리오를 추가하고, 메뉴/도구 상자 붙이기 커맨드가 입력 핸들러 경로를 사용하도록 정리했다.

## 변경 내용

### 1. Task 871 전용 회귀 e2e 추가

- 파일: `rhwp-studio/e2e/task-871-clipboard-priority.test.mjs`
- npm script: `npm run e2e:clipboard-priority`
- 검증 시나리오:
  - rhwp-studio marker가 있는 HTML 붙여넣기는 내부 클립보드를 사용한다.
  - 내부 클립보드가 남아 있어도 marker 없는 외부 plain text는 외부 값을 붙여넣는다.
  - 내부 클립보드가 남아 있어도 marker 없는 외부 HTML은 `pasteHtml` 경로를 사용한다.

기존 `copy-paste.test.mjs`는 macOS headless Chrome에서 키보드 단축키 기반 copy 이벤트가 안정적으로 발생하지 않는 제약이 있었다. 신규 테스트는 `document.execCommand('copy')`로 앱의 `onCopy` 경로를 통과시킨 뒤, `ClipboardEvent('paste')`에 명시적인 `DataTransfer`를 넣어 클립보드 우선순위만 분리 검증한다.

### 2. 메뉴/도구 상자 붙이기 커맨드 경로 정리

- `InputHandler.performPaste()`를 추가했다.
- `edit:paste` 커맨드는 직접 `document.execCommand('paste')`를 호출하지 않고 `services.getInputHandler()?.performPaste()`를 호출한다.
- 커맨드 실행 시 숨겨진 편집 textarea에 포커스를 먼저 맞추므로 키보드 붙이기 경로와 이벤트 대상이 일관된다.

브라우저 보안 정책상 사용자가 직접 발생시킨 paste 이벤트가 아니면 `execCommand('paste')`는 여전히 거부될 수 있다. 이번 변경은 커맨드 경로의 포커스/위임 일관성을 맞추는 보강이며, 외부 클립보드 우선순위 자체는 Stage 2의 marker 판별 로직이 담당한다.

## 검증

### 통과

```bash
cd rhwp-studio
npm run build
```

- `tsc && vite build` 성공
- 기존 Vite chunk size warning만 발생

```bash
cd rhwp-studio
node --check e2e/task-871-clipboard-priority.test.mjs
```

- 신규 e2e 파일 문법 검증 성공

### 브라우저 로드 확인

- Vite dev server: `http://127.0.0.1:7700/`
- Codex 인앱 Browser에서 `rhwp-studio` 로드 확인
- console `error`/`warning` 없음

### 미완료 검증

```bash
cd rhwp-studio
env CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  VITE_URL="http://127.0.0.1:7700" \
  npm run e2e:clipboard-priority -- --mode=headless
```

- sandbox 내부에서는 headless Chrome 프로세스 실행 실패
- sandbox 외부 재실행 승인은 자동 심사에서 거부되어 완료하지 못함
- 테스트 코드는 추가되었으므로 사용자의 로컬 승인 환경 또는 CDP host 모드에서 재실행 가능

## 다음 단계

Stage 4에서 최종 검증 결과를 정리하고, 미완료 e2e 실행 제약을 포함한 최종 보고서를 작성한다.
