# C-block_2x2-all

- family: `covered`
- command: `csv-to-table`
- sample: `synthetic/merge-pattern.hwpx`
- table: 0 (4×4)
- mode: `dry-run`
- exit: 2
- writes: false
- csvRoundtrip: `extract-only`
- invalid: coveredCellNotEmpty,coveredCellNotEmpty,coveredCellNotEmpty
- changedCount: 0
- next: edit set-cell

2×2 블록 병합. covered=3. 한 칸도 안 씀.

## argv

```bash
rhwp csv-to-table synthetic/merge-pattern.hwpx --csv C-block_2x2-all.csv --table 0 --dry-run --json -o out/C-block_2x2-all.hwpx
```

## csv

```csv
blocH0,blocH1,blocH2,blocH3
bloc1_0,bloc1_1,덮인칸값1_2,bloc1_3
bloc2_0,덮인칸값2_1,덮인칸값2_2,bloc2_3
bloc3_0,bloc3_1,bloc3_2,bloc3_3

```

## invalid[]

```json
[
  {
    "reason": "coveredCellNotEmpty",
    "message": "(1,2) 는 병합으로 덮인 칸입니다 — 앵커 (1,1) 를 지정하세요.",
    "row": 1,
    "col": 2,
    "anchorRow": 1,
    "anchorCol": 1
  },
  {
    "reason": "coveredCellNotEmpty",
    "message": "(2,1) 는 병합으로 덮인 칸입니다 — 앵커 (1,1) 를 지정하세요.",
    "row": 2,
    "col": 1,
    "anchorRow": 1,
    "anchorCol": 1
  },
  {
    "reason": "coveredCellNotEmpty",
    "message": "(2,2) 는 병합으로 덮인 칸입니다 — 앵커 (1,1) 를 지정하세요.",
    "row": 2,
    "col": 2,
    "anchorRow": 1,
    "anchorCol": 1
  }
]
```

## 점유

- cellCount: 13
- coveredCount: 3
- mergedAnchorCount: 1
- areaSum: 16 / grid 16
