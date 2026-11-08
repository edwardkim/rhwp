# 셀 그림의 편집 소유와 그리기 계층

원 PR #8061의 같은 재현에서 필요한 기하 정보를 명시하고 속성을 구분한 HWPX 8개다.
초기 불완전한 내보내기 자체를 정상 한컴 저장본이라고 주장하지 않는다.
각 파일은 같은 바이트를 MCP Hancom 2020 Print로 출력해 독립 PDF와 대조했다.
정상 실물 문서의 추가 수집이나 제외 문서의 복원은 하지 않았다.

- [Print PDF](../../../pdf/pr8061_review/)
- [생성 방식·전쪽 Native/fresh WASM TSV·대표 review/overlay·소스 해시](../../../mydocs/pr/assets/cell_picture_layer_qualified_20261026/README.md)
- [독립 PDF CTM·색상·파일 SHA-256](expected.json)
- [정식 관계·좌표·Native PNG 회귀](../../cases/verified_cell_picture_layer_scope.rs)

`expected.json`의 좌표는 96 DPI의 독립 PDF 이미지 CTM이다. 각 쪽의 양쪽 raw 100%와
직접 판독을 먼저 완료했으므로 ±2px를 허용하며 소유·순서·포함 관계도 함께 유지한다.
raw 100%만으로 가림 색상이나 RGB 경계의 동일성을 판정하지 않는다.
