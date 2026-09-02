# T-recipe02-rfc4180

- family: `table-to-csv`
- command: `table-to-csv`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `extract`
- exit: 0
- writes: false
- csvRoundtrip: `allowed`
- invalid: —
- changedCount: 0
- next: 판독기가 "" 를 한 따옴표로 되돌리는지 본다

값은 가,나"다 . CSV 는 "가,나""다".

## argv

```bash
rhwp table-to-csv samples/hwp_table_test.hwp --table 0 --json
```

## csv

```csv
"가,나""다",담당자,세부 내용
,,
,,
,,

```

## 점유

- cellCount: 12
- coveredCount: 0
- mergedAnchorCount: 0
- areaSum: 12 / grid 12
