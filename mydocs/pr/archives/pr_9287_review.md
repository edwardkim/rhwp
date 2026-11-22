# PR #9287 리뷰 — 색상표 없는 WMF 1비트 DIB의 레코드 경계 보존 (#9285)

## #9316 리베이스 후 주장 검증

고정 기준은 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`, 검증한 제품 소스는 `954226e426517dc02ef0a15cc739899ab54b7fae`다. 같은 원본과 독립 한컴 Print로 주장 문서를 전쪽 대조했다.
아래의 이전 검토 자료는 각각 명시한 실행 소스의 기록이다.

| 입력 | 경로 | 쪽수 | 최저값 | 대표 쪽 | 전쪽 TSV 및 이미지 |
| --- | --- | ---: | ---: | ---: | --- |
| 9285 | native | 9 | 99.78% | 4 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/9285/native/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/9285/native/review/review_004.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/9285/native/overlay/overlay_004.png) |
| 9285 | wasm | 9 | 99.66% | 4 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/9285/wasm/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/9285/wasm/review/review_004.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/9285/wasm/overlay/overlay_004.png) |

합성 `fallback.ttf`와 `RHWPHftEmptyPoint.otf` 및 전용 검사는 PR 커밋 범위에서 제거했다. [글꼴 공급 및 출력 증거](../assets/planet6897_9240_9282_20261121/after-9316/README.md)

## 판정

**원 PR 변경 검증 완료.** 고정 devel과 통합 후보에서 원본 전체 9쪽의 SVG를 대조했다. 4쪽의 WMF 변환을
복구했고 나머지 8쪽 SVG는 바이트 단위로 같다. 공식 한컴 Print 전쪽 최저값은 Native 99.78%, fresh WASM
99.66%다.

**통합 검증 완료.** 원 PR의 변경 경계와 전체 Rust 회귀를 확인했다. 동일 공급 조건의 정본 보호 문서 46조건·4,046쪽에서 기존 보호값 및 고정 기준 대비 하락은 0쪽이다. [통합 전쪽 대조](../assets/planet6897_9240_9282_20261121/protected-candidate-v5/fixed-base-comparison.json)

## 접수와 통합

- [원 PR #9287](https://github.com/edwardkim/rhwp/pull/9287) · planet6897 · devel · 리뷰어 `jangster77`
- 관련 이슈 [#9285](https://github.com/edwardkim/rhwp/issues/9285)
- 원 head `4e0c590f92a0c9688536b6adda5fbb7684b382c0` · 고정 devel `d7b5286cbf6401678a70eed3d66236a68dd2a72c`
- 원 코드 `adfc01ad8c0a68837c72a5aed0535d4a20c0167b` → 통합 `3fb3913b9e57f97623bced9522cb16fe3431e81e`
- 원 회귀와 fixture `4e0c590f92a0c9688536b6adda5fbb7684b382c0` → 통합 `19646e69fc99ba76f9ff0350c2a425fe07ec1e88`

두 커밋을 충돌 없이 적용했다. 본문 조판 대신 원본 WMF를 읽고 그리는 경로가 바뀌므로 그림이 놓인
4쪽과 기존 변환이 유지돼야 하는 나머지 쪽을 함께 확인한다.

## 원인과 구현 검토

원본 HWPX의 `BinData/image7.wmf`를 직접 읽었다. 512×512 1비트 마스크 네 건 모두 DIB 헤더와
픽셀을 제외한 레코드 안의 색상표 공간이 0바이트다. `SRCPAINT`와 `SRCAND` 레코드가 네 쌍이다.
색상표 크기는 DIB 헤더와 이를 담은 레코드를 함께 봐야 한다. 통합 코드는 레코드 내의 색상표 공간으로
항목 수를 제한하고, 색상표 없는 1비트 DIB는 검정·흰색으로 처리한다.
[원본 레코드 확인](../assets/planet6897_9240_9282_20261121/wmf-9285/source-records.json) ·
[MS-WMF §2.2.2.9](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/7376542a-cce9-4625-8ead-585e9538f9f1)

고정 devel에서는 4쪽 SVG에 변환되지 않은 `image/x-wmf`가 한 개 있었다. 통합 후보의 4쪽 SVG에서
그 항목이 사라졌고 나머지 쪽에는 변경이 없었다.
[SVG 전쪽 구조 대조](../assets/planet6897_9240_9282_20261121/wmf-9285/svg-structure.json)

## 공식 비교 입력

- [공개 원본 HWPX](../../../samples/issue9285/child-product-safety-plan-wmf-mask-dib.hwpx) · 9쪽 · SHA-256 `45d1785c7c6db19dc595af3a3a66b5f60d820407db16331d3a43403b01ede8eb`
- [기준 PDF](../../../pdf/issue9285/child-product-safety-plan-wmf-mask-dib-2020.pdf) · Windows 데스크톱 한컴 2020 `11.0.0.9136` Hancom PDF Print · 9쪽 · SHA-256 `d1da90986b7aeffff25765075c4e3c1a5f3d70755af13a1fa060be861610554b`
- 첨부된 한컴 2022 PDF를 확인한 뒤 같은 원본을 한컴 2024로 인쇄했다. 2024 결과의 텍스트 추출에서
  3·7·9쪽 영문·숫자 56자가 빠져 원본 저장 제품과 같은 한컴 2020으로 다시 인쇄했다. 2020 결과의 쪽별 영문·숫자·한글
  문자 집계는 2022 참고본과 같다. [인쇄 영수증과 문자 대조](../assets/planet6897_9240_9282_20261121/wmf-9285/print-verification.json)

통합 시각 검증은 공식 2020 Print를 기준으로 Visual Sweep `--compat 2022 --dpi 96`의 2px raw 실루엣을
비교한다. 원 PR의 한컴 2022 기준 측정은 해당 head의 참고 이력이다.

## 통합 Native 검증

고정 devel과 WMF 통합 후보에 동일한 입력·Print PDF·글꼴 공급과 래스터 조건을 적용했다. 4쪽은
**86.99% → 99.93%**이며, 후보 전쪽 최저는 **9쪽 99.78%**다. 기준보다 하락한 쪽은 없다.
4쪽 하단의 그림 네 조각이 모두 보이며 기준 PDF의 순서와 구성을 유지한다.
[Native 검증 영수증](../assets/planet6897_9240_9282_20261121/wmf-9285/native/validation.json) ·
[기준 TSV](../assets/planet6897_9240_9282_20261121/wmf-9285/native/base-silhouette.tsv) ·
[후보 TSV](../assets/planet6897_9240_9282_20261121/wmf-9285/native/silhouette.tsv)

- 고정 devel 4쪽 [review](../assets/planet6897_9240_9282_20261121/wmf-9285/native/base-review-004.png) ·
  [overlay](../assets/planet6897_9240_9282_20261121/wmf-9285/native/base-overlay-004.png)
- 통합 후보 4쪽 [review](../assets/planet6897_9240_9282_20261121/wmf-9285/native/review-004.png) ·
  [overlay](../assets/planet6897_9240_9282_20261121/wmf-9285/native/overlay-004.png) ·
  [실제 출력](../assets/planet6897_9240_9282_20261121/wmf-9285/native/raw-004.png) ·
  [한컴 Print 래스터](../assets/planet6897_9240_9282_20261121/wmf-9285/native/pdf-004.png)

## 통합 fresh WASM 검증

고정 devel의 Canvas 내보내기는 그림 디코딩 오류로 중단돼 전쪽 점수가 없다. 통합 후보는 새로 빌드한
WASM을 사용해 9쪽을 모두 출력했다. 4쪽은 **99.80%**, 후보 전쪽 최저는 **99.66%**다. 4쪽 하단의
그림 네 조각을 실제 출력과 한컴 Print에서 각각 확인했다.
[WASM 검증 영수증](../assets/planet6897_9240_9282_20261121/wmf-9285/wasm/validation.json) ·
[후보 TSV](../assets/planet6897_9240_9282_20261121/wmf-9285/wasm/silhouette.tsv)

- 통합 후보 4쪽 [review](../assets/planet6897_9240_9282_20261121/wmf-9285/wasm/review-004.png) ·
  [overlay](../assets/planet6897_9240_9282_20261121/wmf-9285/wasm/overlay-004.png) ·
  [실제 출력](../assets/planet6897_9240_9282_20261121/wmf-9285/wasm/raw-004.png) ·
  [한컴 Print 래스터](../assets/planet6897_9240_9282_20261121/wmf-9285/wasm/pdf-004.png)

최소 글꼴 환경에서 이 PR의 두 검사를 포함한 집중 회귀 12건을 통과했다. 신규 HWPX의 세 탐지기 검사도
한 건을 실제로 실행해 통과했다.

## 실제 Studio 출력

최신 제품 코드 `19646e69fc9`의 Studio에서 4쪽 화면·인쇄 6건을 다시 확인했다. Canvas2D 실제 출력은 6건이며 그중 4건은 CanvasKit 요청이 글꼴 준비 단계에서 대체된 결과다. 전체 60건의 초기화 응답이 새 WASM 해시와 일치했다. [Studio 최신 빌드 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/actual-studio-v4.json)

## 최신 Native·fresh WASM 전쪽 확인

제품 소스 `65b6fe383c3bd8cbcf83de7909c7a6c24eb32c35`의 Native와 fresh WASM을 각각 사용해 전쪽 9쪽을 다시 출력했다. 공식 두 자리 raw 실루엣의 최저값은 Native **99.78%**, fresh WASM **99.66%**다. 이전 검토에서 비교한 모든 쪽의 RHWP 및 Print PNG 픽셀, TSV가 이번 결과와 일치했다. [최신 전쪽 비교 및 생산물 SHA](../assets/planet6897_9240_9282_20261121/validation/claims-v5.json)

## Merge 후 contributor PR comment 계획

- [#9285](https://github.com/edwardkim/rhwp/issues/9285) 재현 문서 4쪽의 WMF 그림 네 조각을 한컴 2020 Hancom PDF Print에서 대응시킨다. Native와 fresh WASM 4쪽의 review와 overlay를 표시한다.
- 색상표가 없는 1비트 레코드에서 다음 레코드의 시작을 밀지 않게 한 기여 내용을 적는다. 고정 WASM은 디코딩 실패 상태로 표시하고, 후보는 9쪽 전쪽의 일치율을 제시한다.

[Visual Sweep 공식 코멘트 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 연결한다. 통합 merge SHA에 실제 포함된 원본 출력·한컴 PDF 래스터·review·overlay만 사용한다. 최신 CI와 원본 및 Print 영수증, 실제 확인한 쪽수와 자동 비교 수치, 직접 판독 결론을 함께 기록한다. 게시 전에 PNG 원격 응답과 검증 자산의 SHA를 대조한다.
