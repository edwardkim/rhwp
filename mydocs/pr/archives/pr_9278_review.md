# PR #9278 리뷰 — fix(typeset): 나뉜 표 끝 조각 바로 아래에 조판된 빈 줄은 옛 좌표 안내 줄로 감추지 않는다 (#9272)

## #9316 리베이스 후 주장 검증

고정 기준은 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`, 검증한 제품 소스는 `954226e426517dc02ef0a15cc739899ab54b7fae`다. 같은 원본과 독립 한컴 Print로 주장 문서를 전쪽 대조했다.
아래의 이전 검토 자료는 각각 명시한 실행 소스의 기록이다.

| 입력 | 경로 | 쪽수 | 최저값 | 대표 쪽 | 전쪽 TSV 및 이미지 |
| --- | --- | ---: | ---: | ---: | --- |
| 9272 | native | 7 | 99.98% | 7 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/9272/native/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/9272/native/review/review_007.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/9272/native/overlay/overlay_007.png) |
| 9272 | wasm | 7 | 99.98% | 7 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/9272/wasm/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/9272/wasm/review/review_007.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/9272/wasm/overlay/overlay_007.png) |

합성 `fallback.ttf`와 `RHWPHftEmptyPoint.otf` 및 전용 검사는 PR 커밋 범위에서 제거했다. [글꼴 공급 및 출력 증거](../assets/planet6897_9240_9282_20261121/after-9316/README.md)

## 최종 판정

**통합 검증 완료.** 원 PR의 변경 경계와 전체 Rust 회귀를 확인했다. 동일 공급 조건의 정본 보호 문서 46조건·4,046쪽에서 기존 보호값 및 고정 기준 대비 하락은 0쪽이다. [통합 전쪽 대조](../assets/planet6897_9240_9282_20261121/protected-candidate-v5/fixed-base-comparison.json)

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

## 최신 Native·fresh WASM 전쪽 확인

제품 소스 `65b6fe383c3bd8cbcf83de7909c7a6c24eb32c35`의 Native와 fresh WASM을 각각 사용해 전쪽 7쪽을 다시 출력했다. 공식 두 자리 raw 실루엣의 최저값은 Native **99.98%**, fresh WASM **99.98%**다. 이전 검토에서 비교한 모든 쪽의 RHWP 및 Print PNG 픽셀, TSV가 이번 결과와 일치했다. [최신 전쪽 비교 및 생산물 SHA](../assets/planet6897_9240_9282_20261121/validation/claims-v5.json)

## Merge 후 contributor PR comment 계획

- [#9272](https://github.com/edwardkim/rhwp/issues/9272) 재현 문서의 6쪽 분할 표 아래 빈 줄과 7쪽 후속 표의 소속을 한컴 2024 Hancom PDF Print와 비교한 결과를 적는다. 6·7쪽 review와 overlay를 함께 표시한다.
- 빈 줄을 흡수해 두 표가 겹치던 경계와 옛 저장 좌표의 안내 줄을 구분하는 검사 결과를 기록한다. 원 PR이 별도 결함으로 연결한 [#9274](https://github.com/edwardkim/rhwp/issues/9274)의 상태는 마무리 시점에 따로 확인한다.

[Visual Sweep 공식 코멘트 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 연결한다. 통합 merge SHA에 실제 포함된 원본 출력·한컴 PDF 래스터·review·overlay만 사용한다. 최신 CI와 원본 및 Print 영수증, 실제 확인한 쪽수와 자동 비교 수치, 직접 판독 결론을 함께 기록한다. 게시 전에 PNG 원격 응답과 검증 자산의 SHA를 대조한다.
