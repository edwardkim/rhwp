# 편람 205쪽 Studio 실화면

2026-10-30 Linux Chrome 156.0.8078.47의 실제 Studio screen 경로에서 각 원본을
열어 383쪽임을 확인하고 205쪽을 캡처했다. 실제 브라우저가 받은 표준 최적화 fresh WASM의
SHA-256은 `d6dd0b6ce157ea50b6291b541854b2fec8437d2104e51e0597dccff7634df408`이며,
렌더링 제품 소스는 `5c33032fe1`이다. 캔버스 해상도는 741×1006, 캡처 시 이미지 로드
실패는 없었다. 글꼴은 실행 환경에서 공급하며 글꼴 파일을 배포하지 않는다.

| 원본 | 요청 경로 | 실제 경로 | 화면 |
| --- | --- | --- | --- |
| HWP | Canvas2D | Canvas2D | [205쪽](handbook-hwp-p205-canvas2d.png) |
| HWPX | Canvas2D | Canvas2D | [205쪽](handbook-hwpx-p205-canvas2d.png) |
| HWP·HWPX | CanvasKit software | Canvas2D로 전환 | 각 행의 Canvas2D 화면과 동일 |

CanvasKit 요청은 두 원본 모두 383쪽이 준비 상한 128쪽을 초과해
`pageLimitExceeded`와 `canvaskitResourcePreparationFailed`로 Canvas2D에 전환됐다.
이 결과를 CanvasKit의 실제 편람 출력 성공으로 표시하지 않는다. 별도의 합성 글리프
CanvasKit 브라우저 검사는 통과했으며, 이 실화면은 Print 프로필의 전쪽 TSV·독립 PDF
[비교](../README.md)를 대신하지 않는다.
