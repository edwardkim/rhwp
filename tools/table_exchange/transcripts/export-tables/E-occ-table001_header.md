# E-occ-table001_header

- family: `export-tables`
- command: `export-tables`
- sample: `samples/table-001.hwp`
- table: 0 (19×9)
- mode: `scan`
- exit: 0
- writes: false
- csvRoundtrip: `extract-only`
- invalid: —
- changedCount: 0
- next: edit set-cell

covered=6, areaSum=171 <= 171, roundtrip=extract-only.

## argv

```bash
rhwp export-tables samples/table-001.hwp --json
```

## 점유

- cellCount: 165
- coveredCount: 6
- mergedAnchorCount: 3
- areaSum: 171 / grid 171
