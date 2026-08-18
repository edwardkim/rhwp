# R-recipe02-edited

- family: `dry-run`
- command: `csv-to-table`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `dry-run`
- exit: 0
- writes: false
- csvRoundtrip: `allowed`
- invalid: —
- changedCount: 9
- next: csv-to-table --verify

헤더 3칸은 old==new 라 changed 에서 빠진다. 12-3=9.

## argv

```bash
rhwp csv-to-table samples/hwp_table_test.hwp --csv R-recipe02-edited.csv --table 0 --dry-run --json -o out/R-recipe02-edited.hwp
```

## csv

```csv
제목,담당자,세부 내용
서버 이관,홍길동,1차 완료
DB 백업,김철수,진행중
문서 정리,박영희,대기

```

## 점유

- cellCount: 12
- coveredCount: 0
- mergedAnchorCount: 0
- areaSum: 12 / grid 12
