# 17 — 쉼표·따옴표 (RFC 4180)

권위: `rfc4180_quoting_survives_a_round_trip_through_the_document`.

값 `가,나"다` 는 CSV 에서 `"가,나""다"`.

## 확인

```bash
# 계약은 set-cell 로 값을 넣고 table-to-csv 로 다시 뽑는다
rhwp table-to-csv 문서.hwpx --table I --json \
  | jq -r '.tables[0].csv' | head -c 20
# "가,나""다"
```

손으로 `가,나"다,다음열` 을 쓰면 열이 늘어 `colCountMismatch`.

픽스처: [../fixtures/envelopes/table_to_csv_rfc4180.json](../fixtures/envelopes/table_to_csv_rfc4180.json),
[../fixtures/csv/table0_quoted.csv](../fixtures/csv/table0_quoted.csv).

닫히지 않은 따옴표는 `csvParse` ([../fixtures/envelopes/csv_to_table_csv_parse.json](../fixtures/envelopes/csv_to_table_csv_parse.json)).
