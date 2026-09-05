# 08 — table-extract

켤 때: `table_count > 0`. 우선순위 75. skill `rhwp-table-exchange`.

```
rhwp export-tables <file> --json
```

## why

- 병합 없음: `표 N개 — 격자를 CSV 로 뽑아 고치고 되돌리기`
- 병합 있음: `표 N개(병합 셀 포함 M개) — …`

병합 개수는 표를 다시 세지 않고, 이미 뽑힌 격자에서 rowSpan/colSpan>1
인 표의 수다.

## 다음

1. `export-tables --json` 으로 좌표·병합을 확인한다
2. 실제 CSV 가 필요하면 `table-to-csv`
3. 되넣기는 `csv-to-table` — 이 스킬이 아니라 table-exchange

표가 메뉴에 없다고 표가 없음을 보장하지 않는다. 엔진이 못 센 표는
직접 `export-tables` 를 칠 수 있다 (X09).

이 장이 set-cell 이나 csv-to-table 을 재구현하지 않는다.
