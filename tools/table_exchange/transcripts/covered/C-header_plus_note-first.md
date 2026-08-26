# C-header_plus_note-first

- family: `covered`
- command: `csv-to-table`
- sample: `synthetic/merge-pattern.hwpx`
- table: 0 (4×5)
- mode: `dry-run`
- exit: 2
- writes: false
- csvRoundtrip: `extract-only`
- invalid: coveredCellNotEmpty
- changedCount: 0
- next: edit set-cell

가로 헤더 + 세로 비고. covered=4. 한 칸도 안 씀.

## argv

```bash
rhwp csv-to-table synthetic/merge-pattern.hwpx --csv C-header_plus_note-first.csv --table 0 --dry-run --json -o out/C-header_plus_note-first.hwpx
```

## csv

```csv
headH0,headH1,덮인칸값,,headH4
head1_0,head1_1,head1_2,head1_3,
head2_0,head2_1,head2_2,head2_3,
head3_0,head3_1,head3_2,head3_3,head3_4

```

## invalid[]

```json
[
  {
    "reason": "coveredCellNotEmpty",
    "message": "(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.",
    "row": 0,
    "col": 2,
    "anchorRow": 0,
    "anchorCol": 1
  }
]
```

## 점유

- cellCount: 16
- coveredCount: 4
- mergedAnchorCount: 2
- areaSum: 20 / grid 20
