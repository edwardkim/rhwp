# 08 — 열 수 불일치 (`colCountMismatch`, exit 2)

남는 열을 붙이거나, 쉼표를 인용하지 않아 필드가 늘어난 경우.

## 명령

```bash
rhwp csv-to-table samples/hwp_table_test.hwp \
  --csv fixtures/csv/table0_col_long.csv --table 0 --dry-run --json
```

## 기대

exit 2. `invalid` 에 행마다 `colCountMismatch` (`actual:4`, `expected:3`).
`changedCount: 0`. 파일 없음.

`samples/table-001.hwp` 에 2×2 CSV 를 넣으면 행·열이 **함께** 모인다
(playbook §10-5, expected 19 / 9).

픽스처: [../fixtures/envelopes/csv_to_table_col_mismatch.json](../fixtures/envelopes/csv_to_table_col_mismatch.json),
[../fixtures/envelopes/csv_to_table_table001_both_mismatch.json](../fixtures/envelopes/csv_to_table_table001_both_mismatch.json).

처방: RFC 라이브러리로 다시 쓰고, 뽑은 격자 열 수를 유지한다.
