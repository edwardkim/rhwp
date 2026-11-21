# PR #9269 리뷰 — fix(layout): 캡션 문단에 걸린 떠 있는 표를 그리고, 나눌 수 없는 한 행 자리차지 표는 저장된 다음 쪽으로 통째 넘긴다 (#9239)

## 최종 판정

**머지 보류 — 검증 진행 중.** 원 PR의 시각 검증과 변경 경계의 집중 회귀를 마쳤다. 전체 Rust 검증은 통과했고 정본 보호 문서의 기존 점수 재현을 확인 중이다.

## 접수 정보

- [원 PR #9269](https://github.com/edwardkim/rhwp/pull/9269) · planet6897 · devel
- 원 head `e22e6e3e97b089a55cab83e3cec8dcb1e0d236d7` · 지정 리뷰어 `jangster77`
- 통합 비교 기준 `d7b5286cbf6401678a70eed3d66236a68dd2a72c`
- 원 커밋 `2a10ad72e9361ba11ef0d5e7d5bc25288b573f04` → 통합 커밋 `aee67fee11936144f8862531e438cde906115d11`
- 원 커밋 `e22e6e3e97b089a55cab83e3cec8dcb1e0d236d7` → 통합 커밋 `ca9cca0f5c93fd8f3b63a47ee5a8ff3b5319f839`

## 변경과 검토 범위

43쪽의 캡션 안 표와 11→12쪽의 단일 행 표 전체 이동 및 각주 영역을 확인한다.

## 검증 입력과 결과

- 원본 `samples/issue9239/senior-employment-policy-caption-float-table.hwp` · SHA-256 `7ee3c3ef43b95204383b800fbe36644e7eb5730f399065ebc90bab3c269082a2`
- 기준 `pdf/issue9239/senior-employment-policy-caption-float-table-hancom-2020.pdf` · 62쪽 · SHA-256 `2f7fbdd3cc9f9fe94b02b8ad524667fec96672609e324ec2fe6bfba7698f6151`
- Windows 데스크톱 한컴 2020 `11.0.0.9136` Hancom PDF Print · MCP 작업 `08562403-37a1-4c73-a95b-ad9b5bb8a4c1`. [인쇄 영수증](../assets/planet6897_9240_9282_20261121/mcp-print-verification.json)
- 웹한글기안기 PDF는 공식 비교 자료로 사용하지 않는다.

첫 한컴 2024 인쇄본에서 HCI 글자 프로그램이 빠진 페이지를 확인했다. 정상 출력한 같은 입력의 한컴 2020 Hancom PDF Print를 비교 기준으로 고정했다.

`cef9ff9acc08f92431e89450945462514fffa772` 제품 코드의 Visual Sweep 96 DPI·2px raw 실루엣을 확인했다. Native 전쪽 최저 **98.87%**, fresh WASM 전쪽 최저 **98.86%**다. Native와 fresh WASM 각각 고정 base와 후보의 전쪽에서 2%p 초과 하락은 0쪽이다. [Native 기준·후보 TSV](../assets/planet6897_9240_9282_20261121/native-base-comparison/results.json) · [fresh WASM 기준·후보 TSV](../assets/planet6897_9240_9282_20261121/wasm-base-comparison/results.json) · [Native 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/silhouette.tsv) · [fresh WASM 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/silhouette.tsv)

## 시각 증적과 남은 차이

| 쪽 | Native raw | fresh WASM raw | 시각 증거 |
| ---: | ---: | ---: | --- |
| 11 | 99.99% | 99.99% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/review_011.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/overlay_011.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/review_011.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/overlay_011.png) |
| 12 | 99.97% | 99.97% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/review_012.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/overlay_012.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/review_012.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/overlay_012.png) |
| 15 | 99.98% | 99.89% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/review_015.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/overlay_015.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/review_015.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/overlay_015.png) |
| 16 | 99.97% | 99.97% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/review_016.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/overlay_016.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/review_016.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/overlay_016.png) |
| 21 | 99.94% | 99.98% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/review_021.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/overlay_021.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/review_021.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/overlay_021.png) |
| 43 | 99.56% | 99.61% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/review_043.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/overlay_043.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/review_043.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/wasm/overlay_043.png) |

43쪽의 캡션 표와 11→12쪽에 걸린 제목 표가 원 PR의 변경 범위다. 12·16쪽 굴림체 작은 네모와 15·21쪽 OLE 프레임·내부 글줄을 메인터너가 추가 보정했다. 12·15·21·43쪽의 독립 좌표는 ±2px로 보호한다. 24쪽에 남은 미세 차이도 전쪽 TSV에 보존했다.

최소 글꼴 환경 집중 회귀 10건을 통과했다. [회귀 실행 기록](../assets/planet6897_9240_9282_20261121/coordinate-regression-validation.json) · [독립 기대 좌표](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/evidence.json) · [원인과 메인터너 보정](../assets/planet6897_9240_9282_20261121/maintainer-repair.md)

통합 회귀 13,874건과 Native Skia 라이브러리 4,034건을 통과했다. [Rust 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/rust.json)

실제 Studio에서 주장 경계 9쪽의 화면·인쇄 출력을 확인했다. Canvas2D 실제 선택 50건 중 32건은 CanvasKit 글꼴 준비 실패에 따른 정상 대체이고, CanvasKit compat 2건과 default 2건은 각 backend가 직접 렌더했다. 브라우저 초기화 응답 54건의 WASM 해시를 빌드와 대조했다. [Studio 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/actual-studio.json)

최신 제품 코드 `19646e69fc9`의 Studio에서 12·15·21·43쪽 화면·인쇄 24건을 다시 확인했다. Canvas2D 실제 출력은 24건이며 그중 16건은 CanvasKit 요청이 글꼴 준비 단계에서 대체된 결과다. 전체 60건의 초기화 응답이 새 WASM 해시와 일치했다. [Studio 최신 빌드 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/actual-studio-v4.json)

### OLE 편집·저장 보존

같은 제품 코드의 CLI에서 검증 사본을 편집해 저장하고, 다시 열어 편집·재저장한 뒤 한 번 더 열었다.
HWP는 두 저장본 모두 원본의 26바이트 OLE 레코드 두 개와 BinData 바이트를 보존했다.
HWP에서 만든 HWPX도 같은 순서로 두 번 저장했다. OLE 인스턴스 ID, 크기,
테두리, BinData 참조와 바이트가 유지됐다. 두 형식에서 모두 편집 문자열이 최종 재열기 때 한 번만
검색됐으며, 각 저장의 `--verify` 결과도 IR 차이 0건이었다.
[반복 저장과 독립 CFB·ZIP 검사 결과](../assets/planet6897_9240_9282_20261121/validation/ole-repeat-save-v4.json)

이중선 보정을 반영한 제품 코드 `65b6fe383c`에서도 같은 저장 순서를 재실행했다. HWP·HWPX의
저장본 5개가 독립 CFB·ZIP 검사를 마친 기존 산출물과 바이트 단위로 같았다.
[최신 Native 반복 저장 영수증](../assets/planet6897_9240_9282_20261121/validation/ole-repeat-save-v5.json)

## Merge 후 원 PR 코멘트

통합 merge SHA의 원본 출력·한컴 PDF raster·review·overlay를 해당 주장 페이지별로 고정하여 본문에 표시한다. 실제 CI와 사용한 한컴 Print 제품 및 MCP 영수증을 함께 남긴다.
