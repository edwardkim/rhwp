# 01_export_pdf

왜: 인쇄본이 필요할 때

## request.json

```json
{
  "doc": "공문.hwpx",
  "symptom": "인쇄본이 필요할 때",
  "params": {},
  "goal": "export-pdf"
}
```

## 루프
- 표 안 goal `export-pdf` → export-pdf
- 게이트: `pdf-magic`
- 성공 시 `out/공문.pdf`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
