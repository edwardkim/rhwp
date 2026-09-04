# Task M100 #3126 Stage 0 — 현행 인쇄 계약과 surface 비교

- 작성일: 2026-07-23
- 기준 브랜치: `codex/3126-print-pdf-ux`
- 결정 댓글: [#3126 issuecomment-5054453628](https://github.com/edwardkim/rhwp/issues/3126#issuecomment-5054453628)
- 판정: **same-origin iframe 우선안 채택**

## 1. 현행 경로

변경 전 `file:print`의 순서는 다음과 같았다.

1. `renderPageSvg()`로 모든 페이지 생성
2. 5페이지마다 event loop 양보
3. 완료 뒤 `window.open('', '_blank')`
4. `about:blank` 창에 SVG와 rhwp 자체 `인쇄`·`닫기` 버튼 구성
5. 사용자가 rhwp `인쇄` 버튼을 다시 클릭

이 순서는 비동기 SVG 생성 뒤 popup을 열기 때문에 사용자 activation을 잃을 수 있고,
`about:blank`의 브라우저 zoom 기억을 상속할 수 있다. 또한 PDF 저장 경로를 이미 아는 사용자만
`Ctrl+P → 대상 → PDF로 저장`을 발견할 수 있다.

## 2. surface 비교

| 항목 | same-origin iframe | same-origin 전용 창 |
|---|---|---|
| popup 차단 | popup 없음 | 사용자 동기 클릭 구간에서 먼저 열어야 함 |
| `about:blank` | 전용 `print.html`로 제거 | 전용 URL을 쓰면 제거 가능 |
| 진행 상태 | 부모 Studio 상태바에서 유지 | 새 창 loading UI 별도 필요 |
| 자동 `print()` | Chrome에서 동작 확인 | 가능하나 창 lifecycle·조기 닫힘 처리 필요 |
| 혼합 `@page`/SVG id | 기존 DOM pipeline 재사용 | 기존 DOM pipeline 재사용 |
| 확장 CSP | inline script 없는 same-origin 정적 문서 | 동일하나 popup 권한/정책 변수 추가 |
| 정리 | 호출 뒤 iframe 제거 | 취소·조기 닫힘·창 소유권 처리 필요 |

iframe은 popup과 추가 rhwp 클릭을 동시에 없애며, 전용 창보다 실패 상태가 적다. 따라서 iframe을
기본 surface로 선택하고, 향후 특정 브라우저에서 named `@page` 또는 native print 호출 결함이
실증될 때만 same-origin 창을 fallback 후보로 재검토한다.

## 3. 브라우저 관찰

### in-app Chromium

- same-origin iframe이 `/print.html`을 로드하고 부모가 DOM을 구성할 수 있음을 확인했다.
- `iframe.contentWindow.print()` 호출 지점까지 도달하고 별도 top-level window는 생기지 않았다.
- 자동화 컨텍스트는 사용자 activation 만료 뒤 popup도 허용해 popup 차단 자체를 충실히 재현하지
  못했다. 따라서 이 항목은 기존 호출 순서와 브라우저 정책상 위험으로 기록한다.

### macOS Chrome

- 파일 메뉴의 `PDF로 저장…` 한 번 클릭 뒤 macOS/Chrome native print UI가 자동으로 열렸다.
- 별도 rhwp 인쇄 버튼과 top-level popup은 없었다.
- native print UI는 자동화 범위 밖에서 page script를 block하므로 취소 뒤 상태 불변은 mock
  E2E로 별도 검증한다.

### Edge

- 현재 macOS 환경에 Microsoft Edge가 설치되어 있지 않아 native dialog 비교를 수행하지 못했다.
- 구현을 Edge 전용 분기 없이 표준 same-origin iframe/`Window.print()`로 유지했다.
- merge/issue close 전 Windows Edge 수동 검증을 잔여 게이트로 둔다.

## 4. 상태 불변 관찰점

E2E에서 print 호출 전 sentinel file handle과 파일명을 주입하고 dirty 상태를 만든다. 다음 세
시점에서 동일성을 검사한다.

1. 명령 실행 전
2. 가로챈 `iframe.contentWindow.print()` 호출 시점
3. print 반환과 iframe 정리 뒤

브라우저는 저장과 취소 결과를 페이지에 알려주지 않으므로 성공 toast는 제품 계약에서 제외한다.

## 5. Stage 0 결론

- `about:blank`과 `window.open()`을 제거한다.
- same-origin 정적 `print.html` iframe을 실행별로 만들고 `finally`에서 제거한다.
- 기존 페이지별 named `@page`, SVG id namespace, 혼합 크기 로직을 그대로 재사용한다.
- `인쇄…`과 `PDF로 저장…`은 하나의 pipeline을 사용하고 안내 문구만 분리한다.
- Edge native-dialog 검증은 환경 의존 잔여 게이트다.
