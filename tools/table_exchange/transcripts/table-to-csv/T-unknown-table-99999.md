# T-unknown-table-99999

- family: `table-to-csv`
- command: `table-to-csv`
- sample: `samples/hwp_table_test.hwp`
- table: 99999 (0×0)
- mode: `extract`
- exit: 1
- writes: false
- csvRoundtrip: `forbidden`
- invalid: —
- changedCount: 0
- next: export-tables --json

본문 최상위 표 없음. export-tables 의 실제 index 를 본다.

## argv

```bash
rhwp table-to-csv samples/hwp_table_test.hwp --table 99999 --json
```

## 점유

- cellCount: 0
- coveredCount: 0
- mergedAnchorCount: None
- areaSum: None / grid None
