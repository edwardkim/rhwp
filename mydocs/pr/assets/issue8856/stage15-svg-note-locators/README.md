# 실제 출력 확인 후 SVG 주석 번호 조회 정정

제품 Runtime은 stage14 `c88877ee5a` 그대로다. 전체 회귀의 **13,151 PASS / 2 FAIL / 45 skip**에서
실패한 두 검사는 생성 번호가 SVG 한 문자열 안에 있다는 가정을 사용했다. 공통 전진을 소비하는
출력은 번호를 연속 `<text>`로 나눌 수 있다. 원본 입력·기존 PDF·판정 문턱·좌표 기대값은 바꾸지 않았다.

## 원래 보호 계약과 변경

- `inline_note_page_field_source_ownership`: 도입 커밋 `8c738f5434`, 통합 PR #8319
  (merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`)의 원문·부분 문단 소유와 독립 Print 원점
  보호다. 같은 내용을 가진 단일 text 또는 같은 부모·기준선의 연속 `1`·`)`로 참조를 찾고,
  PAGE 필드보다 먼저 그리는 순서를 확인한다. 원점 4198/1947·709/2098, CTM 0.119935/0.119869,
  72→96dpi 환산과 **±2px**는 그대로다. 과거 잘못된 참조 x 113.38667은 이 검사에서 계속 실패한다.
- `issue_1052_footnote_in_textbox`: 원 수정 `8f007066df`는 ShapeTextBox 안 각주를 하단 영역에
  수집하는 변경이다. 기존 세 본문 존재 검사는 통과했다. 번호 존재 검사도 이 파일에 이미 있는
  `svg_text_sequence`를 사용하도록 고쳤다. 본문·글상자·각주 번호의 존재 조건을 삭제하지 않았으며,
  이 관계 검사가 물리 위치까지 고정한다고 설명하지 않는다.

두 묶음의 집중 검사는 수정 후 **6 PASS / 0 FAIL**다. 함수 이름·중복 wrapper·새 CI 호출은 추가하지 않았다.
전체 재실행은 **13,153 PASS / 0 FAIL / 45 skip**,334.650초로 완료했다. Rust/Native Skia·manifest·unit tier·글꼴·브라우저 게이트도 모두 통과했다([게이트 결과](final-gates.json)).
정본 보호 TSV와 실제 소비 경로 검증은 별도로 마무리한다.

## 시각 선행 조건과 같은 페이지 판독

Mac 최소 글꼴 격리, Native Skia/fresh WASM 실제 Canvas, 96dpi·compat2022·PDFium·색상 경계 보정 전
raw 2px로 대조했다. 미주 경계는 전체 3쪽, 글상자 각주는 전체 1쪽의 두 경로가 각각 **100.00%**다.
첫 검사의 완료 3쪽은 고정형 원점 검사를 유지하며, 글상자 검사는 내용 존재를 검사하는 관계형이다.
raw 2px 100%를 RGB 완전 동일로 표현하지 않는다.

- 미주 경계 3쪽: [Native raw](endnote-boundary-native-p003-raw.png) · [review](endnote-boundary-native-p003-review.png) · [overlay](endnote-boundary-native-p003-overlay.png)
- 같은 쪽: [WASM raw](endnote-boundary-wasm-p003-raw.png) · [review](endnote-boundary-wasm-p003-review.png) · [overlay](endnote-boundary-wasm-p003-overlay.png)
- 글상자 1쪽: [Native raw](textbox-footnote-native-p001-raw.png) · [review](textbox-footnote-native-p001-review.png) · [overlay](textbox-footnote-native-p001-overlay.png)
- 같은 쪽: [WASM raw](textbox-footnote-wasm-p001-raw.png) · [review](textbox-footnote-wasm-p001-review.png) · [overlay](textbox-footnote-wasm-p001-overlay.png)

미주 참조·다음 줄의 `3뒤`·하단 주석과 글상자/본문의 `1)`·`2)` 및 하단 두 각주 본문이 실제로
보존됐는지 같은 페이지 review·overlay를 직접 열어 확인했다. 전쪽 TSV·입력/producer 해시는
[검증 JSON](validation.json)에 연결했다.

## 글상자 원본의 독립 Print 재확인

기존 PDF를 덮어쓰지 않고 [동일 HWPX](../../../../../samples/hwpx/footnote-tbox-01.hwpx)를
2024 MCP client의 engine 2020으로 다시 인쇄했다. 입력의 `lastSavedWith`도 한컴 2020임을 확인했다.
job `862bcc93-c2db-420e-a285-a8bb5e044965`는 12초에 성공했으며 실제 제품 11.0.0.9136,
Print method 0·`hancom2020_pdf_driver_one_up`·preprocess none이다.
[Print PDF](../../../../../pdf/issue8856/issue8856-textbox-footnote-2020.pdf)는 14,529바이트이며
SHA-256은 `846fb17709e22ac66fbc91efb2e0919fddb7513da67b18a0b3882d24cf539dc5`다.
[서비스의 Print·다운로드 확인](mcp-textbox-print.json)에는 인증 정보와 서버 URL을 기록하지 않았다.

동일한 현재 Native/WASM PNG를 기존 `pdf/hwpx/footnote-tbox-01.pdf`와도 대조해 두 경로 모두 100%를
확인했다. 새 PDF는 Print 실행 경로를 명시적으로 확인한 추가 근거이며 기존 PDF도 보존한다.
