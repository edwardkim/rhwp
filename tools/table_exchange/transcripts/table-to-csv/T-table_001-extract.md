# T-table_001-extract

- family: `table-to-csv`
- command: `table-to-csv`
- sample: `samples/table-001.hwp`
- table: 0 (19×9)
- mode: `extract`
- exit: 0
- writes: true
- csvRoundtrip: `extract-only`
- invalid: —
- changedCount: 0
- next: edit set-cell

행마다 필드 수 = colCount. 병합 채움이 빠지면 열이 밀린다.

## argv

```bash
rhwp table-to-csv samples/table-001.hwp --table 0 -o table0.csv --json
```

## csv

```csv
구 분,5월,,,6월,,,비고,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,
,,,,,,,,

```

## 점유

- cellCount: 165
- coveredCount: 6
- mergedAnchorCount: 3
- areaSum: 171 / grid 171
