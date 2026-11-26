# KTX-003의 본문 겹침 검사

2026-11-23에 [KTX-003 원본](../../../../../samples/basic/KTX-003.hwp)과
[독립 한컴 2020 Print](../../../../../pdf/basic/KTX-003-hwp-2020.pdf)를 96 DPI로 대조했다.
보정 전 2px 실루엣은 Native 99.99%, fresh WASM 100.00%다. 양쪽에서 1쪽을 출력했다.
Native 구조 검사에서 글줄 겹침 0건을 확인하고 겹침 원장의 적용 대상으로 되돌렸다.

[Native TSV](native.tsv), [fresh WASM TSV](wasm.tsv), [1쪽 비교](page-01-comparison.png)
