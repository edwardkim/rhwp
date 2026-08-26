# 09 — 덮인 칸에 값 (`coveredCellNotEmpty`)

병합 표 CSV 의 빈 칸을 스프레드시트가 채웠을 때.

## 명령

```bash
rhwp csv-to-table samples/table-001.hwp \
  --csv covered.csv --table 0 --dry-run --json
```

## 기대

exit 2.

```json
{
  "changedCount": 0,
  "invalid": [
    {
      "reason": "coveredCellNotEmpty",
      "row": 0,
      "col": 2,
      "message": "(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요."
    }
  ]
}
```

## 다음 수

1. 값을 앵커 `(0,1)` 로 옮기고 `(0,2)` 는 `""`
2. 또는 [12](12_merged_fallback_set_cell.md) — `edit set-cell --row 0 --col 1`

덮인 칸에 쓰는 플래그를 발명하지 않는다.

픽스처: [../fixtures/envelopes/csv_to_table_covered.json](../fixtures/envelopes/csv_to_table_covered.json).
