# R-hwp_table_test_t0-ctrl-lf

- family: `dry-run`
- command: `csv-to-table`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `dry-run`
- exit: 2
- writes: false
- csvRoundtrip: `allowed`
- invalid: controlCharacter
- changedCount: 0
- next: LF/TAB 을 공백으로 치환 후 --dry-run

RFC 인용으로 감싸도 파싱된 값을 본다.

## argv

```bash
rhwp csv-to-table samples/hwp_table_test.hwp --csv R-hwp_table_test_t0-ctrl-lf.csv --table 0 --dry-run --json -o out/R-hwp_table_test_t0-ctrl-lf.hwp
```

## csv

```csv
제목,담당자,세부 내용
"줄
바꿈",,
,,
,,

```

## invalid[]

```json
[
  {
    "reason": "controlCharacter",
    "message": "셀 값에 줄바꿈·탭은 v1 에서 허용하지 않습니다.",
    "row": 1,
    "col": 0
  }
]
```

## 점유

- cellCount: 12
- coveredCount: 0
- mergedAnchorCount: 0
- areaSum: 12 / grid 12
