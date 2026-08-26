# 좌표계 — `index` · `row`/`col` · `containerPath`

권위: 지식지도 §2-3 · §좌표 표,
`cli_commands.md` §export-tables / §table-to-csv / §csv-to-table / §edit set-cell,
`table_csv_contract.rs` (머리말 표가 0),
`edit_set_cell_contract.rs` (`containerPath` 없는 첫 표).

네 명령이 **같은 말**을 쓴다. 다른 말을 발명하지 마라.

## 1. 공유 주소

| 말 | 뜻 | 누가 쓰나 |
|---|---|---|
| `index` / `--table` | 본문 최상위 표 번호 | 넷 다 |
| `row` / `--row` | 격자 행, 0 기준 | export-tables, csv-to-table changed, set-cell |
| `col` / `--col` | 격자 열, 0 기준 | 위와 같음 |
| `section`/`paragraph`/`control` | 문서 안 위치 | export-tables 만 (인용) |
| `containerPath` | 글상자·머리말·각주·중첩 | export-tables. 있으면 왕복 제외 |

`extract-pages` 의 `--from`/`--to` 만 1 기준이다. 표 좌표에 섞지 마라.

차트 `--chart` 는 1부터이고 발견 명령이 없다. 표와 다른 공간이다.

## 2. `index` ≠ 배열 순번

```json
"tables": [
  {"index": 0, "containerPath": [{"kind": "header"}]},
  {"index": 12, "rows": 8, "cols": 6}
]
```

`tables[0].index` 는 0 이지만 머리말이다. `tables[1].index` 는 12 다.
`--table 1` 은 "두 번째 배열 원소"가 아니다. `--table 12` 다.

지식지도 원문: `index` 는 **0부터 시작하지 않을 수 있다**.

픽스처: [../fixtures/envelopes/export_tables_jichi_index_not_zero.json](../fixtures/envelopes/export_tables_jichi_index_not_zero.json),
[../fixtures/matrices/coordinate_index.json](../fixtures/matrices/coordinate_index.json).

jq:

```bash
rhwp export-tables 문서.hwpx --json \
  | jq '[.tables[] | {i:.index, rows, cols, box:.containerPath}]'
```

복사하는 값은 `.i` 이지 배열 위치 `$k` 가 아니다.

## 3. 본문 최상위 필터

`table_csv_contract.rs` 는 병합 표를 고를 때
`containerPath` 가 null 인 것만 본다.

```
body = tables.filter(t => t.containerPath == null)
if body is empty:
    there is no csv-to-table target
    nested-only documents print "중첩 표는 v1 범위 밖"
```

머리말·글상자 표를 수정하는 명령을 이 스킬이 만들지 않는다.

## 4. 표본별 `index` 메모

[../fixtures/matrices/sample_catalog.json](../fixtures/matrices/sample_catalog.json)

| 표본 | tableCount | 왕복에 쓸 index | 메모 |
|---|---:|---|---|
| `samples/hwp_table_test.hwp` | 10 | 0 (3×4) | 레시피 02. 0이 본문 |
| `samples/table-001.hwp` | 1 | 0 | 병합 20. 되돌리기는 set-cell |
| `samples/multi-table-001.hwp` | 6 | 고를 것 | 2쪽 |
| `samples/inner-table-01.hwp` | 1 최상위 | 0 (바깥) | 안쪽 24칸은 nested |
| `samples/basic/treatise sample.hwp` | 3 | containerPath 없는 것 | info 는 1개만 |
| `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` | 53 | 0 아님 | 0 = 머리말 |
| `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | 5 | 규제표는 1 | 0은 1열 개요 |
| `samples/hwpx/basic-table-01.hwpx` | (cli 예) | 확인 후 | 매뉴얼 사용 예 |
| `samples/복학원서.hwp` | 3 | 확인 후 | 누름틀 0 |
| `samples/추진일정.hwp` | 1 | 0일 가능성 큼 | 싼 왕복 |

숫자를 이 표에서 외워 `--table` 에 박지 마라. 문서가 바뀌면 `index` 가
바뀐다. 매번 `export-tables` 가 이긴다.

## 5. `row`/`col` 은 앵커만

`export-tables` 의 `cells[]` 에는 덮인 좌표가 없다.
`(0,1)` 이 `colSpan:3` 이면 `(0,2)` `(0,3)` 은 목록에 없다.

`table-to-csv` 는 그 자리를 `""` 로 채운다. 격자 주소는 그대로다.
CSV 의 0행 2열은 문서의 `(0,2)` 이지, "세 번째 앵커"가 아니다.

`csv-to-table` 의 `changed[].row`/`col` 도 같은 격자다.

`set-cell` 에 덮인 좌표를 주면 앵커를 알려 주며 거절한다.

## 6. `isHeader` 와 0행

`isHeader` 는 문서 속성이다. CSV 첫 줄 여부와 독립이다.
어떤 표는 0행이 헤더가 아니고, 어떤 표는 헤더가 병합되어 1행부터
데이터가 있다. `export-tables` 를 보고 판단한다.

`csv-to-table` 은 `isHeader` 를 건너뛰지 않는다.

## 7. `section`/`paragraph`/`control`

같은 문단에 표가 둘이면 `control` 이 갈린다.
`--table` 은 이 주소를 받지 않는다. `index` 만 받는다.

인용할 때("3절 문단 8의 표")는 `section`/`paragraph` 를 글로 쓰고,
명령에는 `index` 를 넣는다.

## 8. 쪽 번호와 혼동 금지

`changedPages` 는 0 기준 쪽 목록이다. `--table` 이 아니다.
`export-svg -p N` 의 N 도 0 기준(명령 문서를 따른다).
표 번호와 쪽 번호를 같은 슬롯에 넣지 마라.

## 9. MCP 이름

| CLI | MCP |
|---|---|
| `export-tables` | `hwp_export_tables` |
| `table-to-csv` | `hwp_table_to_csv` |
| `csv-to-table` | `hwp_csv_to_table` |
| `edit set-cell` | (세션/무상태 정의는 capabilities --mcp) |

도구 이름을 지어내지 마라. `rhwp-mcp-session` 이 단일 출처다.
이 파동은 MCP 스킬을 고치지 않는다.

## 10. 잘못된 주소 예

| 잘못 | 무엇이 일어나는지 |
|---|---|
| `--table 0` 습관 | 머리말 표 또는 1×1 래퍼 |
| `--table` = 배열 위치 | 없는 표 exit 1 |
| 중첩 `index` | v1 밖 메시지 |
| 덮인 `(row,col)` | set-cell stderr 앵커 / csv coveredCellNotEmpty |
| 1 기준 행 | 한 줄씩 밀림. 헤더가 값으로 |
| `--chart` 번호 | 다른 공간 |

## 11. 체크리스트

- [ ] `export-tables --json` 을 이 문서에서 한 번 돌렸다
- [ ] `containerPath` 없는 `index` 를 복사했다
- [ ] 병합이면 set-cell 앵커만
- [ ] CSV 열 위치 = 격자 `col` (앵커 개수 세지 않음)
- [ ] 재독도 같은 `index`/`row`/`col`

예: [../examples/01_coordinate_scan.md](../examples/01_coordinate_scan.md),
[../examples/18_index_not_zero.md](../examples/18_index_not_zero.md).
