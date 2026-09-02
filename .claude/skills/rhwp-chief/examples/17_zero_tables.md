# 17_zero_tables

왜: 표 0개는 성공

## request.json

```json
{
  "doc": "plain.hwpx",
  "symptom": "표 0개는 성공",
  "params": {},
  "goal": "extract-tables"
}
```

## 루프
- 표 안 goal `extract-tables` → export-tables, table-to-csv
- 게이트: `csv-count`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
