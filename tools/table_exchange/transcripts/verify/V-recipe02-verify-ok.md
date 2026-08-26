# V-recipe02-verify-ok

- family: `verify`
- command: `csv-to-table`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `verify`
- exit: 0
- writes: true
- csvRoundtrip: `allowed`
- invalid: —
- changedCount: 9
- next: csv-to-table --verify

changedCount 9, identical true, outputFormat hwp5.

## argv

```bash
rhwp csv-to-table samples/hwp_table_test.hwp --csv V-recipe02-verify-ok.csv --table 0 --json -o out/V-recipe02-verify-ok.hwp --verify
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
