# PR #9287 리뷰 — 색상표 없는 WMF 1비트 DIB의 레코드 경계 보존 (#9285)

## 판정

**원 PR 변경 검증 완료.** 고정 devel과 통합 후보에서 원본 전체 9쪽의 SVG를 대조했다. 4쪽의 WMF 변환을
복구했고 나머지 8쪽 SVG는 바이트 단위로 같다. 공식 한컴 Print 전쪽 최저값은 Native 99.78%, fresh WASM
99.66%다. 통합 후보 전체의 회귀와 보호 문서 검증은 확인 중이다.

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
