# Task #871 Stage 2 완료 보고서 — 외부 클립보드 우선순위 정정

## 1. 범위

Stage 2에서는 승인된 범위 안에서 rhwp-studio의 내부 클립보드 marker와 붙여넣기 routing을 수정했다.

수정 파일:

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/src/engine/input-handler.ts`

`rhwp-studio/src/core/wasm-bridge.ts`는 수정하지 않았다. `clearClipboard()` 래퍼 없이도 marker exact match 방식으로 stale 내부 클립보드를 회피할 수 있기 때문이다.

## 2. 구현 내용

### 2.1 rhwp 내부 복사 marker 추가

`input-handler-keyboard.ts`에 rhwp-studio 내부 복사 식별용 helper를 추가했다.

- `createRhwpClipboardToken()`
- `prepareRhwpInternalClipboardHtml()`
- `writeTextHtmlToClipboard()`
- 내부 marker 판별 함수

marker 형식:

```html
<!--rhwp-studio-clipboard:{token}-->
```

marker는 `<!--StartFragment-->` 앞에 삽입한다. 따라서 JS는 HTML 전체에서 marker를 판별할 수 있고, WASM `pasteHtml` 파서는 fragment 내부만 문서 내용으로 처리하므로 marker가 본문에 삽입되지 않는다.

### 2.2 텍스트 선택 복사 경로 보강

텍스트 선택 `onCopy()`에서:

1. `wasm.copySelection()` 또는 `wasm.copySelectionInCell()`로 내부 클립보드 저장
2. `exportSelectionHtml()` 또는 `exportSelectionInCellHtml()`로 HTML 생성
3. HTML에 rhwp marker 삽입
4. 시스템 클립보드의 `text/html`에 marker 포함 HTML 기록

HTML 내보내기가 실패해도 plain text 기반 fallback HTML을 만들어 marker를 유지한다.

### 2.3 객체/표 복사 경로 보강

그림/도형 객체 경로:

- `copyControl()` 후 `exportControlHtml()` 결과에 marker를 삽입한다.
- 이미지 객체는 기존 `image/png` 포함 클립보드 기록을 유지한다.
- HTML이 없는 도형/수식 계열도 fallback HTML에 marker를 넣어 내부 `pasteControl` 판별이 가능하게 했다.

표 객체 경로:

- 기존에는 `copyControl()` 후 `navigator.clipboard.writeText()`만 호출했다.
- Stage 2에서 `exportControlHtml()` + marker 포함 `text/html` 기록으로 보강했다.
- 키보드 직접 처리와 메뉴/도구 상자 `performCopy()` 양쪽 모두 반영했다.

### 2.4 붙여넣기 routing 변경

기존:

```text
hasInternalClipboard() true
  -> pasteInternal / pasteControl
  -> return
외부 clipboardData 처리
```

변경:

```text
clipboardData에서 text/html, text/plain, items 수집

if 현재 rhwp marker와 내부 클립보드가 모두 있음
  -> pasteInternal / pasteControl
  -> return

if 이미지 파일 있음
  -> pasteImageFile
  -> return

if 외부 HTML 있음
  -> pasteHtml / pasteHtmlInCell
  -> return

if 외부 plain text 있음
  -> InsertTextCommand / SplitParagraphCommand
```

이제 내부 클립보드가 남아 있어도 시스템 클립보드에 현재 rhwp marker가 없으면 외부 값을 우선한다.

## 3. 검증

### 3.1 TypeScript/Vite 빌드

명령:

```bash
cd rhwp-studio
npm run build
```

결과:

- `tsc` 통과
- `vite build` 통과
- chunk size warning은 기존 빌드 경고이며 실패 아님

### 3.2 브라우저 로딩 확인

Codex in-app Browser로 `http://127.0.0.1:7700/` 로딩 확인.

확인 결과:

- title: `rhwp-studio`
- 메뉴바/도구 상자/서식 도구 모음 DOM 표시 확인
- console error/warn 없음

### 3.3 paste routing 직접 검증

headless Chrome에서 앱을 로드한 뒤 paste 이벤트 데이터를 직접 주입해 marker 경로와 외부 경로를 확인했다.

검증 1: 내부 marker가 있는 paste

- 입력: `abcdefg`
- 내부 복사 수행: `document.execCommand("copy")`
- marker 포함 `text/html` paste 이벤트 주입
- 결과: `abcdefgabcdefg`

검증 2: 내부 클립보드가 남아 있는 상태에서 외부 plain text paste

- 입력: `abcdefg`
- 내부 복사 수행: `document.execCommand("copy")`
- marker 없는 `text/plain=OUTSIDE` paste 이벤트 주입
- 결과: `abcdefgOUTSIDE`

실행 결과:

```json
{"internal":"abcdefgabcdefg","external":"abcdefgOUTSIDE"}
```

### 3.4 기존 e2e 참고

`rhwp-studio/e2e/copy-paste.test.mjs --mode=headless`는 macOS headless Chrome에서 `Control+C`가 copy 이벤트를 발생시키지 않아 실패했다.

확인 내용:

- `Control+A`는 앱 selection을 만들었다.
- `Control+C`와 `Meta+C` 모두 headless 환경에서 브라우저 copy 이벤트를 발생시키지 않았다.
- `document.execCommand("copy")`는 copy 이벤트를 발생시키고 내부 클립보드와 marker token을 정상 생성했다.

따라서 Stage 2에서는 기존 e2e 실패를 코드 회귀로 보지 않고, Stage 3에서 macOS/headless 호환 가능한 회귀 테스트 방식으로 정리한다.

## 4. 영향 분석

| 영역 | 결과 |
|------|------|
| 내부 텍스트 복사/붙여넣기 | marker match 시 내부 `pasteInternal` 경로 유지 |
| 외부 plain text 붙여넣기 | stale 내부 클립보드가 있어도 외부 텍스트 우선 |
| 표 객체 복사 | `text/html` 기록 추가로 내부 표 paste 판별 가능 |
| 이미지 객체 복사 | 기존 `image/png` 기록 유지, HTML marker 추가 |
| WASM/Rust 엔진 | 수정 없음 |

## 5. 남은 작업

Stage 3에서 다음을 진행한다.

1. 기존 `copy-paste.test.mjs` 또는 신규 e2e를 macOS/headless 환경에서도 신뢰 가능하게 정리한다.
2. 메뉴/도구 상자 붙이기(`document.execCommand("paste")`)의 실제 브라우저 제한을 확인하고, 유지/보완/문서화 여부를 판정한다.
3. 외부 HTML/표 붙여넣기와 이미지 붙여넣기 회귀 범위를 추가 확인한다.

## 6. 다음 승인 요청

Stage 3 진행 승인을 요청한다. Stage 3에서는 e2e 테스트 보강과 메뉴/도구 상자 붙이기 경로 판정을 수행한다.
