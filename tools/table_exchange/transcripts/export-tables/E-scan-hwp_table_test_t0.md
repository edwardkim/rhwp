# E-scan-hwp_table_test_t0

- family: `export-tables`
- command: `export-tables`
- sample: `samples/hwp_table_test.hwp`
- table: 0 (4×3)
- mode: `scan`
- exit: 0
- writes: false
- csvRoundtrip: `allowed`
- invalid: —
- changedCount: 0
- next: containerPath 없는 표의 index 로 --table

tableCount=1. index 는 배열 순번이 아니다.

## argv

```bash
rhwp export-tables samples/hwp_table_test.hwp --json
```

## 점유

- cellCount: 12
- coveredCount: 0
- mergedAnchorCount: 0
- areaSum: 12 / grid 12
