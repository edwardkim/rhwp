# 기여자 공통 보호 검사

제품 소스 `9e601ad6a862a30b019ad9563a1d597f6a33222b`의 Native Skia 실행 파일과 최적화 브라우저 WASM으로 등록된 공개 문서 4종을 검사했다. 각 문서의 원본과 독립 한컴 Print를 저장소의 검사 목록에서 읽어 96 DPI의 2px 실루엣을 대조했다. [검사 결과·정본·도구 SHA-256과 실행 명령](contributor-gate.json)

| 문서 | 경로 | 1쪽 raw | 원본 TSV | 인쇄 비교 | 겹쳐 보기 |
| --- | --- | ---: | --- | --- | --- |
| 라틴 줄바꿈 | native | 100.00% | [TSV](latin-native.tsv) | [비교](latin-native-review.png) | [겹침](latin-native-overlay.png) |
| 라틴 줄바꿈 | wasm | 100.00% | [TSV](latin-wasm.tsv) | [비교](latin-wasm-review.png) | [겹침](latin-wasm-overlay.png) |
| 좁은 셀의 줄바꿈 | native | 100.00% | [TSV](narrow-native.tsv) | [비교](narrow-native-review.png) | [겹침](narrow-native-overlay.png) |
| 좁은 셀의 줄바꿈 | wasm | 100.00% | [TSV](narrow-wasm.tsv) | [비교](narrow-wasm-review.png) | [겹침](narrow-wasm-overlay.png) |
| 그림 자르기 | native | 100.00% | [TSV](crop-native.tsv) | [비교](crop-native-review.png) | [겹침](crop-native-overlay.png) |
| 그림 자르기 | wasm | 100.00% | [TSV](crop-wasm.tsv) | [비교](crop-wasm-review.png) | [겹침](crop-wasm-overlay.png) |
| 그룹 도형 | native | 100.00% | [TSV](group-native.tsv) | [비교](group-native-review.png) | [겹침](group-native-overlay.png) |
| 그룹 도형 | wasm | 100.00% | [TSV](group-wasm.tsv) | [비교](group-wasm-review.png) | [겹침](group-wasm-overlay.png) |
