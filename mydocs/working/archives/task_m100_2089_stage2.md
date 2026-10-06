# 단계 완료 보고 — Task M100 #2089 (R11): 가로쓰기 셀 본문 통이동

- 작성일: 2026-07-09 / goal 루프 2/4 (자체 검증)

## 수행 내용

셀 루프의 가로쓰기 else 본문(1,229줄·분기 126)을 `layout_horizontal_cell_paragraphs`
로 통이동. `HorizontalCellVars`(Copy 17필드, §6) + 참조 직접(tree/table_node/cell_node/
cell/composed_paras/table/styles/bin_data_content/enclosing_cell_ctx/row_y 등).
- 컴파일러 수렴 3건(E0425: table_node/row_y/r) + 이중 &mut 7건(E0596) 해소.
- 탈출 5곳 전부 블록-지역 루프 소속 확인(사전 실측) — 외부 제어 흐름 무변.

## 게이트 (전수 통과)

fmt ✓ / clippy 0 / `--tests` **2,945/0** / issue_1116 13/13 / OVR 5샘플 회귀 **0건**.

## 계측 (표적 공식 CC)

| 함수 | 시작 (r10) | 현재 |
|---|---|---|
| `layout_table_cells` | **124** (전체 1위) | **42** |
| 신규 `layout_horizontal_cell_paragraphs` | — | **83** — §5 심사: 셀 문단 방출 단일 국면 응집, 후속 분해 후보 등재 |
| (파일 1위) `layout_table` | 90 | 90 (기존) |
