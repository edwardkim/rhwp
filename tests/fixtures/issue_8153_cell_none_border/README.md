# #8153: cellzone이 없는 기존 #6311 합성 입력

`6311_synthetic_no_cellzone.hwpx`는 기존
[`issue_6311_table_outer_border_fill.rs`](../../cases/issue_6311_table_outer_border_fill.rs)의
`callout_document()`가 만든 2×2 표다. 생성 함수를 바꾸지 않고 입력을 추출했으며,
후속 검사는 이 바이트를 사용한다. 실제 #6311 문서의 회색 프레임 출처는 #6619의
cellzone 38·39이며, 이 합성 입력에는 zone이 없다.

- 이전 source: `be1b4f2ddb7c730075d15cea1d844dbf7826d024`.
- 이전 검사 Git blob: `a2d5ec56a4fd016b9ff4e65505ba6dac111f4040`.
- 입력 SHA-256: `60ac8d56f51ce01310297c3d3559e69d4cc07add3a44ed28e5ed261a0c9e675a`.
- [독립 Print PDF](../../../pdf/issue8153/6311-synthetic-no-cellzone-2020.pdf):
  SHA-256 `2814cb331b49356b1884d3eb70db85719cdae7b44ed8ca1b548e8565924b1ea5`.
- MCP job: `59b27acf-4ba7-4091-b857-dcb734d12597`.
- 저장 메타데이터 11.x에 맞는 실제 한컴 2020 `11.0.0.9136`, PrintMethod 0,
  `hancom2020_pdf_driver_one_up`, 입력 전처리 없음, 1쪽.

독립 출력은 제목 칸의 네 변만 표시한다. 표 전체 왼쪽·아래와 제목 왼쪽의 빈 칸을
닫는 과거 기대값을 정상 계약으로 사용하지 않는다. 제목 테두리 보존과 다른 칸의
개방을 칸 소유·끝점 관계로 검사한다. 글꼴·글자 정확도를 검증하는 입력은 아니다.

96 DPI의 같은 원본·독립 PDF에서 제품 보정 전 Native는 raw 34.35374%·미대응
386픽셀이었다. 공통 NONE 보정 source `be1b4f2ddb`의 Native와 표준 최적화 fresh
WASM은 각각 raw100%·미대응 0픽셀이다. 실제 양쪽 확대 비교를 확인한 뒤 기존 검사
기대값을 정정한다. 상세 경로·원문 명령·실행 결과는
[보정 기록](../../../mydocs/report/issue_8153_cell_border_none_20261026.md)에 연결한다.
