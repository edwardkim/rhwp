# Issue #1133 자동 쪽번호 여백 대조군

원본은 `samples/hwpx/issue_1133.hwpx`입니다. 두 HWPX는 원본 ZIP의 다른 멤버 바이트를 모두 보존하고 `Contents/section0.xml`의 쪽 여백 속성 하나만2834→4252HU로 바꿨습니다. 원본 저장본과 구분되는 **생성 대조군**입니다.

- `page-number-footer15.hwpx`: 꼬리말 여백만10→15mm, 아래쪽 여백은10mm 유지.
- `page-number-bottom15.hwpx`: 아래쪽 여백만10→15mm, 꼬리말 여백은10mm 유지.

동일 입력을 `hwp2024-mcp-convert`의 `engine=2020`으로 한컴에서 다시 출력한 기준 PDF는 `pdf/issue1133/`에 있습니다. 세 쪽 모두 원본/꼬리말 대조군의 쪽번호 기준선은1080.41927px(96dpi), 아래쪽 여백 대조군은1061.55990px입니다. 꼬리말 변화는0px, 아래쪽 여백 변화는−18.859375px입니다. 입력과 PDF의 SHA-256 및 변경 속성은 `MANIFEST.json`에 고정했습니다.

공식 검사는 원본HWP/HWPX와 대조군의 **TextRun 상단 + baseline**을 비교합니다. 상자 상단이나 글자 존재만으로 실제 쪽번호 위치를 입증하지 않습니다. 생성 대조군의 통과를 원본 문서 전체 시각 일치로 보고하지 않습니다.
