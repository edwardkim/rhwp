# 16 — 여러 문서 표 수확 (`batch export-tables`)

권위: `cli_commands.md` §batch, `batch_axes_contract.rs`.

## 명령

```bash
printf '%s\n' samples/hwp_table_test.hwp samples/table-001.hwp \
  | rhwp batch export-tables --json \
  | jq -c '{source, tableCount}'
```

## 기대

레코드마다 단건 `export-tables --json` 과 같은 스키마.

```
{"source":"samples/hwp_table_test.hwp","tableCount":10}
{"source":"samples/table-001.hwp","tableCount":1}
```

한 파일이 실패해도 스트림은 이어진다. 종료 코드로 전부를 버리지 말고
레코드 `error` 를 본다.

`batch csv-to-table` 은 없다. 되돌리기는 문서·표마다 단건.

픽스처: [../fixtures/envelopes/batch_export_tables_ndjson.json](../fixtures/envelopes/batch_export_tables_ndjson.json).
