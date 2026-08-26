# 11 — chart-extract

켤 때: `chart_count > 0`. 우선순위 60. skill `rhwp-table-exchange`.
confidence high.

```
rhwp chart-to-csv <file> --json
```

## why

`차트 N개 — 계열·카테고리 수치를 CSV 로 추출`

표와 차트가 같이 있으면 표(75)가 차트(60)보다 위다. 둘 다 같은
table-exchange 스킬이지만 명령이 다르다. 차트 수치를 `export-tables` 로
읽지 않는다. OLE 차트 파서를 이 스킬이 만지지 않는다.

차트 1개도 항목을 켠다. 메뉴에 없다고 차트가 없음을 보장하지 않는다
(정직한 휴리스틱). 직접 `chart-to-csv` 를 치는 것은 금지 가 아니다.
