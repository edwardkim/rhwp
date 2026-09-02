# D-hwp_table_test_t0-row_short_1

- family: `dimension`
- command: `csv-to-table`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `dry-run`
- exit: 2
- writes: false
- csvRoundtrip: `allowed`
- invalid: rowCountMismatch
- changedCount: 0
- next: 뽑은 CSV 를 표 치수에 맞춰 재생성

4×3 치수 계약. 조용한 절삭 금지.

## argv

```bash
rhwp csv-to-table samples/hwp_table_test.hwp --csv D-hwp_table_test_t0-row_short_1.csv --table 0 --dry-run --json -o out/D-hwp_table_test_t0-row_short_1.hwp
```

## csv

```csv
제목,담당자,세부 내용
,,
,,

```

## invalid[]

```json
[
  {
    "reason": "rowCountMismatch",
    "message": "CSV 행 수 3 가 표 0 의 행 수 4 와 다릅니다 — 표 크기는 바꾸지 않습니다.",
    "expected": 4,
    "actual": 3
  }
]
```

## 점유

- cellCount: 12
- coveredCount: 0
- mergedAnchorCount: 0
- areaSum: 12 / grid 12
