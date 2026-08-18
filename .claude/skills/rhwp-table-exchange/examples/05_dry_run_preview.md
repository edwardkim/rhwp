# 05 — 쓰기 전에 미리보기 (`--dry-run`)

권위: [dry_run_verify.md](../references/dry_run_verify.md).

## 명령

```bash
rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv table0_edited.csv --table 0 --dry-run --json \
  | jq '{changedCount, invalid, dryRun, changedPages, output}'
```

## 기대

```json
{
  "changedCount": 9,
  "invalid": [],
  "dryRun": true,
  "changedPages": null,
  "output": null
}
```

디스크에 `*_csv.hwp` 가 생기면 계약 위반이다. `-o` 를 붙여도 안 쓴다.

`changedPages: null` 을 `[]` 로 읽지 마라.

`invalid` 가 있으면 [07](07_row_count_mismatch.md) 등으로 분기. 저장하지 않는다.

픽스처: [../fixtures/envelopes/csv_to_table_dry_run.json](../fixtures/envelopes/csv_to_table_dry_run.json).
