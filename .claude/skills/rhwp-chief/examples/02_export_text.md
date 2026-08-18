# 02_export_text

왜: 본문만 검색·이관할 때

## request.json

```json
{
  "doc": "회의록.hwp",
  "symptom": "본문만 검색·이관할 때",
  "params": {},
  "goal": "export-text"
}
```

## 루프
- 표 안 goal `export-text` → export-text
- 게이트: `json-envelope`
- 성공 시 `out/text.json`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
