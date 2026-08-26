# 09_path_escape

왜: 폴더 밖 거부

## request.json

```json
{
  "doc": "../secret.hwp",
  "symptom": "폴더 밖 거부",
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
