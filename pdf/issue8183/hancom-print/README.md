# #8183 저장본의 한컴 Print 대조

원본 PDF를 제품 source `cfc5be8fe`의 CLI 양쪽 코덱 빌드로 HWPX에 가져온 뒤 한컴 2024
13.0.0.3901·Print method 0으로 출력했다. 원본 PDF 가져오기의 독립 기준과 구분한 저장본
호환성 대조다. 입력·출력 hash는 [provenance.json](provenance.json)에 있다.

9종 중 7종은 Native/fresh WASM 모두 99% 이상이었다. `jpx-alpha`는 27.39104%,
`jpx-premult`는 52.99065%로 **Print 대조는 미달**이다. 수치·TSV를 숨기거나 통과로 바꾸지 않는다.
한컴으로 다시 저장한 HWP에서도 원래 4×2 PNG의 RGBA 바이트가 보존됐으나 Print는 동일한
1비트 mask(`c0e0`, `50d0`)로 변환했고 두 경로의 인쇄 픽셀이 일치했다.

원본 PDF/독립 Ghostscript 기준과 RHWP Native/fresh WASM의 alpha 합성은 일치한다.
원래 alpha를 손실시켜 한컴의 Print를 흉내 내지 않는다. 이 자료는 한컴 인쇄 제한의 재현 근거이며
미달 Print 비교의 자동 면제나 PR gate 통과 판정이 아니다. 아래 메인테이너의 별도 수용 결정을 함께 따른다.
[검증 결과와 TSV](../../../mydocs/pr/assets/issue_8183/hancom-print/)를 함께 확인한다.

## 2026-10-24 메인테이너 결정

2026-10-24 메인테이너가 두 4×2 반투명 사례의 RHWP 처리를 정답으로 판정하고 PR 진행을 결정했다.
Print의 미달 수치·TSV·원본은 보존하고 해당 두 Print를 alpha 정답지로 사용하지 않는다.
수치 임계값·제품 alpha·회귀 기대값은 바꾸지 않는다.
