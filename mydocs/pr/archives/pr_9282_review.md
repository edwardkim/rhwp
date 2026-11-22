# PR #9282 리뷰 — 글자처럼 취급 표 줄 뒤 같은 문단의 도형 줄이 있으면 문단 뒤 간격을 마지막 줄 뒤에 둔다 (#9279)

## #9316 리베이스 후 주장 검증

고정 기준은 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`, 검증한 제품 소스는 `954226e426517dc02ef0a15cc739899ab54b7fae`다. 같은 원본과 독립 한컴 Print로 주장 문서를 전쪽 대조했다.
아래의 이전 검토 자료는 각각 명시한 실행 소스의 기록이다.

| 입력 | 경로 | 쪽수 | 최저값 | 대표 쪽 | 전쪽 TSV 및 이미지 |
| --- | --- | ---: | ---: | ---: | --- |
| 9279 | native | 3 | 100.00% | 2 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/9279/native/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/9279/native/review/review_002.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/9279/native/overlay/overlay_002.png) |
| 9279 | wasm | 3 | 100.00% | 2 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/9279/wasm/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/9279/wasm/review/review_002.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/9279/wasm/overlay/overlay_002.png) |

합성 `fallback.ttf`와 `RHWPHftEmptyPoint.otf` 및 전용 검사는 PR 커밋 범위에서 제거했다. [글꼴 공급 및 출력 증거](../assets/planet6897_9240_9282_20261121/after-9316/README.md)

## 최종 판정

**통합 검증 완료.** 원 PR의 변경 경계와 전체 Rust 회귀를 확인했다. 동일 공급 조건의 정본 보호 문서 46조건·4,046쪽에서 기존 보호값 및 고정 기준 대비 하락은 0쪽이다. [통합 전쪽 대조](../assets/planet6897_9240_9282_20261121/protected-candidate-v5/fixed-base-comparison.json)

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

최신 제품 코드 `19646e69fc9`의 Studio에서 2쪽 화면·인쇄 6건을 다시 확인했다. Canvas2D 실제 출력은 2건이며 그중 0건은 CanvasKit 요청이 글꼴 준비 단계에서 대체된 결과다. CanvasKit 직접 출력은 4건이다. 전체 60건의 초기화 응답이 새 WASM 해시와 일치했다. [Studio 최신 빌드 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/actual-studio-v4.json)

## 최신 Native·fresh WASM 전쪽 확인

제품 소스 `65b6fe383c3bd8cbcf83de7909c7a6c24eb32c35`의 Native와 fresh WASM을 각각 사용해 전쪽 3쪽을 다시 출력했다. 공식 두 자리 raw 실루엣의 최저값은 Native **100.00%**, fresh WASM **100.00%**다. 이전 검토에서 비교한 모든 쪽의 RHWP 및 Print PNG 픽셀, TSV가 이번 결과와 일치했다. [최신 전쪽 비교 및 생산물 SHA](../assets/planet6897_9240_9282_20261121/validation/claims-v5.json)

## Merge 후 contributor PR comment 계획

- [#9279](https://github.com/edwardkim/rhwp/issues/9279) 재현 문서 2쪽에서 담당 부서 표와 같은 문단에 있는 로고 줄의 위 위치를 한컴 2024 Hancom PDF Print와 대조한다. 표와 로고가 함께 보이는 2쪽 review와 overlay를 표시한다.
- 문단 뒤 간격이 마지막 저장 줄 뒤에 놓이고 표 바로 뒤에 추가되지 않는 관계를 확인한 결과를 적는다. 3쪽 전쪽의 Native와 fresh WASM 수치, Studio의 실제 backend 결과를 각각 표시한다.

[Visual Sweep 공식 코멘트 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 연결한다. 통합 merge SHA에 실제 포함된 원본 출력·한컴 PDF 래스터·review·overlay만 사용한다. 최신 CI와 원본 및 Print 영수증, 실제 확인한 쪽수와 자동 비교 수치, 직접 판독 결론을 함께 기록한다. 게시 전에 PNG 원격 응답과 검증 자산의 SHA를 대조한다.
