# PR #1281 검토 - 찾기 대화상자 Enter 키 라우팅

- PR: https://github.com/edwardkim/rhwp/pull/1281
- 작성일: 2026-06-03
- 작성자: @lidge-jun
- 제목: `fix: keep find dialog Enter inside search`
- base: `main`
- head: `fix/find-dialog-enter-routing` / `f887dca46fee37383012625a9227b3c599545a36`
- 상태: open, non-draft
- PR 댓글: 없음

## 1. PR 요약

PR #1281은 `rhwp-studio`의 modeless 찾기/찾아 바꾸기 대화상자가 열려 있을 때
`Enter` 키가 편집기 본문으로 새지 않도록 처리한다.

기존 동작에서는 찾기창을 연 뒤 포커스가 입력칸 밖 또는 편집면으로 이동하면,
반복 `Enter`가 다음 찾기가 아니라 편집기 입력/선택 동작으로 처리될 수 있었다.

이번 PR은 찾기창이 열려 있는 동안 document capture 단계의 `keydown` handler를 설치해
plain `Enter`, `Shift+Enter`, `Escape`를 찾기창 쪽에서 먼저 처리하도록 한다.

## 2. 변경 범위

| file | 변경 |
|---|---|
| `rhwp-studio/src/ui/find-dialog.ts` | document capture keydown handler 추가, input-local Enter/Escape 처리 제거 |

핵심 변경:

- `FindDialog.show()`에서 `installKeyCaptureHandler()` 호출
- `FindDialog.hide()`에서 capture handler 제거
- plain `Enter`는 다음 찾기, `Shift+Enter`는 이전 찾기로 라우팅
- replace input에서 plain `Enter`는 기존처럼 `doReplace()` 호출
- `Escape`는 열린 찾기창 handler에서 닫기 처리
- `Alt/Ctrl/Meta` 조합 및 IME composing 상태는 검색 Enter로 처리하지 않음

## 3. 코드 검토

### 3.1 capture-phase handler

`document.addEventListener('keydown', handler, true)`를 사용하므로,
편집기 textarea/input handler보다 먼저 `Enter`/`Escape`를 가로챌 수 있다.

`hide()`에서 `removeKeyCaptureHandler()`를 호출하고 handler 참조를 `null`로 되돌리므로,
대화상자 닫힘 이후 키 이벤트가 남는 누수 위험은 낮다.

### 3.2 replace input Enter

replace mode에서 `target === this.replaceInput && !e.shiftKey`이면 `doReplace()`를 호출한다.
따라서 바꾸기 입력칸의 plain `Enter` 동작은 기존과 동일하게 유지된다.

`Shift+Enter`는 이전 찾기로 처리된다. 찾기 입력칸과 replace 입력칸에서 같은 방향 검색
계약을 유지하는 쪽이라 수용 가능하다.

### 3.3 modifier / IME guard

`isFindEnter()`가 `altKey`, `ctrlKey`, `metaKey`, `isComposing`을 제외하므로,
편집기 전역 단축키나 IME 조합 중 Enter까지 빼앗는 범위 확대는 피했다.

### 3.4 기존 dialog 패턴과의 정합

`CommandPalette`, `SymbolsDialog`, `ModalDialog` 등도 document capture 단계에서 키 이벤트를
처리하고 편집기 전파를 차단한다. 이번 변경은 기존 UI 레이어의 키 이벤트 차단 패턴과
같은 계열이다.

주의점:

- 일반 타이핑은 input의 기본 동작으로 유지되어야 하므로 실제 브라우저 동작 확인이 필요하다.
- 이번 자동 검증은 DOM/browser 상호작용을 직접 재현하지 않는다. 메인테이너 수동 테스트가
  최종 게이트가 되어야 한다.

## 4. 로컬 검증

통합 브랜치:

```text
local/pr1281-integration
```

적용:

```text
git cherry-pick origin/pr/1281
```

결과: 충돌 없음.

단위 테스트:

```text
cd rhwp-studio
npm test
```

결과:

```text
tests 7
pass 7
fail 0
```

빌드:

```text
cd rhwp-studio
npm run build
```

결과: 통과.

참고:

- CanvasKit `fs`/`path` browser externalization warning은 기존 경고 계열이다.
- large chunk warning도 기존 Vite 빌드 경고 계열이다.

## 5. 수동 판정 체크리스트

| 항목 | 기대 동작 | 판정 |
|---|---|---|
| 찾기창 open + 입력칸 focus + Enter | 다음 찾기 | 성공 |
| 찾기창 open + 입력칸 focus + Shift+Enter | 이전 찾기 | 성공 |
| 찾기창 open + 편집면 focus + Enter | 편집기 입력이 아니라 다음 찾기 | 성공 |
| 찾기창 open + 편집면 focus + Shift+Enter | 편집기 줄바꿈이 아니라 이전 찾기 | 성공 |
| 찾아 바꾸기 mode + 바꿀 내용 focus + Enter | 바꾸기 실행 | 성공 |
| 찾기창 open + Escape | 찾기창 닫힘 | 성공 |
| IME composing 중 Enter | 찾기 Enter로 오인하지 않음 | 성공 |

메인테이너 동작 테스트:

```text
2026-06-03 성공
```

## 6. 권장 처리

권장: 수용.

근거:

- 변경 범위가 `FindDialog` 하나로 좁다.
- 문제 원인인 modeless dialog keyboard leak을 직접 차단한다.
- handler 설치/해제가 명확하고 modifier/IME guard가 있다.
- `rhwp-studio` 단위 테스트와 production build가 통과했다.

진행 절차:

1. 메인테이너 수동 동작 판정 완료
2. 판정 통과 시 `devel` 병합
3. 원격 push
4. PR #1281 종료 처리
