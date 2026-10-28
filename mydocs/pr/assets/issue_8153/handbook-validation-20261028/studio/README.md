# #8153 편람의 실제 Studio 화면 검증

2026-10-29, 제품 소스 `a1e511dbebcc27e187ff847d5510358fa9fbada4`로 만든
최적화 fresh WASM과 Chrome 156.0.8078.47의 실제 Studio CanvasView 화면을
사용했다. viewport 1500×1600, DPR 1, screen profile에서 페이지를 실제로
스크롤해 해당 페이지 사각형을 캡처했으며 별도 그림·DOM plane도 화면에 포함한다.
앱 초기화 때 수신한 WASM SHA-256은 빌드 산출물과 같은
`3dbc71ea881554b86eeae35fcc124779946906c1e0d5b6145747a7d5fa040c77`이다.
빌드 원본 JS SHA-256은 `b94532f0b0cacd11a2de548272542b57ab6b96829ffe77ae081b6bc98b9943c6`,
Vite 변환을 거쳐 실제 초기화 때 받은 JS 응답 SHA-256은
`800d78d7482062684fa4672299d1c7931c03c7b0cc046384ea28df6e6d7e70c7`이다.
입력과 독립 Print의 SHA·전쪽 Print 결과는 [상위 검증 기록](../README.md)에 있다.

| 쪽 | 실제 Canvas2D 화면 | PNG SHA-256 |
| ---: | --- | --- |
| 9 | [HWP·HWPX 동일 화면](handbook-p009-canvas2d-screen.png) | `3ef071379c30b9048e8db71ee19ca07da1e8c6ef70c53448c1d87b6caeb49fe1` |
| 260 | [HWP·HWPX 동일 화면](handbook-p260-canvas2d-screen.png) | `d5455431217e4c85c2955b502722fd2e4e31824f0ea3152e83a7912f63ee2c33` |
| 311 | [HWP·HWPX 동일 화면](handbook-p311-canvas2d-screen.png) | `968f3392f946e9c972f0ae7f02a59ef84e9d9c211e9450d8578566ad02016dc6` |

두 원본을 각각 열어 총 6개 화면을 캡처했고, 같은 쪽끼리 PNG 바이트가 일치해
사본을 하나씩만 보존했다. 직접 판독에서 9쪽 제목·목차, 260쪽 그림과 본문,
311쪽 법령 대조표가 보인다. 화면에는 편집 가이드와 컬러 이미지가 있으므로
회색 Print의 실루엣 검증과 별도다. 화면 캡처의 260쪽 높이는 1005px이고
독립 PDF 래스터는 1006px이어서 두 이미지를 그대로 동일 크기라고 주장하지 않는다.

CanvasKit software를 요청한 두 원본은 둘 다 전체 383쪽이 사전 검사 상한
128쪽을 넘어 `pageLimitExceeded`를 반환하고 Canvas2D로 대체됐다. 사전 검사
제한은 고정 base `9418ed0c7c`에도 존재한다. 같은 세 쪽을 대체 화면으로
각각 다시 캡처해 직접 Canvas2D PNG와 바이트 일치를 확인했다. 이것은
**CanvasKit 화면 통과 증거가 아니다**. 실제 CanvasKit 소비 경로의 별도 검증과
브라우저의 선택 글꼴 프로그램별 바이트 출처 확인은 이 화면 캡처만으로 완료되지
않았다. 미검증 사항을 다른 backend의 결과로 대신하지 않는다. 검증 환경의
시스템 글꼴은 내부에만 보관했으며 새 글꼴 파일은 배포하지 않았다.

## 회귀 검사 정리 후 재검증

2026-10-29 제품 코드 `412426515e1d367edf4c3cb8a0c38f31eb454b29`를
사용해 fresh WASM을 다시 빌드하고 같은 두 원본의 9·260·311쪽을 실제 Studio에서
캡처했다. WASM SHA-256은
`9454c941a1a1bd16ea87d1a744695aa1bddc2fe48d167c086ce656b445f0f902`이며,
실행 중 브라우저가 받은 WASM의 해시와 일치한다. JS 원본과 Vite 응답의 해시는 위에
기록한 값과 같다. 검증용 작업 트리의 Git 기준점은 `a1e511dbebcc27e187ff847d5510358fa9fbada4`이고,
제품 코드와 테스트는 `412426515e`의 내용과 바이트 단위로 대조했다.

직접 Canvas2D를 요청한 6개 화면과 CanvasKit 요청 후 Canvas2D로 대체된 6개 화면은
각각 위의 해당 쪽 정식 PNG와 SHA-256이 일치한다. CanvasKit 직접 화면은 여전히
쪽수 제한 때문에 검증되지 않았으며, 이 재검증으로 대체 화면을 CanvasKit 통과로
판정하지 않는다. 검증 글꼴은 내부 시스템에서만 사용했다.
