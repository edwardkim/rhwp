# 06 — 표 → CSV

갈래: **수확**. 장: `20_표와_데이터.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`1번 표를 CSV 로.`

## 명령

```bash
rhwp table-to-csv samples/basic/issue2007_nested_cell_pagination_42065.hwp --table 1 -o t1.csv --json
```

되돌리기는 `csv-to-table` + `--dry-run` + `--verify`. 정본은 rhwp-table-exchange.
