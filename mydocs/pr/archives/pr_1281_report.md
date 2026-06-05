# PR #1281 처리 보고서

- PR: https://github.com/edwardkim/rhwp/pull/1281
- 작성일: 2026-06-03
- 작성자: @lidge-jun
- 처리 브랜치: `local/pr1281-integration`
- 통합 방식: 최신 `devel` 기준 cherry-pick

## 1. 반영 내용

PR #1281의 단일 커밋을 `local/pr1281-integration`에 cherry-pick했다.

```text
b1496922 fix: keep find dialog enter inside search
```

변경 내용:

- 찾기/찾아 바꾸기 대화상자가 열려 있는 동안 document capture 단계 `keydown` handler 설치
- plain `Enter` / `Shift+Enter`를 다음/이전 찾기로 라우팅
- replace input의 plain `Enter`는 `doReplace()`로 유지
- `Escape`는 같은 capture handler에서 찾기창 닫기로 처리
- 대화상자 닫힘 시 capture handler 제거

## 2. 검토 결과

수용 가능하다.

확인한 점:

- modeless 찾기창이 열린 상태에서 편집기 본문으로 `Enter`가 새는 문제를 직접 막는다.
- `Alt/Ctrl/Meta` 조합과 IME composing Enter는 검색 동작에서 제외한다.
- handler lifecycle이 `show()`/`hide()`에 묶여 있어 닫힌 뒤 키 이벤트 누수 가능성이 낮다.
- 기존 `CommandPalette`, `SymbolsDialog`, `ModalDialog`의 document capture keyboard 처리 패턴과 방향이 같다.

주의점:

- 브라우저 DOM 입력 동작은 단위 테스트만으로 완전히 검증되지 않는다.
- 최종 게이트는 메인테이너의 rhwp-studio 수동 동작 판정으로 둔다.

## 3. 검증

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

빌드 경고:

- CanvasKit `fs`/`path` browser externalization warning
- Vite large chunk warning

위 경고는 기존 경고 계열이며 빌드 실패는 아니다.

## 4. 수동 판정 필요 항목

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

## 5. 판정

자동 검증 및 메인테이너 동작 테스트 기준으로 통합 가능하다.

## 6. 후속 UX 보강

PR #1281 반영 후 같은 키보드 UX 범위에서 페이지 이동 대화창도 보강했다.

- 페이지 이동 대화창의 쪽 번호 입력칸에서 `Enter`를 누르면 확인 버튼과 동일하게 이동 적용
- 잘못된 쪽 번호를 입력하면 대화창을 닫지 않고 오류 메시지 유지
- `Alt/Ctrl/Meta` 조합 및 IME composing 중 Enter는 이동으로 오인하지 않음

검증:

```text
cd rhwp-studio
npm test
npm run build
```

결과: 통과.

메인테이너 동작 테스트:

```text
2026-06-03 성공
```

## 7. 남은 절차

1. 후속 UX 보강 커밋
2. 원격 `devel` push
