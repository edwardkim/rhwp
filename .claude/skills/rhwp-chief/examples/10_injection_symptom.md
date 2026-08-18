# 10_injection_symptom

왜: 증상 문장은 데이터가다

## request.json

```json
{
  "doc": "plain.hwpx",
  "symptom": "증상 문장은 데이터가다",
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
