# Task #945 Stage 1 — navigation keymap 현재 동작 재현 및 테스트 설계

## 1. 확인 범위

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/src/engine/input-handler-text.ts`
- `rhwp-studio/src/engine/cursor.ts`
- `rhwp-studio/e2e/helpers.mjs`
- `rhwp-studio/e2e/shift-end.test.mjs`
- `rhwp-studio/e2e/text-flow.test.mjs`
- `rhwp-studio/tests/*.test.ts`

구현 코드는 수정하지 않았다.

## 2. 현재 동작 재현 근거

### 2.1 Ctrl/Meta 선처리

`input-handler-keyboard.ts:837-840`:

```ts
if (e.ctrlKey || e.metaKey) {
  this.handleCtrlKey(e);
  return;
}
```

`Ctrl` 또는 `Meta`가 있으면 본문 `Arrow`/`Home`/`End` switch에 도달하지 않고 `handleCtrlKey()`로 빠진다.

### 2.2 Windows/Linux `Ctrl+←/→` 오동작 지점

`handleCtrlKey()` 내부:

- `input-handler-keyboard.ts:1137-1142`: `Ctrl/Meta+ArrowLeft` → `moveToLineStart()`
- `input-handler-keyboard.ts:1145-1150`: `Ctrl/Meta+ArrowRight` → `moveToLineEnd()`

따라서 Windows/Linux에서 기대되는 `Ctrl+←/→` 단어 이동이 아니라 줄 처음/끝 이동으로 처리된다.

### 2.3 macOS `Option+←/→` 경로는 이미 존재

`input-handler-keyboard.ts:937-949`:

- `Alt/Option+ArrowLeft/Right` → `cursor.moveToWordBoundary(-1 | 1)`
- `Shift`가 있으면 `setAnchor()`, 없으면 `clearSelection()`

즉 macOS 단어 이동 primitive는 이미 연결되어 있다. 다만 `Ctrl/Meta` 선처리보다 아래에 있으므로 `Command` 계열과는 별도다.

### 2.4 단어 이동 primitive

`cursor.ts:595-672`:

- 본문: `moveToWordBoundary()`
- 표 셀: `moveToWordBoundaryInCell()`
- 문단 경계에서는 `moveHorizontal()`로 이전/다음 문단 또는 셀 이동

단어 경계 알고리즘은 이미 한글/라틴/숫자/공백/문장부호를 구분한다.

### 2.5 Home/End 기존 공통 동작

`input-handler-keyboard.ts:992-1016`:

- `Home` → `moveToLineStart()`
- `End` → `moveToLineEnd()`
- `Shift` 선택 확장 유지

이 경로는 플랫폼 공통 기대 동작과 맞으므로 보존 대상이다.

## 3. 문단 이동 확인

`handleCtrlKey()` 내부:

- `Meta+ArrowUp/Down` 단독 → `moveToDocumentStart()/moveToDocumentEnd()`
- 그 외 `Ctrl+ArrowUp/Down` → `moveToParagraphBoundary(-1 | 1)`

이슈 #945 본문은 macOS `Command+↑/↓`를 한컴 기준 문단 이동 후보로 제시한다. 현재 코드는 주석상 "macOS 표준"을 우선해 문서 시작/끝으로 이동한다. Stage 2에서 다음 중 하나를 명시 결정해야 한다.

| 선택지 | 내용 | 위험 |
|--------|------|------|
| A | #945 기대대로 macOS `Command+↑/↓`를 문단 이동으로 변경 | 기존 macOS 표준 동작 회귀 가능 |
| B | #945의 `Command+↑/↓`는 별도 검토로 남기고 이번 PR은 단어/줄 이동만 정정 | 범위가 작고 안전 |

권장: **B**. 이슈 제목과 문제 관찰의 핵심은 단어/줄 이동 충돌이다.

## 4. 테스트 설계

### 4.1 E2E 테스트 추가 위치

권장 신규 파일:

- `rhwp-studio/e2e/navigation-shortcuts.test.mjs`

기존 helper:

- `createNewDocument()`
- `clickEditArea()`
- `typeText()`
- `moveCursorTo()`
- `getCursorPosition()`

### 4.2 테스트 케이스

#### Windows/Linux keymap

1. 새 문서 생성
2. `Hello World 123 가나다` 입력
3. 커서를 문단 끝으로 이동
4. `Ctrl+ArrowLeft`
5. 기대: 줄 시작이 아니라 직전 단어 경계로 이동
6. `Ctrl+Shift+ArrowLeft`
7. 기대: selection anchor 유지 및 단어 단위 선택

#### macOS keymap

1. 새 문서 생성
2. 동일 텍스트 입력
3. `Alt/Option+ArrowLeft/Right`
4. 기대: 단어 이동
5. `Meta+ArrowLeft/Right`
6. 기대: 줄 처음/끝 이동

Playwright/Puppeteer의 키 입력은 실행 OS 영향을 받으므로, platform 판별 helper는 테스트에서 override 가능한 형태가 좋다.

### 4.3 단위 테스트 후보

E2E가 무거우므로 keymap 결정 로직은 별도 순수 함수로 분리하면 `node --test`에서 빠르게 검증할 수 있다.

권장 파일:

- 구현: `rhwp-studio/src/engine/navigation-keymap.ts`
- 테스트: `rhwp-studio/tests/navigation-keymap.test.ts`

검증 대상:

- macOS + `Meta+ArrowLeft/Right` → line start/end
- macOS + `Alt+ArrowLeft/Right` → word backward/forward
- Windows/Linux + `Ctrl+ArrowLeft/Right` → word backward/forward
- Windows/Linux + `Home/End` → line start/end
- command shortcut(`Ctrl+S`, `Meta+S`, `Ctrl+C`)은 navigation helper가 처리하지 않음

## 5. 구현 계획 방향

1. `handleCtrlKey()` 전체를 플랫폼 분기로 찢지 않는다.
2. `onKeyDown()`에서 `Ctrl/Meta` 공통 처리 전에 navigation shortcut만 선처리한다.
3. 일반 command shortcut은 기존 `handleCtrlKey()`와 `shortcut-map.ts` 경로를 그대로 사용한다.
4. `Shift` 선택 확장 처리는 기존 패턴을 그대로 재사용한다.
5. `processPendingNav()`는 현재 `ctrlKey/metaKey` 값을 저장하지만 사용하지 않는다. IME 조합 중 `Ctrl/Meta/Alt+Arrow` 재생이 필요하면 Stage 2에서 동일 helper를 호출하도록 설계한다.

## 6. 검증 결과

`rhwp-studio` 기존 단위 테스트:

```text
npm test
tests 7
pass 7
fail 0
```

## 7. Stage 1 결론

- 결함 원인: `Ctrl/Meta` 공통 선처리 + `handleCtrlKey()`의 `ArrowLeft/Right` 줄 처음/끝 고정 처리.
- 단어 이동 primitive는 이미 있으므로 구현 범위는 keymap 라우팅 정리로 제한 가능.
- 테스트는 순수 keymap 단위 테스트 + 신규 E2E 1개 조합이 적절하다.
- Stage 2에서는 `navigation-keymap.ts` 도입 여부와 macOS `Command+↑/↓` 범위를 명시해야 한다.
