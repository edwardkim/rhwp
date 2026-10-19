# Issue #7642: #1116 HWP3 원본의 독립 Print

`tests/issue_1116.rs`에서 사용하는 `samples/hwp3-sample16.hwp`를 내용 변경 없이
문서화된 MCP `engine: 2020`으로 전쪽 Print 출력했다. 한컴 11.0.0.9136,
PrintMethod 0, `hancom2020_pdf_driver_one_up`이다. 정확한 입력·PDF 해시와
쪽수·job은 [출력 근거](print-provenance.json)에 보존한다.

출처를 확정한 대조군이며 Native/fresh WASM 전쪽 검증·직접 판독은 진행 중이다.
PDF 확보만으로 시각 통과·회귀 변경을 판정하지 않는다.

## 후속 검증

code cc4255a58d에서 원본 전체64쪽의 Native/fresh WASM raw 최저는
94.88041%/94.94043%이며 Print와 전체 쪽수가 같다. 6쪽 수식과 앞뒤 본문을
직접 대조했다. 원본/한컴 변환 HWPX의 같은 영역, 최신 TSV와 검사 교정 범위는
[SAMPLE16_SOURCE_EQUATION_PRINT.md](SAMPLE16_SOURCE_EQUATION_PRINT.md)에 연결한다.
이는 RHWP 왕복 저장본이나 브랜치 전체의 완료를 대신하지 않는다.
