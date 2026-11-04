# 큰 표 뒤 문단의 독립 Print 계약

원 PR #8036의 같은 두 최소 재현을 보존한다. `zero.hwpx`는 뒤 문단의 저장 vpos가
0이고, `continuing.hwpx`는 67200이다. 원래 검사의 `fixture()`에서 산출한 같은
HWPX를 사용하며, 이번 시각 검증에 맞춰 내용·줄 정보·용지를 바꾸지 않았다.

두 입력 모두 독립 한컴 2020 Print와 기준 devel에서 **1쪽**이다. 저장 vpos가
0이라는 이유만으로 2쪽을 기대하던 검사는 잘못됐으므로 제거했다. 이 증거는
원 PR 본문에서 언급한 별도의 13쪽 문서까지 검증했다는 뜻이 아니다.

- 기준 PDF: [zero](../../../pdf/pr8036_stored_tall_line/zero-2020.pdf),
  [continuing](../../../pdf/pr8036_stored_tall_line/continuing-2020.pdf).
- [전쪽 Native/fresh WASM TSV와 출처](../../../mydocs/pr/assets/pr8036_print_contract_20261026/README.md).
- 새 검사: [verified_stored_tall_line_print_flow.rs](../../cases/verified_stored_tall_line_print_flow.rs).

새 검사는 쪽 소속·내용 보존·표 안 포함·뒤 문단과의 순서를 기본으로 확인한다.
두 입력의 완료 페이지는 Native258/fresh WASM075가 각각 raw 100%이며 직접 판독을
마쳐, 독립 PDF 괘선의 잉크 범위를 96 DPI·좌상단 좌표계에서 ±2px로도 보호한다.
PDFium path에서 읽은 범위는 `(x, y, width, height) =
(74.840001, 115.073324, 641.094650, 866.733348)`이다. 새 devel 리베이스 뒤에도
양쪽 98% 이상을 유지하는지 다시 확인해야 한다.
