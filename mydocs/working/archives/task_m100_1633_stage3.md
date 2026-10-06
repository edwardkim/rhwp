# Task M100 #1633 Stage 3 작업 문서

## 목표

빈 문서/파일 미선택 상태에서 `입력` 메뉴의 `각주`, `미주` 항목이 활성화되는 UI 상태를 한컴 동작에 맞게 비활성화한다.

## 관찰

- 현재 rhwp-studio는 HWP 파일을 선택하지 않은 초기 상태에서도 `각주`, `미주` 메뉴가 활성화되어 보인다.
- 이 상태에서는 본문 커서/문단 대상이 없어 각주/미주 삽입 명령을 수행할 수 없다.

## 작업 범위

1. `insert:footnote`, `insert:endnote` 명령의 활성 조건을 `hasDocument` 기준으로 제한한다.
2. 명령 실행 경로에서도 문서가 없으면 즉시 반환해 메뉴 상태와 직접 호출 상태를 일치시킨다.
3. 기존 문서 로드 후 각주/미주 삽입 가능 상태는 유지한다.

## 검증 계획

- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- Vite dev 서버 `7700`에서:
  - 빈/파일 미선택 상태: 입력 메뉴의 `각주`, `미주` 비활성
  - 새 문서 생성 또는 문서 로드 후: `각주`, `미주` 활성

## 구현 결과

- `insert:footnote`, `insert:endnote`의 `canExecute`를 `ctx.hasDocument` 기준으로 제한했다.
- 명령 실행 경로에서도 `services.getContext().hasDocument`가 false이면 즉시 반환하도록 보강했다.

## 검증 결과

- `cd rhwp-studio && npx tsc --noEmit` 통과.
- `cd rhwp-studio && npm test` 통과: 147개.
- Browser plugin은 `iab`가 없어 사용할 수 없었다. `agent.browsers.list()` 결과가 빈 배열이라 Playwright fallback을 시도했다.
- Playwright fallback 중 390px viewport에서 상단 메뉴 타이틀이 보이지 않아 메뉴 클릭 검증은 타임아웃으로 종료됐다.
- 작업지시자가 빈 문서 상태의 각주/미주 메뉴 비활성 시각 검증을 완료했다.
