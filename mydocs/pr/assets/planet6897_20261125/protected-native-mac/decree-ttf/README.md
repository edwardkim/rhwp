# 시행령 Native 휴먼고딕 공급 검증

시행령 9쪽의 `(A) 사업자의 수 또는 범위 제한`은 원본 글자모양의 한글과 영문
슬롯 모두 휴먼고딕이다. 기존 보호 PDF와 MCP로 다시 산출한 한컴 2020
`11.0.0.9172` Print도 이 줄에 휴먼고딕을 사용한다. 기존 보호 PDF의 파일명은
`-2020.pdf`이며 `pdfinfo`의 Creator는 `Hwp 2022 0.0.0.0`이다.
[한컴 2020 인쇄 영수증](mcp-reprint-receipt.json)

설치된 `HMKMG.otf`와 검증된 `HMKMG.TTF`는 괄호의 전진폭이 다르다. Native
공급 목록에 TTF를 추가하자 기존 Print의 9·32·50·66쪽이 100.00%로 복원됐고,
나머지와 합쳐 74쪽 전체의 값이 조정 전 정본 수치와 일치했다. 새 Print에 대조한
이 네 쪽은 모두 99.98%다. 보호값은 기존 Print의 전쪽 대조 결과로 복원한다.

[Native 74쪽 TSV](silhouette.tsv) · [공급 및 실행 신원](validation.json) ·
[한컴 2020 Print 9쪽 review](new-print-review_009.png) · [overlay](new-print-overlay_009.png)

## 한컴 2024 재인쇄

같은 HWPX를 한컴 2024 `13.0.0.4225`의 Hancom PDF로 인쇄해 74쪽을 확인했다.
Native와 최적화 fresh WASM의 PNG를 각각 비교하니 9·32·50·66쪽은 모두 99.98%였고,
16쪽이 각각 53.09%·53.35%로 가장 낮았다. 2020 재인쇄와 2024 재인쇄의 PDFium PNG는
74쪽 중 73쪽이 같았다.

16쪽의 기존 보호 PDF에는 Palatino Linotype과 한양신명조(`H2mjsM`)가 포함된다.
새 한컴 2020·2024 인쇄본은 이 쪽에 굴림을 포함하며, `학계·법조계·산업계`가 끊기는
위치와 여러 줄의 말미가 두 새 인쇄본에서 동일하게 바뀐다. 이 차이는 같은 Native PNG를
두 기준에 대조해서 확인했다. 기준 파일 교체 검토용 자료로 새 인쇄 결과를 기록하고,
보호값 판정은 정본 manifest에 고정된 PDF와 공급 파일로 수행했다.

[한컴 2024 인쇄 영수증](mcp-2024-reprint-receipt.json) · [Native 74쪽 TSV](mcp-2024-native.tsv) ·
[fresh WASM 74쪽 TSV](mcp-2024-wasm.tsv) ·
[Native 인쇄본 대조](mcp-2024-native-comparison.json) · [fresh WASM 인쇄본 대조](mcp-2024-wasm-comparison.json) ·
[16쪽 review](mcp-2024-review_016.png) · [overlay](mcp-2024-overlay_016.png)
