# Task #945 Stage 3 — keymap helper + 단위 테스트

## 1. 구현 범위

이번 단계에서는 실제 키보드 핸들러 연결은 하지 않고, 플랫폼별 navigation shortcut 결정 로직과 단위 테스트만 추가했다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `rhwp-studio/src/engine/navigation-keymap.ts` | 플랫폼 판별 + navigation action 결정 순수 함수 추가 |
| `rhwp-studio/tests/navigation-keymap.test.ts` | macOS / Windows-Linux keymap 단위 테스트 추가 |

## 3. keymap 결정

### macOS

| 입력 | action |
|------|--------|
| `Option+←` | `wordBackward` |
| `Option+→` | `wordForward` |
| `Command+←` | `lineStart` |
| `Command+→` | `lineEnd` |
| `Command+↑/↓` | 이번 범위에서 처리하지 않음 |

### Windows/Linux

| 입력 | action |
|------|--------|
| `Ctrl+←` | `wordBackward` |
| `Ctrl+→` | `wordForward` |
| `Ctrl+↑` | `paragraphBackward` |
| `Ctrl+↓` | `paragraphForward` |
| `Alt+←/→` | 처리하지 않음 |

### 공통

| 입력 | action |
|------|--------|
| `Home` | `lineStart` |
| `End` | `lineEnd` |
| `Shift` 조합 | action은 동일, 선택 확장은 Stage 4 handler 연결에서 처리 |

## 4. 테스트 항목

`navigation-keymap.test.ts`에서 다음을 고정했다.

- macOS platform/userAgent 판별
- Windows/Linux platform 판별
- 테스트 전용 `globalThis.__rhwpTestPlatformKind` override
- macOS `Option+Arrow` 단어 이동
- macOS `Command+ArrowLeft/Right` 줄 처음/끝
- macOS `Ctrl+Arrow`, `Command+ArrowUp/Down` 미처리
- Windows/Linux `Ctrl+ArrowLeft/Right` 단어 이동
- Windows/Linux `Ctrl+ArrowUp/Down` 문단 이동
- Windows/Linux `Alt+Arrow` 미처리
- `Home/End` 공통 줄 처음/끝
- `Ctrl+S`, `Meta+S`, `Ctrl+C`, `Meta+C` 같은 command shortcut 미처리
- IME pending navigation처럼 `key=Process`인 경우 `code` 기반 판별

## 5. 검증 결과

`rhwp-studio` 단위 테스트:

```text
npm test
tests 19
pass 19
fail 0
```

`npm run build`:

```text
sh: tsc: command not found
```

이 worktree의 `rhwp-studio/node_modules`가 준비되어 있지 않아 TypeScript build는 실행하지 못했다. Stage 5에서 의존성 설치 또는 기존 개발 환경 정렬 후 다시 검증한다.

## 6. Stage 3 결론

- keymap 결정 로직은 소스에서 독립된 순수 함수로 분리했다.
- 한컴 기준에 맞춰 Windows/Linux `Alt+Arrow` 단어 이동은 action 없음으로 고정했다.
- command shortcut은 navigation helper가 처리하지 않도록 테스트로 고정했다.
- 다음 Stage 4에서 `input-handler-keyboard.ts`와 `input-handler-text.ts`에 helper를 연결한다.
