# PR #9282 리뷰 — 글자처럼 취급 표 줄 뒤 같은 문단의 도형 줄이 있으면 문단 뒤 간격을 마지막 줄 뒤에 둔다 (#9279)

## 최종 판정

**머지 보류 — 검증 진행 중.** 원 PR의 시각 검증과 변경 경계의 집중 회귀를 마쳤다. 전체 Rust 검증은 통과했고 정본 보호 문서의 기존 점수 재현을 확인 중이다.

## 접수 정보

- [원 PR #9282](https://github.com/edwardkim/rhwp/pull/9282) · planet6897 · devel
- 원 head `99c71c1ea73fd9c319bc60e5f1b96eb95a688d3c` · 지정 리뷰어 `jangster77`
- 통합 비교 기준 `d7b5286cbf6401678a70eed3d66236a68dd2a72c`
- 원 커밋 `03a5a9cbcad2b177703e4dfa7fe45e6d36b4024d` → 통합 커밋 `17caef6163e97174be42fd8e7cb8072f9deae810`
- 원 커밋 `99c71c1ea73fd9c319bc60e5f1b96eb95a688d3c` → 통합 커밋 `9751f5edec301f4faf4562c8a691f3c6b8d872bb`

## 변경과 검토 범위

2쪽에서 글자처럼 취급된 표 줄과 도형 줄의 간격 및 문단 뒤 간격을 확인한다.

## 검증 입력과 결과

- 원본 `samples/issue9279/forest-press-release-tac-table-shape-tail.hwpx` · SHA-256 `4763ea3fac592f39dccc36808d022edd0be57db2746e38a50cb9dfe5e6726ea9`
- 기준 `pdf/issue9279/forest-press-release-tac-table-shape-tail-2024.pdf` · 3쪽 · SHA-256 `1a5a6c2d74197b6e691807b86e1bf3f346779abe623f2ee7e687054de5bde097`
- Windows 데스크톱 한컴 2024 `13.0.0.3901` Hancom PDF Print · MCP 작업 `996a401a-f7c0-44d7-9f2c-7484fcca303a`. [인쇄 영수증](../assets/planet6897_9240_9282_20261121/mcp-print-verification.json)
- 웹한글기안기 PDF는 공식 비교 자료로 사용하지 않는다.


`cef9ff9acc08f92431e89450945462514fffa772` 제품 코드의 Visual Sweep 96 DPI·2px raw 실루엣을 확인했다. Native 전쪽 최저 **100.00%**, fresh WASM 전쪽 최저 **100.00%**다. Native와 fresh WASM 각각 고정 base와 후보의 전쪽에서 2%p 초과 하락은 0쪽이다. [Native 기준·후보 TSV](../assets/planet6897_9240_9282_20261121/native-base-comparison/results.json) · [fresh WASM 기준·후보 TSV](../assets/planet6897_9240_9282_20261121/wasm-base-comparison/results.json) · [Native 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9279/native/silhouette.tsv) · [fresh WASM 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9279/wasm/silhouette.tsv)

## 시각 증적과 남은 차이

| 쪽 | Native raw | fresh WASM raw | 시각 증거 |
| ---: | ---: | ---: | --- |
| 2 | 100.00% | 100.00% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9279/native/review_002.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9279/native/overlay_002.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9279/wasm/review_002.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9279/wasm/overlay_002.png) |

2쪽에서 표와 사각형이 한 번씩 그려지고 문단 뒤 간격은 두 줄 뒤에 적용된다. Native와 fresh WASM의 전체 3쪽이 100.00%이며 2쪽 표와 도형 위 경계를 ±1px로 검사한다.

최소 글꼴 환경 집중 회귀 10건을 통과했다. [회귀 실행 기록](../assets/planet6897_9240_9282_20261121/coordinate-regression-validation.json) · [독립 기대 좌표](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9279/evidence.json) · [원인과 메인터너 보정](../assets/planet6897_9240_9282_20261121/maintainer-repair.md)

통합 회귀 13,874건과 Native Skia 라이브러리 4,034건을 통과했다. [Rust 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/rust.json)

실제 Studio에서 주장 경계 9쪽의 화면·인쇄 출력을 확인했다. Canvas2D 실제 선택 50건 중 32건은 CanvasKit 글꼴 준비 실패에 따른 정상 대체이고, CanvasKit compat 2건과 default 2건은 각 backend가 직접 렌더했다. 브라우저 초기화 응답 54건의 WASM 해시를 빌드와 대조했다. [Studio 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/actual-studio.json)

## Merge 후 원 PR 코멘트

통합 merge SHA의 원본 출력·한컴 PDF raster·review·overlay를 해당 주장 페이지별로 고정하여 본문에 표시한다. 실제 CI와 사용한 한컴 Print 제품 및 MCP 영수증을 함께 남긴다.
