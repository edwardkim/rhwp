# T-hwp_table_test_t0-extract

- family: `table-to-csv`
- command: `table-to-csv`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `extract`
- exit: 0
- writes: true
- csvRoundtrip: `allowed`
- invalid: —
- changedCount: 0
- next: 외부 편집 후 csv-to-table --dry-run

행마다 필드 수 = colCount. 병합 채움이 빠지면 열이 밀린다.

## argv

```bash
rhwp table-to-csv samples/hwp_table_test.hwp --table 0 -o table0.csv --json
```

## csv

```csv
제목,담당자,세부 내용
,,
,,
,,

```

## 점유

- cellCount: 12
- coveredCount: 0
- mergedAnchorCount: 0
- areaSum: 12 / grid 12
