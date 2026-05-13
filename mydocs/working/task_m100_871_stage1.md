# Task #871 Stage 1 완료 보고서 — 클립보드 경로 진단 및 marker 설계

## 1. 범위

Stage 1에서는 소스 코드를 수정하지 않고, rhwp-studio의 copy/paste 경로와 내부 클립보드 오판 원인을 확인했다.

대상 파일:

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `src/document_core/commands/clipboard.rs`
- `src/document_core/commands/html_import.rs`

## 2. 진단 결과

### 2.1 핵심 원인 확정

`input-handler-keyboard.ts::onPaste()`는 외부 시스템 클립보드 데이터를 읽기 전에 내부 WASM 클립보드를 먼저 검사한다.

현재 흐름:

1. `onPaste(e)` 진입
2. `this.wasm.hasInternalClipboard()` 검사
3. 내부 클립보드가 있으면 `pasteControl` 또는 `pasteInternal` 실행
4. 즉시 `return`
5. 외부 클립보드 이미지, `text/html`, `text/plain` 처리는 도달하지 않음

따라서 rhwp-studio 내부에서 한 번 복사한 뒤 외부 앱에서 새 값을 복사해도, 내부 클립보드가 남아 있으면 외부 값이 무시된다.

### 2.2 내부 복사 경로

텍스트 선택 복사:

- `onCopy()`에서 `wasm.copySelection()` 또는 `wasm.copySelectionInCell()` 호출
- WASM 내부 클립보드에 서식 포함 paragraph 저장
- 시스템 클립보드에는 `text/plain`과 `text/html`을 설정
- HTML은 `exportSelectionHtml()` 또는 `exportSelectionInCellHtml()` 결과

그림/도형 객체 선택 복사:

- 일부 경로는 `copyControl()` 후 `exportControlHtml()`을 호출
- 이미지 객체는 `writeImageToClipboard()`로 `text/plain`, `text/html`, `image/png`를 함께 기록
- 도형은 현재 `navigator.clipboard.writeText()` fallback 중심이라 HTML marker 삽입 경로 보강 필요

표 객체 선택 복사:

- 현재 Ctrl+C 직접 처리와 `performCopy()` 모두 `copyControl()` 후 `navigator.clipboard.writeText()`만 호출
- 즉, 내부 클립보드는 표 컨트롤을 보유하지만 시스템 클립보드는 `[표]` plain text만 가진다.
- Stage 2에서 `exportControlHtml()` 기반 `text/html` 기록으로 보강해야 내부 표 붙여넣기 보존과 marker 판별이 동시에 가능하다.

### 2.3 HTML 내보내기와 가져오기 구조

WASM HTML 내보내기:

- `export_selection_html_native()`는 `<html><body>` 뒤 `<!--StartFragment-->`와 `<!--EndFragment-->`를 포함한다.
- `export_selection_in_cell_html_native()`도 같은 구조다.
- `export_control_html_native()`도 control HTML을 같은 fragment 구조에 넣는다.

WASM HTML 가져오기:

- `parse_html_to_paragraphs()`는 `<!--StartFragment-->`와 `<!--EndFragment-->` 사이만 추출한다.
- marker를 `StartFragment` 앞에 두면 pasteHtml 파서에는 들어가지 않는다.
- 따라서 rhwp marker는 HTML 전체에서는 판별 가능하지만 HWP 문서 내용으로 삽입되지 않도록 `StartFragment` 앞에 두는 방식이 안전하다.

## 3. marker 설계 확정

### 3.1 marker 포맷

Stage 2에서는 HTML에 다음 형태의 marker를 삽입한다.

```html
<!--rhwp-studio-clipboard:{token}-->
```

삽입 위치:

```html
<html><body>
<!--rhwp-studio-clipboard:{token}-->
<!--StartFragment-->
...
<!--EndFragment-->
</body></html>
```

이 위치를 선택한 이유:

- `pasteHtml` 파서는 `StartFragment` 이후만 문서 내용으로 처리하므로 marker가 본문으로 들어가지 않는다.
- 외부 HTML과 충돌 가능성이 낮다.
- JS에서 정규식 또는 문자열 검색으로 쉽게 판별할 수 있다.

### 3.2 token 소유권

copy 성공 시 JS 쪽에서 token을 생성하고 InputHandler 인스턴스에 저장한다.

예상 필드:

```typescript
this._rhwpClipboardToken = token;
```

paste 시 다음 조건을 모두 만족할 때만 내부 클립보드를 사용한다.

1. `e.clipboardData.getData('text/html')`에 rhwp marker가 있다.
2. marker token이 현재 `this._rhwpClipboardToken`과 일치한다.
3. `this.wasm.hasInternalClipboard()`가 true다.

조건 하나라도 실패하면 외부 클립보드로 처리한다.

## 4. Stage 2 수정 범위 확정

### 4.1 `input-handler-keyboard.ts`

추가할 헬퍼 후보:

- `createRhwpClipboardToken()`
- `markRhwpClipboardHtml(html, token)`
- `readRhwpClipboardToken(html)`
- `hasCurrentRhwpClipboardMarker(this, html)`
- `writeHtmlToClipboard(text, html)`

수정할 경로:

- 텍스트 선택 `onCopy()`:
  - `copySelection*()` 성공 후 token 생성
  - `exportSelection*Html()` 결과에 marker 삽입
  - `text/html`에 marker 포함 HTML 기록
- 그림/도형 객체 Ctrl+C/Ctrl+X:
  - `copyControl()` 성공 후 token 생성
  - `exportControlHtml()` 결과에 marker 삽입
  - 이미지 객체는 기존 `image/png` 포함 경로 유지
  - 도형은 HTML이 없으면 plain text fallback 유지
- 표 객체 Ctrl+C/Ctrl+X:
  - 현재 `writeText()`만 쓰는 경로를 `exportControlHtml()` + HTML marker 경로로 보강
- `onPaste()`:
  - 먼저 `e.clipboardData`에서 `items`, `text/html`, `text/plain` 수집
  - rhwp marker exact match인 경우에만 `pasteInternal` 또는 `pasteControl`
  - marker가 없으면 이미지, HTML, plain text 순서로 외부 붙여넣기

### 4.2 `input-handler.ts`

메뉴/도구 상자 복사 경로:

- `performCopy()`의 그림/표 객체 경로도 키보드 경로와 같은 helper를 사용하도록 정리한다.
- 텍스트 선택은 `document.execCommand('copy')`를 통해 `onCopy()`로 들어가므로 marker 삽입 대상이다.

메뉴/도구 상자 붙이기:

- 현재 `edit:paste`는 `document.execCommand('paste')`를 호출한다.
- 브라우저 보안 정책상 직접 paste 호출은 제한될 수 있다.
- Stage 3에서 실제 동작 확인 후 유지 또는 제한 사항 문서화로 정리한다.

### 4.3 `wasm-bridge.ts`

`src/wasm_api.rs`에는 `clearClipboard()` 바인딩이 이미 존재한다. TypeScript `WasmBridge`에는 래퍼가 없다.

Stage 2에서 꼭 필요하지 않으면 추가하지 않는다. marker exact match 방식이면 stale 내부 클립보드를 지우지 않아도 외부 붙여넣기를 우선할 수 있기 때문이다.

## 5. 예상 정정 후 흐름

```text
onPaste(e)
  clipboard = e.clipboardData
  html = clipboard.getData('text/html')
  text = clipboard.getData('text/plain')

  if html has current rhwp marker && wasm.hasInternalClipboard()
    if wasm.clipboardHasControl() && body context
      pasteControl()
    else
      pasteInternal()
    return

  if clipboard has image file
    pasteImageFile()
    return

  if html exists
    pasteHtml()
    return

  if text exists
    InsertTextCommand / SplitParagraphCommand
```

## 6. 검증 계획

Stage 2 이후 수동 검증:

1. rhwp-studio 내부 텍스트 복사 후 바로 붙여넣기 → 내부 서식 보존
2. rhwp-studio 내부 텍스트 복사 후 외부 앱 텍스트 복사 후 붙여넣기 → 외부 텍스트 우선
3. rhwp-studio 내부 표 복사 후 바로 붙여넣기 → 표 컨트롤 보존
4. 외부 HTML/표 붙여넣기 → `pasteHtml` 경로 유지
5. 외부 plain text 다중 줄 붙여넣기 → 줄바꿈 유지
6. 이미지 클립보드 붙여넣기 → 이미지 삽입 유지

Stage 3 자동 검증 후보:

- `rhwp-studio/e2e/copy-paste.test.mjs` 확장 또는 신규 테스트 추가
- Playwright clipboard 주입으로 내부 복사 후 외부 텍스트 우선 붙여넣기 확인

## 7. 판정

Stage 1의 결론은 다음과 같다.

- 결함 원인은 `onPaste()`의 내부 클립보드 선검사로 확정한다.
- 외부 클립보드 우선 처리는 단순히 순서만 바꾸면 내부 서식 보존 붙여넣기가 깨질 수 있으므로, rhwp marker exact match 방식을 채택한다.
- Stage 2에서는 `input-handler-keyboard.ts` 중심으로 copy marker 삽입과 paste routing을 수정한다.
- 표 객체 복사 경로는 현재 `writeText()`만 사용하므로 Stage 2 수정 범위에 포함한다.

## 8. 다음 승인 요청

Stage 2에서 소스 수정을 시작한다. 수정 범위는 다음으로 제한한다.

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- 필요 시 `rhwp-studio/src/core/wasm-bridge.ts`

작업지시자 승인 후 Stage 2를 진행한다.
