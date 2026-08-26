# V-hwpx_basic_01-identical

- family: `verify`
- command: `csv-to-table`
- sample: `samples/hwpx/basic-table-01.hwpx`
- table: 0 (2×2)
- mode: `verify`
- exit: 0
- writes: true
- csvRoundtrip: `allowed`
- invalid: —
- changedCount: 0
- next: csv-to-table --verify

outputFormat=hwpx. identical_csv_writes_nothing_and_verifies.

## argv

```bash
rhwp csv-to-table samples/hwpx/basic-table-01.hwpx --csv V-hwpx_basic_01-identical.csv --table 0 --json -o out/V-hwpx_basic_01-identical.hwpx --verify
```

## csv

```csv
이름,점수
가,90

```

## 점유

- cellCount: 4
- coveredCount: 0
- mergedAnchorCount: 0
- areaSum: 4 / grid 4
