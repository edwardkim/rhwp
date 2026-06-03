# 구현 계획서 — Task #945: rhwp-studio 플랫폼별 단어/줄 이동 단축키 정리

- 이슈: [#945](https://github.com/edwardkim/rhwp/issues/945)
- 수행 계획서: [task_m100_945.md](task_m100_945.md)
- Stage 1 보고서: [task_m100_945_stage1.md](../working/task_m100_945_stage1.md)
- 브랜치: `feature/issue-945-platform-nav-shortcuts`

## 1. 공식 기준

한컴 공식 단축키 표 기준:

| 기능 | Mac 한/글 | Windows 한/글 |
|------|-----------|----------------|
| 한 단어 오른쪽/왼쪽 | `Option+→/←` | `Ctrl+→/←` |
| 줄 처음/끝 | `Command+←/→` 또는 `Home/End` | `Home/End` |
| 한 문단 아래/위 | `Command+↓/↑` | `Ctrl+↓/↑` |

참조:

- Mac/Windows 비교표: <https://help.hancom.com/hoffice_mac/ko-KR/hwp/view/toolbar/shortcut%28table%29.htm>
- Windows 단축키 일람: <https://help.hancom.com/hoffice/multi/ko_kr/hwp/view/toolbar/shortcut%28table%29.htm>

## 2. 이번 구현 범위

### 포함

1. Windows/Linux `Ctrl+←/→`를 단어 이동으로 정정
2. Windows/Linux `Ctrl+Shift+←/→`를 단어 선택으로 정정
3. macOS `Option+←/→`, `Option+Shift+←/→` 단어 이동/선택 동작 유지 및 테스트 고정
4. macOS `Command+←/→`, `Command+Shift+←/→` 줄 처음/끝 이동/선택 동작 유지 및 테스트 고정
5. 공통 `Home/End`, `Shift+Home/End` 줄 처음/끝 이동/선택 동작 보존
6. IME 조합 종료 후 pending navigation도 동일 keymap을 쓰도록 정리
7. Windows/Linux `Alt+←/→`의 기존 단어 이동 동작 제거

### 보류

macOS `Command+↑/↓`는 공식 표상 문단 이동이지만, 현재 코드는 macOS 표준에 맞춰 문서 시작/끝으로 이동한다. 이번 이슈의 핵심 충돌은 단어/줄 이동이므로, `Command+↑/↓` 변경은 이번 PR에서 제외하고 별도 후속으로 남긴다.

이유:

- 기존 주석도 "macOS 표준"으로 명시되어 있어 동작 변경 시 사용자 체감 회귀 가능성이 있다.
- #945 본문도 `Command+↑/↓`는 "별도 검토"로 표현한다.
- 단어/줄 이동 정정만으로 현재 관찰된 `Ctrl+←/→` 충돌과 `Option+←/→` 테스트 공백을 해결할 수 있다.

## 3. 수정 파일

| 파일 | 작업 |
|------|------|
| `rhwp-studio/src/engine/navigation-keymap.ts` | 신규. 플랫폼별 navigation keymap 순수 함수 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | Ctrl/Meta 공통 처리 전에 navigation shortcut 선처리 |
| `rhwp-studio/src/engine/input-handler-text.ts` | IME pending navigation 처리에서 동일 keymap 재사용 |
| `rhwp-studio/tests/navigation-keymap.test.ts` | 신규 단위 테스트 |
| `rhwp-studio/e2e/navigation-shortcuts.test.mjs` | 신규 E2E 테스트 |
| `mydocs/working/task_m100_945_stage{N}.md` | 단계 보고서 |
| `mydocs/orders/20260518.md` | 진행 상태 갱신 |

## 4. 설계

### 4.1 순수 keymap 모듈

신규 파일:

```ts
// rhwp-studio/src/engine/navigation-keymap.ts
export type PlatformKind = 'mac' | 'other';

export type NavigationAction =
  | 'wordBackward'
  | 'wordForward'
  | 'lineStart'
  | 'lineEnd'
  | 'paragraphBackward'
  | 'paragraphForward';

export interface NavigationKeyInput {
  key: string;
  code?: string;
  shiftKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
}
```

핵심 함수:

```ts
export function detectPlatformKind(nav: Navigator = navigator): PlatformKind;
export function getNavigationAction(input: NavigationKeyInput, platform: PlatformKind): NavigationAction | null;
```

결정표:

| 입력 | macOS | other |
|------|-------|-------|
| `Alt+ArrowLeft` | `wordBackward` | null |
| `Alt+ArrowRight` | `wordForward` | null |
| `Meta+ArrowLeft` | `lineStart` | null |
| `Meta+ArrowRight` | `lineEnd` | null |
| `Ctrl+ArrowLeft` | null | `wordBackward` |
| `Ctrl+ArrowRight` | null | `wordForward` |
| `Ctrl+ArrowUp` | null | `paragraphBackward` |
| `Ctrl+ArrowDown` | null | `paragraphForward` |
| `Home` | `lineStart` | `lineStart` |
| `End` | `lineEnd` | `lineEnd` |

주의:

- `Alt+Arrow`의 other 플랫폼 동작은 기존 코드에 있었지만 한컴 Windows/Linux 기준과 충돌하므로 제거한다.
- `Ctrl+Home/End`, `Meta+Home/End`, `Meta+ArrowUp/Down`은 이번 helper에서 처리하지 않는다. 기존 `handleCtrlKey()` 동작을 유지한다.

### 4.2 keyboard handler 연결

`input-handler-keyboard.ts`의 `Ctrl/Meta` 공통 처리 직전에 helper를 호출한다.

의사 코드:

```ts
const navAction = getNavigationAction(toNavigationKeyInput(e), detectPlatformKind());
if (navAction && executeNavigationAction.call(this, navAction, e.shiftKey)) {
  e.preventDefault();
  return;
}

if (e.ctrlKey || e.metaKey) {
  this.handleCtrlKey(e);
  return;
}
```

`executeNavigationAction()`은 기존 cursor primitive를 그대로 쓴다.

| action | cursor 호출 |
|--------|-------------|
| `wordBackward` | `cursor.moveToWordBoundary(-1)` |
| `wordForward` | `cursor.moveToWordBoundary(1)` |
| `lineStart` | `cursor.moveToLineStart()` |
| `lineEnd` | `cursor.moveToLineEnd()` |
| `paragraphBackward` | `cursor.moveToParagraphBoundary(-1)` |
| `paragraphForward` | `cursor.moveToParagraphBoundary(1)` |

선택 처리:

```ts
if (shiftKey) this.cursor.setAnchor();
else this.cursor.clearSelection();

// move...
this.updateCaret();
if (shiftKey) this.updateSelection();
```

### 4.3 IME pending navigation

현재 `input-handler-keyboard.ts`는 IME 조합 중 navigation을 `_pendingNavAfterIME`에 저장하지만 `altKey`를 누락한다.

변경:

```ts
this._pendingNavAfterIME = {
  code: e.code,
  key: e.key,
  shiftKey: e.shiftKey,
  ctrlKey: e.ctrlKey,
  metaKey: e.metaKey,
  altKey: e.altKey,
};
```

`input-handler-text.ts`의 `processPendingNav()`는 같은 `getNavigationAction()`을 먼저 시도하고, action이 없으면 기존 plain arrow/Home/End fallback을 유지한다.

### 4.4 테스트 가능성

플랫폼 판별은 테스트에서 `PlatformKind`를 직접 넘겨 검증한다. 실제 `navigator` mock에 의존하지 않는다.

E2E에서는 실제 OS의 `navigator.platform` override가 불안정할 수 있으므로 다음 방식 중 하나를 사용한다.

1. 단위 테스트로 keymap을 확정하고 E2E는 현재 플랫폼에서 가능한 키만 검증
2. `window.__rhwpTestPlatformKind = 'mac' | 'other'` 같은 테스트 전용 override를 helper가 읽도록 추가

권장: **2**. E2E에서 macOS/Windows keymap을 같은 환경에서 모두 검증할 수 있다. production에서는 해당 값이 없으므로 `navigator` 기반 판별만 사용한다.

## 5. 구현 단계

### Stage 3 — keymap helper + 단위 테스트

작업:

1. `navigation-keymap.ts` 신규 추가
2. `navigation-keymap.test.ts` 신규 추가
3. macOS/other 결정표 단위 테스트 작성
4. `npm test` 통과 확인

완료 조건:

- `npm test` 통과
- command shortcut 입력(`Ctrl+S`, `Meta+S`, `Ctrl+C`)에 대해 helper가 `null` 반환

### Stage 4 — keyboard/IME 연결 + E2E 보강

작업:

1. `input-handler-keyboard.ts`에서 navigation shortcut 선처리
2. 기존 `Alt+Arrow` 단어 이동 분기를 플랫폼별 helper 경로로 흡수하여 Windows/Linux에서는 제거
3. `input-handler-text.ts`에서 IME pending nav 처리 통합
4. `navigation-shortcuts.test.mjs` 작성
5. `Shift` 선택 확장 검증

완료 조건:

- Windows/Linux `Ctrl+←/→` 단어 이동 확인
- macOS `Option+←/→`, `Command+←/→` 확인
- Windows/Linux `Alt+←/→`가 단어 이동으로 처리되지 않음 확인
- 기존 `Shift+End` 선택 동작 유지

### Stage 5 — 빌드 및 회귀 검증

작업:

1. `npm run build` (`rhwp-studio`)
2. `npm test` (`rhwp-studio`)
3. 신규 E2E `navigation-shortcuts.test.mjs`
4. 기존 `shift-end.test.mjs`
5. 필요 시 `text-flow.test.mjs`

완료 조건:

- TypeScript build 통과
- 단위 테스트 통과
- navigation E2E 통과
- 기존 Shift+End 회귀 없음

### Stage 6 — 보고서/PR 준비

작업:

1. 최종 보고서 `mydocs/report/task_m100_945_report.md`
2. `orders/20260518.md` 완료 상태 갱신
3. commit 준비
4. PR 본문에 #945와 #223 관계 명시

## 6. 위험 평가

| 위험 | 평가 | 대응 |
|------|------|------|
| `Ctrl/Meta` 공통 shortcut 회귀 | 중 | navigation key만 선처리하고 나머지는 기존 `handleCtrlKey()` 유지 |
| macOS 판별 오류 | 중 | 순수 함수 단위 테스트 + E2E test override |
| `Ctrl+Shift+←/→` 선택 하이라이트 누락 | 중 | `updateSelection()` 호출을 helper 실행 경로에 포함 |
| IME 조합 중 Option/Command/Ctrl navigation 누락 | 중 | `_pendingNavAfterIME.altKey` 추가 + 동일 helper 재사용 |
| 기존 Windows/Linux `Alt+Arrow` 사용자 회귀 | 낮음 | 한컴 공식 기준 정합을 우선하고 테스트로 명시 |
| `Command+↑/↓` 기대와 불일치 | 낮음 | 이번 PR 제외 사항으로 문서화, 후속 이슈 권장 |
| 표 셀/글상자 내부 이동 회귀 | 중 | 기존 `CursorState` primitive 재사용, E2E에서 본문 우선 검증 후 필요 시 셀 케이스 추가 |

## 7. 승인 필요 결정

구현 전 작업지시자 승인 필요 항목:

1. `Command+↑/↓`를 이번 PR에서 제외하는 범위 결정
2. 테스트 전용 `window.__rhwpTestPlatformKind` override 허용 여부
3. Windows/Linux `Alt+Arrow` 기존 단어 이동 제거 여부

권장 결정:

- `Command+↑/↓`: 이번 PR 제외
- 테스트 override: 허용
- Windows/Linux `Alt+Arrow`: 제거 (**작업지시자 결정: 한컴 기준 엄격 적용**)

## 8. 진행 규칙

- 본 구현 계획 승인 전 소스 수정 금지
- Stage 3 완료 후 보고서 작성 및 승인 요청
- Stage 4 완료 후 보고서 작성 및 승인 요청
- 회귀 발견 시 즉시 범위 축소 또는 revert
