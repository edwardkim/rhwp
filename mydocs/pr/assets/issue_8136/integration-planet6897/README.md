# issue_8136 통합 후보 Print 대조

2026-11-01 KST, 96 DPI에서 한컴 2024 독립 Print와 Native Skia·fresh WASM Canvas 전쪽의 2px raw 실루엣을 비교했다. 입력은 2쪽이다. 기호와 영문 슬롯별 글자 및 앞뒤 글자의 내용을 같은 쪽에서 직접 대조했다.

- 입력 `samples/issue8136/lang-slot-ranges.hwpx` SHA-256: `575de965af86769779a8533b7eea73f79d4ab76737b9daacc514728c92d8e0eb`.
- Print PDF `pdf/issue8136/lang-slot-ranges-2024.pdf` SHA-256: `27a3d1916d078fa064be171c777807d4f278cd62059c810decaf7a0d233f327d`.
- Native 실행 파일 SHA-256: `4b03859cfa21cdba1a766d5141d260adf70d3c986c6481a3e8440d1ab992ae96`.
- WASM SHA-256: `cee8e58be7191a14febfb73e8b79b2ad59946ac1c7f13332d46ae92157aef55f`.
- 제품 코드 생산 커밋 `6be28314c5` 이후 통합 제품 소스는 동일하다. 시스템 글꼴을 사용했고 글꼴 파일은 추가하지 않았다.
- [Native 전쪽 TSV](native.tsv): 1쪽 99.90%, 2쪽 98.96%; 최저 98.96%.
- [WASM 전쪽 TSV](wasm.tsv): 1쪽 99.86%, 2쪽 98.37%; 최저 98.37%.
- 직접 판독한 각 쪽의 PNG는 전쪽 TSV 생산에 쓴 같은 쪽 PNG와 SHA-256이 동일하다.

- native 1쪽: [review](native-review-001.png) · [overlay](native-overlay-001.png)
- native 2쪽: [review](native-review-002.png) · [overlay](native-overlay-002.png)
- wasm 1쪽: [review](wasm-review-001.png) · [overlay](wasm-overlay-001.png)
- wasm 2쪽: [review](wasm-review-002.png) · [overlay](wasm-overlay-002.png)
