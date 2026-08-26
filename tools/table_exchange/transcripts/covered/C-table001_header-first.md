# C-table001_header-first

- family: `covered`
- command: `csv-to-table`
- sample: `samples/table-001.hwp`
- table: 0 (19×9)
- mode: `dry-run`
- exit: 2
- writes: false
- csvRoundtrip: `extract-only`
- invalid: coveredCellNotEmpty
- changedCount: 0
- next: edit set-cell

table-001 문서화된 헤더 병합만. covered=6. 한 칸도 안 씀.

## argv

```bash
rhwp csv-to-table samples/table-001.hwp --csv C-table001_header-first.csv --table 0 --dry-run --json -o out/C-table001_header-first.hwp
```

## csv

```csv
구 분,5월,덮인칸값,,6월,,,비고,tablH8
tabl1_0,tabl1_1,tabl1_2,tabl1_3,tabl1_4,tabl1_5,tabl1_6,,tabl1_8
tabl2_0,tabl2_1,tabl2_2,tabl2_3,tabl2_4,tabl2_5,tabl2_6,,tabl2_8
tabl3_0,tabl3_1,tabl3_2,tabl3_3,tabl3_4,tabl3_5,tabl3_6,tabl3_7,tabl3_8
tabl4_0,tabl4_1,tabl4_2,tabl4_3,tabl4_4,tabl4_5,tabl4_6,tabl4_7,tabl4_8
tabl5_0,tabl5_1,tabl5_2,tabl5_3,tabl5_4,tabl5_5,tabl5_6,tabl5_7,tabl5_8
tabl6_0,tabl6_1,tabl6_2,tabl6_3,tabl6_4,tabl6_5,tabl6_6,tabl6_7,tabl6_8
tabl7_0,tabl7_1,tabl7_2,tabl7_3,tabl7_4,tabl7_5,tabl7_6,tabl7_7,tabl7_8
tabl8_0,tabl8_1,tabl8_2,tabl8_3,tabl8_4,tabl8_5,tabl8_6,tabl8_7,tabl8_8
tabl9_0,tabl9_1,tabl9_2,tabl9_3,tabl9_4,tabl9_5,tabl9_6,tabl9_7,tabl9_8
tabl10_0,tabl10_1,tabl10_2,tabl10_3,tabl10_4,tabl10_5,tabl10_6,tabl10_7,tabl10_8
tabl11_0,tabl11_1,tabl11_2,tabl11_3,tabl11_4,t
…
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

- cellCount: 165
- coveredCount: 6
- mergedAnchorCount: 3
- areaSum: 171 / grid 171
