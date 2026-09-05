# 10. goal=extract-tables

두 명령을 순서대로 친다. 둘 다 기존 표면이다.

```
rhwp export-tables <doc> --json
# tables[].index 마다
rhwp table-to-csv <doc> --table <index> -o out/table_<index>.csv
```

`needs:export-tables,table-to-csv`

## 게이트 (C20)

- `export-tables` exit 0, stdout JSON
- `tables` 가 빈 배열이면 **성공** (`표가 없다 (0개)`, artifacts []).
  표 없음은 오류가 아니다.
- 각 index 에 대해 CSV 파일이 실존해야 한다. 하나라도 실패면 그 즉시
  `failed` (나머지 표를 부분 성공으로 회신하지 않는다).

## 산출

- `out/table_0.csv`, `out/table_1.csv`, …
- `summary`: `표 N개 CSV 수확`

## 하지 않는 것

- 표를 다시 문서에 집어넣기 (`csv-to-table`) — 왕복은
  `rhwp-table-exchange` 층. 이 표의 goal 은 수확만.
- `--table` 을 증상 문장에서 고르기. 전 표다.
- 새 `export-csv` CLI.
