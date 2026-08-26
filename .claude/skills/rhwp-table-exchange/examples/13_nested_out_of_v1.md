# 13 — 중첩 표는 v1 밖

표본: `samples/inner-table-01.hwp`.

## 명령

```bash
rhwp export-tables samples/inner-table-01.hwp --json \
  | jq '.tables[0].cells[] | select(.nested != null) | {row,col,nested}'
```

## 기대

한 칸에 `nested[0].cellCount == 24`,
`nested[0].containerPath[].kind == "tableCell"`.

## 하지 않는 것

- `nested[0].index` 를 `--table` 에 넣기
- 안쪽 24칸 CSV 왕복 명령을 만들기
- 바깥 CSV 를 늘려 안쪽을 펼치기

바깥 최상위만 `table-to-csv --table <그 index>`.
안쪽은 사용자에게 v1 밖이라고 알린다.

픽스처: [../fixtures/envelopes/export_tables_inner_table.json](../fixtures/envelopes/export_tables_inner_table.json),
[../fixtures/loops/nested_reject.json](../fixtures/loops/nested_reject.json).
