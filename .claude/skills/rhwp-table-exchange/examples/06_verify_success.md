# 06 — 저장하고 자기검증 (`--verify`)

05 가 `invalid: []` 일 때만.

## 명령

```bash
rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv table0_edited.csv --table 0 \
  -o output/table_updated.hwp --verify --json
```

## 기대

```json
{
  "changedCount": 9,
  "invalid": [],
  "dryRun": false,
  "outputFormat": "hwp5",
  "changedPages": [0],
  "verify": {"diffCount": 0, "identical": true}
}
```

exit 0. `output` 이 있다.

같은 CSV 를 다시 넣으면 `changedCount: 0` 이어도 `identical: true` 가 정상.

픽스처: [../fixtures/envelopes/csv_to_table_verify_ok.json](../fixtures/envelopes/csv_to_table_verify_ok.json),
[../fixtures/envelopes/csv_to_table_identical_zero.json](../fixtures/envelopes/csv_to_table_identical_zero.json).

재독은 [14](14_roundtrip_hwp_table_test.md).
`identical: false` 는 [15](15_verify_exit3.md).
