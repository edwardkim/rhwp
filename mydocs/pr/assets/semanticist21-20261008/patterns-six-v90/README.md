---
kind: evidence
status: active
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-10-10
---

# #7655 무늬 6종의 같은 입력 인쇄 대조

[같은 입력 HWPX](input.hwpx)는 기존 API/모델 편집으로 만든 캡처다. 실제 한컴에서 작성한 원본·새 정상 한컴 fixture로 분류하지 않는다. SHA-256은 `216166ae3b7d760fd03d9ebb297e8bb268fca5a13e0534380a8a8465901a58ae`이다. 독립 기준은 같은 입력의 실제 한컴2020 PrintMethod0/1-up [Print PDF](../../../../../pdf/semanticist21-20261008/maintainer_object_boxes-0000-216166ae3b7d-2020.pdf), 1쪽, SHA-256 `9ac1553a70c91cb86d890ea9d9e2b50cd5c34bc230420db3512a2a9f7e9b41e4`다. 이미 커밋한 Print 파일을 그대로 사용했다.

## 실제 검증과 직접 판독

| 경로 | 전체 쪽수 / 최저 일치율 | review / overlay |
| --- | --- | --- |
| 현재 Native SVG + webfont raster | 1 / 100% | [review](native-review_001.png), [overlay](native-overlay_001.png) |
| 현재 WASM 실제 Canvas2D | 1 / 100% | [review](wasm-canvas2d-review_001.png), [overlay](wasm-canvas2d-overlay_001.png) |
| 현재 WASM 실제 CanvasKit software | 1 / 100% | [review](wasm-canvaskit-review_001.png), [overlay](wasm-canvaskit-overlay_001.png) |

Native는 실제 HCRBatang을 우선 공급한 v91 retry1에 새 export/raster, 두 WASM Canvas는 v88에서 실제 현재 패키지로 새 캡처한 PNG의 재사용이다. 6종의 수평·수직·두 대각선·격자·대각 격자와 노랑 바탕/주황 선의 색상, 방향, 셀 소속을 독립 Print와 review에서 직접 대조했다. CanvasKit review의 판독은 진행72, Native review는 진행74에 연결한다. 새 Native v91 review를 직접 열어 같은 항목을 다시 대조했다. 제목 자간과 작은 AA·선 두께/위상 차이는 남는다. 높은 실루엣 점수만으로 픽셀 완전 일치로 보고하지 않는다.

## Source와 실행 기록

Production source는 `9b53bd699126205b7a93fff326167f575252f1ea`에 커밋한 v86 retry2다. Native binary SHA-256 `a41dbbebdf51f7c2fa928c6f2a7d5bd80a80a91bb9c904b3320fbd1afed8f2e2`, WASM SHA-256 `ab5e01324ad024dd34979f03a08d2da6844d26ff31c49069c17de0fe849779b6`다. 실제 빌드는 부모3874 위 당시 미커밋 변경에서 수행했다. Native v91 retry1은 96dpi/print profile로 실행했으며 SVG에 실제 HCRBatang 원본 TTF SHA8664669bd2d6…가 포함됨을 확인했다. CSS face 이름만으로 공급 완료를 판정하지 않았다. CanvasKit은 같은 HCR Batang face를 직접 등록했고 SHA-256 `8664669bd2d6a4fe8b7538706d2bb2a1004c074aa02dca59051e515e731294f5`다. registeredface1/fallback0/unsupported0/rendererror0/hiddenCanvas2dOverlayfalse를 확인했다.

정확한 명령·원본/Print/font SHA·gate는 ignored `output/pr-review/semanticist21-20261008/native-actual-original-faces-patterns-six-v91-retry1-command.json`, `native-actual-original-faces-patterns-six-v91-retry1/patterns6/run_manifest.json`, `browser-patterns-supplied-font-v88.json`, `actual-pattern-backends-supplied-font-v88/metrics.json`에 있다. Native sweep terminal exit0/39.365초/source불변이며 1대1쪽/gatepassed다. TSV·로그·중간 JSON은 이 asset 경로에 커밋하지 않는다.

Native Skia의 직접 PNG paint 또는 다른 편집 입력/백엔드의 지원 증거로 확대하지 않는다. 새 test/golden/baseline은 추가하지 않았다. 초기18개 전체 PR의 완료·최종 통합 head 검증·CI/merge와는 별도다. source가 바뀌면 영향을 받는 범위를 새 head에서 다시 실행한다.
