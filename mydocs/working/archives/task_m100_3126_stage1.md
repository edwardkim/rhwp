# Task M100 #3126 Stage 1 — 발견 가능한 PDF UX와 iframe print pipeline

- 작성일: 2026-07-23
- 판정: 구현·단위 검증 완료

## 1. 구현

- 파일 메뉴에 별도 `PDF로 저장…` 명령을 추가했다.
- tooltip에 `브라우저 인쇄 창에서 ‘대상 → PDF로 저장’을 선택합니다.`를 표시한다.
- 기존 `인쇄…`과 새 명령을 `runBrowserPrint(services, intent)` 하나로 통합했다.
- 출력 의도에 따라 `PDF 준비 중…`/`인쇄 준비 중…`을 표시한다.
- PDF 경로는 준비 시작 시 남은 브라우저 단계를 toast로 안내하고, 저장 성공 메시지는 표시하지
  않는다.
- `public/print.html`을 same-origin iframe으로 로드하고, font/layout 준비 뒤 `print()`를 자동
  호출한다.
- 실행 중 중복 명령을 차단하고 `finally`에서 iframe과 실행 상태를 정리한다.
- `window.open('', '_blank')`, `about:blank`, rhwp 자체 인쇄·닫기 버튼을 제거했다.

## 2. surface 준비 계약

`createPrintSurface()`는 다음을 보장한다.

- 현재 Studio `baseURI` 기준으로 `print.html` URL 해석
- 동일 id의 오래된 iframe 제거
- load/error/10초 timeout 처리
- `contentWindow`·`contentDocument` 존재와 exact origin 확인
- dispose 멱등성

`waitForPrintSurfaceReady()`는 `document.fonts.ready`, animation frame 두 번, layout read 순서로
인쇄 호출 전 준비를 마친다.

## 3. 유지한 기존 회귀 계약

- CSS px → mm 변환
- 페이지마다 독립된 named `@page`
- 서로 다른 페이지 크기·방향
- SVG 내부 id와 `url(#id)`/hash 참조의 페이지 namespace
- 마지막 페이지의 불필요한 page break 제거

## 4. 단위 검증

focused Studio test 12개가 통과했다.

- 두 명령의 shared pipeline
- 명시적 print profile 호출
- 별도 메뉴와 tooltip
- 저장 상태 비변경 source contract
- 혼합 용지 named `@page`
- SVG id namespace
- PDF/인쇄 진행 문구
- same-origin `print.html`, `about:blank` 부재

전체 Studio test는 513/513, TypeScript와 production build도 통과했다.
