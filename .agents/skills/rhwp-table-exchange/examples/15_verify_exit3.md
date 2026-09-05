# 15 — `--verify` 차이 (exit 3, 데이터)

권위: [dry_run_verify.md](../references/dry_run_verify.md) §4.

## 봉투

```json
{
  "changedCount": 9,
  "invalid": [],
  "output": "table_updated.hwp",
  "verify": {"diffCount": 2, "identical": false}
}
```

exit **3**. 예외가 아니다. 산출물은 있다.

## 다음 수

```bash
rhwp export-tables table_updated.hwp --json > actual.json
# CSV 와 cells[].text 를 diff
# 병합·중첩 혼재면 set-cell 축으로
```

같은 `csv-to-table` 을 즉시 재시도하지 않는다.
`invalid` 가 비어 있으므로 치수 문제가 아니다.

픽스처: [../fixtures/envelopes/csv_to_table_verify_fail.json](../fixtures/envelopes/csv_to_table_verify_fail.json),
[../fixtures/loops/verify_exit3.json](../fixtures/loops/verify_exit3.json).
