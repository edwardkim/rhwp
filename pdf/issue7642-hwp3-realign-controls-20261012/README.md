# HWP3 외부 문서 재조판 대조군

원본을 바꾸지 않고 한컴 2020(11.0.0.9136)에서 각각 Print PDF 및 HWPX로 저장했다.
PDF는 PrintMethod 0의 1-up 인쇄다. HWPX는 수입 속성·저장 줄 해석 진단이며 기준 PDF를 대신하지 않는다.
SHA·실제 job·쪽수와 인쇄 메타데이터는 `provenance.json`에 보존했다.

- `hwp3-table-cell-order.hwp`: 외부 플래그 bit 0 설정.
- `issue1892_hwp3_tab_roundtrip.hwp`: 외부 플래그 bit 0 설정.
- `hwp3-table-grid-gap.hwp`: 외부 플래그 bit 0 해제.

기준 자료 공급이며 RHWP 시각 통과를 뜻하지 않는다. 시각 TSV·로그·중간 JSON은 ignored output에 남긴다.
