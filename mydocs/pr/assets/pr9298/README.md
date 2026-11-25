# 규제영향분석서 8쪽 JPEG 방향 검증

원본은 [규제영향분석서 HWP](../../../../samples/issue1921/59043_regulatory_analysis.hwp)이며 SHA-256은 `da9f9c9862475d1710f6ecf93e3ce134726054fe0be0d9523b9d60d1a90c82ef`이다. 한컴에서 만든 독립 [2022 PDF](../../../../pdf/issue1921/59043_regulatory_analysis-2022.pdf)와 [2024 PDF](../../../../pdf/semanticist21-20261123/59043_regulatory_analysis-2024.pdf)를 96 DPI로 비교했다. [현재 판단](../../archives/pr_9298_review.md)

| 확인한 항목 | 실측 근거 |
| --- | --- |
| 8쪽 사진의 회전 | [한컴 인쇄와 실제 Chromium Studio 전후](studio-source-comparison.png). 한컴은 안내판 글씨를 세로 사진 안에 똑바로 인쇄한다. 화면이 고쳐진 후에도 오른쪽 사진이 안내판 일부를 가린다. |
| 37쪽 문서의 기존 미달 | [#9007과 최신 devel의 전쪽 TSV](source-history.tsv). 한컴 2022·2024 기준 모두 12·14·15·16·20·25·28·37쪽이 90% 미만이다. |
| JPEG 마커의 채움 바이트 | [원본과 변형의 파일 해시·독립 인쇄 결과](print-control.json). 한컴 2024 직접 인쇄 PDF의 1쪽이 96·192 DPI 모두 같다. |
| 37쪽 원본 저장 | [8개 저장본의 비교](full-source-roundtrip.tsv). 원본과 동일한 JPEG 15개가 HWP·HWPX 두 세대의 저장·재열기를 거친다. |
| 같은 칸에 있는 두 사진의 배치 | [원본 8쪽](page8-neighbor.png) · [두 줄 배치 설정의 한컴·Native 비교](cell-wrap-controls.png) · [HWP·HWPX의 인쇄 좌표·raw 결과](cell-wrap-controls.tsv). |
| 글꼴 공급 | [원본·대조군의 목록](source-fonts.tsv) · [중첩 표 Native의 21개 파일과 해시](nested-fonts.tsv). |

38개 글꼴 슬롯을 사용한 추가 측정과 일반 공급 조건의 결과는 별도로 취급한다. 회전과 겹침은 2px 실루엣 수치와 함께 같은 사진의 색상과 글씨 방향을 직접 비교했다.

## 전체 비교

- [Native·fresh WASM의 원본·한 쪽 대조군 결과 152행](target-comparison.tsv)과 [38개 글꼴 슬롯으로 재측정한 원본 148행](source-expanded.tsv)에 각 쪽의 백분율과 PNG 해시를 기록했다. 두 공급 조건 모두 기본 브랜치와 JPEG 보완본의 인쇄용 출력이 같다.
- [보호 문서 전쪽 4,046행](protected-comparison.tsv)에서 기본 브랜치와 보완본의 PNG SHA-256이 4,046쌍 모두 같다. 쪽 누락은 0건이다. Native 2,023쪽은 모두 정본 하한 이상이다. [WASM 14쪽](protected-below.tsv)은 양쪽 모두 공식값 100.00%에 대해 99.87~99.99%이며, 각각 [직전 검토에서 기록한 PNG](protected-evidence.json)와도 SHA-256이 같다. 화학물질 표시 기준 26쪽은 양쪽 모두 기존 99.99% 기준에서 100.00%였다.
- [빌드 대상·바이너리 해시와 중첩 표 대조](builds.json)에 원 기여 SHA, 비교한 `devel`·제품 수정 SHA, 글꼴 16·21종 대조 결과를 기록했다. 중첩 표 Native 17쪽은 두 공급 조건에서 모두 100.00%였다. WASM은 두 조건 모두 17쪽에서 99.98%였다.
- 실제 Studio의 [화면 컬러 측정](studio-colour.json), [한 쪽 사진 화면 비교](studio-photo-comparison.png), [그림 키·화면 SVG·inline 디코딩](browser-decoding.json)과 [한 쪽 문서 12개 저장 결과](roundtrip-jpeg.tsv)를 인쇄용 전쪽 측정과 함께 확인했다.
