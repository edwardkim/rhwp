# 실제 Studio backend 검증

검증 code: `20d93fe1d4226acab640b66875964d09c7d6d59b`. 아래27개 실제 캡처와5개 확대 시트는 이 code의 fresh WASM에서 다시 생성·직접 판독했다.

동일한 fresh WASM을 초기 fetch 응답의 복제 바이트에서 SHA-256으로 확인했다. 브라우저 안에서 기존 response를 clone해 관찰했으며 별도 사후 fetch를 생산 증거로 사용하지 않았다. JS/WASM과 Studio public 동기화 증거는 상위 README에 있다.

실제 초기 WASM SHA-256: `4907d462d9bace404d5f0b11a3d8b7d1115612ed5bde92db4cbd018f0b938e62`.

실제 `PageRenderer.renderPage`의 Canvas2D·CanvasKit, 공개 `renderPageSvgWithProfile(...,"print")`의 SVG DOM을 각각 캡처했다. Canvas2D는 흐름 이미지의 DOM 레이어도 포함한 container를 screenshot했고 폰트·이미지 decode 뒤 재렌더했다. CanvasKit은 실제 software surface, fallback 없음, runtime readiness PASS와 unsupported op 없음으로 확인했다. SVG 문자열을 파일로 쓰지 않고 브라우저 DOM으로 raster했다.

5개 입력9쪽을 각 backend에서 실행했다. SVG와 실제 CanvasKit은 각각9쪽 모두 독립 Print 대비100%다. Studio Canvas2D의 전체 UI screenshot은 `page-renderer.ts::drawMarginGuides`가 그리는 편집용 L자 여백 안내선도 포함한다. 이 값(아래 표)은 안내선 없는 독립 Print 전쪽 gate로 보고하지 않는다. 안내선을 지우거나 마스킹하지 않았으며 원본 전체 캡처도 함께 보존했다. 별도 실제 Print Canvas2D 전쪽 TSV는29문서42쪽 전부100%이고 상위 README에 연결한다.

5개 확대 시트를 실제 열어 본문 탭, 그림 바로 뒤 PAGE1, 좁은 문단의 PAGE1/12 줄바꿈, 양쪽 정렬의 PAGE1/2/12 및 후속 글자 간격, 새 쪽 단 구분선을 대조했다. 독립 Print와 대응하며 내용 누락·소속 이동은 관찰하지 않았다.

[확대1](inspection-1.png) · [확대2](inspection-2.png) · [확대3](inspection-3.png) · [확대4](inspection-4.png) · [확대5](inspection-5.png)

| 입력 / 쪽 | 실제 경로 | 전체 캡처 실루엣% | 원본 캡처 |
| --- | --- | --- | --- |
| pr7663-cell-hwpx / 1 | canvas2d | 92.18581 | [pr7663-cell-hwpx-canvas2d-1.png](pr7663-cell-hwpx-canvas2d-1.png) |
| pr7663-cell-hwpx / 1 | actual SVG browser raster | 100.0 | [pr7663-cell-hwpx-svg-1.png](pr7663-cell-hwpx-svg-1.png) |
| pr7671-picture-hwpx / 1 | canvas2d | 73.43173 | [pr7671-picture-hwpx-canvas2d-1.png](pr7671-picture-hwpx-canvas2d-1.png) |
| pr7671-picture-hwpx / 1 | actual SVG browser raster | 100.0 | [pr7671-picture-hwpx-svg-1.png](pr7671-picture-hwpx-svg-1.png) |
| pr7671-body-page-fields-narrow-restart12-hwpx / 1 | canvas2d | 68.78613 | [pr7671-body-page-fields-narrow-restart12-hwpx-canvas2d-1.png](pr7671-body-page-fields-narrow-restart12-hwpx-canvas2d-1.png) |
| pr7671-body-page-fields-narrow-restart12-hwpx / 1 | actual SVG browser raster | 100.0 | [pr7671-body-page-fields-narrow-restart12-hwpx-svg-1.png](pr7671-body-page-fields-narrow-restart12-hwpx-svg-1.png) |
| pr7671-body-page-fields-narrow-restart12-hwpx / 2 | canvas2d | 53.54839 | [pr7671-body-page-fields-narrow-restart12-hwpx-canvas2d-2.png](pr7671-body-page-fields-narrow-restart12-hwpx-canvas2d-2.png) |
| pr7671-body-page-fields-narrow-restart12-hwpx / 2 | actual SVG browser raster | 100.0 | [pr7671-body-page-fields-narrow-restart12-hwpx-svg-2.png](pr7671-body-page-fields-narrow-restart12-hwpx-svg-2.png) |
| pr7671-body-page-field-justification-hwpx / 1 | canvas2d | 69.31818 | [pr7671-body-page-field-justification-hwpx-canvas2d-1.png](pr7671-body-page-field-justification-hwpx-canvas2d-1.png) |
| pr7671-body-page-field-justification-hwpx / 1 | actual SVG browser raster | 100.0 | [pr7671-body-page-field-justification-hwpx-svg-1.png](pr7671-body-page-field-justification-hwpx-svg-1.png) |
| pr7671-body-page-field-justification-hwpx / 2 | canvas2d | 70.09736 | [pr7671-body-page-field-justification-hwpx-canvas2d-2.png](pr7671-body-page-field-justification-hwpx-canvas2d-2.png) |
| pr7671-body-page-field-justification-hwpx / 2 | actual SVG browser raster | 100.0 | [pr7671-body-page-field-justification-hwpx-svg-2.png](pr7671-body-page-field-justification-hwpx-svg-2.png) |
| pr7671-body-page-field-justification-hwpx / 3 | canvas2d | 70.92867 | [pr7671-body-page-field-justification-hwpx-canvas2d-3.png](pr7671-body-page-field-justification-hwpx-canvas2d-3.png) |
| pr7671-body-page-field-justification-hwpx / 3 | actual SVG browser raster | 100.0 | [pr7671-body-page-field-justification-hwpx-svg-3.png](pr7671-body-page-field-justification-hwpx-svg-3.png) |
| pr7673-line-hwpx / 1 | canvas2d | 46.53465 | [pr7673-line-hwpx-canvas2d-1.png](pr7673-line-hwpx-canvas2d-1.png) |
| pr7673-line-hwpx / 1 | actual SVG browser raster | 100.0 | [pr7673-line-hwpx-svg-1.png](pr7673-line-hwpx-svg-1.png) |
| pr7673-line-hwpx / 2 | canvas2d | 73.04348 | [pr7673-line-hwpx-canvas2d-2.png](pr7673-line-hwpx-canvas2d-2.png) |
| pr7673-line-hwpx / 2 | actual SVG browser raster | 100.0 | [pr7673-line-hwpx-svg-2.png](pr7673-line-hwpx-svg-2.png) |
| pr7663-cell-hwpx / 1 | canvaskit | 100.0 | [pr7663-cell-hwpx-canvaskit-1.png](pr7663-cell-hwpx-canvaskit-1.png) |
| pr7671-picture-hwpx / 1 | canvaskit | 100.0 | [pr7671-picture-hwpx-canvaskit-1.png](pr7671-picture-hwpx-canvaskit-1.png) |
| pr7671-body-page-fields-narrow-restart12-hwpx / 1 | canvaskit | 100.0 | [pr7671-body-page-fields-narrow-restart12-hwpx-canvaskit-1.png](pr7671-body-page-fields-narrow-restart12-hwpx-canvaskit-1.png) |
| pr7671-body-page-fields-narrow-restart12-hwpx / 2 | canvaskit | 100.0 | [pr7671-body-page-fields-narrow-restart12-hwpx-canvaskit-2.png](pr7671-body-page-fields-narrow-restart12-hwpx-canvaskit-2.png) |
| pr7671-body-page-field-justification-hwpx / 1 | canvaskit | 100.0 | [pr7671-body-page-field-justification-hwpx-canvaskit-1.png](pr7671-body-page-field-justification-hwpx-canvaskit-1.png) |
| pr7671-body-page-field-justification-hwpx / 2 | canvaskit | 100.0 | [pr7671-body-page-field-justification-hwpx-canvaskit-2.png](pr7671-body-page-field-justification-hwpx-canvaskit-2.png) |
| pr7671-body-page-field-justification-hwpx / 3 | canvaskit | 100.0 | [pr7671-body-page-field-justification-hwpx-canvaskit-3.png](pr7671-body-page-field-justification-hwpx-canvaskit-3.png) |
| pr7673-line-hwpx / 1 | canvaskit | 100.0 | [pr7673-line-hwpx-canvaskit-1.png](pr7673-line-hwpx-canvaskit-1.png) |
| pr7673-line-hwpx / 2 | canvaskit | 100.0 | [pr7673-line-hwpx-canvaskit-2.png](pr7673-line-hwpx-canvaskit-2.png) |

관찰 스크립트는 ignored `output/pr-review/semanticist21-20261015/cache-provenance-studio-backend-probe-20261016.mjs`, raw 결과/response hash/renderer 진단은 같은 output의 `cache-provenance-studio-backends/result.json`에 보존한다. 폰트는 시스템 원본2개를 런타임에만 공급했고 바이너리는 배포하지 않는다.
