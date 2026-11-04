# 편람 중간 전쪽 검증: 2026-10-29

제품 코드 `d412f3cc58`에서 두 원본의 전체 383쪽을 기존 독립 한컴 2024 Print PDF와
96dpi PDFium 기준으로 대조했다. 지표는 색상 보정 전 2px 실루엣이며 TSV는 소수 둘째
자리로 표시한다. 네 경로 모두 130쪽은 99.96%, 나머지는 100.00%다. 130쪽의 WMF는
사용자가 별도 예외로 지정했다. 이 결과는 #8188 복구 이후 최종 head의 검증을 대신하지 않는다.

| 원본 | Native | fresh WASM |
| --- | --- | --- |
| HWP | [TSV](handbook-hwp-native.tsv) | [TSV](handbook-hwp-wasm.tsv) |
| HWPX | [TSV](handbook-hwpx-native.tsv) | [TSV](handbook-hwpx-wasm.tsv) |

원본과 PDF의 경로·SHA, 기준 인쇄 및 사용 글꼴 환경은 [기존 근거](../README.md)를 따른다.
Native 실행 파일 SHA-256은 `b0a046ca1555fec69cb218445f96a2a5c4dd961cfb4185a7e7d40c7414c0db65`,
최적화된 WASM SHA-256은 `a250ce30c101da3b1e56cfdcc9005684548d6e16bba5de9c7f27666e76a8ce2c`다.
사용 글꼴은 로컬 검증 환경에서 공급하며 이 자료에 글꼴 파일을 포함하지 않는다.
