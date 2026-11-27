# 반복 점검표의 2→3쪽 이어받기

[원본 HWP](../../../../../samples/issue2439/issue2439_repeat_table_overlap.hwp)와
[한컴 2024 가로 1-up Print](../../../../../pdf/issue2439/issue2439_hancom2024_landscape.pdf)를
2026-11-23에 96 DPI로 대조했다. 보정 전 2px 실루엣은 Native와 fresh WASM 모두
10쪽 전체 100.00%다.

2쪽 오른쪽 끝 셀은 ‘2개월 1회 주기’에서 끊기고 3쪽 같은 셀에서 ‘체크권장’으로
이어진다. 2쪽 마지막 괘선은 본문 아래쪽에서 원본 표의 아래 바깥여백을 뺀 곳에
닫힌다. 4쪽은 서명 본문 앞에 같은 문단의 두 표가 순서대로 있다. 최소 글꼴
환경에서 쪽·셀 소유와 후속 요소의 순서를 검사한다.

[Native TSV](native.tsv), [fresh WASM TSV](wasm.tsv),
[2→3쪽 확대 비교](pages-02-03-repeat-grid.png)

| 자료 | SHA-256 |
| --- | --- |
| 원본 HWP | `674eabe66ea0ba783ad2cd398519c9893ba94956a22d9cb94b084db00d4d2c3d` |
| 독립 Print PDF | `f36a747c5f848d90e755abe2e730d932429a324fad9b7e822964934cd8f8eca4` |
| Native TSV | `cd69186771b706c9ee855490c9c46fc4471589a8658df8076b89a360b68ddd4e` |
| fresh WASM TSV | `cd69186771b706c9ee855490c9c46fc4471589a8658df8076b89a360b68ddd4e` |
