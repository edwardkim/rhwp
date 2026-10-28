# Task M100 #1633 Stage 2 작업 문서

## 목표

한컴처럼 표 셀 컨텍스트 메뉴에서 `셀 테두리/배경 - 하나의 셀처럼 적용`은 여러 셀이 선택된 경우에만 활성화한다.

## 관찰

- 한컴은 단일 셀 선택 또는 셀 선택 범위가 없는 상태에서는 `하나의 셀처럼 적용`을 비활성화한다.
- 여러 셀이 선택된 경우에는 `각 셀마다 적용`과 `하나의 셀처럼 적용`이 모두 선택 가능하다.
- 현재 rhwp-studio는 단일 셀 상태에서도 `하나의 셀처럼 적용`을 실행 가능한 메뉴로 표시한다.

## 작업 범위

1. rhwp-studio의 표 셀 선택 상태에서 다중 셀 선택 여부를 판정하는 기존 상태/헬퍼를 확인한다.
2. 컨텍스트 메뉴의 `table:border-one` 항목을 다중 셀 선택이 아닐 때 disabled 처리한다.
3. 명령 실행 경로에서도 단일 셀일 때 `table:border-one` 실행을 막아 UI 외 호출과 상태 불일치를 피한다.
4. 기존 `table:border-each` 동작은 유지한다.

## 검증 계획

- `cd rhwp-studio && npx tsc --noEmit`
- 필요 시 `cd rhwp-studio && npm test`
- Vite dev 서버 `7700`에서 `대각선샘플.hwp`를 열고:
  - 단일 셀 우클릭: `하나의 셀처럼 적용` 비활성
  - 여러 셀 선택 후 우클릭: `하나의 셀처럼 적용` 활성

## 구현 결과

- `EditorContext`에 `hasMultiCellSelection`을 추가했다.
- `InputHandler.hasMultiCellSelection()`에서 셀 선택 범위가 2개 이상인지 판정한다.
- `table:border-one`의 `canExecute`를 `hasMultiCellSelection` 기준으로 바꾸고, 실행 함수에서도 같은 조건을 한 번 더 확인한다.
- `table:border-each`는 기존처럼 표 셀 내부에서 계속 활성화된다.

## 검증 결과

- `cd rhwp-studio && npx tsc --noEmit` 통과.
- `cd rhwp-studio && npm test` 통과: 147개.
- Browser plugin은 `iab`가 없어 사용할 수 없었다. `agent.browsers.list()` 결과가 빈 배열이라 Playwright fallback으로 검증했다.
- Playwright 검증:
  - 단일 셀 우클릭: `table:border-one` DOM class가 `md-item disabled`.
  - 다중 셀 선택(F5, F5, 오른쪽 방향키) 후 우클릭: `table:border-one` DOM class가 `md-item`.
  - 두 상태 모두 콘솔 error/warning 없음.

## 제외 범위

- 실제 cellzone 편집 정책 변경
- 셀 선택 UX 전체 재설계
