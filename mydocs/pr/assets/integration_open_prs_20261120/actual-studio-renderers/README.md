# 최종 Studio 실제 렌더러 검증

시행령 7·16쪽, 정책연구용역 신청서 9쪽, 한 줄 입력 결재문서 1쪽을 `screen`과
`print`에서 각각 열었다. 브라우저 캡처 24건에서 Canvas2D 8건과 CanvasKit 소프트웨어
16건(compat·default 각 8건)의 실제 렌더러 및 프로필이 요청과 일치했다. 대상 쪽에서
글꼴 폴백, 준비 실패 또는 숨은 Canvas2D 합성이 재발한 건은 0건이었다.
[캡처별 입력·PNG 및 런타임 검증](validation.json).

초기 headless 검증 세션은 설치 글꼴의 파일 바이트를 전달하지 않아 시행령과 신청서의
CanvasKit 글꼴 준비 단계에서 폴백했다. 시행령에서 나눔바른고딕을 쓰는 두 글꼴 조합은
모두 Bold를 요청했다. Regular를 공급한 탐색에서도 이 요청이 남았다. 검증 세션에 한해
실제 Bold 프로그램과 두 문서의 나머지 로컬 글꼴을 기존 Studio 호스트 글꼴 공급자 API로
제공했다. 공급 원본은 바이트 해시로 확인했다. 모든 공급 설정은 로컬 검사 출력에 있다.
시행령 12건에서 앱 초기화가 사용한 WASM HTTP 응답도 최종 WASM과 같은 해시였다.
[나눔바른고딕 배포 원본](https://hangeul.naver.com/fonts/search?f=nanum).

아래 PNG는 Studio `print`의 실제 캡처다. 기준 PDF 대비 점수와 앞뒤 판독은
[공식 Print 대조 및 전쪽 TSV](../README.md#최적화한-fresh-wasm-재검증)에서 확인한다.

| 문서·쪽 | Canvas2D | CanvasKit compat | CanvasKit default |
| --- | --- | --- | --- |
| 시행령 7 | [PNG](decree-terminal-p007-canvas2d-print.png) | [PNG](decree-terminal-p007-canvaskit-compat-print.png) | [PNG](decree-terminal-p007-canvaskit-default-print.png) |
| 시행령 16 | [PNG](decree-terminal-p016-canvas2d-print.png) | [PNG](decree-terminal-p016-canvaskit-compat-print.png) | [PNG](decree-terminal-p016-canvaskit-default-print.png) |
| 정책연구용역 신청서 9 | [PNG](issue9040-nested-p009-canvas2d-print.png) | [PNG](issue9040-nested-p009-canvaskit-compat-print.png) | [PNG](issue9040-nested-p009-canvaskit-default-print.png) |
| 한 줄 입력 결재문서 1 | [PNG](squeeze-title-p001-canvas2d-print.png) | [PNG](squeeze-title-p001-canvaskit-compat-print.png) | [PNG](squeeze-title-p001-canvaskit-default-print.png) |
