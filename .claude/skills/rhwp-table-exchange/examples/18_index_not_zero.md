# 18 — `index` 가 0이 아니거나 머리말인 경우

권위: [coordinate_index.md](../references/coordinate_index.md).
표본: 지자체 양식 (표 53, 0=머리말), issue2007 (규제표 index 1).

## 명령

```bash
rhwp export-tables "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx" --json \
  | jq '[.tables[] | {index, rows, cols, box:.containerPath}] | .[:4]'
```

## 기대

첫 원소가 `index:0` + `containerPath.kind==header` 일 수 있다.
그 `0` 을 `--table` 에 넣지 마라.

```bash
BODY=$(jq '[.tables[] | select(.containerPath==null) | .index][0]')
rhwp table-to-csv 양식.hwpx --table "$BODY" -o t.csv --json
```

issue2007 규제 표:

```bash
rhwp table-to-csv samples/basic/issue2007_nested_cell_pagination_42065.hwp --table 1 -o t1.csv --json
# tables[0].index == 1
```

`--table 0` 은 `Ⅰ. 규제 심사(안) 개요` 1열 표다.

없는 번호:

```
rhwp table-to-csv 양식.hwpx --table 99999 --json
# exit 1, stdout 0
```

픽스처: [../fixtures/envelopes/export_tables_jichi_index_not_zero.json](../fixtures/envelopes/export_tables_jichi_index_not_zero.json),
[../fixtures/envelopes/table_to_csv_unknown_table_exit1.json](../fixtures/envelopes/table_to_csv_unknown_table_exit1.json),
[../fixtures/envelopes/export_tables_issue2007.json](../fixtures/envelopes/export_tables_issue2007.json).
