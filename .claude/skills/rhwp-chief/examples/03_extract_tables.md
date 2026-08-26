# 03_extract_tables

왜: 표를 스프레드시트로

## request.json

```json
{
  "doc": "예산.hwpx",
  "symptom": "표를 스프레드시트로",
  "params": {},
  "goal": "extract-tables"
}
```

## 루프
- 표 안 goal `extract-tables` → export-tables, table-to-csv
- 게이트: `csv-count`
- 성공 시 `out/table_0.csv`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
