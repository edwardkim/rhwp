# Task #945 Stage 4 — keyboard/IME 연결 + E2E 보강

## 1. 구현 범위

Stage 3에서 만든 플랫폼별 keymap helper를 실제 키보드 입력 경로와 IME pending navigation 경로에 연결했다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | `Ctrl/Meta` 공통 처리 전에 navigation shortcut 선처리 |
| `rhwp-studio/src/engine/input-handler-text.ts` | IME 조합 종료 후 pending navigation에 동일 keymap 적용 |
| `rhwp-studio/src/engine/navigation-keymap.ts` | Windows/Linux `Alt+←/→` suppress helper 추가 |
| `rhwp-studio/tests/navigation-keymap.test.ts` | suppress 동작 단위 테스트 추가 |
| `rhwp-studio/e2e/navigation-shortcuts.test.mjs` | 플랫폼별 navigation E2E 신규 추가 |
| `rhwp-studio/src/engine/selection-renderer.ts` | 기존 `shift-end.test.mjs`가 찾는 `.selection-highlight` 클래스 부여 |

## 3. 동작 정리

### macOS

- `Option+←/→`: 단어 이동
- `Option+Shift+←/→`: 단어 선택
- `Command+←/→`: 줄 처음/끝
- `Command+Shift+←/→`: 줄 처음/끝 선택
- `Command+↑/↓`: 이번 범위에서 변경하지 않음

### Windows/Linux

- `Ctrl+←/→`: 단어 이동
- `Ctrl+Shift+←/→`: 단어 선택
- `Ctrl+↑/↓`: 문단 이동
- `Alt+←/→`: 단어 이동으로 처리하지 않음

### 공통

- `Home/End`: 줄 처음/끝
- `Shift+Home/End`: 줄 처음/끝 선택
- `Ctrl+S`, `Meta+S`, `Ctrl+C`, `Meta+C` 등 command shortcut은 기존 `handleCtrlKey()` / `shortcut-map.ts` 경로 유지

## 4. 구현 상세

1. `handleNavigationShortcut()`을 `Ctrl/Meta` 공통 처리보다 앞에 배치했다.
2. `getNavigationAction()`이 action을 반환하면 기존 cursor primitive를 호출한다.
   - 단어: `moveToWordBoundary()`
   - 줄: `moveToLineStart()/moveToLineEnd()`
   - 문단: `moveToParagraphBoundary()`
3. `Shift`가 있으면 기존 selection anchor 모델을 유지하고 `updateSelection()`까지 호출한다.
4. Windows/Linux `Alt+←/→`는 `shouldSuppressUnmappedNavigation()`으로 앱 단어 이동도, 일반 Arrow 이동도 하지 않게 막았다.
5. IME 조합 중 pending navigation 저장값에 `key`와 `altKey`를 추가하고, composition end에서 동일 helper를 사용한다.

## 5. 검증 결과

### 단위 테스트

```text
npm test
tests 19
pass 19
fail 0
```

### 빌드

검증 전 `pkg/`가 없는 새 worktree 상태에서는 `@wasm/rhwp.js` 해석 실패가 발생했다. 기존 작업 트리의 gitignored `pkg/` 산출물을 검증용으로 복사한 뒤 빌드 성공.

```text
npm run build
✓ built
```

### E2E

검증 환경:

- dev server: `http://127.0.0.1:7702/`
- browser: local Chrome headless
- `pkg/`: 기존 작업 트리의 gitignored WASM 산출물 복사 사용

신규 navigation E2E:

```text
node e2e/navigation-shortcuts.test.mjs --mode=headless
PASS: Ctrl+← 단어 이동
PASS: Alt+← 미처리로 커서 유지
PASS: Ctrl+Shift+← 단어 선택
PASS: Option+← 단어 이동
PASS: Command+←/→ 줄 처음/끝
PASS: Command+Shift+→ 줄 선택
```

기존 Shift+End 회귀 E2E:

```text
node e2e/shift-end.test.mjs --mode=headless
PASS: Shift+End 후 선택 상태
PASS: 선택 하이라이트 표시
```

## 6. 참고 사항

- `npm ci`, `pkg/`, `dist/`, `node_modules/`, E2E screenshots/report는 검증용 산출물이며 gitignore 대상이다.
- `selection-renderer.ts`의 클래스 추가는 시각 변경이 아니라 기존 E2E가 기대하던 DOM 식별자 복원이다.
- Stage 5에서 최종 회귀 검증 및 보고서 작성 전 `git status --ignored`로 산출물 비포함 상태를 재확인한다.

## 7. Stage 4 결론

- 이슈 #945의 핵심인 플랫폼별 단어/줄 이동 keymap이 실제 keyboard/IME 경로에 연결됐다.
- Windows/Linux `Alt+←/→` 단어 이동은 제거됐다.
- 신규 E2E와 기존 Shift+End E2E가 모두 통과했다.
