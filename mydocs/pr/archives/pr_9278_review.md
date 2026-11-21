# PR #9278 리뷰 — fix(typeset): 나뉜 표 끝 조각 바로 아래에 조판된 빈 줄은 옛 좌표 안내 줄로 감추지 않는다 (#9272)

## 최종 판정

**머지 보류 — 검증 진행 중.** 원 PR의 시각 검증과 변경 경계의 집중 회귀를 마쳤다. 전체 Rust 검증은 통과했고 정본 보호 문서의 기존 점수 재현을 확인 중이다.

## 접수 정보

- [원 PR #9278](https://github.com/edwardkim/rhwp/pull/9278) · planet6897 · devel
- 원 head `083da555605faa5ac9e70108481d204c27093fc6` · 지정 리뷰어 `jangster77`
- 통합 비교 기준 `d7b5286cbf6401678a70eed3d66236a68dd2a72c`
- 원 커밋 `5b8d969f77d9c0f25ec9b088160ced637be741d9` → 통합 커밋 `7fe1b8ec03002c72f5336f5d477fa6f64775ae08`
- 원 커밋 `083da555605faa5ac9e70108481d204c27093fc6` → 통합 커밋 `b42b10b97bfe24183305ee44efd9fe710bbf749f`

## 변경과 검토 범위

6쪽에서 이어받은 표 아래 빈 줄의 소속과 7쪽에서 다시 시작하는 후속 표를 확인한다.

## 검증 입력과 결과

- 원본 `samples/issue9272/trade-remedy-decree-table-tail-empty-lines.hwpx` · SHA-256 `37624fffaa34704360298c9edfc3d636f5a0fabd0db040a7bc6a161b73a1301d`
- 기준 `pdf/issue9272/trade-remedy-decree-table-tail-empty-lines-2024.pdf` · 7쪽 · SHA-256 `25f1443410590604f527e07739b0b33f3f34ba5d207e85b3f2fccac87c859fc7`
- Windows 데스크톱 한컴 2024 `13.0.0.3901` Hancom PDF Print · MCP 작업 `874933b1-3886-446d-aa8e-6778590d7d2d`. [인쇄 영수증](../assets/planet6897_9240_9282_20261121/mcp-print-verification.json)
- 웹한글기안기 PDF는 공식 비교 자료로 사용하지 않는다.


`cef9ff9acc08f92431e89450945462514fffa772` 제품 코드의 Visual Sweep 96 DPI·2px raw 실루엣을 확인했다. Native 전쪽 최저 **99.98%**, fresh WASM 전쪽 최저 **99.98%**다. Native와 fresh WASM 각각 고정 base와 후보의 전쪽에서 2%p 초과 하락은 0쪽이다. [Native 기준·후보 TSV](../assets/planet6897_9240_9282_20261121/native-base-comparison/results.json) · [fresh WASM 기준·후보 TSV](../assets/planet6897_9240_9282_20261121/wasm-base-comparison/results.json) · [Native 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/native/silhouette.tsv) · [fresh WASM 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/wasm/silhouette.tsv)

## 시각 증적과 남은 차이

| 쪽 | Native raw | fresh WASM raw | 시각 증거 |
| ---: | ---: | ---: | --- |
| 6 | 100.00% | 100.00% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/native/review_006.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/native/overlay_006.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/wasm/review_006.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/wasm/overlay_006.png) |
| 7 | 100.00% | 100.00% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/native/review_007.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/native/overlay_007.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/wasm/review_007.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/wasm/overlay_007.png) |

6쪽에서 표 아래 빈 줄을 보존했고 7쪽이 후속 표를 소유한다. 두 쪽의 Native와 fresh WASM이 모두 100.00%이며 독립 Print의 두 표 아래 경계를 ±1px로 검사한다.

최소 글꼴 환경 집중 회귀 10건을 통과했다. [회귀 실행 기록](../assets/planet6897_9240_9282_20261121/coordinate-regression-validation.json) · [독립 기대 좌표](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9272/evidence.json) · [원인과 메인터너 보정](../assets/planet6897_9240_9282_20261121/maintainer-repair.md)

통합 회귀 13,874건과 Native Skia 라이브러리 4,034건을 통과했다. [Rust 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/rust.json)

실제 Studio에서 주장 경계 9쪽의 화면·인쇄 출력을 확인했다. Canvas2D 실제 선택 50건 중 32건은 CanvasKit 글꼴 준비 실패에 따른 정상 대체이고, CanvasKit compat 2건과 default 2건은 각 backend가 직접 렌더했다. 브라우저 초기화 응답 54건의 WASM 해시를 빌드와 대조했다. [Studio 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/actual-studio.json)

최신 제품 코드 `19646e69fc9`의 Studio에서 6·7쪽 화면·인쇄 12건을 다시 확인했다. Canvas2D 실제 출력은 12건이며 그중 8건은 CanvasKit 요청이 글꼴 준비 단계에서 대체된 결과다. 전체 60건의 초기화 응답이 새 WASM 해시와 일치했다. [Studio 최신 빌드 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/actual-studio-v4.json)

## Merge 후 원 PR 코멘트

통합 merge SHA의 원본 출력·한컴 PDF raster·review·overlay를 해당 주장 페이지별로 고정하여 본문에 표시한다. 실제 CI와 사용한 한컴 Print 제품 및 MCP 영수증을 함께 남긴다.
