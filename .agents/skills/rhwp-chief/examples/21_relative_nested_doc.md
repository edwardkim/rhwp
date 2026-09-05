# 21_relative_nested_doc

왜: 하위 상대경로는 허용

## request.json

```json
{
  "doc": "docs/본문.hwpx",
  "symptom": "하위 상대경로는 허용",
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
