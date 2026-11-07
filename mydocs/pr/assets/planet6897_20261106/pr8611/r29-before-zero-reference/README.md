# 0번 마커 글자 모양 참조의 수정 전 증거

제품 `91109ff833`의 원본 `samples/basic/sungeo.hwp`와 기존 독립 한컴 Print
`pdf/basic/sungeo-hwp-2020.pdf`를 96 DPI·색상 경계 보정 전 2px raw로 전쪽 비교했다.
Native/fresh WASM 모두 94쪽이고 최저와 검사 5쪽은 각각 98.93%다. 이 작업의
base 대비 0.1%p 조건은 충족하지 못하므로 최종 승인이나 무회귀 자료가 아니다.

- `native/full.tsv`·`wasm/full.tsv`: 실제 전쪽 TSV 원본.
- 각 경로의 `review-005.png`·`overlay-005.png`: 같은 실제 원본 5쪽의 대응 이미지.
- `provenance.json`: 입력·독립 PDF·producer·PNG·TSV 해시와 실제 측정 경로.
- `source-bullet-records.json`: 원본 DocInfo BULLET의 실제 바이트.
- `before-regression-result.json`: 같은 검사 본문으로 수정 전 2 FAIL / 정상 대조 1 PASS.
- `equivalent-id-diagnostic.json`: rhwp로 직렬화한 진단 HWPX 쌍의 ID 별칭 반례다.
  원본 HWP의 최종 측정이나 한컴이 새로 저장한 원본으로 취급하지 않는다.

0번 글자 모양은 14pt·#000080·양각이고 본문은 다른 모양의 13pt다.
크기·꾸밈과 동일 모양 ID 별칭의 불변성을 관계 검사로 보호하며 특정 설치 글꼴이나
절대 px 핀을 요구하지 않는다. 보정 후에는 실제 원본을 Native/fresh WASM으로 다시 검증한다.
