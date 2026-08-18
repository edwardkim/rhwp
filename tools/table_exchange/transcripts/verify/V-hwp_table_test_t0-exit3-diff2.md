# V-hwp_table_test_t0-exit3-diff2

- family: `verify`
- command: `csv-to-table`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `verify`
- exit: 3
- writes: true
- csvRoundtrip: `allowed`
- invalid: —
- changedCount: 9
- next: csv-to-table --verify

고장이 아니라 판정 데이터. invalid [] 이고 outputKept true.

## argv

```bash
rhwp csv-to-table samples/hwp_table_test.hwp --csv V-hwp_table_test_t0-exit3-diff2.csv --table 0 --json -o out/V-hwp_table_test_t0-exit3-diff2.hwp --verify
```

## csv

```csv
제목,담당자,세부 내용
v1_0,v1_1,v1_2
v2_0,v2_1,v2_2
v3_0,v3_1,v3_2

```

## 점유

- cellCount: 12
- coveredCount: 0
- mergedAnchorCount: 0
- areaSum: 12 / grid 12
