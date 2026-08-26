# T-hwp_table_test_t0-bom

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
- next: 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라.

bom_flag_only_affects_the_file_not_the_envelope.

## argv

```bash
rhwp table-to-csv samples/hwp_table_test.hwp --table 0 -o table0_bom.csv --bom --json
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
