# issue_7092 통합 후보 Print 대조

2026-11-01 KST, 96 DPI에서 한컴 2020 MCP의 독립 Print PDF와 Native Skia·fresh WASM Canvas 전쪽의 2px raw 실루엣을 비교했다. 입력은 7쪽이다. 곡선 따옴표와 점 리더 및 앞뒤 글자의 내용을 같은 쪽에서 직접 대조했다.

- 입력 `samples/basic/treatise sample.hwp` SHA-256: `1aec0ca0a958d24199591b96916a2535d04f1c654f60dda1027ecbaf4839f828`.
- Print PDF `pdf/issue7092/treatise-sample-2020.pdf` SHA-256: `a90c88f4d42d06b91e49edb0da5172ee64d8bc3e13bcbfb406e0bfbd5b21fbd6`.
- Native 실행 파일 SHA-256: `4b03859cfa21cdba1a766d5141d260adf70d3c986c6481a3e8440d1ab992ae96`.
- WASM SHA-256: `cee8e58be7191a14febfb73e8b79b2ad59946ac1c7f13332d46ae92157aef55f`.
- 제품 코드 생산 커밋 `6be28314c5` 이후 통합 제품 소스는 동일하다. 시스템 글꼴을 사용했고 글꼴 파일은 추가하지 않았다.
- [Native 전쪽 TSV](native.tsv): 3쪽 98.21%, 4쪽 98.35%; 전쪽 최저 96.96%.
- [WASM 전쪽 TSV](wasm.tsv): 3쪽 98.26%, 4쪽 98.29%; 전쪽 최저 97.73%.
- 직접 판독한 3·4쪽의 RHWP와 PDF PNG는 새 Print 기준 전쪽 TSV 생산에 쓴 같은 쪽 PNG와 각각 SHA-256이 동일하다. MCP 인쇄 실행 정보는 [Print 출처](print-provenance.json)에 기록했다.

- native 3쪽: [review](native-review-003.png) · [overlay](native-overlay-003.png)
- native 4쪽: [review](native-review-004.png) · [overlay](native-overlay-004.png)
- wasm 3쪽: [review](wasm-review-003.png) · [overlay](wasm-overlay-003.png)
- wasm 4쪽: [review](wasm-review-004.png) · [overlay](wasm-overlay-004.png)
