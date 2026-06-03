# 최종 보고서 — Task #945: rhwp-studio 플랫폼별 단어/줄 이동 단축키 정리

- 이슈: [#945](https://github.com/edwardkim/rhwp/issues/945)
- 관련 이슈: [#223](https://github.com/edwardkim/rhwp/issues/223)
- 브랜치: `feature/issue-945-platform-nav-shortcuts`
- worktree: `/private/tmp/rhwp-issue-945`

## 1. 요약

`rhwp-studio`의 navigation shortcut을 한컴 공식 기준에 맞춰 플랫폼별로 분리했다.

- macOS: `Option+←/→` 단어 이동, `Command+←/→` 줄 처음/끝
- Windows/Linux: `Ctrl+←/→` 단어 이동, `Home/End` 줄 처음/끝
- Windows/Linux: 기존 `Alt+←/→` 단어 이동은 한컴 기준과 맞지 않아 제거
- 선택 확장(`Shift`)은 기존 anchor 모델을 재사용
- IME 조합 종료 후 pending navigation도 같은 keymap 사용

## 2. 사전 확인 보정

공개 사이트 <https://edwardkim.github.io/rhwp/> 기준으로 macOS `Option+←/→` 단어 이동은 이미 동작하는 것으로 확인됐다.

따라서 본 작업의 본질은 "macOS Option 단어 이동 신규 구현"이 아니라, 다음 충돌 정리다.

1. Windows/Linux `Ctrl+←/→`가 줄 처음/끝으로 처리되는 문제 정정
2. macOS `Command+←/→` 줄 처음/끝 동작 유지
3. Windows/Linux `Alt+←/→` 단어 이동 제거
4. 위 동작을 단위 테스트와 E2E로 고정

## 3. 변경 파일

| 파일 | 내용 |
|------|------|
| `rhwp-studio/src/engine/navigation-keymap.ts` | 플랫폼별 navigation action 결정 helper 추가 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | Ctrl/Meta 공통 shortcut 처리 전 navigation shortcut 선처리 |
| `rhwp-studio/src/engine/input-handler-text.ts` | IME pending navigation에 동일 keymap 적용 |
| `rhwp-studio/src/engine/selection-renderer.ts` | 선택 하이라이트에 `.selection-highlight` class 부여 |
| `rhwp-studio/tests/navigation-keymap.test.ts` | keymap 단위 테스트 추가 |
| `rhwp-studio/e2e/navigation-shortcuts.test.mjs` | 플랫폼별 navigation E2E 추가 |
| `mydocs/orders/20260518.md` | 작업 상태 갱신 |
| `mydocs/plans/task_m100_945.md` | 수행 계획서 |
| `mydocs/plans/task_m100_945_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_945_stage1.md` | Stage 1 보고서 |
| `mydocs/working/task_m100_945_stage3.md` | Stage 3 보고서 |
| `mydocs/working/task_m100_945_stage4.md` | Stage 4 보고서 |

## 4. 최종 동작

### macOS

| 입력 | 동작 |
|------|------|
| `Option+←/→` | 단어 이동 |
| `Option+Shift+←/→` | 단어 선택 |
| `Command+←/→` | 줄 처음/끝 이동 |
| `Command+Shift+←/→` | 줄 처음/끝 선택 |
| `Home/End` | 줄 처음/끝 이동 |
| `Shift+Home/End` | 줄 처음/끝 선택 |

### Windows/Linux

| 입력 | 동작 |
|------|------|
| `Ctrl+←/→` | 단어 이동 |
| `Ctrl+Shift+←/→` | 단어 선택 |
| `Ctrl+↑/↓` | 문단 이동 |
| `Home/End` | 줄 처음/끝 이동 |
| `Shift+Home/End` | 줄 처음/끝 선택 |
| `Alt+←/→` | 단어 이동으로 처리하지 않음 |

## 5. 범위 제외

macOS `Command+↑/↓`는 공식 표상 문단 이동이지만, 현재 코드는 macOS 표준 동작으로 문서 시작/끝 이동을 유지한다. #945 본문도 "별도 검토"로 표현하므로 이번 PR에서는 변경하지 않았다.

## 6. 검증 결과

### 단위 테스트

```text
npm test
tests 19
pass 19
fail 0
```

### 빌드

```text
npm run build
✓ built
```

### E2E

검증 환경:

- dev server: `http://127.0.0.1:7702/`
- browser: local Chrome headless
- WASM: gitignored `pkg/` 산출물 사용

```text
node e2e/navigation-shortcuts.test.mjs --mode=headless
PASS
```

검증 항목:

- Windows/Linux `Ctrl+←` 단어 이동
- Windows/Linux `Alt+←` 미처리
- Windows/Linux `Ctrl+Shift+←` 단어 선택
- macOS `Option+←` 단어 이동
- macOS `Command+←/→` 줄 처음/끝
- macOS `Command+Shift+→` 줄 선택

```text
node e2e/shift-end.test.mjs --mode=headless
PASS
```

검증 항목:

- `Shift+End` 후 선택 상태
- 선택 하이라이트 표시

## 7. 사용자 수동 검증

작업지시자가 dev server에서 직접 검증 완료.

확인 사항:

- 공개 사이트에서도 macOS `Option+←/→` 단어 이동은 이미 가능
- 이번 작업의 수정 범위는 Windows/Linux `Ctrl+←/→`, Windows/Linux `Alt+←/→` 제거, 플랫폼별 keymap 고정으로 재정리

## 8. 산출물 관리

다음은 검증용 산출물이며 gitignore 대상이다. PR에 포함하지 않는다.

- `pkg/`
- `rhwp-studio/node_modules/`
- `rhwp-studio/dist/`
- `rhwp-studio/e2e/screenshots/`
- `output/`

## 9. 결론

Task #945 완료. 플랫폼별 단어/줄 이동 단축키 충돌을 정리했고, command shortcut 경로는 기존대로 보존했다. 최종 검증은 통과했다.
