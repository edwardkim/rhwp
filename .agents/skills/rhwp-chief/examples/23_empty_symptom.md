# 23_empty_symptom

왜: symptom 은 선택

## request.json

```json
{
  "doc": "a.hwpx",
  "symptom": "symptom 은 선택",
  "params": {},
  "goal": "export-pdf"
}
```

## 루프
- 표 안 goal `export-pdf` → export-pdf
- 게이트: `pdf-magic`
- 성공 시 `out/a.pdf`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
