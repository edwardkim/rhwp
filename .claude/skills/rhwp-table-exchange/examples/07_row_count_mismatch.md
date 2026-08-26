# 07 — 행 수 불일치 (`rowCountMismatch`, exit 2)

권위: `row_count_mismatch_is_invalid_and_writes_nothing`.

헤더를 빼고 값 3줄만 넣은 CSV (`table0_header_dropped` 와 같은 치수).

## 명령

```bash
rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv fixtures/csv/table0_row_short.csv --table 0 --dry-run --json
```

## 기대

exit 2. 파일을 만들지 않음. 봉투는 있다.

```json
{
  "changedCount": 0,
  "changed": [],
  "invalid": [
    {
      "reason": "rowCountMismatch",
      "actual": 3,
      "expected": 4,
      "message": "CSV 행 수 3 가 표 0 의 행 수 4 와 다릅니다 — 표 크기는 바꾸지 않습니다."
    }
  ]
}
```

stdout 을 버리지 마라. `expected` 가 표의 `rowCount` 다.

처방: `table-to-csv` 산출(4행)을 다시 고친다. 표 크기를 줄이는 명령을
발명하지 않는다.

픽스처: [../fixtures/envelopes/csv_to_table_row_mismatch.json](../fixtures/envelopes/csv_to_table_row_mismatch.json).
