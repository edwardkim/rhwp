# 주 입력의 최적화 fresh WASM 전쪽 재검증

> 이 문서는 당시 단계의 분석 기록이다. 최종 결과와 대표 증적은 [검증 기록](../verification-record.md)과 [최종 대상 증적](../final/main/README.md)을 따른다. 원시 로그·중간 분석 JSON·중간 PNG는 제출에서 제외했다.

2026-11-15에 `wasm-canvas-coverage`의 실제 최적화 제품 package로
`samples/issue9060/86036-rowbreak-first-line-fragment.hwpx` 전 14쪽을 재출력했다.
입력·기존 독립 Print PDF·제품 JS/WASM·기존 글꼴 공급 목록의 해시는 [실행 기록](../verification-record.md)에 있다.
공급 목록만 재사용했고 PNG는 새 제품에서 다시 생성했다. exporter는 공급 프로그램의 SHA를
모두 재확인했다. 글꼴 파일은 저장소에 추가하지 않았다.

Print·compat 2022·96 DPI·PDFium·색상 경계 보정 전 2px raw의
[전체 TSV의 검증 요약](../verification-record.md)는 **2쪽 99.99%, 나머지 13쪽 100.00%**다.
동일 입력의 제품 Native는 [앞 단계](../native-print-stage/README.md)에서 전 14쪽 100.00%다.
이 단계는 원 contributor source의 과거 WASM 출력을 최종 제품의 실행 결과로 재인용하지 않는다.

원 PR의 주장은 5쪽 표 첫 줄의 실제 수용, 6쪽의 나머지 두 줄, 7쪽 후속 본문의 순서 보존이다.
아래 이미지는 그 주장에 직접 대응하는 5·6쪽이며 양쪽 raw는 각각 100.00%다.
전체 문서의 내용 보존 대조는 최종 리뷰에서 Native/fresh WASM의 실제 결과와 함께 기록한다.
보호 대상 전체의 잔여 하락·전체 필수 검증·원격 CI는 아직 완료되지 않았다.

| 페이지 | 실제 제품과 독립 Print | 같은 쪽 겹침 |
| --- | --- | --- |
| 5쪽: 표 첫 줄 | [review](../verification-record.md) | [overlay](../verification-record.md) |
| 6쪽: 나머지 두 줄 | [review](../verification-record.md) | [overlay](../verification-record.md) |
