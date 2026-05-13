# Task M100 #871 최종 보고서

## 이슈

- GitHub Issue: #871
- 브랜치: `local/task871`
- 증상: macOS rhwp-studio web에서 내부 복사 후 외부 앱에서 새 값을 복사해도 붙여넣기 시 stale 내부 클립보드 값이 삽입됨

## 원인

`rhwp-studio/src/engine/input-handler-keyboard.ts`의 `onPaste()`가 paste 이벤트의 시스템 클립보드 데이터보다 `wasm.hasInternalClipboard()`를 먼저 신뢰했다. 그 결과 rhwp-studio 내부 클립보드가 남아 있으면 외부 앱에서 새로 복사한 `text/plain` 또는 `text/html` 값이 무시되었다.

Stage 4 검증 중 답안지 상단 `성명` 칸에서 추가 문제가 확인되었다. 해당 위치는 중첩 표이며, 외부 클립보드가 `text/html`을 함께 제공할 때 기존 `pasteHtmlInCell()` 경로가 전체 `cellPath`를 표현하지 못해 바깥 표 문단에 삽입될 수 있었다.

## 수정 요약

### 1. 내부 클립보드 marker 도입

rhwp-studio 내부 복사 HTML에 다음 marker를 삽입한다.

```html
<!--rhwp-studio-clipboard:{token}-->
```

붙여넣기 시 `text/html`에 현재 token과 일치하는 marker가 있을 때만 내부 `pasteInternal` 또는 `pasteControl` 경로를 사용한다. marker가 없으면 외부 클립보드로 보고 이미지, HTML, plain text 순서로 처리한다.

### 2. 복사 경로 보강

- 텍스트 선택 복사: `exportSelectionHtml()` 결과에 marker 삽입
- 셀 내부 텍스트 복사: `exportSelectionInCellHtml()` 결과에 marker 삽입
- 그림/도형/표 객체 복사: `exportControlHtml()` 결과에 marker 삽입
- 메뉴/도구 상자 `performCopy()` 경로도 동일하게 보강

### 3. 붙이기 커맨드 경로 정리

`edit:paste` 커맨드는 직접 `document.execCommand('paste')`를 호출하지 않고 `InputHandler.performPaste()`를 호출하도록 변경했다. 커맨드 실행 시 숨겨진 편집 textarea에 포커스를 먼저 맞춘다.

### 4. PR #865 코드 반영

답안지 `성명` 칸 같은 중첩 표 hit-test/edit path 문제를 해결하기 위해 PR #865의 코드 변경을 Task #871 브랜치에 반영했다.

주요 효과:

- 중첩 표 hit-test 결과가 루트 기준 전체 `cellPath`를 보존
- 중첩 표 커서 사각형 계산 보정
- 입력 루프의 page overlay image 조회 비용 축소

### 5. 중첩 표 외부 HTML fallback

중첩 표 내부에서 외부 `text/html`과 `text/plain`이 함께 들어오면, 현재는 `pasteHtmlInCell()` 대신 plain text 삽입 경로를 사용한다. 이 경로는 `InsertTextCommand`를 통해 `insertTextInCellByPath()`를 호출하므로 중첩 표 위치가 정확히 보존된다.

## 변경 파일

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/command/commands/edit.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/src/view/page-renderer.ts`
- `rhwp-studio/e2e/task-871-clipboard-priority.test.mjs`
- `rhwp-studio/package.json`
- `src/document_core/queries/cursor_rect.rs`
- `src/document_core/queries/rendering.rs`
- `src/wasm_api.rs`
- `tests/issue_850_answer_sheet_name_hit_test.rs`

## 검증

```bash
cd rhwp-studio
npm run build
```

- 성공
- 기존 Vite chunk size warning만 발생

```bash
node --check e2e/task-871-clipboard-priority.test.mjs
```

- 성공

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
```

- 3 passed

```bash
docker-compose --env-file .env.docker run --rm wasm
```

- 성공

수동 검증:

- 일반 본문 텍스트: 외부 앱에서 복사한 값이 정상 붙여넣기됨
- `exam_social.hwp` 답안지 상단 `성명` 칸: 외부 텍스트가 중첩 표 셀 내부에 정상 붙여넣기됨
- 작업지시자가 동일 환경에서 정상 동작 확인

## 남은 제약

- 중첩 표 내부 외부 HTML 붙여넣기는 서식 보존 없이 plain text로 fallback한다.
- 브라우저 보안 정책상 메뉴/도구 상자 붙이기는 사용자가 직접 발생시킨 paste 이벤트가 아니면 제한될 수 있다.
- macOS headless Chrome의 copy 이벤트 제한으로 신규 e2e 실실행은 현재 sandbox에서 완료하지 못했다.

## 결론

Issue #871의 핵심 문제인 stale 내부 클립보드 우선 사용은 marker exact match 방식으로 해결했다. 추가로 검증 중 발견된 답안지 중첩 표 붙여넣기 문제는 PR #865 코드 반영과 중첩 표 plain text fallback으로 해결했다.

Issue close는 작업지시자 승인 후 별도로 진행한다.
