# 12_already_processed

왜: result.json 있으면 건너뜀

## request.json

```json
{
  "doc": "done.hwpx",
  "symptom": "result.json 있으면 건너뜀",
  "params": {},
  "goal": "export-pdf"
}
```

## 루프
- 표 안 goal `export-pdf` → export-pdf
- 게이트: `pdf-magic`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
