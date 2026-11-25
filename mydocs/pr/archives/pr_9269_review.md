# PR #9269 리뷰 — fix(layout): 캡션 문단에 걸린 떠 있는 표를 그리고, 나눌 수 없는 한 행 자리차지 표는 저장된 다음 쪽으로 통째 넘긴다 (#9239)

## #9316 리베이스 후 주장 검증

고정 기준은 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`, 검증한 제품 소스는 `954226e426517dc02ef0a15cc739899ab54b7fae`다. 같은 원본과 독립 한컴 Print로 주장 문서를 전쪽 대조했다.
아래의 이전 검토 자료는 각각 명시한 실행 소스의 기록이다.

| 입력 | 경로 | 쪽수 | 최저값 | 대표 쪽 | 전쪽 TSV 및 이미지 |
| --- | --- | ---: | ---: | ---: | --- |
| 9239 | native | 62 | 98.96% | 43 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/9239/native/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/9239/native/review/review_043.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/9239/native/overlay/overlay_043.png) |
| 9239 | wasm | 62 | 98.96% | 43 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/9239/wasm/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/9239/wasm/review/review_043.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/9239/wasm/overlay/overlay_043.png) |

합성 `fallback.ttf`와 `RHWPHftEmptyPoint.otf` 및 전용 검사는 PR 커밋 범위에서 제거했다. [글꼴 공급 및 출력 증거](../assets/planet6897_9240_9282_20261121/after-9316/README.md)

## 최종 판정

**통합 검증 완료.** 원 PR의 변경 경계와 전체 Rust 회귀를 확인했다. 동일 공급 조건의 정본 보호 문서 46조건·4,046쪽에서 기존 보호값 및 고정 기준 대비 하락은 0쪽이다. [통합 전쪽 대조](../assets/planet6897_9240_9282_20261121/protected-candidate-v5/fixed-base-comparison.json)

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

최신 Native에서 OLE·EMF 보정의 원본 출력과 기준 이미지를 같은 전쪽 실행에서 보존했다. [SHA 검증 기록](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/ole-claim-media-v5.json)

- 15쪽 [Native 원본 PNG](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/raw_015.png) · [한컴 Print PNG](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/reference_015.png).
- 16쪽 [Native 원본 PNG](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/raw_016.png) · [한컴 Print PNG](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/reference_016.png).
- 21쪽 [Native 원본 PNG](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/raw_021.png) · [한컴 Print PNG](../assets/planet6897_9240_9282_20261121/coordinate-evidence/9239/native/reference_021.png).

## 최신 Native·fresh WASM 전쪽 확인

제품 소스 `65b6fe383c3bd8cbcf83de7909c7a6c24eb32c35`의 Native와 fresh WASM을 각각 사용해 전쪽 62쪽을 다시 출력했다. 공식 두 자리 raw 실루엣의 최저값은 Native **98.87%**, fresh WASM **98.86%**다. 이전 검토에서 비교한 모든 쪽의 RHWP 및 Print PNG 픽셀, TSV가 이번 결과와 일치했다. [최신 전쪽 비교 및 생산물 SHA](../assets/planet6897_9240_9282_20261121/validation/claims-v5.json)

## Merge 후 contributor PR comment 계획

- [#9239](https://github.com/edwardkim/rhwp/issues/9239) 원본의 12쪽 제목 표와 43쪽 캡션에 걸린 떠 있는 표를 한컴 2020 Hancom PDF Print에서 대조한다. 11쪽 각주 영역을 제목 표가 침범하지 않고 43쪽의 내부 표가 한 번만 보이는지 확인한 결과를 적는다.
- 볼 수 있는 경계가 각각 달라 11·12·43쪽과 메인터너 보정의 15·16·21쪽을 따로 표시한다. OLE와 EMF 글자 위치를 보정한 이유, 두 번 실제 편집·저장·재열기로 OLE 참조를 확인한 결과를 포함한다.

[Visual Sweep 공식 코멘트 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 연결한다. 통합 merge SHA에 실제 포함된 원본 출력·한컴 PDF 래스터·review·overlay만 사용한다. 최신 CI와 원본 및 Print 영수증, 실제 확인한 쪽수와 자동 비교 수치, 직접 판독 결론을 함께 기록한다. 게시 전에 PNG 원격 응답과 검증 자산의 SHA를 대조한다.
