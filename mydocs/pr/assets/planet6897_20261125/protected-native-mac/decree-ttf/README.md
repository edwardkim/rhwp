# 시행령 Native 휴먼고딕 공급 검증

시행령 9쪽의 `(A) 사업자의 수 또는 범위 제한`은 원본 글자모양의 한글과 영문
슬롯 모두 휴먼고딕이다. 기존 한컴 2020 Print와 MCP로 다시 산출한 한컴 2020
`11.0.0.9172` Print도 이 줄에 휴먼고딕을 사용한다. [새 Print 영수증](mcp-reprint-receipt.json)

설치된 `HMKMG.otf`와 검증된 `HMKMG.TTF`는 괄호의 전진폭이 다르다. Native
공급 목록에 TTF를 추가하자 기존 Print의 9·32·50·66쪽이 100.00%로 복원됐고,
나머지와 합쳐 74쪽 전체의 값이 조정 전 정본 수치와 일치했다. 새 Print에 대조한
이 네 쪽은 모두 99.98%다. 보호값은 기존 Print의 전쪽 대조 결과로 복원한다.

[Native 74쪽 TSV](silhouette.tsv) · [공급 및 실행 신원](validation.json) ·
[새 Print 9쪽 review](new-print-review_009.png) · [overlay](new-print-overlay_009.png)
