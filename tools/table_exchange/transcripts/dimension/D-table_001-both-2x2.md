# D-table_001-both-2x2

- family: `dimension`
- command: `csv-to-table`
- sample: `samples/table-001.hwp`
- table: 0 (19×9)
- mode: `dry-run`
- exit: 2
- writes: false
- csvRoundtrip: `extract-only`
- invalid: rowCountMismatch,colCountMismatch,colCountMismatch
- changedCount: 0
- next: edit set-cell

playbook §10-5. rowCountMismatch + colCountMismatch.

## argv

```bash
rhwp csv-to-table samples/table-001.hwp --csv D-table_001-both-2x2.csv --table 0 --dry-run --json -o out/D-table_001-both-2x2.hwp
```

## csv

```csv
a,b
c,d

```

## invalid[]

```json
[
  {
    "reason": "rowCountMismatch",
    "message": "CSV 행 수 2 가 표 0 의 행 수 19 와 다릅니다 — 표 크기는 바꾸지 않습니다.",
    "expected": 19,
    "actual": 2
  },
  {
    "reason": "colCountMismatch",
    "message": "CSV 0행 필드 수 2 가 표 0 의 열 수 9 와 다릅니다 — 표 크기는 바꾸지 않습니다.",
    "row": 0,
    "expected": 9,
    "actual": 2
  },
  {
    "reason": "colCountMismatch",
    "message": "CSV 1행 필드 수 2 가 표 0 의 열 수 9 와 다릅니다 — 표 크기는 바꾸지 않습니다.",
    "row": 1,
    "expected": 9,
    "actual": 2
  }
]
```

## 점유

- cellCount: 165
- coveredCount: 6
- mergedAnchorCount: 3
- areaSum: 171 / grid 171
