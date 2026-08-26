# R-hwp_table_test_t0-preview

- family: `dry-run`
- command: `csv-to-table`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `dry-run`
- exit: 0
- writes: false
- csvRoundtrip: `allowed`
- invalid: —
- changedCount: 9
- next: csv-to-table --verify

changedPages=null, output=null, -o 를 줘도 파일 없음.

## argv

```bash
rhwp csv-to-table samples/hwp_table_test.hwp --csv R-hwp_table_test_t0-preview.csv --table 0 --dry-run --json -o out/R-hwp_table_test_t0-preview.hwp
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
