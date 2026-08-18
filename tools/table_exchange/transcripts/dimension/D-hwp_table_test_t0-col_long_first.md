# D-hwp_table_test_t0-col_long_first

- family: `dimension`
- command: `csv-to-table`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `dry-run`
- exit: 2
- writes: false
- csvRoundtrip: `allowed`
- invalid: colCountMismatch
- changedCount: 0
- next: 뽑은 CSV 를 표 치수에 맞춰 재생성

4×3 치수 계약. 조용한 절삭 금지.

## argv

```bash
rhwp csv-to-table samples/hwp_table_test.hwp --csv D-hwp_table_test_t0-col_long_first.csv --table 0 --dry-run --json -o out/D-hwp_table_test_t0-col_long_first.hwp
```

## csv

```csv
제목,담당자,세부 내용,남는열
,,
,,
,,

```

## invalid[]

```json
[
  {
    "reason": "colCountMismatch",
    "message": "CSV 0행 필드 수 4 가 표 0 의 열 수 3 와 다릅니다 — 표 크기는 바꾸지 않습니다.",
    "row": 0,
    "expected": 3,
    "actual": 4
  }
]
```

## 점유

- cellCount: 12
- coveredCount: 0
- mergedAnchorCount: 0
- areaSum: 12 / grid 12
