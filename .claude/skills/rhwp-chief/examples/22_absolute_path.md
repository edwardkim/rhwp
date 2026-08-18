# 22_absolute_path

왜: 절대경로 거부

## request.json

```json
{
  "doc": "C:/tmp/a.hwp",
  "symptom": "절대경로 거부",
  "params": {},
  "goal": "export-text"
}
```

## 루프
- 표 안 goal `export-text` → export-text
- 게이트: `json-envelope`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
